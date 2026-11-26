# ADR-002：Kotlin 变异测试插件版本

- 状态：**提案（待 T-004 复核兼容性）**
- 日期：2026-11-26
- 关联任务：T-001

## 背景

T-001 任务卡指定 Kotlin 变异测试插件为 `pl.droidsonroids.pitest 0.2.25`。

**事实核查结果（2026-11-26）：该版本不存在。**

Maven Central 坐标 `pl/droidsonroids/pitest/pl.droidsonroids.pitest.gradle.plugin`
下**全部已发布版本**只有：

| 版本 | 状态 |
|---|---|
| 0.2.10 | 存在 |
| 0.2.11 | 存在 |
| 0.2.12 | 存在 |
| **0.3.0** | 存在（最新） |
| 0.2.25 | **不存在** |

另：**Gradle 插件标记坐标是 `pl.droidsonroids.pitest`（无 `.gradle.plugin` 后缀出现在
buildscript 声明里）**，实际写法为：

```kotlin
plugins {
    id("pl.droidsonroids.pitest") version "0.3.0"
}
```

## 决策（提案）

**采用 `pl.droidsonroids.pitest 0.3.0`（当前最新稳定版）。**

理由：

1. 0.2.25 从未发布，无法使用
2. 0.3.0 是既有版本中最新且跨大版本的一版，包含对较新 AGP/PIT 的适配
3. 不自行降级到 0.2.12（更旧，且与 Kotlin 2.3 / AGP 8.13 的兼容性风险更高）

## 影响

- 阶段 2：核心模块变异覆盖率 ≥ 90%，存活变异体必须 triage
- 阶段 3：Provider 核心 100%，等价变异体显式抑制
- 阶段 1：**不强制**
- 门禁阈值不变

## 风险

1. `0.3.0` 与 Kotlin 2.3.0 / AGP 8.13 的兼容性**未验证**，需在阶段 1 末尾做一次集成冒烟
2. 若 0.3.0 不兼容，备选为 `0.2.12`，届时另开 ADR
3. PIT 依赖的 `pitest` 核心版本需一并确认（当前最新 1.30.2）

## 不采用的备选及原因

- `0.2.12`：更旧，与 Kotlin 2.3 的兼容性更差，仅在 0.3.0 验证失败时回退
- 自研 Gradle 变异插件：违反最小依赖原则，且成本远超收益

## 补充（2026-11-26 工具链基线锁定后）

- pitest 插件坐标：**`pl.droidsonroids.pitest`**，版本 **0.3.0**
- **仅从 Gradle Plugin Portal 获取，不在 Maven Central**
- **必须在 T-004 中复核 0.3.0 与 AGP 9.3.1 / Gradle 9.7.0 的兼容性**
- 若 0.3.0 不兼容，回退候选在 **Gradle Plugin Portal 上按版本倒序排查**，
  **不得默认 0.2.x**
