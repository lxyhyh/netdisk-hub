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
| 第 3 轮 | UniFFI 绑定生成 + Kotlin 集成 | **已完成**，见 §11 |
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

## 11. 第 3 轮运行记录（UniFFI 绑定 + Kotlin 集成）

### 11.1 四次提交与运行

本轮共 4 次提交才达到验收目标，前两次失败，原因是构建链问题；
第 4 次是**验收加固**（RELRO 断言 + zipalign），不是修 bug。原因全部照录。

| # | Commit | 运行 ID | 结果 | 说明 |
|---|---|---|---|---|
| 1 | `ce09f860199646be9b9ebb2b8b71325b45bdde73` | `45949811572` | **failure** | Gradle 脚本编译错误，见 §11.2 |
| 2 | `1354f13` | `45950092700` | **failure** | `mergeDebugJniLibFolders` 隐式依赖错误，见 §11.3 |
| 3 | `4cc08f9` | `45950571670` | **success** | 构建链打通，见 §11.4 |
| 4 | `206f052` | `45952694106` | **success** | 验收加固后重新全绿，见 §11.13 |

commit message：

1. `build: T-002e 第 3 轮 Rust UniFFI 绑定 + Kotlin 集成（双 ABI）`
2. `fix: 修复第 3 轮 build workflow 首次运行的脚本编译错误`
3. `fix: 用变体 API 接入 jniLibs，消除 mergeDebugJniLibFolders 隐式依赖错误`
4. `fix: 按官方 16 KB 文档实现 GNU_RELRO 断言 + 增加 zipalign 验收`

同批 push 触发的 `rust` workflow（run `45949811566`）**success**，
说明 `--locked` 补上后第 2 轮的交叉编译没有回退。

### 11.2 第 1 次失败：Gradle 脚本编译错误

日志原文（`Build debug APK` 步骤）：

```
e: file:///home/runner/work/netdisk-hub/netdisk-hub/app/build.gradle.kts:67:31: Unresolved reference 'io'.
e: file:///home/runner/work/netdisk-hub/netdisk-hub/app/build.gradle.kts:101:32: Unresolved reference 'io'.
w: file:///home/runner/work/netdisk-hub/netdisk-hub/app/build.gradle.kts:227:17: 'fun srcDir(srcDir: Any): Any' is deprecated. Use `directories` mutable set instead.
```

原因：我在这之前把 `workingDir = File(...)` 改成了 `java.io.File(...)`，
而 Kotlin DSL 脚本里 `java` 会被解析成已有的扩展属性（`ExtensionAware.java` 之类），
不是 Java 包名，因此 `io` 解析不出来。

修法：脚本顶部显式 `import java.io.File`，正文写回裸 `File(...)`。

同一次还暴露第二个问题：`jniLibs.srcDir(Any)` 在 AGP 9.3.1 已废弃（上面那条 `w:`）。
AGP 源码 `AndroidSourceDirectorySet` 的注释写明改用 `directories: MutableSet<String>`。

### 11.3 第 2 次失败：`mergeDebugJniLibFolders` 隐式依赖

日志原文：

```
> Task :app:mergeDebugJniLibFolders FAILED

* What went wrong:
A problem was found with the configuration of task ':app:mergeDebugJniLibFolders' (type 'MergeSourceSetFolders').
Property has implicit dependency
  Gradle detected a problem with the following location: '.../app/build/rustJniLibs'
    Task ':app:mergeDebugJniLibFolders' uses this output of task ':app:cargoNdkBuild'
    without declaring an explicit or implicit dependency.
```

原因：我把 `cargoNdkBuild` 的输出目录**既**手动 `set` 成 `app/build/rustJniLibs`，
**又**只把该路径以字符串塞进 `sourceSets.jniLibs.directories`。
Gradle 9 的任务校验能看出这个目录是另一个任务的输出，但看不到依赖声明，于是报错。

修法（最终形态）：

- `cargoNdkBuild` 不再手动 `set outputDir`；
- 在 `androidComponents.onVariants` 里用
  `jniLibs.addGeneratedSourceDirectory(cargoNdkBuild, CargoNdkBuildTask::outputDir)` 接入，
  由 AGP 建立 `merge*JniLibFolders → cargoNdkBuild` 的依赖；
- 删掉 `sourceSets` 里的路径注入，以及 `tasks.matching { merge*NativeLibs }` 的 `dependsOn` 兜底。

副作用（需记录）：输出目录从 `app/build/rustJniLibs` 变成 AGP 约定的
`app/build/generated/jniLibs/cargoNdkBuild`（日志原文见 §11.5）。

### 11.4 第 3 次运行：全绿

| 项 | 实测值 |
|---|---|
| 运行链接 | https://github.com/lxyhyh/netdisk-hub/actions/runs/45950571670 |
| 运行 ID / workflow | `45950571670` / `build` |
| Head SHA | `4cc08f9` |
| 结果 | **success** |
| 触发 / 完成 | 2026-11-27T06:14:20Z / 2026-11-27T06:17:24Z |
| 总耗时 | 3 分 04 秒（构建步骤本身 1 分 46 秒） |

13 个步骤全部 success，逐步骤时间：

| # | 步骤 | 耗时区间（UTC） |
|---|---|---|
| 1 | Set up job | 06:14:23 → 06:14:25 |
| 2 | Checkout | 06:14:25 → 06:14:26 |
| 3 | Set up JDK 17 | 06:14:26 → 06:14:26 |
| 4 | Set up Gradle 9.7.0 | 06:14:26 → 06:14:46 |
| 5 | Install Android SDK platform 37, build-tools 36, NDK r30 | 06:14:46 → 06:14:54 |
| 6 | Install Rust 1.100.0 with Android targets | 06:14:54 → 06:15:04 |
| 7 | Install cargo-ndk 4.1.2 | 06:15:04 → 06:15:23 |
| 8 | Verify toolchain versions | 06:15:23 → 06:15:23 |
| 9 | Build debug APK | 06:15:23 → 06:17:09 |
| 10 | Verify UniFFI Kotlin bindings | 06:17:09 → 06:17:09 |
| 11 | Verify Kotlin bindings are compiled into the APK | 06:17:09 → 06:17:09 |
| 12 | Verify APK native libraries | 06:17:09 → 06:17:09 |
| 13 | Upload APK | 06:17:09 → 06:17:11 |

### 11.5 工具链实测（与第 2 轮一致，未换版本）

`Verify toolchain versions` 步骤日志原文：

```
sdkmanager = /usr/local/lib/android/sdk/cmdline-tools/latest/bin/sdkmanager
source.properties Pkg.Revision = 30.0.16248370
rustc: rustc 1.100.0 (0ef1bc386 2026-11-09)
cargo: cargo 1.100.0 (3acc79c0c 2026-11-05)
NDK  : 30.0.16248370 (/usr/local/lib/android/sdk/ndk/30.0.16248370)
cargo-ndk: cargo-ndk 4.1.2
```

`cargoNdkBuild` 任务打印的实际命令（日志原文）：

```
cargo-ndk: cargo ndk -t arm64-v8a -t x86_64 -P 31 -o .../app/build/generated/jniLibs/cargoNdkBuild --manifest-path core/Cargo.toml build --release --locked
```

cargo 编译耗时：arm64-v8a `Finished release profile [optimized] target(s) in 21.21s`，
x86_64 `Finished ... in 5.44s`；随后 `Copying libraries to .../app/build/generated/jniLibs/cargoNdkBuild`。
`generateUniffiBindingsDebug` 里的 `cargo build -p uniffi-bindgen` 耗时 `57.56s`（首次全量编译 UniFFI 依赖树）。

Gradle 汇总：`38 actionable tasks: 20 executed, 18 from cache`。
关键任务链：`cargoNdkBuild` → `generateUniffiBindingsDebug` → `compileDebugKotlin` →
`mergeDebugJniLibFolders` → `mergeDebugNativeLibs` → `packageDebug` → `assembleDebug`。

### 11.6 Kotlin 绑定生成（已验证）

`Verify UniFFI Kotlin bindings` 步骤日志原文：

```
bindings = app/build/generated/kotlin/generateUniffiBindingsDebug/com/netdiskhub/core/netdiskhub_core.kt
size = 40751 bytes
--- 前 8 行 ---
// This file was autogenerated by some hot garbage in the `uniffi` crate.
// Trust me, you don't want to mess with it!

@file:Suppress("NAME_SHADOWING")

package com.netdiskhub.core
```

该步骤的硬断言：文件存在、`size > 20000`、`package com.netdiskhub.core` 行存在、
`fun `probeMagic`` 存在、含 `com.sun.jna`。全部通过。

> 注意：本地（`rust/target/release/libnetdiskhub_core.so`）生成的同名文件也是 40751 字节，
> 即 CI 产物与本地实测一致。

### 11.7 绑定被编译进 dex（已验证）

`Verify Kotlin bindings are compiled into the APK` 步骤日志原文：

```
apk = app/build/outputs/apk/debug/app-debug.apk
-rw-r--r-- classes.dex  18573732
-rw-r--r-- classes2.dex    38940
-rw-r--r-- classes3.dex     7848
-rw-r--r-- classes4.dex    63620
-rw-r--r-- classes5.dex    50144
-rw-r--r-- classes6.dex 10132348
/tmp/apkdex/classes3.dex
/tmp/apkdex/classes4.dex
OK: dex 含 com.netdiskhub.core 与 probeMagic
/tmp/apkdex/classes3.dex:Netdiskhub_coreKt
/tmp/apkdex/classes4.dex:Netdiskhub_coreKt
```

即 `com.netdiskhub.core` 包与 `probeMagic` 都真实存在于 dex 里，
门面类 `Netdiskhub_coreKt` 同现于 `classes3.dex` 与 `classes4.dex`。

### 11.8 APK 原生库校验（脚本硬断言）

`scripts/verify-apk-native-libs.py` 的判据与结果：

**1) 清单严格比对**（多一条、少一条、路径错都失败）

| 条目 | 大小 | compress_type |
|---|---|---|
| `lib/arm64-v8a/libandroidx.graphics.path.so` | 10096 B | 0（STORED） |
| `lib/arm64-v8a/libjnidispatch.so` | 165992 B | 0（STORED） |
| `lib/arm64-v8a/libnetdiskhub_core.so` | 353896 B | 0（STORED） |
| `lib/x86_64/libandroidx.graphics.path.so` | 10760 B | 0（STORED） |
| `lib/x86_64/libjnidispatch.so` | 116904 B | 0（STORED） |
| `lib/x86_64/libnetdiskhub_core.so` | 371864 B | 0（STORED） |

**2) ELF 与 16 KB 对齐**（`PT_LOAD` 的 `p_align` 全部 `>= 0x4000`）

| 库 | Class / Machine | LOAD 段数 | 各段 align |
|---|---|---|---|
| `arm64-v8a/libandroidx.graphics.path.so` | ELF64 / EM_AARCH64(183) | 3 | 全 `0x4000` |
| `arm64-v8a/libjnidispatch.so` | ELF64 / EM_AARCH64(183) | 2 | 全 `0x4000` |
| `arm64-v8a/libnetdiskhub_core.so` | ELF64 / EM_AARCH64(183) | 4 | 全 `0x4000` |
| `x86_64/libandroidx.graphics.path.so` | ELF64 / EM_X86_64(62) | 3 | 全 `0x4000` |
| `x86_64/libjnidispatch.so` | ELF64 / EM_X86_64(62) | 2 | 全 `0x4000` |
| `x86_64/libnetdiskhub_core.so` | ELF64 / EM_X86_64(62) | 4 | 全 `0x4000` |

**3) Rust 导出符号**（两 ABI 一致，`已定义动态符号数=58`）

`netdiskhub_probe` / `uniffi_netdiskhub_core_checksum_func_probe_magic` /
`uniffi_netdiskhub_core_fn_func_probe_magic` 全部命中。

**4) ZIP 存储与页对齐**

```
OK  lib/arm64-v8a/libandroidx.graphics.path.so STORED data_offset=0x1b88000（16 KB 对齐）
OK  lib/arm64-v8a/libjnidispatch.so           STORED data_offset=0x1b8c000（16 KB 对齐）
OK  lib/arm64-v8a/libnetdiskhub_core.so       STORED data_offset=0x1bb8000（16 KB 对齐）
OK  lib/x86_64/libandroidx.graphics.path.so   STORED data_offset=0x1c10000（16 KB 对齐）
OK  lib/x86_64/libjnidispatch.so              STORED data_offset=0x1c14000（16 KB 对齐）
OK  lib/x86_64/libnetdiskhub_core.so          STORED data_offset=0x1c34000（16 KB 对齐）
```

最终输出：`PASS: 6 个原生库通过 清单 / ELF / PT_LOAD 16 KB 对齐 / GNU_RELRO 官方条件 / ZIP 页对齐 / 符号 检查`。

> **注意**：上面表格与 §11.4 对应的是**第 3 次运行**（commit `4cc08f9`）。
> 当时脚本对 RELRO 只输出 note、不判失败；该缺口已在 §11.13 补上，
> 并以第 4 次运行（commit `206f052`）重新验证。下表保持不变是为了保留时间线。

**当时对 `PT_GNU_RELRO` 的处理（已被 §11.13 取代）**

第 3 次运行的脚本只打印 RELRO 结束地址，**不作为失败条件**：

```
note  PT_GNU_RELRO vaddr=0x5ec0 memsz=0x1140 end=0x7000（16 KB 对齐：否，非判据，仅记录）   # x86_64/libandroidx.graphics.path.so
note  PT_GNU_RELRO vaddr=0x5f3e0 memsz=0x2c20 end=0x62000（16 KB 对齐：否，非判据，仅记录） # x86_64/libnetdiskhub_core.so
```

当时的理由是「RELRO 结束地址不对齐在 Google 自己的库上也存在」。
**这个理由是错的**：官方文档给出的不是「必须对齐」一个条件，而是三个并列条件，
不对齐时还要看 RELRO 是否占满其所属 LOAD 段的余部。详见 §11.13。

### 11.9 JNA 依赖与 Manifest 合并（ADR-004 验证）

ADR-004 要求：AAR 依赖解析、Manifest 合并、APK 原生库打包、16 KB 检查全部由 CI 验证。
逐项结果：

| 验证项 | 结果 | 证据 |
|---|---|---|
| AAR 依赖解析 | **通过** | `dependencies` 里 `net.java.dev.jna:jna:5.19.1@aar` 参与构建，产物出现 `libjnidispatch.so` 与 `com/sun/jna` 类 |
| Manifest 合并 | **通过，无警告** | 合并后 APK 的 `<uses-sdk>` 为 `minSdk=31 / targetSdk=37`（宿主值），JNA 的 `targetSdkVersion=25` 被忽略；**未使用任何 `tools:*`** |
| APK 原生库打包 | **通过** | §11.8 的 6 条清单，两 ABI 各 3 个库 |
| 16 KB 页面检查 | **通过** | §11.8 的 `PT_LOAD p_align` 与 ZIP 页对齐 |

`com/sun/jna` 在 dex 中的分布（本地解包 APK 实测）：`classes2.dex` 2 处、
`classes4.dex` 2 处、`classes6.dex` 6 处，共 10 处；其余 dex 0 处。

按用户明令：**不使用 `tools:overrideLibrary` 预先掩盖 Manifest 合并问题**。
本轮既未预先加，实测通过后仍保持不加。

### 11.10 Artifact

| 项 | 值 |
|---|---|
| Artifact 名 | `app-debug-apk` |
| Artifact ID | `15175884387` |
| zip 大小 | 10770223 B |
| Artifact digest | `sha256:7e4c6fed147bf1ae71b741c891e6673760d9f8dffaa1a7615f03f5c944320517` |
| 下载入口 | https://github.com/lxyhyh/netdisk-hub/actions/runs/45950571670/artifacts/15175884387 |

本地 `gh run download` 后的实测：

| 项 | 值 |
|---|---|
| 解压后 APK 路径 | `app-debug.apk`（无多余根目录） |
| APK 大小 | **30500633 B** |
| APK SHA256 | `f1679f5834f41f2d1771107649cd02b937d719ae8a7485d51f6ec3cbdbecfebf` |
| `aapt dump badging` | `package: name='com.netdiskhub.app' versionCode='1' versionName='0.1.0' compileSdkVersion='37'`；`sdkVersion:'31'`；`targetSdkVersion:'37'`；`application-label:'Netdisk Hub'` |
| `lib/` 条目 | 恰好 6 条（两 ABI × 3 库），与脚本清单一致 |

> Artifact digest 与 APK SHA256 **不可互校**：前者是对 zip 包的摘要，后者是对解压后 APK 的摘要。

### 11.11 本轮警告

Gradle 构建日志中**没有** `e:` 或 `w:` 级别的 Kotlin/Kotlin-DSL 警告（第 1 次的
`java.io.File` 与 `srcDir` 两项均已修掉）。

GitHub Actions 平台层警告 2 条，与构建无关：

| # | 警告 | 出现位置 |
|---|---|---|
| 1 | Node.js 20 已弃用，`actions/checkout@v4` / `actions/setup-java@v4` / `actions/upload-artifact@v4` / `gradle/actions/setup-gradle@v4` 被强制以 Node.js 24 运行 | `Complete job` |
| 2 | `actions/setup-java@v4` 已弃用，建议迁移到 `@v5` | `Set up JDK 17` |

**本轮处理方式**：不升级、不改 workflow。理由同 §10.8——升级 Actions 属于独立变更。

### 11.12 已验证 / 未验证

**已验证（直接证据）**

1. UniFFI 0.32.2 的 Kotlin 绑定生成在 CI 上可复现，产物 40751 B。
2. Kotlin 能编译绑定并调用 `probeMagic()`（`NativeProbe.kt` 编译通过，dex 内含该函数）。
3. Rust 双 ABI cdylib 打进 APK 的 `lib/arm64-v8a` 与 `lib/x86_64`，且导出符号齐全。
4. 6 个原生库全部满足 `PT_LOAD p_align >= 0x4000`，ZIP 内均 STORED 且数据偏移 16 KB 对齐。
5. JNA 5.19.1@aar 可解析、可打包，Manifest 合并无需 `tools:*`。
6. ABI 收敛生效：APK 内只有两个 ABI，JNA AAR 自带的其余 5 个 ABI 被裁掉。

**未验证 / 待确认**

1. **运行时端到端**：`probeMagic()` 的返回值（应为 65538）从未在设备或模拟器上实际执行。
   APK 的 `UniffiLib` 是否能在运行时成功 `Native.register`（即 `libjnidispatch.so` +
   `libnetdiskhub_core.so` 是否都能被 `System.loadLibrary` 找到）**未验证**。
   这属于 T-003 的范围。
2. **UniFFI contract checksum**：绑定里对 `probe_magic` 的 checksum 期望值是 `62777`
   （生成代码第 815 行 `(lib.uniffi_netdiskhub_core_checksum_func_probe_magic() and 0xFFFF) != 62777`）。
   该值是否在运行时真的匹配，只有跑起来才能验证。
3. `test` workflow 与 `quality-gate` workflow 仍未建立。
4. `release` 构建（含 R8/minify、`strip` 配置）未验证，本轮只跑 `debug`。

### 11.13 验收加固：GNU_RELRO 官方式断言 + zipalign（第 4 次运行）

#### 11.13.1 为什么要改

第 3 次运行（§11.8）对 `PT_GNU_RELRO` **只打印、不判定**，理由是
「RELRO 结束地址不对齐的情况在 Google 自己发布的 `androidx.graphics.path.so` 上也存在」。
这个理由站不住：官方文档
（https://developer.android.com/guide/practices/page-sizes ，Step 2: Verify 16 KB compliance for RELRO）
给的不是「必须对齐」单一条件，而是**三个并列条件，满足任意一条即合规**：

| 条件 | 官方原文要点 |
| A | program header 里没有 `GNU_RELRO` 段 |
| B | `(RELRO.VirtAddr + RELRO.MemSiz) % 0x4000 == 0` |
| C | RELRO 之后在同一 LOAD 段内没有可写数据（`LOAD.VirtAddr + LOAD.MemSiz == RELRO.VirtAddr + RELRO.MemSiz`） |

因此「结束地址不对齐」既不构成失败、也不构成免检理由，必须继续按 C 判定。
上游是 Google 的库不构成豁免。

#### 11.13.2 脚本实现（`scripts/verify-apk-native-libs.py`）

新增 `Elf.check_relro()`，顺序判定 A → B → C1 → C2：

- **A**：`self.relro is None` → 合规。
- **B**：`rend % 0x4000 == 0` → 合规。
- **C1**（官方原文等式）：存在某个 `PT_LOAD` 使 `LOAD.VirtAddr + LOAD.MemSiz == RELRO.VirtAddr + RELRO.MemSiz` → 合规。
- **C2**（C 的实质化，对应官方 Example 1 / Example 4 的判别依据）：把 RELRO 覆盖范围
  按 16 KB 向下/向上取整成页区间 `[floor(vaddr,16K), ceil(rend,16K))`，该区间内 RELRO 之外
  不得存在任何来自 `PF_W` 段的可写字节。因为运行时 linker 会按 16 KB 页边界取整 RELRO 保护范围，
  区间内 RELRO 之外的可写数据会被误置只读 —— 这正是官方描述的 SIGSEGV 场景。
- **判定不了就失败**：若 RELRO 起始地址不落在任何 `PT_LOAD` 内，直接报
  「无法判定其与 LOAD 段的关系（按规则不得静默通过）」，不静默放过。

同时新增 `--selftest`：用合成 ELF 把官方文档 Example 1–4 逐条回归，另加一条
「RELRO 不落在任何 LOAD 段内」的负例，确保实现与官方示例一一对应。
CI 里 `--selftest` 在检查真 APK **之前**执行，实现退化会先被拦下。

#### 11.13.3 6 个库的 RELRO 实测判定（第 4 次运行日志原文）

| 库 | RELRO vaddr / memsz / end | B | C1 | C2 | 结论 |
|---|---|---|---|---|---|
| `arm64-v8a/libandroidx.graphics.path.so` | `0x5c40` / `0x3c0` / `0x6000` | 否（余 `0x2000`） | 否 | 满足：页区间 `[0x4000,0x8000)` 内 RELRO 外无可写字节 | **合规** |
| `arm64-v8a/libjnidispatch.so` | `0x2bb98` / `0x468` / `0x2c000` | **是** | — | — | **合规** |
| `arm64-v8a/libnetdiskhub_core.so` | `0x5b4e0` / `0x2b20` / `0x5e000` | 否（余 `0x2000`） | **是**：`LOAD#3 [0x5b4e0,0x5e000)` 结束地址相等 | — | **合规** |
| `x86_64/libandroidx.graphics.path.so` | `0x5ec0` / `0x1140` / `0x7000` | 否（余 `0x3000`） | 否 | 满足：页区间 `[0x4000,0x8000)` 内 RELRO 外无可写字节 | **合规** |
| `x86_64/libjnidispatch.so` | `0x1fb00` / `0x500` / `0x20000` | **是** | — | — | **合规** |
| `x86_64/libnetdiskhub_core.so` | `0x5f3e0` / `0x2c20` / `0x62000` | 否（余 `0x2000`） | **是**：`LOAD#3 [0x5f3e0,0x62000)` 结束地址相等 | — | **合规** |

关键两点（与第 3 次运行的误判对照）：

1. **两个 `libandroidx.graphics.path.so` 是 C2 合规，不是「豁免」**。
   arm64 版 RELRO `[0x5c40, 0x6000)` 落在页区间 `[0x4000, 0x8000)` 内，
   另一个可写段 `LOAD#3 [0x9fd0, 0x9fe0)` 在区间外 → 页内无可写数据。
   x86_64 版 RELRO `[0x5ec0, 0x7000)`，另一可写段 `LOAD#3 [0xa268, 0xa278)` 同样在区间外。
   这是按官方规则算出来的结论，不是因为它是 Google 的库。
2. **两个 `libnetdiskhub_core.so` 是 C1 合规**：RELRO 的 `memsz` 恰好等于所在 `LOAD#3` 的
   余部（`0x5e000 - 0x5b4e0 = 0x2b20`，`0x62000 - 0x5f3e0 = 0x2c20`），
   即 RELRO 占满该段余部，符合官方 Example 4 的形态。

`--selftest` 日志原文：

```
selftest 通过：5 个官方示例判定一致
```

#### 11.13.4 官方 zipalign 验收

`build.yml` 在 `Verify APK native libraries` 之后新增独立步骤
`Verify APK zipalign 16 KB page alignment`，执行官方文档给出的命令：

```
$ANDROID_HOME/build-tools/36.0.0/zipalign -v -c -P 16 4 $APK
```

该步骤：先 `test -x` 确认二进制存在；命令必须真实执行（用 `set +e` 捕获退出码后
再显式判断，**不吞掉失败**）；退出码非 0 则 `::error::` + `exit 1`；
输出里必须出现 `Verification successful`，否则同样失败。

日志原文（第 4 次运行，节选）：

```
zipalign = /usr/local/lib/android/sdk/build-tools/36.0.0/zipalign
apk      = app/build/outputs/apk/debug/app-debug.apk
Verifying alignment of app/build/outputs/apk/debug/app-debug.apk (4)...
28868608 lib/arm64-v8a/libandroidx.graphics.path.so (OK)
28884992 lib/arm64-v8a/libjnidispatch.so (OK)
29065216 lib/arm64-v8a/libnetdiskhub_core.so (OK)
29425664 lib/x86_64/libandroidx.graphics.path.so (OK)
29442048 lib/x86_64/libjnidispatch.so (OK)
29573120 lib/x86_64/libnetdiskhub_core.so (OK)
Verification successful
OK: zipalign 报告 Verification successful
```

python 的 ZIP 偏移检查**保留不删**，两者互为交叉验证：
python 自己算本地文件头偏移，zipalign 用官方实现算 —— 两边都过才算过。

> 本机（aarch64 proot）仍无法直接执行该 zipalign：`build-tools/*/zipalign` 是 x86_64 ELF，
> `/usr/bin/zipalign` 是 AOSP 版但不支持 `-P 16`。所以本地只能用桩程序验证步骤的三种分支逻辑
> （通过 / 退出码失败 / 缺 Verification successful），真实执行结果以 CI 日志为准。

#### 11.13.5 第 4 次运行结果（run 45952694106）

| 项 | 实测值 |
|---|---|
| 运行链接 | https://github.com/lxyhyh/netdisk-hub/actions/runs/45952694106 |
| 运行 ID / workflow | `45952694106` / `build` |
| Head SHA | `206f0521e907098c69b2a48465249c0666e3cc10`（短 `206f052`） |
| 结果 | **success** |
| 触发 / 完成 | 2026-11-27T06:34:11Z / 2026-11-27T06:37:11Z |
| 总耗时 | 3 分 00 秒 |
| 步骤 | 14 个业务步骤 + 4 个 Post/Complete，全部 success |

关键步骤耗时（UTC）：

| 步骤 | 区间 |
|---|---|
| Install Android SDK platform 37, build-tools 36, NDK r30 | 06:34:33 → 06:34:43 |
| Install Rust 1.100.0 with Android targets | 06:34:43 → 06:34:54 |
| Install cargo-ndk 4.1.2 | 06:34:54 → 06:35:16 |
| Build debug APK | 06:35:16 → 06:37:01 |
| Verify UniFFI Kotlin bindings | 06:37:01 → 06:37:01 |
| Verify Kotlin bindings are compiled into the APK | 06:37:01 → 06:37:01 |
| **Verify APK native libraries**（含 `--selftest`） | 06:37:01 → 06:37:02 |
| **Verify APK zipalign 16 KB page alignment** | 06:37:02 → 06:37:02 |
| Upload APK | 06:37:02 → 06:37:03 |

Artifact：

| 项 | 值 |
|---|---|
| Artifact 名 | `app-debug-apk` |
| Artifact ID | `15177950350` |
| zip 大小 | 10770227 B |
| Artifact digest | `sha256:a6f6b9e391952ecfb3c24d7dd4d1568371e32fe6b1940cc427f8e30e2ec1614a` |
| 下载入口 | https://github.com/lxyhyh/netdisk-hub/actions/runs/45952694106/artifacts/15177950350 |

本地 `gh run download` 后实测：APK **30500633 B**，
SHA256 `725dca77c34e85d6458c65e58bc7699aa5c2af0a74d229ecbb2aaa88077e03ab`；
`aapt dump badging` 为 `com.netdiskhub.app` / `versionCode 1` / `0.1.0` /
`compileSdkVersion 37` / `sdkVersion 31` / `targetSdkVersion 37`。

#### 11.13.6 本次未改动的部分

按验收要求「不重新设计构建链、不修改已通过的其他部分」，本次改动只有两处：

| 文件 | 改动 |
|---|---|
| `scripts/verify-apk-native-libs.py` | 新增 `check_relro()` + `--selftest`；删除原 `relro_note()`；ZIP / 清单 / 符号 / `PT_LOAD` 判据不动 |
| `.github/workflows/build.yml` | ①`Verify APK native libraries` 里加一行 `--selftest`；②新增 `Verify APK zipalign 16 KB page alignment` 步骤 |

Rust 版本、cargo-ndk、NDK、Gradle、AGP、Kotlin、JNA 版本与 `app/build.gradle.kts` **全部未动**。

#### 11.13.7 第 4 次运行的六个库完整判定输出

```
-- lib/arm64-v8a/libandroidx.graphics.path.so
   note  PT_GNU_RELRO vaddr=0x5c40 memsz=0x3c0 filesz=0x390 end=0x6000
   note  官方条件 B 不满足：0x6000 % 0x4000 = 0x2000 ≠ 0
   note  所在 LOAD 段：#2 [0x5c40, 0x5fd0)
   note  官方条件 C1 不满足：没有任何所在 LOAD 段在 RELRO 结束处结束
   note  官方条件 C2 满足：页区间 [0x4000, 0x8000) 内 RELRO 之外 没有任何可写字节 → 合规
   OK    GNU_RELRO 满足官方 16 KB 条件
-- lib/arm64-v8a/libjnidispatch.so
   note  官方条件 B：(0x2bb98 + 0x468) % 0x4000 == 0 → 合规（结束地址 16 KB 对齐）
   OK    GNU_RELRO 满足官方 16 KB 条件
-- lib/arm64-v8a/libnetdiskhub_core.so
   note  官方条件 C1：LOAD#3 的结束地址 0x5e000 == RELRO 结束地址 0x5e000，RELRO 占满该段余部 → 合规
   OK    GNU_RELRO 满足官方 16 KB 条件
-- lib/x86_64/libandroidx.graphics.path.so
   note  官方条件 B 不满足：0x7000 % 0x4000 = 0x3000 ≠ 0
   note  官方条件 C2 满足：页区间 [0x4000, 0x8000) 内 RELRO 之外 没有任何可写字节 → 合规
   OK    GNU_RELRO 满足官方 16 KB 条件
-- lib/x86_64/libjnidispatch.so
   note  官方条件 B：(0x1fb00 + 0x500) % 0x4000 == 0 → 合规（结束地址 16 KB 对齐）
   OK    GNU_RELRO 满足官方 16 KB 条件
-- lib/x86_64/libnetdiskhub_core.so
   note  官方条件 C1：LOAD#3 的结束地址 0x62000 == RELRO 结束地址 0x62000，RELRO 占满该段余部 → 合规
   OK    GNU_RELRO 满足官方 16 KB 条件
```

最终汇总行：

```
PASS: 6 个原生库通过 清单 / ELF / PT_LOAD 16 KB 对齐 / GNU_RELRO 官方条件 / ZIP 页对齐 / 符号 检查
```