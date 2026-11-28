import org.gradle.api.DefaultTask
import org.gradle.api.file.ConfigurableFileCollection
import org.gradle.api.file.DirectoryProperty
import org.gradle.api.provider.ListProperty
import org.gradle.api.provider.Property
import org.gradle.api.tasks.Input
import org.gradle.api.tasks.InputFiles
import org.gradle.api.tasks.OutputDirectory
import org.gradle.api.tasks.TaskAction
import org.gradle.process.ExecOperations
import org.jetbrains.kotlin.gradle.dsl.JvmTarget
import java.io.File
import javax.inject.Inject

plugins {
    alias(libs.plugins.android.application)
    alias(libs.plugins.kotlin.compose)
}

// ---------------------------------------------------------------------------
// T-002e 第 3 轮：Rust 交叉编译 + UniFFI 绑定生成 + Kotlin 源码接入
//
// 三个任务的分工（都跑在 rust/ 工作目录，全部带 --locked，锁 rust/Cargo.lock）：
//   cargoNdkBuild          cargo ndk 交叉编译 rust/core → app/build/rustJniLibs/<abi>/
//   generateUniffiBindings 先 cargo build uniffi-bindgen，再用它从 .so 里抽元数据
//                          生成 Kotlin 绑定 → app/build/generated/uniffi/kotlin/<variant>/
//
// 为什么写自定义任务类型而不是 tasks.register<Exec>：AGP 的
// SourceDirectories.addGeneratedSourceDirectory 要求任务暴露一个 @OutputDirectory
// 属性，裸 Exec 没有该属性，无法把生成目录接进 Kotlin 编译。
// ---------------------------------------------------------------------------

/** 用 cargo-ndk 把 rust/core 交叉编译成各 ABI 的 .so。 */
abstract class CargoNdkBuildTask : DefaultTask() {
    @get:InputFiles
    abstract val rustSources: ConfigurableFileCollection

    @get:Input
    abstract val ndkHome: Property<String>

    @get:Input
    abstract val rustDir: Property<String>

    @get:Input
    abstract val androidAbis: ListProperty<String>

    @get:OutputDirectory
    abstract val outputDir: DirectoryProperty

    @get:Inject
    abstract val execOps: ExecOperations

    @TaskAction
    fun build() {
        val ndk = ndkHome.get()
        val args = mutableListOf("cargo", "ndk")
        androidAbis.get().forEach { abi ->
            args += listOf("-t", abi)
        }
        args += listOf(
            "-P", "31",
            "-o", outputDir.get().asFile.absolutePath,
            "--manifest-path", "core/Cargo.toml",
            "build", "--release", "--locked",
        )
        logger.lifecycle("cargo-ndk: " + args.joinToString(" "))
        execOps.exec {
            workingDir = File(rustDir.get())
            // cargo-ndk 4.x 按 ANDROID_NDK_HOME → ANDROID_NDK_ROOT → ANDROID_NDK_PATH → NDK_HOME
            // 顺序找 NDK，这里三个都指到 AGP 解析出的同一个目录，避免命中别处的老版本。
            environment("ANDROID_NDK_HOME", ndk)
            environment("ANDROID_NDK_ROOT", ndk)
            environment("NDK_HOME", ndk)
            // 展开成 vararg：直接传 List 会被当成单个参数，cargo 报 "no such subcommand"
            commandLine(*args.toTypedArray())
        }
    }
}

/** 编译 uniffi-bindgen，再从 cdylib 抽取元数据生成 Kotlin 绑定。 */
abstract class GenerateUniffiBindingsTask : DefaultTask() {
    @get:InputFiles
    abstract val rustSources: ConfigurableFileCollection

    @get:InputFiles
    abstract val libraries: ConfigurableFileCollection

    @get:Input
    abstract val targetAbi: Property<String>

    @get:Input
    abstract val rustDir: Property<String>

    @get:OutputDirectory
    abstract val outputDir: DirectoryProperty

    @get:Inject
    abstract val execOps: ExecOperations

    @TaskAction
    fun generate() {
        val rustDirFile = File(rustDir.get())
        val bindgen = rustDirFile.resolve("target/release/uniffi-bindgen")

        execOps.exec {
            workingDir = rustDirFile
            commandLine("cargo", "build", "--release", "--locked", "-p", "uniffi-bindgen")
        }
        check(bindgen.isFile) { "uniffi-bindgen 未生成: " + bindgen.absolutePath }

        // libraries 里放的是 cargoNdkBuild 的输出**目录**，不是散装文件；
        // ConfigurableFileCollection.from(Provider<Directory>) 解析出的就是这个目录本身。
        val libRoot = libraries.files.single()
        val lib = libRoot.resolve(targetAbi.get()).resolve("libnetdiskhub_core.so")
        check(lib.isFile) { "找不到 " + targetAbi.get() + " 的 cdylib: " + lib.absolutePath }
        val out = outputDir.get().asFile
        out.deleteRecursively()
        out.mkdirs()

        execOps.exec {
            workingDir = rustDirFile
            // 坑：uniffi-bindgen 0.32.2 的 --library 是布尔 flag（已废弃），
            // .so 路径必须是**位置参数**，写成 --library <path> 会被当成两个位置参数解析失败。
            commandLine(
                bindgen.absolutePath, "generate",
                "--language", "kotlin",
                "--out-dir", out.absolutePath,
                "--no-format",
                lib.absolutePath,
            )
        }
    }
}

val nativeAbis = listOf("arm64-v8a", "x86_64")
val rustProjectDir = rootProject.layout.projectDirectory.dir("rust")

val cargoNdkBuild = tasks.register<CargoNdkBuildTask>("cargoNdkBuild") {
    group = "rust"
    description = "用 cargo-ndk 交叉编译 rust/core 到 app/build/rustJniLibs"
    rustSources.from(
        rustProjectDir.dir("core/src"),
        rustProjectDir.file("Cargo.toml"),
        rustProjectDir.file("Cargo.lock"),
        rustProjectDir.file("rust-toolchain.toml"),
        rustProjectDir.file("core/Cargo.toml"),
        rustProjectDir.file("core/uniffi.toml"),
    )
    androidAbis.set(nativeAbis)
    rustDir.set(rustProjectDir.asFile.absolutePath)
    // outputDir 不在这里 set：通过 androidComponents 的
    // addGeneratedSourceDirectory 接入时，AGP 会用 convention 指定输出目录，
    // 并自动建立 merge*JniLibFolders → cargoNdkBuild 的任务依赖。
    // 手动 set 会覆盖 convention，反而丢掉这层依赖声明。
    // 由 AGP 自己解析 ndkVersion 对应的目录，避免和 rust.yml 里的路径写法分叉：
    // 这里取到的就是 $ANDROID_HOME/ndk/30.0.16248370。
    ndkHome.set(androidComponents.sdkComponents.ndkDirectory.map { it.asFile.absolutePath })
}

androidComponents {
    onVariants { variant ->
        val variantName = variant.name.replaceFirstChar { it.uppercase() }
        val generateBindings = tasks.register<GenerateUniffiBindingsTask>(
            "generateUniffiBindings" + variantName,
        ) {
            group = "rust"
            description = "从 " + variant.name + " 的 cdylib 生成 UniFFI Kotlin 绑定"
            dependsOn(cargoNdkBuild)
            rustSources.from(
                rustProjectDir.dir("core/src"),
                rustProjectDir.file("Cargo.toml"),
                rustProjectDir.file("Cargo.lock"),
                rustProjectDir.file("core/Cargo.toml"),
                rustProjectDir.file("core/uniffi.toml"),
                rustProjectDir.dir("uniffi-bindgen"),
            )
            // 必须把 DirectoryProperty 落成 File：直接 from(Provider<DirectoryProperty>)
            // 拿到的是属性对象本身而不是目录，files.single() 会解析错。
            libraries.from(cargoNdkBuild.map { it.outputDir.asFile })
            targetAbi.set("arm64-v8a")
            rustDir.set(rustProjectDir.asFile.absolutePath)
            // outputDir 不在这里设：AGP 的 addGeneratedSourceDirectory 会把它
            // 注入到 variant 专用的 build/generated 子目录，提前设会被覆盖反而误导。
        }
        // jniLibs 也必须走 addGeneratedSourceDirectory：只往 sourceSets 里塞路径，
        // Gradle 9 会在 merge*JniLibFolders 上抛「uses this output without declaring
        // an explicit or implicit dependency」的校验错误（第 3 轮 CI 实测）。
        // 这里接进来之后，merge*JniLibFolders 会自动依赖 cargoNdkBuild。
        val jniLibs = variant.sources.jniLibs
        if (jniLibs != null) {
            jniLibs.addGeneratedSourceDirectory(cargoNdkBuild, CargoNdkBuildTask::outputDir)
        } else {
            throw GradleException("变体 " + variant.name + " 没有 jniLibs 源目录")
        }

        val kotlinSources = variant.sources.kotlin
        if (kotlinSources != null) {
            kotlinSources.addGeneratedSourceDirectory(
                generateBindings, GenerateUniffiBindingsTask::outputDir,
            )
        } else {
            // AGP 9 内置 Kotlin 下正常不会走到这里；留兜底而不是静默跳过，
            // 否则生成的绑定不进编译路径，APK 里少类却不报错。
            val javaSources = variant.sources.java
                ?: throw GradleException("变体 " + variant.name + " 既没有 kotlin 也没有 java 源目录")
            javaSources.addGeneratedSourceDirectory(
                generateBindings, GenerateUniffiBindingsTask::outputDir,
            )
        }
    }
}

android {
    namespace = "com.netdiskhub.app"

    // SDK 37 的 SDK 平台包名是 platforms;android-37.0，
    // compileSdk = 37 时 AGP 解析到的平台哈希为 android-37.0；
    // CI 中同时建立 android-37 符号链接作为兜底（见 workflow）。
    compileSdk = 37

    // NDK 版本写死 r30，与 rust.yml 交叉编译用的 NDK 完全一致；
    // 本机没有 NDK，构建只在 CI 上发生。
    ndkVersion = "30.0.16248370"

    defaultConfig {
        applicationId = "com.netdiskhub.app"
        minSdk = 31
        targetSdk = 37
        versionCode = 1
        versionName = "0.1.0"

        // 原生核心只支持 arm64-v8a 与 x86_64（ADR-004）。
        // 不写 abiFilters 的话 JNA 的 AAR 会把 7 个 ABI 的 libjnidispatch.so
        // 全部打进 APK，APK 体积膨胀且与「仅两个 ABI」的声明不一致。
        ndk {
            abiFilters += nativeAbis
        }

        // T-003：Android 运行时测试（仪器测试）用 AndroidX Test 的 runner。
        // 只跑在设备/模拟器上，宿主机 JVM 测试不能替代（.so 与 JNA 只有真机才加载得起来）。
        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
    }

    buildFeatures {
        compose = true
    }

    buildTypes {
        release {
            isMinifyEnabled = false
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

}

// AGP 9 移除了 android.kotlinOptions，必须用顶层 kotlin 块
kotlin {
    compilerOptions {
        jvmTarget.set(JvmTarget.JVM_17)
    }
}

dependencies {
    implementation(platform(libs.compose.bom))
    implementation(libs.compose.ui)
    implementation(libs.compose.material3)
    implementation(libs.androidx.activity.compose)

    // UniFFI 生成的 Kotlin 绑定直接 import com.sun.jna.*（direct mapping，
    // 用 Native.register 而非 Native.load），所以 JNA 是编译期与运行期依赖。
    // 必须带 @aar：普通 jar 里没有 jni/<abi>/libjnidispatch.so，运行期注册必失败。
    // 版本在 gradle/libs.versions.toml 的 [versions].jna 锁定。
    implementation("net.java.dev.jna:jna:" + libs.versions.jna.get() + "@aar")

    // T-003 / T-103：仪器测试依赖。见 docs/ADR/ADR-005-android-instrumentation-test.md。
    // 断言只用 JUnit4 内置 org.junit.Assert，不引 MockK / Mockito / Espresso。
    androidTestImplementation(libs.androidx.test.ext.junit)
    androidTestImplementation(libs.androidx.test.runner)
    androidTestImplementation(libs.androidx.test.core)
}