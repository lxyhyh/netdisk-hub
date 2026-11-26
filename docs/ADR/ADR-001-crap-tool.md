# ADR-001：CRAP 度量工具选型

- 状态：**提案（Rust 侧已定，Kotlin 侧待 T-004 定稿）**
- 日期：2026-11-26
- 关联任务：T-001
- 决策者：用户（待批）

## 背景

T-001 任务卡指定的 CRAP 工具是 `itos-cc 0.14.3`。

**事实核查结果（2026-11-26）：该包在所有公开仓库均不存在。**

| 仓库 | 查询 | 结果 |
|---|---|---|
| crates.io | `itos-cc` | 404 |
| npm | `itos-cc` | 404 |
| PyPI | `itos-cc` | 404 |
| Maven Central | `itos-cc` | 404 |
| crates.io 搜索 | `itos` | 仅返回 ito-common / ito-templates / ito-config / ito-domain / ito-core / ito-logging / ito-web / ito-backend / ito-cli / ito-canvas，无 itos-cc |

结论：`itos-cc` 无法作为依赖使用。用户已确认**作废**。

## 候选方案

| 方案 | 版本 | 语言 | 状态 | 说明 |
|---|---|---|---|---|
| **cargo-crap** | 0.10.0 | Rust | 存在 | "Change Risk Anti-Patterns (CRAP) metric for Rust projects"，直接对应 Rust 代码 |
| crap4rs | 0.6.2 | Rust | 存在 | 同类，社区较小 |
| jci-coverage | 0.0.21 | 多语言 | 存在 | 偏 CI 覆盖率聚合，非专做 CRAP |
| 自研脚本 | — | — | — | 违反"不新增自研工具链"的克制原则 |

## 决策（提案）

**采用 `cargo-crap 0.10.0` 作为 Rust 侧 CRAP 度量工具。**

理由：

1. 原名即 CRAP，语义完全对应任务卡要求
2. 直接面向 Rust 项目，与 `cargo test` / `cargo nextest` 链路一致
3. 版本 0.10.0 为当前发布版，可用
4. 不引入非 Rust 技术栈

## 影响

- 阶段 1 CI 硬门禁：CRAP ≤ 6
- 阶段 3 收紧至 ≤ 4
- **门禁阈值不变**，只换工具，不改变验收口径

## 待确认

1. `cargo-crap` 是否支持从 `cargo nextest` 的覆盖率产物读取覆盖数据（需在实际接入时验证）
2. Kotlin 侧 CRAP 是否有对应工具？当前无（见 ADR-002 讨论范围）
3. 若 `cargo-crap` 不能覆盖 Kotlin，Kotlin 侧 CRAP 门禁如何执行 —— 需补 ADR

## 不采用的备选及原因

- `crap4rs`：功能重叠但社区与维护活跃度低于 cargo-crap
- `jci-coverage`：定位是覆盖率聚合，不是 CRAP 度量，语义不符

## 补充（2026-11-26 工具链基线锁定后）

- **cargo-crap 0.10.0 用于 Rust 侧 CRAP 计算**
- **Kotlin / Android 侧 CRAP 工具尚未定稿**，候选：
  - Gradle 插件 CRAP report and quality gate（需查准确 group ID 和最新版本）
  - 自写 Gradle task 解析 JaCoCo XML + 复杂度
- Kotlin 侧选型**必须在 T-004 中完成**，**不得默认复用 cargo-crap**
- 两侧 CRAP 工具都需**复核与 Gradle 9.7.0 / AGP 9.3.1 的兼容性**
