# ADR-005：引入 AndroidX Test（instrumentation）用于 T-003 的 Android 运行时验证

- 状态：**已批准（2026-11-28 用户批准）**
- 日期：2026-11-28
- 关联任务：T-003（UniFFI 构建链最小化集成验证）、T-103（Kotlin UseCase 层）
- 影响范围：app 模块的 `androidTestImplementation` 依赖、`defaultConfig.testInstrumentationRunner`、`.github/workflows/build.yml` 新增一个 instrumentation 测试 step

## 背景

T-003 要求「必须验证真实的 Android ABI 原生库加载和运行，不能使用宿主机 Rust 测试代替」，
并要求覆盖 APK 加载 `libnetdiskhub_core.so` 与 `libjnidispatch.so`、`probeMagic()` 返回 `0x00010002`、
`providerIds()` 返回两个 id、登录拿到不透明 Session、九项文件操作、跨 UniFFI 边界的错误传播、
以及「运行过程不把密码/验证码/ticket/会话写入日志」。

这些断言只有一种可靠做法：把测试跑在真实 Android 运行时里（instrumentation），
让 APK 的 dex 通过 JNA 加载 APK 内 x86_64 / arm64-v8a 的 `.so` 并实际执行 FFI 调用。
当前工程的 `app/src/androidTest` 目录**不存在**，`defaultConfig` 里也没有 `testInstrumentationRunner`，
`dependencies` 里没有任何 instrumentation 依赖。

## 决策

| 项 | 值 |
| --- | --- |
| 测试框架 | AndroidX Test + JUnit4 instrumentation（`androidx.test.runner.AndroidJUnitRunner`） |
| `androidx.test:runner` | 1.7.0（稳定版） |
| `androidx.test.ext:junit` | 1.3.0（稳定版） |
| `androidx.test:core` | 1.7.0（稳定版） |
| 断言库 | JUnit4 内置 `org.junit.Assert`（不自带任何第三方断言库） |
| 运行方式 | 仅 CI：workflow 内用官方 `sdkmanager` / `avdmanager` / `emulator` / `adb` 启 x86_64 模拟器，再跑 `gradle :app:connectedDebugAndroidTest`（不使用第三方 Action） |
| ABI 覆盖 | 模拟器为 x86_64，APK 内同时含 arm64-v8a 与 x86_64，实际加载 x86_64 那份 |

版本来自 `https://dl.google.com/dl/android/maven2/**/maven-metadata.xml` 实测（2026-11-28）：
`androidx.test:runner` 最新稳定 1.7.0（`release` 字段是 1.8.0-alpha04，不用 alpha），
`androidx.test.ext:junit` 最新稳定 1.3.0，`androidx.test:core` 最新稳定 1.7.0。

## 理由

### 1. 为什么不用「宿主机 JVM 测试 + JNA」代替

宿主机 JVM 上也能 `Native.register` 一个 `.so`，但那样验证不到三件事：

1. `.so` 是否真的被打进了 APK（`lib/<abi>/`）；
2. Android 的 `System.loadLibrary` / JNA 的 `Native.register` 在 Android 上能否找到该库（依赖 `libjnidispatch.so` 与 `NativeLibrary` 搜索路径）；
3. Android 运行时对 16 KB 页、API 31+ 的 `unsatisfiedLinkError` 行为。

用户指令明确写「不能用宿主机 Rust 测试代替」，宿主机 JVM 测试同样属于代替方案。

### 2. 为什么用 AndroidX Test 而不是自研 Activity/Instrumentation

AndroidX Test 是 Google 官方 instrumentation 入口，`AndroidJUnitRunner` 已经是 `adb shell am instrument` 的标准封装，
不需要引入 Robolectric（它不是真实运行时）、不需要 UI 自动化框架（Espresso 与本任务无关）。
依赖面收敛到三个官方 artifact，且全部是稳定版。

### 3. 为什么不引入 third-party assertion / mocking

本任务的断言都是「等值比较 + 异常类型判断」，JUnit4 的 `assertEquals` / `assertThrows` 足够。
不引入 MockK / Mockito 是因为 UseCase 层的可测性来自构造注入与真实 MockProvider，不需要 mock 框架。

### 4. 为什么 CI 用 x86_64 模拟器

GitHub Actions 的 hosted runner 是 x86_64，KVM 可用；`system-images;android-37.0;google_apis;x86_64`
在 `dl.google.com/android/repository/sys-img/google_apis/sys-img2-3.xml` 中有实测条目（2143899425 字节），
同页也有 `google_apis_ps16k;x86_64`（2081411720 字节，16 KB 页镜像）。

## 备选方案与被否决理由

| 方案 | 结论 | 理由 |
| --- | --- | --- |
| 只跑宿主机 `cargo test` | 否决 | 用户明确禁止用宿主机测试代替；验证不到 APK 内原生库加载 |
| 宿主机 JVM + JNA 直接加载 `rust/target/.../libnetdiskhub_core.so` | 否决 | 同上，且 JNA 在非 Android JVM 与 Android 上的库搜索路径不同 |
| Robolectric 单元测试 | 否决 | 不加载真实 `.so`，`Native.register` 会失败或被打桩，无法验证 10 项 |
| 自研 `Instrumentation` 子类 + 手写 `am instrument` | 否决 | 重复造 AndroidJUnitRunner，维护成本高 |
| Espresso / Compose UI Test | 否决 | T-003 不验证 UI，T-103 只验证 UseCase 层逻辑 |

## 影响

### 新增依赖（全部为 `androidTestImplementation`，不进 release APK）

```kotlin
androidTestImplementation("androidx.test:runner:1.7.0")
androidTestImplementation("androidx.test.ext:junit:1.3.0")
androidTestImplementation("androidx.test:core:1.7.0")
```

三者都只进入 debug 的 androidTest 变体，**不改变 APK 的原生库清单与 dex 内容断言**。

### `defaultConfig` 新增一行

```kotlin
testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
```

### CI 新增

- 五个新 step：`Enable KVM access`（校验并放开 `/dev/kvm`）、`Install emulator and x86_64 system image`（`system-images;android-37.0;google_apis;x86_64`）、`Create AVD`、`Run Android instrumented tests on emulator`（后台起 `emulator -no-window`，轮询 `sys.boot_completed` 后跑 `gradle :app:connectedDebugAndroidTest --no-daemon --stacktrace`）、`Verify instrumented test results`。
- 上传 `app/build/reports/androidTests/` 与 logcat 作为 artifact（失败时用于定位）。

### 风险

1. **模拟器启动耗时**：首次约 3–6 分钟，冷启动可能触发 runner 超时，需要设置 `timeout-minutes`。
2. **必须自己启动模拟器**：本实现刻意不使用任何第三方 Action，全部用官方命令行工具（`sdkmanager` / `avdmanager` / `emulator` / `adb`）。代价是要自己处理启动等待（轮询 `sys.boot_completed`，不能只靠 `adb wait-for-device`，它在 init 阶段就会返回）。
3. **API 37 镜像的稳定性**：`android-37.0` 是当前最新 API，模拟器镜像可能比 36 慢或不稳。若 CI 反复失败，退到 `api-level: 36`（APK 的 `minSdk 31 / targetSdk 37` 在 API 36 上可安装运行，只是不会验证 API 37 特有行为）。

## 批准结论

- 2026-11-28 用户明确批准引入上述三个 `androidTestImplementation` 依赖。
- 用户同时批准继续完成 T-002e 第 3 轮之后的工作；本 ADR 采纳「不用第三方 Action、纯官方命令行工具启动模拟器」这一路径，因此 CI 不引入任何额外的 Action 依赖。
