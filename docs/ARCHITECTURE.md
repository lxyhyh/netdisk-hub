# 架构（ARCHITECTURE）

- 状态：草案 v0.3
- 产出任务：T-001
- 最后更新：2026-11-26（工具链基线已锁定，见 ADR-003）

## 1. Android 分层

```
Compose (UI)
    ↓ UiState / Intent
ViewModel (Kotlin)
    ↓
UseCase (Kotlin)
    ↓
UniFFI interface        ← 跨语言边界，只在这里
    ↓
Rust 核心
    ├── core/          统一模型、调度、错误
    └── providers/     各网盘 Adapter（签名/指纹/分片/验证码全在这层）
```

依赖方向单向：UI 不碰网盘细节，Kotlin 不知道任何网盘特有参数。

## 2. Provider 深模块草图

### UniFFI interface（必须小、通用）

```rust
// 通用操作，不含任何网盘特有概念
interface NetdiskProvider {
    Session connect(AuthInput auth);
    Page list(Session s, RemotePath path, ListOptions opts);
    void download(Session s, RemotePath from, LocalPath to, TransferOpts opts);
    void upload(Session s, LocalPath from, RemotePath to, TransferOpts opts);
    void remove(Session s, RemotePath path);
    void rename(Session s, RemotePath from, RemotePath to);
    void mkdir(Session s, RemotePath path);
    Quota quota(Session s);
}

interface Session { /* 不透明句柄，不暴露 token */ }
interface RemoteFile { name, size, modified, isDir, id }
interface AuthInput { /* 账号密码 / 短信验证码 / 扫码 三种形态 */ }
```

### 明确不进 interface 的东西

网盘特有的一切都**藏在 Rust Adapter 内部**，Kotlin 侧完全不可见：

- 签名参数、签名算法
- 设备指纹、UA、版本号、包名模拟
- TLS 指纹
- 分片协议、分片大小、并发数细节
- 验证码流程、滑块、短信风控
- 各网盘专有的 API 路径、header、字段名
- token 刷新细节
- 秒传 / 离线下载 / 分享等单网盘独有能力

### 真 seam 判定

**至少两个 Adapter 实现后才算验证通过。** 只有一个实现时，接口不算验证过。

## 3. Rust Adapter 组织

```
rust/
├── core/
│   ├── model.rs        统一模型（RemoteFile / RemotePath / Session）
│   ├── error.rs        统一错误分类
│   ├── queue.rs        任务队列
│   └── ffi.rs          UniFFI 导出
└── providers/
    ├── common/
    │   ├── http.rs     HTTP 客户端抽象
    │   ├── client_env.rs  客户端环境模拟（UA/版本/指纹/签名）
    │   └── auth_store.rs  token 交给 Kotlin 存 Keystore
    ├── tianyi/
    ├── nutstore/
    ├── baidu/
    └── ...
```

每个 provider 目录内：auth / list / download / upload / sign / env。

## 4. 登录相关模块归属

- **登录流程编排**：Kotlin ViewModel + UseCase
- **登录协议实现**：Rust Adapter 内部 `auth` 模块
- **客户端环境模拟**（UA、版本号、包名、设备指纹、签名、TLS 指纹）：
  Rust Adapter 内部 `client_env` 模块
- **token 存储**：Android Keystore，Kotlin 侧封装，**Rust 侧不持有持久化 token**
- **UniFFI interface 只暴露** `connect(auth: AuthInput): Session` 这类通用方法，
  **不暴露任何 OAuth、WebView、cookie 相关方法**
- **Kotlin 层不知道**任何网盘特有的登录参数、签名、设备指纹逻辑

## 5. 工程基线

平台：Android 原生，minSdk = 31（Android 12），compileSdk = 37，targetSdk = 37

语言：Kotlin 2.4.21（UI/编排） + Rust 1.100.0（核心逻辑）

桥接：UniFFI 0.32.2，通过自定义 Gradle Exec 任务调用 uniffi-bindgen 生成 Kotlin 绑定
  - 不使用 Gobley，改用 cargo-ndk 4.1.2 做 Rust 交叉编译

构建：Gradle 9.7.0（Kotlin DSL） + AGP 9.3.1 + JDK 17 + Build Tools 36.0.0 + NDK r30
  - ndkVersion 必须显式设为 "30.0.16248370"，不依赖 AGP 默认值
  - AGP 9.3.1 默认启用内置 Kotlin，必须按内置 Kotlin 机制显式固定 KGP 2.4.21
  - 不得重复应用 org.jetbrains.kotlin.android

Kotlin 测试：JUnit 5.14.x + JaCoCo 0.8.14

Rust 测试：cargo test / cargo nextest

CRAP：cargo-crap 0.10.0（Rust 侧）+ Kotlin 侧待 ADR-001 定稿

Kotlin 变异：pl.droidsonroids.pitest 0.3.0（ADR-002）

Rust 变异：cargo-mutants 27.1.0

UI：Jetpack Compose + Material 3

## 6. 版本兼容性

版本兼容性声明（已锁定）：

AGP 9.3.1 要求 Gradle ≥ 9.5.0，当前 Gradle 9.7.0 满足。

Kotlin 官方兼容性表明确支持：
- KGP 2.4.21：Gradle 7.6.3～9.7.0
- KGP 2.4.21：AGP 8.5.2～9.3.1

因此 AGP 9.3.1 + Gradle 9.7.0 + Kotlin 2.4.21 属于官方完全支持组合。
文档中不得再出现"Kotlin/Gradle/AGP 兼容性待验证"类表述。

UniFFI 0.32.2 没有官方 Gradle 版本兼容矩阵。
UniFFI 集成验证属于阶段1前置任务，不是版本选型待办，见 TASKS.md T-003。

NDK r30 需在 build.gradle.kts 中显式锁定，不依赖 AGP 默认值。

结论：本基线视为已锁定。UniFFI 集成验证属于阶段1前置任务，不是版本选型待办。

## 7. 构建链

构建链组成：
- Gradle 9.7.0 + AGP 9.3.1 + JDK 17 + Build Tools 36.0.0
- cargo-ndk 4.1.2 交叉编译 Rust 到 Android ABI
- 自定义 Gradle Exec 任务调用 uniffi-bindgen 0.32.2 生成 Kotlin 绑定
- 不用 Gobley，不用第三方 UniFFI Gradle 插件
- KGP 2.4.21 按 AGP 内置 Kotlin 机制固定，不重复应用 kotlin.android 插件

### 7.1 支持的 ABI（明确声明）

**当前原生核心只支持两个 ABI：`arm64-v8a` 与 `x86_64`。其余 ABI 不支持、不承诺。**

| 项 | 值 |
|---|---|
| 支持 | `arm64-v8a`（`aarch64-linux-android`）、`x86_64`（`x86_64-linux-android`） |
| 不支持 | `armeabi`、`armeabi-v7a`、`mips`、`mips64`、`x86` |
| 声明位置 | `app/build.gradle.kts` 的 `defaultConfig.ndk { abiFilters += listOf("arm64-v8a","x86_64") }` |
| 依据 | ADR-004 批准 JNA 5.19.1@aar 时同步批准 ABI 收敛 |
| 校验 | CI 的 `scripts/verify-apk-native-libs.py` 做 APK 内 `lib/` 条目严格清单比对，多一条即失败 |

影响：JNA AAR 自带 7 个 ABI 的 `libjnidispatch.so`，其中 5 个会被 `abiFilters` 裁掉，
APK 内只剩两个 ABI 各 3 个库。若日后要支持 32 位设备，需先改这里与 ADR-004 的决策。

## 8. 质量门禁

| 门禁项 | 阶段 1 | 阶段 2 | 阶段 3+ |
|---|---|---|---|
| CRAP | ≤6（CI 硬门禁） | ≤6 | 目标 ≤4，CI 硬门禁 ≤4 |
| Kotlin 变异 | 不强制 | 核心模块 ≥90% | Provider 核心 100%，等价变异体显式抑制 |
| Rust 变异 | 不强制 | 核心模块 ≥90% | Provider 核心 100%，CI 对未 triage 存活体报错 |
| lint/typecheck/test | 全绿 | 全绿 | 全绿 |

**100% 变异覆盖率不是阶段 1 硬门禁。**

## 9. CI-first 开发约束

开发机不具备本地编译条件，所有编译、构建、测试、打包必须在 GitHub Actions 完成。
本地只做：代码编写、静态分析、Git 操作、文档编辑。

影响：
- 任何需要验证编译的任务（包括 T-003 UniFFI 集成验证、T-004 工具兼容性复核、
  所有测试、CRAP 门禁、变异测试），都在 CI 上执行
- 本地不可完成的验证，不得写成"已本地验证"
- agent 每完成一批改动即 push，等 CI 反馈，不假设改动正确

CI 基础设施要求：
- GitHub Actions workflow 至少覆盖：Kotlin 编译、Rust 交叉编译、UniFFI 绑定生成、
  APK 打包、单元测试、CRAP 门禁、变异测试
- Ubuntu runner 需安装：JDK 17、Android SDK（compileSdk 37 / Build Tools 36.0.0）、
  NDK r30、Rust 1.100.0、cargo-ndk 4.1.2
- workflow 分档：
  - 日常 push：build + test
  - PR / 手动触发：quality-gate（CRAP + 变异测试）
  - 文档改动：paths 过滤，不触发 build
- workflow 产物（APK / AAB / 测试报告 / CRAP 报告 / 变异报告）可下载

CI 验证结果作为唯一编译验证依据。本地无法编译不得作为"跳过验证"的理由。
