# CI 环境记录（ci-setup）

- 文档状态：T-002e 执行中（第 1、2 轮已归档；第 3 轮未开始）
- 最后更新：2026-11-27
- 性质：本文件是 CI runner 环境的**唯一基准**（对应 RISKS.md R24），不写计划，只写实测结果

## 1. 概述

本地开发机为 Android proot Ubuntu，没有 Android SDK / NDK / Gradle 完整环境，**本地无法编译**。
所有编译、构建、测试、打包一律通过 GitHub Actions 完成，CI 结果（日志 + artifact）是唯一验证依据。

T-002e 分三轮，每轮只增加一个变量：

| 轮次 | 内容 | 状态 |
|---|---|---|
| 第 1 轮 | 最小 Android 工程 + APK 打包（不含 Rust） | **已完成**（commit `0773aad`） |
| 第 2 轮 | Rust 交叉编译独立 workflow（cargo-ndk → arm64-v8a / x86_64 .so） | **已完成**（commit `b8e1b08`） |
| 第 3 轮 | UniFFI 绑定生成 + Kotlin 集成 | 未开始 |

三轮之后才补 quality-gate 占位。

## 2. workflow 清单

| 文件 | workflow name | 触发 | 本轮状态 |
|---|---|---|---|
| `.github/workflows/build.yml` | `build` | push / pull_request（master）+ workflow_dispatch | 已跑通 |
| `.github/workflows/rust.yml` | `rust` | push / pull_request（master，paths: `rust/**`、`.github/workflows/rust.yml`）+ workflow_dispatch | 已跑通 |

第 1 轮**只有** `build.yml`，没有 test / quality-gate workflow（按 T-002e 分轮方案，第 1 轮不强制单元测试与 lint）。
第 2 轮新增 `rust.yml`，与 `build.yml` 相互独立、并行运行；仍然没有 test / quality-gate workflow。

## 3. 第 1 轮运行记录（build.yml）

| 项 | 实测值 |
|---|---|
| Commit SHA | `0773aad65d9777451c896c156f082bb15a6ac8c0` |
| Commit message | `build: 最小 Android 工程骨架 + 第 1 轮 CI（assembleDebug）` |
| 运行链接 | https://github.com/lxyhyh/netdisk-hub/actions/runs/45863557547 |
| 运行 ID | `45863557547` |
| Workflow | `build` |
| 结果 | **success** |
| 触发时间 | 2026-11-26T16:58:03Z |
| 完成时间 | 2026-11-26T17:00:32Z |
| 总耗时 | 2 分 29 秒 |
| 构建步骤耗时 | 1 分 39 秒（日志原文 `BUILD SUCCESSFUL in 1m 39s`，`36 actionable tasks: 36 executed`） |

各步骤结果全部 success：`Set up job` / `Checkout` / `Set up JDK 17` / `Set up Gradle 9.7.0` /
`Install Android SDK platform 37` / `Build debug APK` / `Upload APK` / 三个 Post 步骤 / `Complete job`。

关键任务确实执行（非 UP-TO-DATE / NO-SOURCE）：`compileDebugKotlin` → `dexBuilderDebug` →
`mergeProjectDexDebug` → `packageDebug` → `assembleDebug`。
`compileDebugJavaWithJavac` 为 `NO-SOURCE`（工程是纯 Kotlin，属预期）。

## 4. Artifact 与产物核验

| 项 | 实测值 |
|---|---|
| Artifact 名 | `app-debug-apk` |
| Artifact ID | `15138789375` |
| 下载入口 | https://github.com/lxyhyh/netdisk-hub/actions/runs/45863557547/artifacts/15138789375 |
| 线上 zip 大小 | 10204286 B |
| 解压后 APK 大小 | **29209031 B** |
| **Artifact Digest（GitHub Actions 页面提供）** | `a59ffff27f92f483bff0eb306abc8dde8dadd4c08a00d893b4e554e9a7802eee` |
| 解压后 APK 的 SHA256（本地实测） | `1038561f0ee8302b47006dd34edc0cac71762c56a0196a0713fc7c7312fd0928` |

> **Artifact Digest ≠ APK 的 SHA256。** 前者是 GitHub Actions 对上传的 artifact（zip 压缩包）计算的摘要，
> 后者是对解压出来的 APK 计算的摘要，两者对象不同，不能互相校验。
> 若要校验 APK 完整性，用本行的 APK SHA256；若要校验下载的 artifact 未被篡改，用 Digest。

> zip 大小与 APK 大小不同是正常的：前者是 artifact 的 zip 压缩包，后者是解压出来的 APK，不是两次产物。

`aapt dump badging` 实测结果：

```
package: name='com.netdiskhub.app' versionCode='1' versionName='0.1.0' compileSdkVersion='37'
sdkVersion:'31'
targetSdkVersion:'37'
application-label:'Netdisk Hub'
```

APK 内容结构：5 个 dex（`classes.dex` 18573732 B / `classes2.dex` 38608 B / `classes3.dex` 7244 B /
`classes4.dex` 50144 B / `classes5.dex` 9930100 B）+ 4 个 ABI 的 `libandroidx.graphics.path.so`。
最小的 `classes3.dex` 才是本工程自己的代码（`com.netdiskhub.app.*`），其余是 Compose 依赖。

## 5. runner 环境（实测）

| 项 | 值 |
|---|---|
| Runner | `ubuntu-latest` |
| JDK | Temurin 17（`JAVA_HOME=/opt/hostedtoolcache/Java_Temurin-Hotspot_jdk/17.0.21-9/x64`） |
| Gradle | 9.7.0（`gradle/actions/setup-gradle@v4`，不提交 Gradle wrapper） |
| Android SDK 平台 | `platforms;android-37.0` |
| Build Tools | `build-tools;36.0.0` |
| sdkmanager 路径 | `/usr/local/lib/android/sdk/cmdline-tools/latest/bin/sdkmanager` |
| 符号链接 | `$ANDROID_HOME/platforms/android-37` → `$ANDROID_HOME/platforms/android-37.0`（新建的是 `android-37`，指向真实目录 `android-37.0`） |

SDK 安装链路已验证可用：安装后 `ls $ANDROID_HOME/platforms` 输出含 `android-37` 与 `android-37.0`，
`ls $ANDROID_HOME/build-tools` 输出含 `36.0.0`；随后 `:app:assembleDebug` 成功。

workflow 里的 sdkmanager 定位逻辑先 `command -v sdkmanager`，取不到再 `find "$ANDROID_HOME" -maxdepth 4`，
本轮走的是前者（日志 `sdkmanager = /usr/local/lib/android/sdk/cmdline-tools/latest/bin/sdkmanager`）。

## 6. 版本实际生效证据

本节只记录**从构建产物里直接读出来的**证据，与配置文件里声明什么无关。

### 6.1 Kotlin 编译器版本：2.4.x（已验证）

把 APK 的 `classes3.dex`（含本工程代码）反汇编后，`com/netdiskhub/app/MainActivity.smali` 与
`com/netdiskhub/app/ComposableSingletons$MainActivityKt.smali` 的 `kotlin/Metadata` 注解均为：

```
.annotation runtime Lkotlin/Metadata;
    mv = {
        0x2,
        0x4,
        0x0
    }
```

即 metadata version = **2.4.0**，这是 Kotlin 2.4.x 编译器写入的值（2.2.x 会写 `2.2.0`）。
因此实际编译本工程的 Kotlin 编译器是 2.4.x 系列。

### 6.2 Compose 依赖版本（已验证，读 APK 内 `META-INF/*.version`）

| 构件 | 实际版本 |
|---|---|
| `androidx.compose.ui:ui` | 1.12.1 |
| `androidx.compose.runtime:runtime` | 1.12.1 |
| `androidx.compose.material3:material3` | 1.4.0 |
| `androidx.activity:activity-compose` | 1.13.0 |

对应 `compose-bom` `2026.09.00`，与 `gradle/libs.versions.toml` 的声明一致。

### 6.3 KGP 精确版本号：未在 CI 日志中明文打印（待确认）

CI 日志里**没有** `2.4.21` 或 `2.2.10` 的明文，也没有 Gradle 打印的 KGP 版本行
（`--stacktrace` 成功时不展开依赖解析树）。

当前结论是**推论链，不是日志原文**：

1. AGP 9.3.1 的 POM 对 `kotlin-gradle-plugin` 有 2.2.10 运行时依赖（AGP 9.0–9.5 的最低 KGP 版本）；
2. 顶层 `plugins` 块声明了 `org.jetbrains.kotlin.plugin.compose:2.4.21`；
3. Android 官方 Built-in Kotlin 文档：顶层声明其它 Kotlin 插件且版本 ≥2.2.10 时，Gradle 会把 KGP 对齐到该版本；
4. 产物 `mv = 2.4.0` 与「对齐到 2.4.21 生效」一致。

若需要那行明文，后续可加一步 `gradle buildEnvironment` 或
`gradle :app:dependencies --configuration kotlin-extension` 打印。本轮未做（不新增 workflow 步骤）。

## 7. 已知警告

本轮构建过程中出现两类警告：一类来自 Android 构建（§7.1），一类来自 GitHub Actions 平台（§7.2）。
两类都**不是**本次 Android 构建失败。

### 7.1 so strip 警告

日志原文（`Build debug APK` 步骤）：

```
Unable to strip the following libraries, packaging them as they are: libandroidx.graphics.path.so. Run with --info option to learn more.
```

**来源（有产物证据）**：该 `.so` 来自 AndroidX graphics-path 原生组件，APK 内同时存在对应的
版本标记 `META-INF/androidx.graphics_graphics-path.version`，其内容为 `1.0.1`。

关于它是否为传递依赖引入：本次只核实到「工程 `dependencies` 块未直接声明
`androidx.graphics:graphics-path`，而 APK 内出现了该组件」这一事实，
**没有**从 Gradle 依赖树打印出它经由哪条边引入的证据，因此本文件不写具体传递路径。

本 APK 中该 `.so` 的实际文件信息（`unzip -l` 实测）：

| ABI | 大小 | APK 内路径 |
|---|---|---|
| `arm64-v8a` | 10096 B | `lib/arm64-v8a/libandroidx.graphics.path.so` |
| `armeabi-v7a` | 7252 B | `lib/armeabi-v7a/libandroidx.graphics.path.so` |
| `x86` | 9284 B | `lib/x86/libandroidx.graphics.path.so` |
| `x86_64` | 10760 B | `lib/x86_64/libandroidx.graphics.path.so` |

本次构建的事实：`BUILD SUCCESSFUL`，APK 成功打包，5 个 dex + 4 个 ABI 的 `.so` 都在产物里，
`aapt dump badging` 可正常解析。**该警告没有导致本次构建失败。**

该警告对产物有无功能影响、release 构建是否仍需 strip 配置，本轮未验证。

### 7.2 GitHub Actions 平台警告

这两条是 **GitHub Actions 平台层面**的提示，与本次 Android 构建无关，**不是构建失败**：

| 警告 | 内容 |
|---|---|
| Node.js 20 deprecated | 相关工作流 Action 被平台强制以 Node.js 24 运行 |
| `actions/setup-java@v4` 弃用 | 该 Action 已弃用，不再接收更新，GitHub 提示迁移到 `v5` |

本轮**不修改 workflow、不升级 Actions 版本**；迁移到 `v5` 留待后续轮次单独处理。

## 8. 已验证 / 未验证

### 8.1 已验证（直接证据）

- Android 工程可编译，APK 可生成
- GitHub Actions workflow 可运行
- 实际编译 Kotlin 版本为 2.4.x（依据 dex 中 `kotlin/Metadata` `mv={2,4,0}`）
- Compose 依赖实际版本：ui 1.12.1 / runtime 1.12.1 / material3 1.4.0 / activity-compose 1.13.0，对应 compose-bom 2026.09.00
- SDK 安装链路正常：`platforms;android-37.0` 可装，`android-37` 符号链接生效
- APK 产物可下载，包名/版本号/minSdk/targetSdk 与配置一致
- **Rust → Android 交叉编译链路可用**（Rust 1.100.0 + cargo-ndk 4.1.2 + NDK 30.0.16248370，
  产出 arm64-v8a / x86_64 两个 ELF64 `.so`，导出符号 `netdiskhub_probe`，见 §10）

### 8.2 未验证 / 待确认

- KGP 精确版本号 2.4.21 未在 CI 日志中明文打印（见 §6.3）
- UniFFI 绑定生成
- Kotlin ↔ Rust 调用
- APK 真实设备安装和运行
- 单元测试、lint、CRAP、变异测试（第 1 轮按分轮方案不要求）

## 9. 后续轮次

| 轮次 | 目标 | 验收 |
|---|---|---|
| 第 2 轮 | `rust/` + `.github/workflows/rust.yml`，cargo-ndk 产出 arm64-v8a 与 x86_64 的 `.so` 并上传 artifact | **已完成**，见 §10 |
| 第 3 轮 | UniFFI 绑定生成 + Kotlin 集成 | APK 含 `.so` 与 Kotlin 绑定，Kotlin 能调用 Rust 导出接口 |
| 收尾 | `quality-gate.yml` 占位（仅 workflow_dispatch + PR 触发 + 注释） | 工具等 T-004 定稿后接入 |

每轮完成后在本文件追加对应小节，不攒到最后。

## 10. 第 2 轮运行记录（rust.yml）

### 10.1 提交与运行

| 项 | 实测值 |
|---|---|
| Commit SHA | `b8e1b0898cabf6b273d17c8f01f69292d9de73c5` |
| Commit message | `build: add standalone Rust Android cross-compilation workflow` |
| 运行链接 | https://github.com/lxyhyh/netdisk-hub/actions/runs/45938734221 |
| 运行 ID | `45938734221` |
| Workflow | `rust`（`.github/workflows/rust.yml`） |
| 结果 | **success** |
| 触发时间 | 2026-11-27T04:14:03Z |
| 完成时间 | 2026-11-27T04:15:08Z |
| 总耗时 | 1 分 05 秒 |

13 个 step 全部 success：`Set up job` / `Checkout` / `Install and pin Android NDK r30` /
`Install Rust 1.100.0 with Android targets` / `Install cargo-ndk 4.1.2` / `Verify toolchain versions` /
`Verify rust/rust-toolchain.toml takes effect` / `Cross-compile for arm64-v8a and x86_64` /
`Verify artifacts` / `Check generated libs are git-ignored` / `Upload shared libraries` /
`Post Checkout` / `Complete job`。

### 10.2 工具链实测

| 组件 | 实测版本 | 证据来源 |
|---|---|---|
| Rust | **1.100.0** | 日志原文 `rustc 1.100.0 (0ef1bc386 2026-11-09)` |
| Cargo | **1.100.0** | 日志原文 `cargo 1.100.0 (3acc79c0c 2026-11-05)` |
| cargo-ndk | **4.1.2** | 日志原文 `cargo-ndk 4.1.2`（`cargo ndk --version` 输出） |
| NDK | **30.0.16248370** | `source.properties` 的 `Pkg.Revision` |

**NDK 实际路径与版本证据**（`Install and pin Android NDK r30` 步骤日志原文）：

```
NDK_DIR = /usr/local/lib/android/sdk/ndk/30.0.16248370
source.properties Pkg.Revision = 30.0.16248370
```

workflow 用三个变量同时指向同一个完整版本目录（日志 env 原文）：

```
ANDROID_NDK_HOME: /usr/local/lib/android/sdk/ndk/30.0.16248370
ANDROID_NDK_ROOT: /usr/local/lib/android/sdk/ndk/30.0.16248370
ANDROID_NDK:      /usr/local/lib/android/sdk/ndk/30.0.16248370
```

> 背景：runner 镜像预置的 `ANDROID_NDK_HOME` 默认指向 **27.3.13750724**（default NDK），
> 而 cargo-ndk 4.1.2 的 NDK 探测顺序里 `ANDROID_NDK_HOME` 优先。不显式覆盖会静默用错 NDK，
> 因此 workflow 在构建前 assert `Pkg.Revision` 相等，而不是只打印预期值。

**`rust/rust-toolchain.toml` 实际生效证据**（`Verify rust/rust-toolchain.toml takes effect` 步骤，
`working-directory: rust`）：

```
1.100.0-x86_64-unknown-linux-gnu (overridden by '/home/runner/work/netdisk-hub/netdisk-hub/rust/rust-toolchain.toml')
rustc 1.100.0 (0ef1bc386 2026-11-09)
```

括号内的 `overridden by .../rust/rust-toolchain.toml` 是 rustup 自己给出的「该目录下的
toolchain 文件接管了版本选择」的直接证据，不是仅靠 `rustc -V` 推断。

### 10.3 交叉编译结果

编译命令（`working-directory: rust`）：

```
cargo ndk -t arm64-v8a -t x86_64 -P 31 -o target/jniLibs --manifest-path core/Cargo.toml build --release
```

日志原文：

```
    Building arm64-v8a (aarch64-linux-android)
   Compiling netdiskhub-core v0.0.0 (/home/runner/work/netdisk-hub/netdisk-hub/rust/core)
    Finished `release` profile [optimized] target(s) in 4.92s
    Building x86_64 (x86_64-linux-android)
   Compiling netdiskhub-core v0.0.0 (/home/runner/work/netdisk-hub/netdisk-hub/rust/core)
    Finished `release` profile [optimized] target(s) in 0.20s
     Copying libraries to /home/runner/work/netdisk-hub/netdisk-hub/rust/target/jniLibs
```

**arm64-v8a**

| 项 | 实测值 |
|---|---|
| target | `aarch64-linux-android` |
| 文件 | `libnetdiskhub_core.so` |
| 大小 | **449560 B** |
| `file` 输出 | `ELF 64-bit LSB shared object, ARM aarch64, version 1 (SYSV), dynamically linked, not stripped` |
| ELF Class | **ELF64** |
| Machine | **AArch64** |
| 导出符号 | `netdiskhub_probe`（`llvm-nm -D` 原文 `0000000000014f50 T netdiskhub_probe`） |

**x86_64**

| 项 | 实测值 |
|---|---|
| target | `x86_64-linux-android` |
| 文件 | `libnetdiskhub_core.so` |
| 大小 | **424656 B** |
| `file` 输出 | `ELF 64-bit LSB shared object, x86-64, version 1 (SYSV), dynamically linked, not stripped` |
| ELF Class | **ELF64** |
| Machine | **Advanced Micro Devices X86-64** |
| 导出符号 | `netdiskhub_probe`（`llvm-nm -D` 原文 `00000000000143f8 T netdiskhub_probe`） |

以上 ELF Class / Machine 不是只打印，`Verify artifacts` 步骤里是**硬断言**（`case` 匹配
`ELF 64-bit LSB shared object`、`grep -q 'Class:[[:space:]]*ELF64'`、
`grep -q 'Machine:[[:space:]]*AArch64'` / `'Machine:.*X86-64'`），任何一条不匹配即 `exit 1`。

**上传目录文件清单校验**：`Verify artifacts` 把 `rust/target/jniLibs` 下全部普通文件排序后与
预期清单做**严格字符串相等**比较，任何多余文件、缺失文件或路径错误都 `exit 1`。日志原文：

```
--- expected ---            --- actual ---
arm64-v8a/libnetdiskhub_core.so     arm64-v8a/libnetdiskhub_core.so
x86_64/libnetdiskhub_core.so        x86_64/libnetdiskhub_core.so
```

### 10.4 Artifact

| 项 | 实测值 |
|---|---|
| 名称 | `rust-android-libs` |
| Artifact ID | `15172320101` |
| ZIP 大小 | **379922 B** |
| **Artifact Digest（GitHub Actions 页面提供）** | `f9a9d5bb1a9311fc6e1c0d998256139d97e130403e20342a512049cf9dd77575` |
| 下载链接 | https://github.com/lxyhyh/netdisk-hub/actions/runs/45938734221/artifacts/15172320101 |

`gh run download` 解压后**无多余根目录**，只有两个文件：

```
./arm64-v8a/libnetdiskhub_core.so   449560 B    sha256 51da6bada0e38782a33130ef6e8ad21667d116632c1da9b767953537b5f999a5
./x86_64/libnetdiskhub_core.so      424656 B    sha256 35aa38f35aa161deaf770ee9be1b42b9f534f20a70ba6cf7e724dab12c65d087
```

> **Artifact ZIP Digest ≠ 单个 `.so` 文件的 SHA256。** 前者是 GitHub 对上传的 artifact（zip）
> 计算的摘要，后者是对解压出来的单个 `.so` 计算的摘要，两者对象不同，不能互相校验。
> 要校验下载的 artifact 未被篡改用 Digest；要校验某 ABI 的 `.so` 内容用对应 SHA256。

### 10.5 生成产物不进 Git

`Check generated libs are git-ignored` 步骤实测命中：

```
.gitignore:26:rust/target/	rust/target/jniLibs/arm64-v8a/libnetdiskhub_core.so
.gitignore:26:rust/target/	rust/target/jniLibs/x86_64/libnetdiskhub_core.so
```

并额外断言 `git ls-files -- 'rust/target/**'` 为空（防止产物被误提交）。

### 10.6 runner 差异（相对 §5）

| 项 | 第 1 轮 build.yml | 第 2 轮 rust.yml |
|---|---|---|
| `runs-on` | `ubuntu-latest` | **`ubuntu-24.04`**（固定） |
| 原因 | 第 1 轮未涉及 NDK 版本 | 本轮 NDK 环境依据 Ubuntu 24.04 runner 的软件清单设计，不依赖 `ubuntu-latest` 的迁移状态 |
| NDK | 不用 | 显式装 `30.0.16248370` 并钉死 |

### 10.7 Android 构建回归

同一次 push 也触发了原有的 Android `build` workflow（因为没有 paths 过滤）：

| 项 | 实测值 |
|---|---|
| 运行链接 | https://github.com/lxyhyh/netdisk-hub/actions/runs/45938734219 |
| 运行 ID | `45938734219` |
| Workflow | `build` |
| 结果 | **success** |
| 耗时 | 59 秒 |

这构成**既有 Android 构建的回归验证**：新增 `rust/**` 与 `rust.yml` 后，原 APK 打包链路未回退。

> 注：本轮 `rust` 与 `build` 是两个独立 workflow，`.so` **尚未**集成进 APK。

### 10.8 第 2 轮实际出现的警告

本轮 `rust` workflow 只有三条警告，**全部是 GitHub Actions 运行时层面的提示，不是 Rust 编译失败**，
构建结果仍为 `success`：

| # | 警告 | 出现位置 |
|---|---|---|
| 1 | DEP0040：`punycode` 模块已弃用，建议改用 userland 替代实现 | `Upload shared libraries` |
| 2 | DEP0169：`url.parse()` 行为未标准化，建议改用 WHATWG URL API | `Upload shared libraries` |
| 3 | Node.js 20 已弃用，`actions/checkout@v4` 与 `actions/upload-artifact@v4` 声明 Node.js 20，被平台强制以 Node.js 24 运行 | `Complete job` |

日志原文（照录，含 Node 进程号 `4337`）：

```
(node:4337) [DEP0040] DeprecationWarning: The `punycode` module is deprecated. Please use a userland alternative instead.
(Use `node --trace-deprecation ...` to show where the warning was created)
(node:4337) [DEP0169] DeprecationWarning: `url.parse()` behavior is not standardized and prone to errors that have security implications. Use the WHATWG URL API instead. CVEs are not issued for `url.parse()` vulnerabilities.
##[warning]Node.js 20 is deprecated. The following actions target Node.js 20 but are being forced to run on Node.js 24: actions/checkout@v4, actions/upload-artifact@v4. For more information see: https://github.blog/changelog/2025-09-19-deprecation-of-node-20-on-github-actions-runners/
```

说明：

- 前两条是 **Action 内部 Node.js 运行时的弃用提示**（`actions/upload-artifact@v4` 自身用了
  `punycode` 与 `url.parse()`），不涉及本仓库源码，也不影响上传结果（artifact 已成功上传并核对）。
- 第三条是**平台强制迁移**：`actions/checkout@v4` 与 `actions/upload-artifact@v4` 声明的是
  Node.js 20，被 GitHub 强制以 Node.js 24 运行。这是平台行为，不是版本不匹配错误。

**本轮处理方式**：不升级 Actions 版本、不修改 workflow。第 2 轮的范围只有「归档本轮 CI 实测结果」，
升级 `checkout` / `upload-artifact` / `setup-java` 属于独立变更，应单独提交、单独验证，不在归档轮里夹带。

> 与第 1 轮 §7.2 的关系：§7.2 记录的是 build.yml 的两条平台警告（Node.js 20 deprecated、
> `actions/setup-java@v4` 弃用）。本轮 rust.yml 未用 setup-java，因此没有第二条；
> 但 Node.js 20 deprecated 在本轮**依然出现**，说明它不是某一个 workflow 的问题，而是平台级的。
