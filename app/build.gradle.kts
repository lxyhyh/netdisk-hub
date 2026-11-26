import org.jetbrains.kotlin.gradle.dsl.JvmTarget

plugins {
    alias(libs.plugins.android.application)
    alias(libs.plugins.kotlin.compose)
}

android {
    namespace = "com.netdiskhub.app"

    // SDK 37 的 SDK 平台包名是 platforms;android-37.0，
    // compileSdk = 37 时 AGP 解析到的平台哈希为 android-37.0；
    // CI 中同时建立 android-37 符号链接作为兜底（见 workflow）。
    compileSdk = 37

    defaultConfig {
        applicationId = "com.netdiskhub.app"
        minSdk = 31
        targetSdk = 37
        versionCode = 1
        versionName = "0.1.0"
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
}
