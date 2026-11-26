# ADR-003：工具链版本锁定与 compileSdk 37

- 状态：**已采纳**
- 日期：2026-11-26
- 关联任务：T-001
- 取代：本 ADR 的 v0.1 提案版本（compileSdk 36 / NDK r29 待确认）

## 决策

| 项 | 值 |
|---|---|
| compileSdk | **37** |
| targetSdk | **37** |
| AGP | **9.3.1** |
| Gradle | **9.7.0** |
| Kotlin（KGP） | **2.4.21** |
| JDK | **17** |
| Build Tools | **36.0.0** |
| NDK | **r30**，`ndkVersion = "30.0.16248370"` |
| Rust | **1.100.0** |
| cargo-ndk | **4.1.2** |
| UniFFI | **0.32.2** |

## 理由

- API 37 已于 2026 年稳定，AGP 9.3.1 支持
- Kotlin 官方兼容表确认 Gradle 9.7.0 / AGP 9.3.1 均在完全支持范围内

## 回退策略

**仅当实际构建确认是 Gradle 版本问题时**才启用：

- 第一回退：**Gradle 9.6.1**（满足 AGP 最低要求，在 Kotlin 完全支持范围内）
- **禁止回退到 Gradle 9.5.0 以下**
- **禁止回退到 Gradle 9.3.1**（低于 AGP 9.3.1 最低要求）

## 明确不在本 ADR 范围内

- UniFFI 集成验证（见 TASKS.md **T-003**）
- CRAP / pitest 与新工具链的兼容性复核（见 TASKS.md **T-004**）
