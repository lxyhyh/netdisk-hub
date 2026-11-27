# CI 环境记录（ci-setup）

- 文档状态：T-002e 执行中（第 1 轮已归档；第 2、3 轮未开始）
- 最后更新：2026-11-27
- 性质：本文件是 CI runner 环境的**唯一基准**（对应 RISKS.md R24），不写计划，只写实测结果

## 1. 概述

本地开发机为 Android proot Ubuntu，没有 Android SDK / NDK / Gradle 完整环境，**本地无法编译**。
所有编译、构建、测试、打包一律通过 GitHub Actions 完成，CI 结果（日志 + artifact）是唯一验证依据。

T-002e 分三轮，每轮只增加一个变量：

| 轮次 | 内容 | 状态 |
|---|---|---|
| 第 1 轮 | 最小 Android 工程 + APK 打包（不含 Rust） | **已完成**（commit `0773aad`） |
| 第 2 轮 | Rust 交叉编译独立 workflow（cargo-ndk → arm64-v8a / x86_64 .so） | 未开始 |
| 第 3 轮 | UniFFI 绑定生成 + Kotlin 集成 | 未开始 |

三轮之后才补 quality-gate 占位。

## 2. workflow 清单

| 文件 | workflow name | 触发 | 本轮状态 |
|---|---|---|---|
| `.github/workflows/build.yml` | `build` | push / pull_request（master）+ workflow_dispatch | 已跑通 |

第 1 轮**只有** `build.yml`，没有 test / quality-gate workflow（按 T-002e 分轮方案，第 1 轮不强制单元测试与 lint）。

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

### 8.2 未验证 / 待确认

- KGP 精确版本号 2.4.21 未在 CI 日志中明文打印（见 §6.3）
- Rust 交叉编译
- UniFFI 绑定生成
- Kotlin ↔ Rust 调用
- APK 真实设备安装和运行
- 单元测试、lint、CRAP、变异测试（第 1 轮按分轮方案不要求）

## 9. 后续轮次

| 轮次 | 目标 | 验收 |
|---|---|---|
| 第 2 轮 | `rust/` + `.github/workflows/rust.yml`，cargo-ndk 产出 arm64-v8a 与 x86_64 的 `.so` 并上传 artifact | 两个 ABI 的 `.so` 可下载；不含 UniFFI / Kotlin 集成 |
| 第 3 轮 | UniFFI 绑定生成 + Kotlin 集成 | APK 含 `.so` 与 Kotlin 绑定，Kotlin 能调用 Rust 导出接口 |
| 收尾 | `quality-gate.yml` 占位（仅 workflow_dispatch + PR 触发 + 注释） | 工具等 T-004 定稿后接入 |

每轮完成后在本文件追加对应小节，不攒到最后。
