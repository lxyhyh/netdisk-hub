# ADR-004：引入 JNA 5.19.1（Android AAR）作为 UniFFI Kotlin 绑定的运行时依赖

- 状态：**已批准**（2026-11-27 用户批准：JNA 5.19.1@aar + ABI 收敛 arm64-v8a/x86_64）
- 日期：2026-11-27
- 关联任务：T-002e 第 3 轮（UniFFI + Kotlin 集成）
- 影响范围：app 模块的新增运行时依赖、APK 原生库清单、16 KB 页面兼容性

## 背景

T-002e 第 3 轮要把 Rust 侧 `#[uniffi::export]` 的 `probe_magic()` 暴露给 Kotlin。
UniFFI 0.32.2 的 Kotlin 后端不是生成 JNI 代码，而是生成 **JNA direct mapping** 代码：
生成文件 `netdiskhub_core.kt` 里 `internal object UniffiLib` 的 `init` 块直接调用
`com.sun.jna.Native.register(UniffiLib::class.java, findLibraryName(...))`，
并把 `uniffi_netdiskhub_core_fn_func_probe_magic` 等声明为 `external fun`。

本地实测（`rust/target/release/libnetdiskhub_core.so` + `uniffi-bindgen 0.32.2`）：
生成的绑定文件为 `com/netdiskhub/core/netdiskhub_core.kt`，1006 行 / 40751 字节，
其 import 段包含 `com.sun.jna.Library / IntegerType / Native / Pointer / Structure / Callback`
与 `com.sun.jna.ptr.*`，公开入口为 `fun probeMagic(): kotlin.UInt`。
因此 **JNA 不是可选项**，缺少它 Kotlin 编译阶段直接失败。

## 决策

| 项 | 值 |
| --- | --- |
| 依赖坐标 | `net.java.dev.jna:jna:5.19.1@aar` |
| 引入位置 | `gradle/libs.versions.toml`（版本 + library）+ `app/build.gradle.kts` 的 `dependencies` |
| 打包范围 | 随 `abiFilters` 裁剪，最终只进 APK 的 `arm64-v8a` 与 `x86_64` |
| 许可证 | LGPL-2.1-or-later 或 Apache-2.0（JNA 双许可，本项目取 Apache-2.0 分支使用） |

## 理由

### 1. 为什么必须是 JNA

UniFFI 0.32.2 的 Kotlin 后端只支持 JNA 运行时（templates/wrapper.kt 硬编码 import `com.sun.jna.*`）。
官方文档 `docs/manual/src/kotlin/gradle.md` 明确要求 JNA「5.12.0 or greater is required」，
并给出 Android 写法 `implementation "net.java.dev.jna:jna:5.12.0@aar"`。

本项目禁止引入 UniFFI 的第三方 Gradle 插件，也没有自研 JNI 桥的规划，
因此 JNA 是该链路上唯一可行的运行时。

### 2. 为什么必须用 `@aar` 而不是普通 `jar`

`jna-5.19.1.pom` 的 `packaging = jar`，普通 jar 变体里没有 `libjnidispatch.so`；
Android 必须用 AAR 变体，否则运行期 `Native.register` 找不到 `libjnidispatch.so`。
实测 `jna-5.19.1.aar`（525093 字节）内容：

- `classes.jar` 223133 字节，含 `com/sun/jna/Native.class`、`Structure.class`、`Pointer.class`、`Callback.class`、`internal/Cleaner.class` 等；
- `jni/` 下 7 个 ABI 的 `libjnidispatch.so`：arm64-v8a **176520**、armeabi-v7a 126496、armeabi 126980、mips 130556、mips64 150256、x86 124380、x86_64 **126912** 字节；
- `AndroidManifest.xml`（268 字节，`package="com.sun.jna"`、`versionCode="5190199"`、`versionName="5.19.1"`、`<uses-sdk android:targetSdkVersion="25"/>`）。

### 3. 为什么选 5.19.1

- Maven Central `maven-metadata.xml` 共 55 个版本，`latest` 与 `release` 均为 **5.19.1**；
- 满足官方 `>= 5.12.0` 要求；
- 该版本 `classes.jar` 内含 `com/sun/jna/internal/Cleaner.class`，UniFFI 生成代码在存在 object 类型时会用它（本轮探针无 object，不触发，但保留向上兼容）。

### 4. 16 KB 页面对齐（已实测，非推测）

本机 `readelf -lW` 实测：

| 库 | PT_LOAD 段 Align |
| --- | --- |
| `jna-5.19.1.aar` 内 `jni/arm64-v8a/libjnidispatch.so` | 2 个 LOAD 段，全部 `0x4000` |
| `jna-5.19.1.aar` 内 `jni/x86_64/libjnidispatch.so` | 2 个 LOAD 段，全部 `0x4000` |

即 JNA 5.19.1 预编译库已满足 16 KB 页面要求，不需要自行重编或用 `-Wl,-z,max-page-size` 处理。

### 5. manifest 合并风险（待 CI 首次运行验证）

AAR 自带 `<uses-sdk android:targetSdkVersion="25"/>`，低于本项目的 `targetSdk = 37`。
AGP 9 的 manifest merger 对库的 `uses-sdk` 一般直接忽略并采用宿主值，但**本轮尚未在真实构建中验证**。

按用户明确要求：**不使用 `tools:overrideLibrary` 预先掩盖合并问题**，
因此 `app/src/main/AndroidManifest.xml` 保持原样，不加任何 `tools:*`。
若 CI 报 manifest 合并错误，记录完整错误日志并回到本 ADR 重新评估，而不是先加 `tools:*` 试。

## ABI 范围声明

本项目 Rust 核心当前只交叉编译 `arm64-v8a` 与 `x86_64`（见 T-002e 第 2 轮验收）。
因此 Gradle 侧必须用 `ndk { abiFilters += listOf("arm64-v8a", "x86_64") }` 把 ABI 收敛到这两个，
否则 JNA AAR 自带的 armeabi / x86 / mips / mips64 等 5 个 ABI 会被一并打包，
造成 APK 内出现无对应 Rust 实现的 `libjnidispatch.so`。

**当前不承诺支持上述两个之外的任何 ABI。**
`docs/ARCHITECTURE.md` 未对 ABI 范围作过承诺，本 ADR 不构成与既有文档冲突。

## 回退策略

1. 若 5.19.1 在 AGP 9.3.1 下出现 manifest 合并或打包问题，回退到官方文档示例的 5.12.0（同为 `@aar`），并记录原因；
2. 若 JNA 完全不可用，则说明 UniFFI Kotlin 后端与本项目约束不兼容，需回到 T-001 层面重新评估桥接方案（不在本轮范围内，不得擅自改架构）。

## 明确不在本 ADR 范围内

- 不引入 `jna-platform`；
- 不引入任何 Kotlin 协程依赖（本轮为同步接口，官方文档也只要求异步场景才需要）；
- 不做 ABI 扩展到 `armeabi-v7a` 等（Rust 侧未编译，不承诺）；
- 不涉及运行时行为验证（属 T-003）。