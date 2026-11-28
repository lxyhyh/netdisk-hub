# 任务卡（TASKS）

- 最后更新：2026-11-28（**T-101 / T-102 完成**：统一 Provider 接口（模型 / 认证输入 / 不透明会话 / 十类统一错误 / 九项操作 / UniFFI 句柄导出）与两个行为不同的 MockProvider 已落地，Rust 单元测试 12 个全绿（含同一套契约测试对两个实现各跑一遍），CI 新增 Rust 测试步骤且 Android 构建与双 ABI 检查未受影响。<br>此前：2026-11-28（**T-006 最终一致性校正完成**：`t_cuid`/`t_appname` 字段来源更正（索引 38/39 曾写反）、`i()` 已确认进入最终请求、`sf_zid` 七分支与 62 位常量已闭环、验证码 WebView 与坚果云 R27 统一口径并新增「A 密码路径同样存在 50052 挑战分支」；三路径档位不变（A 可行 / B 需攻关 / C 可行）；T-005 历史 No-Go 未改写、MVP 未锁定、未启动阶段 2。<br>此前：2026-11-28（**T-006 补充收口完成**：`libbaiduprotect_sec.so` 已闭环、短信发码已取证但 **B 路径降级为「需攻关」**、扫码 `hostDeviceId` 已闭环为「官方不生成」、`cuid` 官方算法已定位；百度整体仍判「原生登录可行」但成立路径收窄为 A/C；T-005 历史 No-Go 未改写、MVP 未锁定。<br>此前：2026-11-28（T-006 完成：百度改判「原生登录可行（静态可行，动态未验证）」，硬准入数量 1 → 2；T-005 的历史 No-Go 结论未改写、MVP 未锁定；此前 T-002c / T-002d / T-005 完成、T-002e 第 3 轮已归档）
- 一个任务一节

## 编号索引

| 编号 | 任务 | 阶段 | 状态 |
|---|---|---|---|
| T-001 | 建立项目控制面与接口盘点 | 0 | 进行中 |
| T-002a | APK 清单合并 | 0 | 已完成 |
| T-002b | 反编译 1 个 APK 跑通索引格式 | 0 | 已完成 |
| T-002c | 各网盘登录流程侦察与可行性判定 | 0 | **已完成** |
| T-002d | 反编译百度 + Flutter 网盘壳确认 | 0 | **已完成** |
| T-002e | GitHub Actions CI 基础设施搭建 | 1 前置 | 进行中（第 1、2、3 轮完成；test / quality-gate workflow 未做） |
| T-003 | UniFFI 构建链最小化集成验证 | 1 前置 | 待启动（依赖 T-002e） |
| T-004 | CRAP / 变异测试工具兼容性复核 | 1 前置 | 待启动（依赖 T-002e） |
| T-005 | Go/No-Go 决策与 MVP 网盘锁定 | 0 | **已完成（No-Go）** |
| T-006 | 百度网盘原生登录攻关与第二家准入验证 | 0 | **已完成** |
| T-101 | 定义 UniFFI 深接口（统一 Provider 接口） | 1 | **已完成** |
| T-102 | 实现 MockProvider（两个假 Adapter 验证 seam） | 1 | **已完成** |

**编号规则**

- 阶段 0 侦察类任务用 `T-002` 前缀 + 字母后缀（a / b / c），表示"同一个大任务拆分出的子步骤"
- 阶段 1 前置与正式任务用连续数字编号（T-003、T-004、T-005…）
- 新增任务前必须先查本表，确认编号未被占用
- 阶段 1 前置任务中 **T-002e（CI）优先级最高**，T-003 / T-004 均依赖它

---

## T-001 建立项目控制面与接口盘点

- 阶段：0
- 状态：进行中
- 类型：文档 / 侦察（**不写生产业务代码**）

### 目标

建立项目控制面（章程、范围、架构、安全、任务、风险）与接口事实库的初始形态，
冻结 MVP 范围，确定工程基线与分阶段质量门禁，建立 APK 索引，给出纵向切片建议。

### 背景

项目目标：把 9 个网盘整合进一个 Android 客户端，个人自用。
reverse/ 目录用于存放反编译资料（不进 Git）。

### 输入

- 工作区 APK（天翼云盘在根目录，其余在附件区）
- 无网页端资料

### 输出

- docs/PROJECT_CHARTER.md
- docs/SCOPE.md
- docs/ARCHITECTURE.md
- docs/PROVIDERS.md
- docs/API_FACTS.md
- docs/SECURITY.md
- docs/TASKS.md
- docs/RISKS.md
- docs/design/UI_DIRECTION.md（补充产出）
- docs/evidence/apk-inventory.md
- CHANGELOG.md

### 验收标准

1. 9 个文档齐全（PROJECT_CHARTER / SCOPE / ARCHITECTURE / PROVIDERS /
   API_FACTS / SECURITY / TASKS / RISKS / CHANGELOG）
2. MVP 范围冻结
3. 每个网盘有明确未知问题
4. ARCHITECTURE.md 中 UniFFI 深接口草图清晰，网盘特有操作没有泄漏到 interface
5. 分阶段质量门禁表存在，100% 变异覆盖率没有被错误地放在阶段 1 硬门禁
6. 给出建议先做哪 2 个网盘及理由
7. docs/evidence/apk-inventory.md 存在，列出根目录所有 APK 及对应网盘
8. 每个有 APK 的网盘有对应 index 文件
9. 资料缺失的网盘在 PROVIDERS.md 中明确标注

### 登录规则相关验收标准

1. PROJECT_CHARTER.md「登录方式」章节包含核心原则、唯一合格路径、不合格方式、
   判定标准、不可行处理
2. PROJECT_CHARTER.md「登录方式」章节包含登录方式判定表
3. SCOPE.md「非目标」章节包含全部登录相关非目标
4. SCOPE.md「兜底策略」章节包含不合格网盘从 MVP 移除的规则
5. AGENTS.md 硬约束第 9 条已改写为原生登录版本
6. ARCHITECTURE.md「登录相关模块归属」章节存在，client_env 不暴露到 UniFFI
7. API_FACTS.md 每个网盘都有完整「登录流程」字段，含登录可行性结论
8. PROVIDERS.md 能力矩阵有「原生登录可行性」和「登录类型」两列
9. PROVIDERS.md「准入规则」章节存在
10. PROVIDERS.md 里没有任何一个网盘被标成 OAuth/WebView/cookie 可行
11. 资料缺失的网盘在 API_FACTS.md 和 PROVIDERS.md 中明确标“未知/资料缺失”

### 登录规则打回条件（出现任一即打回）

- 任何网盘被标成 OAuth 登录可行
- 任何网盘被标成 WebView 登录可行
- 任何网盘被标成 cookie 提取可行
- 任何网盘被标成 token 手动导入可行
- 登录可行性结论缺失
- 静默降级为间接登录方案

### 约束

- 不写生产业务代码；不新增依赖；不改架构
- 不提交真实账号/token；reverse/ 不得反向加入 Git
- 接口推测必须标"推测"，没有证据的不得写成事实
- 不要同时深入 9 个网盘，先做能力矩阵和 go/no-go 判断
- 不假设"最新版一定兼容"，所有版本兼容性必须在 ARCHITECTURE.md 中逐项确认

---

## T-002a APK 清单合并

- 阶段：0
- 状态：**已完成**（2026-11-26）
- 类型：文档 / 静态盘点（**不写生产业务代码，不反编译**）

### 目标

把工作区根目录与附件只读区的全部 APK 合并登记到
`docs/evidence/apk-inventory.md`，明确每个网盘有哪些资料、哪些缺失。

### 输出

- `docs/evidence/apk-inventory.md`（v0.2，7634 B）

### 完成内容

1. 根目录 1 个 APK（天翼云盘）登记：大小、包名、版本、minSdk/targetSdk、sha256
2. 附件区 **12 个文件**逐个登记，其中 8 个与网盘相关、4 个无关
3. 百度 `.apks` 内部分包明细（4 个分包，合计 409674002 字节）
4. 结构特征表：dex 数、Flutter 判定、加固特征
5. Flutter 判定含义说明（静态反编译对 Flutter 应用无效）
6. 工具链现状（jadx 未安装）
7. 资料缺口 + 反编译价值排序

### 验收标准

| # | 标准 | 状态 |
|---|---|---|
| 1 | 根目录 APK 全部登记 | ✅ |
| 2 | 附件区文件全部登记 | ✅ 12/12 |
| 3 | 每个文件有大小、包名（如适用）、sha256 | ✅ |
| 4 | 明确标出与项目无关的文件 | ✅ 4 个 |
| 5 | 明确标出资料缺失的网盘 | ✅ 3 个 |
| 6 | 未做任何反编译 | ✅ |

### 约束

- 不反编译
- 不拷贝 APK 进工作区
- 不编造任何接口信息

---

## T-002b 反编译 1 个 APK 跑通索引格式

**阶段**：阶段0
**状态**：已完成（2026-11-26）
**依赖**：T-002a（APK 清单合并完成）

**目标**：
只反编译 1 个 APK，产出完整的 index-<网盘名>.md，验证索引格式的信息量，作为后续批量反编译的模板。不批量反，不做登录可行性判定（那是 T-002c）。

**靶子**：
坚果云（nutstore.android v4.26.5），已核实：文件存在、zip 完整、aapt 可解析、
2 个 dex、无 libflutter.so / libapp.so。

若反编译过程中发现坚果云不可用（文件损坏、dex 无法解析、工具报错），
停止并报告，等待我决定，不退回任何 Flutter 网盘。

理由：Flutter 网盘（天翼/123/夸克/阿里）dex 里没有协议逻辑，
用它们验证索引格式的信息量没有参考价值，无法作为后续批量反编译的模板。

**工具准备（本任务第一步）**：

1. 下载 jadx v1.5.7：
   - URL: https://github.com/skylot/jadx/releases/download/v1.5.7/jadx-1.5.7.zip
   - 大小: 74766896 B
   - 发布: 2026-11-18
2. 下载完成后计算 SHA256，与 GitHub release 页面提供值比对（若有）
3. 解压并跑 jadx --version，确认可执行
4. 若下载中断，重试，不改用 apktool
   - 理由：apktool 拿不到 Java 类名/方法名，索引格式验证不完整
5. 记录到 docs/evidence/tooling.md：
   - 工具名、版本号、来源 URL、下载时间、SHA256、运行环境 JDK 版本

**执行步骤**：
1. 反编译靶子 APK 到 reverse/<网盘名>/
2. 建立 docs/evidence/index-<网盘名>.md，内容至少包含：
   - APK 基本信息：文件名、包名、版本、compileSdk、dex 数量、是否加固
   - 反编译工具及版本
   - HTTP 客户端相关类/方法（OkHttp/Retrofit/自定义客户端）
   - 认证相关类/方法（登录、token、cookie 处理）
   - 文件列表相关类/方法
   - 上传相关类/方法
   - 下载相关类/方法
   - 关键字符串常量（域名、路径片段、header 名、参数名）
   - 加固/混淆/证书绑定/设备指纹迹象
   - 每条线索标：类#方法 或 字符串位置 + 可信度（已验证/推测/待确认）
3. 评估索引格式：这套格式对后续 8 个网盘是否够用，缺什么字段，在 index 文件末尾写「格式评估」小节

**约束**：
- 只反 1 个 APK，不批量
- 不写生产代码
- 不登录真实账号、不抓真实请求
- 不确认完整接口，只记录线索
- reverse/ 不进 Git
- 推测必须标「推测」，无证据不得写成事实
- 不为了填满字段而编造类名或方法名

**输出**：
1. reverse/<网盘名>/（反编译产物，不进 Git）
2. docs/evidence/index-<网盘名>.md
3. docs/evidence/tooling.md（反编译工具版本、来源、校验值）
4. index 文件末尾的「格式评估」小节

**验收**：
- index-<网盘名>.md 包含上述所有必填区块，无空区块（无内容时写“未发现”并说明搜索方式）
- 每条线索有来源定位和可信度标注
- 工具版本和来源已记录
- 「格式评估」小节给出明确结论：这套格式是否可直接用于剩余 8 个网盘，需要增删哪些字段

**打回条件**：
- 批量反编译了多个 APK
- 编造类名/方法名/字符串
- 没有来源定位的"线索"
- 没有格式评估小节
- 坚果云不可用时擅自退回 Flutter 网盘，未停止报告
- 擅自改用 apktool 替代 jadx，未说明原因

### 完成情况（2026-11-26）

**产物**：

| # | 产物 | 路径 / 数值 |
|---|---|---|
| 1 | 反编译产物 | `reverse/nutstore/sources/`（8006 个 .java，约 67 MB，不进 Git） |
| 2 | 索引文件 | `docs/evidence/index-坚果云.md`（25612 B / 342 行） |
| 3 | 工具记录 | `docs/evidence/tooling.md`（jadx 1.5.7） |

**工具**：jadx v1.5.7，来源 `https://github.com/skylot/jadx/releases/download/v1.5.7/jadx-1.5.7.zip`，
74766896 B，SHA256 `1755e4242ade71a077438723689f404079592ec7764a3a0572f98fef94ebcbf3`（本地自算，
官方 release 未提供校验值 → 待确认）。运行环境 OpenJDK 21.0.13.1。

**反编译覆盖率**（2026-11-26 复核，口径 v2）：dex 内 nutstore.android 包共 **2753** 个类定义（含 720 个含 `$` 的匿名/内部类），
覆盖 **2752 个 = 100.0%**（1966 主产物 + 78 补出 + 591 内联 + 97 成员类声明 + 20 synthetic 空类）。
唯一未定位类 `WorkspaceRepository$Companion` 零业务逻辑。
**旧口径「2019 类 / 产出 1912 / 缺 107（5.3%）」已作废**——旧统计排除了含 `$` 的类，且 107 条里 76 条能补出、19 条是 synthetic 空类、12 条 dex 中不存在。详见 `docs/evidence/index-坚果云.md` §2.1.1。

**关键技术发现（均已在 index 中标行号 + 可信度）**：

- 无加固、无证书绑定、无设备指纹采集；字符串用每类自带 XOR 解密器混淆（155 个解密器、4804 处调用），
  已建立完整 keyMap 并全量解密
- 认证：`POST /mcmd/loginV2`，密码只做 Base64 不做哈希；token 无刷新机制、无过期时间
- 全部文件接口共用 `sndId`/`sndMagic` 两个 Query 参数（`Long.toHexString`）
- 上传是**裸字节流非 multipart**，单文件硬限 500 MB；**无分片**（checkBlock 系列五接口无调用点，死代码）
- 下载**无 Range 断点续传**（只有错误码常量）→ 服务端是否支持标"待确认"

**验收核对**：

| # | 标准 | 状态 |
|---|---|---|
| 1 | 10 个必填区块齐全、无空区块 | ✅ 全部有内容，未发现项均写明搜索方式 |
| 2 | 每条线索有来源定位 + 可信度 | ✅ §10 汇总 12 条：9 已验证 / 3 待确认 |
| 3 | 工具版本和来源已记录 | ✅ tooling.md + index §2 |
| 4 | 有「格式评估」小节并给出明确结论 | ✅ §11，结论=需增 4 项、降级 2 项 |
| 5 | 只反 1 个 APK，不批量 | ✅ 仅坚果云 |
| 6 | 未编造类名/方法名/字符串 | ✅ 全部带 文件:行号，未解出的写"待确认" |
| 7 | 未登录真实账号、未抓真实请求 | ✅ 全静态 |
| 8 | reverse/ 不进 Git | ✅ 已在 .gitignore |

**格式评估结论摘要**（详见 index §11）：这套格式**基本可用**，但需：

- 必增 4 项：①主体技术栈判定（Java 主体 vs Flutter）②反编译覆盖率（类总数/产出/缺失）
  ③加固与风控 SDK 清单 ④接口清单统一成「路径｜方法｜Query｜Body｜调用点」一行表
- 建议降级 2 项：域名清单只留 API 相关域；HTTP 链路细节注明"仅 Java 主体适用"
- 不建议改：行号定位、可信度三档、"未发现须写搜索方式"

对 4 个 Flutter 网盘（天翼/123/夸克/阿里）的预期：§3–§8 会全部落到"dex 内未发现"，
届时应先探一轮再决定是否继续投入，不要机械填满 10 个区块。

---

## T-002d 反编译百度 + Flutter 网盘壳确认

**阶段**：阶段0
**状态**：**已完成**（2026-11-27，验收见文末「完成情况」）
**依赖**：T-002b（索引模板已落地）

**目标**：
按 index-template.md v2.1 反编译百度网盘，产出 index-百度.md；
对 4 个 Flutter 网盘（天翼 / 123 / 夸克 / 阿里）做壳确认，
产出 index-<网盘名>.md 的"壳状态"版本。

**靶子**：
1. 百度网盘（.apks 已解出 base.apk，20 dex，无加固）
   - 完整反编译，产出完整 index-百度.md
2. 天翼 / 123 / 夸克 / 阿里（Flutter 主体）
   - 不完整反编译，只产出"壳状态"索引：
     技术栈判定 / dex 内容性质 / libapp.so 存在 / assets 关键文件 / 加固情况
   - 明确写"协议逻辑在 libapp.so，静态不可读，需动态抓包"

**执行步骤**：
1. 按 index-template.md v2.1 反编译百度，产出 index-百度.md
2. 对 4 个 Flutter 网盘逐个做壳确认，产出 index-<网盘名>.md
3. 更新 API_FACTS.md 的侦察进度汇总表
4. 更新 PROVIDERS.md 的静态线索

**约束**：
- 百度完整反编译，其余 4 个只做壳确认，不深挖
- 不登录真实账号、不抓真实请求
- 推测必须标「推测」
- reverse/ 不进 Git
- jadx 覆盖率必须按模板 §2.1 的口径 v2 算（分母含 `$` 的类，内联计入覆盖），不许只写百分比；
  关键缺失类用 `--single-class` 补并检查文件大小；反编译命令必须带 `--log-level debug` 重定向到 `reverse/<网盘名>/jadx-full.log`

**输出**：
1. docs/evidence/index-百度.md
2. docs/evidence/index-天翼.md / index-123云盘.md / index-夸克.md / index-阿里云盘.md
3. API_FACTS.md 更新
4. PROVIDERS.md 更新

**验收**：
- 5 份 index 文件齐全，遵循 index-template.md v2.1
- 百度索引含完整接口清单统一表
- 4 个 Flutter 网盘索引明确标"协议需动态抓包"
- API_FACTS 和 PROVIDERS 已同步
- jadx 覆盖率已记录

**打回条件**：
- 对 Flutter 网盘深挖 dex，浪费工时
- 百度索引未含接口清单统一表
- 未记录覆盖率
- 编造类名 / 方法名

---

### 完成情况（2026-11-27）

**产物**

| 文件 | 大小 / 行数 | 说明 |
|---|---|---|
| `docs/evidence/index-百度.md` | 43508 B / 590 行 | 完整索引，11 节 + 附录 A/B/C |
| `docs/evidence/index-天翼.md` | 10940 B / 205 行 | 壳确认模式 |
| `docs/evidence/index-123云盘.md` | 8069 B / 182 行 | 壳确认模式 |
| `docs/evidence/index-夸克.md` | 11176 B / 193 行 | 壳确认模式 |
| `docs/evidence/index-阿里云盘.md` | 10304 B / 187 行 | 壳确认模式 |

**百度反编译**

- 样本：`.apks` 内 `base.apk`，251854857 B，sha256 `3ad2da623e2a32a4924c1a8a3c84887e6994342dbbc497517a7781b3f4d260e4`，20 dex 合计 182513748 B；
- 单次全量反编译**失败**（-Xmx5g，进度 46% 时被系统内存压力终止，日志 `reverse/baidu/jadx-full.log` 1476074 B / 13056 行）→ 改按 dex 分批（`-Xmx3g -Xss2m`），产物 `reverse/baidu/sources` 1.2 GB；
- 覆盖率（口径 v2）：分母 `Lcom/baidu/` 类定义 **63259**，file-main 27424 / renamed-file 6670 / file-rec 26 / nested-decl-in-outer 15630 / synthetic-empty 169 / inline-anon-comment 13340 / **missing 0** → **100.0000%**，缺失类影响核心链路 = **否**；
- 中途 25 个 `DONT_GENERATE` 类两轮补出（第二轮加 `--no-inline-anonymous --no-inline-kotlin-lambda`），全部为 Kotlin lambda 合成类，零业务逻辑；
- 新增两条口径要求（已写入 index §2.1）：识别 jadx 改名注释与 `p000` 非法包名前缀。

**四个 Flutter 网盘壳确认（实测差异）**

| 网盘 | dex 体量 | 壳桩类数 | `libapp.so` | 特殊性 |
|---|---|---|---|---|
| 天翼 | 10136 B | **3** | 18138032 B（arm64）+ 20365916 B（v7a） | **爱加密**：`ijiami.dat` 22378961 B + `ijiami.ajm` 10890680 B |
| 123云盘 | 13916 B | **4** | 18219952 B | 无已知整包壳；`libijm-emulator.so` 来源待确认 |
| 夸克 | 12 dex / 115475 类 | 无（外壳/三方） | **1 字节占位** | 真载荷 `libkernelu4_uc_7z.so` 15938463 B 是 **7-zip**；`libfakedexuc.so` 是 **APK** |
| 阿里云盘 | 25 dex / 70881 类 | 无（壳+jsbridge） | 16794520 B | 混合架构（jsbridge/WindVane）；阿里聚安全 8 文件 |

**关键结论**

1. 百度是 **Java/Kotlin 主体 + Flutter 模块共存**（base.apk 无 `lib/`，但 split 内有 `libflutter.so` + `libapp.so` 20972464 B）→ 修正 `apk-inventory.md` 原「非 Flutter」的表述；
2. 天翼 `versionCode` 实测 **638**，修正 `apk-inventory.md` 原记的 1112；
3. 四个 Flutter 网盘的 dex 侧**确实没有网盘协议**，此判断由实测证据支撑，不再是推测；夸克更进一步：Dart 快照被打包进 7z 容器，连 `libapp.so` 都是 1 字节占位。

**验收核对（17 项）**

| # | 验收项 | 结果 |
|---|---|---|
| 1 | `index-百度.md` 完整 | ✅ |
| 2 | `index-天翼.md` 存在 | ✅ |
| 3 | `index-123云盘.md` 存在 | ✅ |
| 4 | `index-夸克.md` 存在 | ✅ |
| 5 | `index-阿里云盘.md` 存在 | ✅ |
| 6 | 均遵循模板 v2.1，Flutter 用壳状态模式 | ✅ |
| 7 | 百度覆盖率按 class_defs 口径（含 `$` 与内联） | ✅ 63259/63259 |
| 8 | 核心链路缺失类已补出并检查内容 | ✅ 25 个，零业务逻辑 |
| 9 | 百度有统一接口表 | ✅ §8.2 五组 40+ 行 |
| 10 | 每个 APK 有技术栈/dex/libapp.so/资源/加固证据 | ✅ |
| 11 | Flutter 明确标协议需动态分析 | ✅ 每份 §3–§8 与 §11 |
| 12 | API_FACTS 已同步 | ✅ v0.4，五个条目重写 + 汇总表加覆盖率列 |
| 13 | PROVIDERS 已同步 | ✅ v0.4，能力矩阵 + 登录矩阵 + §5 + §6.4 |
| 14 | TASKS 已同步 | ✅ 本节 |
| 15 | CHANGELOG 已同步 | ✅ |
| 16 | reverse/ 未进 Git、无真实凭据 | ✅ `.gitignore` 第 2 行忽略 `reverse/` |
| 17 | 关键结论有路径/行号/可复现命令 | ✅ 每条线索带 `文件:行号` 或 sha256/字节数 |

**未做（按任务范围）**

- 未判定百度原生登录可行性（属 T-002c）；
- 未对四个 Flutter 网盘深挖 `libapp.so`、未研究 Dart AOT、未做动态分析；
- 未新增依赖、未改架构/Gradle/Rust/CI。

**T-002c 启动条件核对**

| 条件 | 要求 | 现状 |
|---|---|---|
| 样本数 | ≥ 3 个已完成静态侦察 | ✅ 6 个（坚果云、百度、天翼、123、夸克、阿里） |
| Java/Kotlin 主体 ≥ 1 | ✅ | ✅ 坚果云、百度 |
| Flutter 主体 ≥ 1 | ✅ | ✅ 天翼、123、夸克、阿里 |

→ **启动条件已全部满足**，T-002c 可进入待启动。

---

## T-002c 各网盘登录流程侦察与可行性判定

**阶段**：阶段0
**依赖**：T-002d（百度完整索引 + 4 个 Flutter 网盘壳确认）+ **样本前置条件**
**状态**：**已完成**（2026-11-28 执行完毕，完成情况见本节末尾）
**启动条件（2026-11-26 追加，必须全部满足才可启动）**：

1. 已完成静态侦察的网盘样本 **≥ 3 个**；
2. 其中 **≥ 1 个是 Java/Kotlin 主体**（技术栈判定 A 类）；
3. 其中 **≥ 1 个是 Flutter 主体**（技术栈判定 B 类）。

**为什么加这条**：截至 T-002b 完成时只有坚果云 1 个 Java 主体样本，
若此刻启动 T-002c，登录能力矩阵会被坚果云单一特征带偏；
且 6 个候选里 4 个是 Flutter 主体（天翼/123/夸克/阿里），
其登录逻辑大概率不在 Java 层，仅凭 dex 静态读取判定「登录可行性」会得出错误结论。

**条件达成情况（2026-11-27，T-002d 完成后核对）**

| 条件 | 要求 | 现状 |
|---|---|---|
| 样本数 | ≥ 3 个 | ✅ **6 个**：坚果云、百度、天翼、123、夸克、阿里 |
| Java/Kotlin 主体 | ≥ 1 | ✅ 2 个：坚果云（100.0%）、百度（100.0000%） |
| Flutter 主体 | ≥ 1 | ✅ 4 个：天翼、123、夸克、阿里 |

**执行后的实际情形（2026-11-28 补记）**：4 个 Flutter 网盘中，天翼与 123 确实无法给出「可行/不可行」，
只能记为「未知（需动态分析）」；但夸克与阿里经定向反编译后，**登录协议完整存在于 dex 的 Java 层**，
因此判为「需攻关」——这一点在启动前是未知的，实际执行结果优于预期。详见 docs/evidence/login-feasibility.md 第 4 节。

**输入**：
- docs/evidence/index-<网盘名>.md（T-002b 产出）
- docs/API_FACTS.md 已有线索
- docs/PROVIDERS.md 现有能力矩阵

**目标**：
对 9 个网盘逐个给出“原生登录可行性”结论，填入 API_FACTS.md 的「登录流程」字段和 PROVIDERS.md 的两列。

**逐网盘必填字段**（按登录规则定义，不得省略）：
- 登录类型：账号密码 / 短信验证码 / 扫码 / 混合 / OAuth / 其他
- 是否可在 APP 内原生完成：是 / 否 / 未知
- 会话换取方式：官方客户端接口 / OAuth / 仅网页端 / 未知
- 是否需要 OAuth 或 WebView 才能完成：是 / 否 / 未知
- 扫码登录 token 来源：客户端协议 / WebView 抓取 / 未知
- token 刷新机制：有 / 无 / 未知
- token 有效期：具体时长 / 未知
- 依据：APK 类#方法、抓包对比、网页端资料
- 登录可行性结论：原生登录可行 / 需攻关 / 不可行 / 未知
- 若需攻关，攻关点：签名 / 设备指纹 / 版本校验 / TLS 指纹 / 其他
- 若不可行，原因及证据

**约束**：
- 不写生产代码
- 不登录真实账号、不抓真实请求
- 推测必须标「推测」，无证据不得写成事实
- Flutter 网盘（天翼/123/夸克/阿里）标“静态资料仅入口壳，协议需动态抓包”，不得强行给“可行”
- UC / 蓝奏 / 迅雷 保持“不可行（资料缺失）”
- 不得降级为 OAuth/WebView/cookie 可行

**输出**：
1. API_FACTS.md 每个网盘的「登录流程」字段填满
2. PROVIDERS.md 的「原生登录可行性」「登录类型」两列填满
3. docs/evidence/login-feasibility.md 汇总表：9 个网盘 × 可行性结论 × 依据 × 攻关点

**验收**：
- 9 个网盘每个都有可行性结论，无“待定”
- 没有任何一个网盘被标成 OAuth/WebView/cookie 可行
- Flutter 网盘的“静态资料不足”结论有明确说明
- 汇总表可直接作为 T-005 决策输入

**打回条件**：
- 登录可行性结论缺失
- 把 Flutter 网盘的“入口壳”当成“协议已拿到”
- 静默降级为间接登录方案


**完成情况（2026-11-28）**

| 项 | 内容 |
|---|---|
| 产出文件 | docs/evidence/login-feasibility.md（728 行 / 50552 B，11 个二级节） |
| 同步文档 | docs/API_FACTS.md v0.5（九家登录流程 12 字段全部填满）、docs/PROVIDERS.md v0.5（登录能力矩阵 + 准入规则 + 6.5 节） |
| 取证范围 | 坚果云（reverse/nutstore）、百度（reverse/baidu）、夸克（reverse/t002c/src/quark/classes7+10）、阿里（reverse/t002c/src/ali/classes9+11） |
| 未做 | 未登录真实账号、未发任何真实请求、未抓流量、未对 Flutter 做 Dart AOT 逆向、未破任何加固壳、未执行 T-005 |

**九网盘结论**

| 网盘 | 结论 | 关键依据 |
|---|---|---|
| 坚果云 | **原生登录可行**（静态可行，动态未验证） | POST /mcmd/loginV2，密码仅 Base64，落库与 Authorization 注入闭环，登录目录搜 WebView/OAuth 0 命中 |
| 百度网盘 | **需攻关** | 账号密码 POST {wap}/wp/api/login（RSA 1024 硬编码）、短信 POST /v2/sapi/login、扫码三步协议；缺 di 指纹裁剪规则、tpl 取值、服务端风控放行 |
| 夸克网盘 | **需攻关** | UCLoginServiceImpl + MTOP over /acs.wsg_access；请求体 M9 加密与 security_sign 均为阿里聚安全 native |
| 阿里云盘 | **需攻关** | MTOP 接口名明文（ApiConstants.java:66-120）；每个登录请求注入 riskControlInfo，MTOP 签名依赖聚安全 native |
| 天翼云盘 | **未知（需动态分析）** | dex 仅 3 个壳桩类、0 条登录线索；业务在 libapp.so + 爱加密载荷 |
| 123云盘 | **未知（需动态分析）** | dex 仅 4 个壳桩类、0 条登录线索；业务在 libapp.so |
| 蓝奏云 / UC 网盘 / 迅雷云盘 | **不可行（资料缺失）** | 沿用 docs/SCOPE.md 第 6 节兜底，不硬凑、不降级 |

**两条必须留档的修正**

1. **夸克/阿里的登录协议在 dex 的 Java 层，不在 Dart 侧**。T-002d 把四家统一记为「协议需动态分析」，对**文件业务协议**成立，对**登录协议**不成立。两家登录链路都可静态读出，结论因此是「需攻关」而不是「未知」。
2. **天翼/123 仍是「未知」且不是遗漏**。dex 侧确实 0 条登录线索，这是技术栈决定的上限，不是没做。

**对 T-005 的输入（本任务不作选择）**

- 硬准入「可行」集合：**只有坚果云 1 个**；
- 「需攻关」集合：百度、夸克、阿里（完成攻关前不计入 Go 的分子）；
- 「未知」集合：天翼、123（不进阶段 2）；
- 「不可行」集合：蓝奏、UC、迅雷（出 MVP）；
- 因此 T-005 的 Go 判据「可行集合 ≥ 2」**当前不成立**，需先完成至少一个网盘的攻关。

**验收核对（11 项）**

| # | 项目 | 结果 |
|---|---|---|
| 1 | 九家网盘每家都有四类结论之一 | 通过 |
| 2 | 坚果云原生登录调用链及会话处理审查完成 | 通过，逐跳行号见 login-feasibility.md 3.1 |
| 3 | 百度 SAPI2 调用链、OAuth 排除、Sdata 缺口审查完成 | 通过，见 3.2；OAuth 常量定性为「只取时间戳」；Sdata 明确「不属于登录链路、未还原」 |
| 4 | 四个 Flutter 各自标未知或需攻关并记阻塞点 | 通过，天翼/123 未知、夸克/阿里需攻关，阻塞点各自不同 |
| 5 | 三个资料缺失网盘沿用不可行 | 通过，见 3.7 至 3.9 |
| 6 | API_FACTS 九家 12 字段齐全 | 通过，每节 1 至 12 编号齐全 |
| 7 | PROVIDERS 已同步 | 通过，登录能力矩阵 + 准入规则 + 6.5 |
| 8 | login-feasibility.md 存在 | 通过，728 行 |
| 9 | 无 OAuth/WebView/cookie/手动 token 降级 | 通过，第 5 节逐网盘排除记录 |
| 10 | 推测与待确认未冒充已验证 | 通过，全篇三档标注，未验证项单列 |
| 11 | T-005 可直接据此筛选候选 | 通过，第 7 节 + 本节输入清单 |

**未完成的动态分析（T-005 之后或另开任务）**

| # | 事项 | 归属 |
|---|---|---|
| 1 | 坚果云 UA 与验证码阈值的服务端行为 | 阶段 2 动态验证 |
| 2 | 百度 tpl 与 appSignKey 常量值定位 | T-002c 遗留 |
| 3 | 阿里 classes18.dex 的 AccountRpc 未反编译 | T-002c 遗留 |
| 4 | 阿里 Havana 二维码流程字段 | T-002c 遗留 |
| 5 | 天翼脱壳、123 Dart AOT | 需新任务卡 |
| 6 | 聚安全 native 是否移植进本项目 | 架构决策，需 ADR |

---


---

## T-002e GitHub Actions CI 基础设施搭建

**阶段**：阶段1前置
**状态**：进行中（第 1、2、3 轮完成；test / quality-gate workflow 未做）
**依赖**：GitHub 公开仓库已创建并首次 push 完成
**优先级**：高于 T-003 / T-004

### 目标

搭建 GitHub Actions workflow，使后续所有编译、构建、测试、打包可在 CI 完成。
本地不具备编译条件，CI 是唯一编译验证通道。

### 必须覆盖的 workflow

1. build：Kotlin 编译（AGP 9.3.1 / Gradle 9.7.0 / JDK 17 / compileSdk 37 / Build Tools 36.0.0）
2. build：Rust 交叉编译（Rust 1.100.0 / cargo-ndk 4.1.2 / NDK r30 → Android ABI）
3. build：UniFFI 绑定生成（uniffi-bindgen 0.32.2）
4. build：APK / AAB 打包
5. test：单元测试（JUnit 5.14.x + cargo test / nextest）
6. quality-gate：CRAP 门禁（cargo-crap 0.10.0 + Kotlin 侧工具，T-004 定稿后接入）
7. quality-gate：变异测试（pl.droidsonroids.pitest 0.3.0 + cargo-mutants 27.1.0，T-004 定稿后接入）

### workflow 分档策略

- build + test：push 触发，paths 过滤排除 docs/ 和 reverse/
- quality-gate：PR 触发 + 手动 workflow_dispatch 触发，不随每次 push 跑
- 文档改动：不触发 build

### 执行步骤

1. 建立 .github/workflows/ 目录
2. 逐条建立 workflow 文件，先用空项目跑通最小构建
3. 每个 workflow 记录：触发条件、runner 环境、依赖安装步骤、产物、失败重试策略
4. 按 SECURITY.md 的 CI 审查清单逐条核对：secrets 引用、无 echo 敏感变量、artifact 脱敏检查
5. workflow 产物上传为 artifact
6. 在 docs/evidence/ci-setup.md 中记录：runner 环境、依赖版本、触发策略、产物说明

### 约束

- 只搭 CI 骨架，不写业务代码
- workflow 必须可复现：从干净 runner 开始，不依赖本地缓存
- 不得把真实 token / cookie / 账号写进 workflow 或 secrets 之外的地方
- 不假设本地可编译，workflow 是唯一编译验证依据
- 遵守 SECURITY.md 的「公开仓库 CI 安全」章节

### 输出

1. .github/workflows/ 下至少 3 个 workflow：build / test / quality-gate
2. docs/evidence/ci-setup.md
3. 一次空项目 CI 全绿记录（日志链接或截图）

### 验收

- 从干净 runner 触发 build workflow，APK 打包成功
- test workflow 跑通单元测试
- quality-gate workflow 可运行（具体工具等 T-004 定稿后接入）
- 产物可从 Actions 页面下载
- ci-setup.md 记录完整
- SECURITY.md CI 审查清单 5 项全部核对通过

### 打回条件

- workflow 依赖本地缓存或本地文件
- 声称 CI 跑通但无日志证据
- 未记录 runner 环境和依赖安装步骤
- CI 日志或 artifact 中出现真实凭据
- commit message 出现"破解 / 绕过 / 封堵"类描述

### 第 1 轮完成情况（最小 Android 工程 + APK 打包）

**状态**：已完成（2026-11-27 归档）

| 项 | 实测值 |
|---|---|
| commit | `0773aad65d9777451c896c156f082bb15a6ac8c0` |
| commit message | `build: 最小 Android 工程骨架 + 第 1 轮 CI（assembleDebug）` |
| CI 运行 | https://github.com/lxyhyh/netdisk-hub/actions/runs/45863557547 |
| 运行 ID / workflow | `45863557547` / `build` |
| 结果 | **success**（触发 16:58:03Z → 完成 17:00:32Z，总 2m 29s，构建 1m 39s） |
| APK artifact | `app-debug-apk`（Artifact ID `15138789375`，zip 10204286 B，解压后 29209031 B） |
| APK SHA256 | `1038561f0ee8302b47006dd34edc0cac71762c56a0196a0713fc7c7312fd0928` |
| APK 元数据 | `com.netdiskhub.app` / versionCode 1 / 0.1.0 / compileSdk 37 / minSdk 31 / targetSdk 37 |

**本轮新增的工程文件（11 个）**：`.gitignore`（追加构建产物规则）、`settings.gradle.kts`、
`gradle.properties`、`gradle/libs.versions.toml`、`build.gradle.kts`、`app/build.gradle.kts`、
`app/src/main/AndroidManifest.xml`、`app/src/main/kotlin/com/netdiskhub/app/MainActivity.kt`、
`app/src/main/res/values/strings.xml`、`app/src/main/res/values/themes.xml`、
`.github/workflows/build.yml`。

**关键结论**

1. 最小 Android 工程可编译、APK 可生成，artifact 可下载。
2. 实际编译 Kotlin 版本为 **2.4.x**（依据产物 dex 中 `kotlin/Metadata` 的 `mv={2,4,0}`）。
3. Compose 依赖实际版本：ui 1.12.1 / runtime 1.12.1 / material3 1.4.0 / activity-compose 1.13.0，
   对应 compose-bom `2026.09.00`。
4. SDK 安装链路正常：`platforms;android-37.0` 可装，`android-37` 符号链接生效，
   `sdkmanager` 走 `command -v` 分支（`/usr/local/lib/android/sdk/cmdline-tools/latest/bin/sdkmanager`）。
5. Android 构建的警告只有 `libandroidx.graphics.path.so` 无法 strip，本次构建仍 `BUILD SUCCESSFUL`；
   另有 GitHub Actions 平台警告（Node.js 20 deprecated、`actions/setup-java@v4` 弃用），
   与 Android 构建无关。详见 `docs/evidence/ci-setup.md` §7.1 / §7.2。

**未验证 / 待确认**：KGP 精确版本号 2.4.21 未在 CI 日志明文打印（结论为推论链，见
`ci-setup.md` §6.3）；Rust 交叉编译、UniFFI 绑定、Kotlin↔Rust 调用、真机安装运行均未开始。

**第 2 轮**：已完成（见下节「第 2 轮完成情况」）。产物为 `rust/` + `.github/workflows/rust.yml`，
cargo-ndk 产出 arm64-v8a 与 x86_64 的 `.so`；不涉及 UniFFI 与 Kotlin 集成，不改 APK 打包流程。

**第 3 轮**：已完成（见下节「第 3 轮完成情况」）。产物为 UniFFI Kotlin 绑定 + Kotlin→Rust 调用 + APK 内原生库。

**证据归档**：`docs/evidence/ci-setup.md`（runner 环境、依赖版本、产物核验、已知警告、已验证/未验证清单）。

### 第 2 轮完成情况（Rust Android 独立交叉编译）

**状态**：已完成（2026-11-27 归档，commit `b8e1b08`）

| 项 | 实测值 |
|---|---|
| commit | `b8e1b0898cabf6b273d17c8f01f69292d9de73c5` |
| commit message | `build: add standalone Rust Android cross-compilation workflow` |
| CI 运行 | https://github.com/lxyhyh/netdisk-hub/actions/runs/45938734221 |
| 运行 ID / workflow | `45938734221` / `rust` |
| 结果 | **success**（触发 2026-11-27T04:14:03Z → 完成 2026-11-27T04:15:08Z，总 1 分 05 秒） |
| 工具链实测 | Rust **1.100.0** / Cargo **1.100.0** / cargo-ndk **4.1.2** / NDK **30.0.16248370** |
| ABI | `arm64-v8a`、`x86_64` |
| 产物 | `rust-android-libs`（Artifact ID `15172320101`，zip 379922 B） |

**本轮新增的工程文件（5 个）**：`rust/Cargo.toml`、`rust/core/Cargo.toml`、`rust/core/src/lib.rs`、
`rust/rust-toolchain.toml`、`.github/workflows/rust.yml`。

**交叉编译产物（CI 内 `Verify artifacts` 步骤硬断言，非人工目测）**

| ABI | target | 文件 | 大小 | ELF Class | Machine | 导出符号 |
|---|---|---|---|---|---|---|
| arm64-v8a | `aarch64-linux-android` | `libnetdiskhub_core.so` | 449560 B | ELF64 | AArch64 | `netdiskhub_probe` |
| x86_64 | `x86_64-linux-android` | `libnetdiskhub_core.so` | 424656 B | ELF64 | Advanced Micro Devices X86-64 | `netdiskhub_probe` |

**关键结论**

1. Rust → Android 交叉编译链路独立可跑通，不依赖 Android 工程与 APK 打包流程。
2. `rust/rust-toolchain.toml` **确实生效**：CI 日志原文
   `1.100.0-x86_64-unknown-linux-gnu (overridden by '.../rust/rust-toolchain.toml')`。
3. NDK 版本被显式钉死在 `30.0.16248370`：runner 预置的 `ANDROID_NDK_HOME` 指向 27.3.13750724，
   workflow 用三个变量（`ANDROID_NDK_HOME` / `ANDROID_NDK_ROOT` / `ANDROID_NDK`）同时覆盖，
   并在构建前 assert `source.properties` 的 `Pkg.Revision`。
4. Artifact 解压后**无多余根目录**，正好两个文件：`arm64-v8a/libnetdiskhub_core.so` 与
   `x86_64/libnetdiskhub_core.so`。
5. 同一 push 触发的 Android `build` workflow（run `45938734219`，59 秒）也 success，
   构成既有 Android 构建的回归验证，未因新增 `rust/**` 而回退。

**未完成（保留）**

- UniFFI 绑定生成：未开始
- Kotlin ↔ Rust 调用：未开始
- `.so` 集成进 APK：未开始
- 真机安装运行：未验证

**证据归档**：`docs/evidence/ci-setup.md` §10（本轮完整实测记录）。

### 第 3 轮完成情况（UniFFI 绑定 + Kotlin 集成）

**状态**：已完成（2026-11-27 归档）。共 4 次提交，其中前两次失败、第三次全绿、第四次补验收加固；失败原因照录在下。

| 项 | 实测值 |
|---|---|
| 提交 1（实现） | `ce09f860199646be9b9ebb2b8b71325b45bdde73` — `build: T-002e 第 3 轮 Rust UniFFI 绑定 + Kotlin 集成（双 ABI）` |
| 提交 2（修脚本编译） | `1354f13` — `fix: 修复第 3 轮 build workflow 首次运行的脚本编译错误` |
| 提交 3（修 jniLibs 接线） | `4cc08f9` — `fix: 用变体 API 接入 jniLibs，消除 mergeDebugJniLibFolders 隐式依赖错误` |
| 提交 4（验收加固） | `206f052` — `fix: 按官方 16 KB 文档实现 GNU_RELRO 断言 + 增加 zipalign 验收` |
| 第 3 次运行（首次全绿） | https://github.com/lxyhyh/netdisk-hub/actions/runs/45950571670（`45950571670` / `build`，13 步全 success，3 分 01 秒） |
| 最终 CI 运行（第 4 次，验收加固后） | https://github.com/lxyhyh/netdisk-hub/actions/runs/45952694106 |
| 运行 ID / workflow | `45952694106` / `build` |
| 结果 | **success**，14 个业务步骤全部 success（触发 2026-11-27T06:34:11Z → 完成 06:37:11Z，总 3 分 00 秒） |
| APK artifact（第 4 次） | `app-debug-apk`（Artifact ID `15177950350`，zip 10770227 B，digest `sha256:a6f6b9e391952ecfb3c24d7dd4d1568371e32fe6b1940cc427f8e30e2ec1614a`） |
| 下载后 APK（第 4 次） | 30500633 B，SHA256 `725dca77c34e85d6458c65e58bc7699aa5c2af0a74d229ecbb2aaa88077e03ab` |
| APK artifact（第 3 次） | Artifact ID `15175884387`，zip 10770223 B，digest `sha256:7e4c6fed147bf1ae71b741c891e6673760d9f8dffaa1a7615f03f5c944320517`；下载后 30500633 B，SHA256 `f1679f5834f41f2d1771107649cd02b937d719ae8a7485d51f6ec3cbdbecfebf` |
| 并行的 rust workflow | run `45949811566` **success**（`--locked` 生效，无回退） |

**本轮新增/修改的文件（15 个，+1602/−18）**

| 文件 | 作用 |
|---|---|
| `rust/core/src/lib.rs` | 新增 `#[uniffi::export] pub fn probe_magic() -> u32` 与 `uniffi::setup_scaffolding!()`；保留第 2 轮的 `netdiskhub_probe` |
| `rust/core/Cargo.toml` | 增加 `uniffi = "=0.32.2"` 依赖 |
| `rust/core/uniffi.toml` | `[bindings.kotlin] package_name = "com.netdiskhub.core"` |
| `rust/uniffi-bindgen/{Cargo.toml,src/main.rs}` | workspace 内 crate 承载 bindgen CLI（`uniffi_bindgen_main()`），不用 `cargo install` |
| `rust/Cargo.toml` | members 增加 `uniffi-bindgen` |
| `rust/Cargo.lock` | **纳入版本控制**，作为 `--locked` 的锁 |
| `app/build.gradle.kts` | `CargoNdkBuildTask` / `GenerateUniffiBindingsTask` 两个任务类型；`androidComponents.onVariants` 里接 jniLibs 与 Kotlin 生成源；`ndkVersion` / `abiFilters` |
| `app/src/main/kotlin/com/netdiskhub/app/NativeProbe.kt` | Kotlin→Rust 的编译期引用 |
| `gradle/libs.versions.toml` | 新增 `jna = "5.19.1"`（坐标在 build.gradle.kts 里拼出 `@aar`） |
| `docs/ADR/ADR-004-jna-runtime.md` | JNA 引入决策（已批准） |
| `scripts/verify-apk-native-libs.py` | APK 原生库静态校验脚本 |
| `.github/workflows/build.yml` | 新增 NDK r30 / Rust 1.100.0 / cargo-ndk 4.1.2 安装与三步产物校验 |
| `.github/workflows/rust.yml` | 交叉编译命令补 `--locked` |
| `.gitignore` | 补 `__pycache__/` 与 `*.pyc` |

> 上表为前三次提交（`ce09f86` + `1354f13` + `4cc08f9`）的合计。第四次提交 `206f052`
> 只改 2 个文件（+280/−27）：`scripts/verify-apk-native-libs.py` 新增 `check_relro()` 与 `--selftest`，
> `.github/workflows/build.yml` 加 `--selftest` 一行并新增 zipalign 验收步骤。

**CI 实际执行的关键任务**

`cargoNdkBuild`（`cargo ndk ... build --release --locked`，arm64 21.21s + x86_64 5.44s）→
`generateUniffiBindingsDebug`（cargo build uniffi-bindgen 57.56s）→ `compileDebugKotlin` →
`mergeDebugJniLibFolders` → `mergeDebugNativeLibs` → `packageDebug`。
Gradle 汇总：`38 actionable tasks: 20 executed, 18 from cache`。

**APK 内原生库（脚本硬断言，非目测）**

| 条目 | 大小 | 说明 |
|---|---|---|
| `lib/arm64-v8a/libnetdiskhub_core.so` | 353896 B | Rust 核心，4 个 PT_LOAD 全 `0x4000` |
| `lib/arm64-v8a/libjnidispatch.so` | 165992 B | JNA 提供 |
| `lib/arm64-v8a/libandroidx.graphics.path.so` | 10096 B | Compose 传递依赖 |
| `lib/x86_64/libnetdiskhub_core.so` | 371864 B | 4 个 PT_LOAD 全 `0x4000` |
| `lib/x86_64/libjnidispatch.so` | 116904 B | JNA 提供 |
| `lib/x86_64/libandroidx.graphics.path.so` | 10760 B | Compose 传递依赖 |

清单严格等于上述 6 条（多一条少一条都判失败）；全部 `ELFCLASS64`；6 条都 `STORED` 且数据偏移 16 KB 对齐。
`libnetdiskhub_core.so` 动态符号含 `netdiskhub_probe` /
`uniffi_netdiskhub_core_checksum_func_probe_magic` /
`uniffi_netdiskhub_core_fn_func_probe_magic`，两 ABI 一致。

**Kotlin 绑定与集成（已验证）**

1. 绑定文件生成在 `app/build/generated/kotlin/generateUniffiBindingsDebug/com/netdiskhub/core/netdiskhub_core.kt`，
   40751 B，第 6 行 `package com.netdiskhub.core`。
2. 绑定被真正编译进 dex：`classes3.dex` 与 `classes4.dex` 都能 grep 到
   `Lcom/netdiskhub/core/` 与 `probeMagic`，门面类 `Netdiskhub_coreKt` 同时出现。
3. APK 内 `classes*.dex` 共 6 个，累计 28208 KB；`com/sun/jna` 在 `classes2/4/6.dex` 命中，
   说明 JNA 随绑定一起进包。
4. Manifest 合并**没有报错**，也未加任何 `tools:*`；合并后 `targetSdkVersion=37` 采用宿主值
   （JNA AAR 自带的 `targetSdkVersion=25` 被忽略）。

**前两次 CI 失败的原因（照录，不掩饰）**

1. 第 1 次（run `45949811572`）失败在 Gradle 脚本编译：
   `e: app/build.gradle.kts:67:31: Unresolved reference 'io'.` 与 `:101:32: Unresolved reference 'io'.`
   —— 我此前把 `File(...)` 改成了 `java.io.File(...)`，Kotlin DSL 里 `java` 是扩展属性，解析不出包名。
   修法：显式 `import java.io.File`，写回裸 `File(...)`。
   同一次还暴露 `jniLibs.srcDir(...)` 在 AGP 9.3.1 已废弃（`w: 'fun srcDir' is deprecated`）。
2. 第 2 次（run `45950092700`）失败在 `mergeDebugJniLibFolders`：
   `uses this output of task ':app:cargoNdkBuild' without declaring an explicit or implicit dependency`。
   —— 我把 `cargoNdkBuild` 的输出目录以字符串塞进 `sourceSets.jniLibs.directories`，AGP 不知道任务依赖。
   修法：`cargoNdkBuild` 不再手动 set `outputDir`，改为在 `androidComponents.onVariants` 里
   用 `jniLibs.addGeneratedSourceDirectory(cargoNdkBuild, CargoNdkBuildTask::outputDir)` 接入，
   依赖关系由 AGP 建立；同时删掉 `sourceSets` 里的路径注入。

**只支持 arm64-v8a 与 x86_64（明确声明）**

当前原生核心**只**交叉编译并打包 `arm64-v8a` 与 `x86_64` 两个 ABI，其余 ABI 不支持、不承诺。
依据：ADR-004 批准的 ABI 收敛；`app/build.gradle.kts` 的 `ndk { abiFilters += listOf("arm64-v8a","x86_64") }`；
APK 内 `lib/` 只有这两个目录（脚本严格清单断言）。

**第 4 次运行：验收加固（GNU_RELRO 官方断言 + zipalign）**

第 3 次运行虽然全绿，但 `PT_GNU_RELRO` **只打印、不判定**，理由是「结束地址不对齐的情况在 Google 自己的
`androidx.graphics.path.so` 上也存在」。该理由不成立：官方文档
（https://developer.android.com/guide/practices/page-sizes）给的是**三个并列条件，满足任一即合规**：

| 条件 | 内容 |
|---|---|
| A | 无 `GNU_RELRO` 段 |
| B | `(RELRO.VirtAddr + RELRO.MemSiz) % 0x4000 == 0` |
| C | RELRO 之后在同一 LOAD 段内没有可写数据 |

因此「结束地址不对齐」既不等于失败，也不构成免检理由；上游是 Google 的库同样不构成豁免。
`scripts/verify-apk-native-libs.py` 已按 A → B → C1 → C2 顺序实现判定：C1 用官方等式
（`LOAD.VirtAddr + LOAD.MemSiz == RELRO.VirtAddr + RELRO.MemSiz`），C2 把 RELRO 覆盖范围按 16 KB 取整成页区间，
要求该区间内 RELRO 之外无可写字节。**RELRO 起始地址不落在任何 `PT_LOAD` 内时直接判失败，不静默通过。**
另加 `--selftest`，用合成 ELF 回归官方 Example 1–4 与一条「RELRO 不在任何 LOAD 内」负例。

6 个库的判定结果（第 4 次运行日志，全部**合规、无需修复**）：

| 库 | RELRO 结束 | 命中条件 |
|---|---|---|
| `arm64-v8a/libandroidx.graphics.path.so` | `0x6000` | B 否；**C2 合规**（页区间 `[0x4000,0x8000)` 内 RELRO 外无可写字节） |
| `arm64-v8a/libjnidispatch.so` | `0x2c000` | **B 合规** |
| `arm64-v8a/libnetdiskhub_core.so` | `0x5e000` | **C1 合规**（`LOAD#3 [0x5b4e0,0x5e000)` 结束地址相等） |
| `x86_64/libandroidx.graphics.path.so` | `0x7000` | B 否；**C2 合规** |
| `x86_64/libjnidispatch.so` | `0x20000` | **B 合规** |
| `x86_64/libnetdiskhub_core.so` | `0x62000` | **C1 合规**（`LOAD#3 [0x5f3e0,0x62000)`） |

新增的官方 zipalign 验收（`build.yml` 独立步骤，失败即令 workflow 失败）：

```
$ANDROID_HOME/build-tools/36.0.0/zipalign -v -c -P 16 4 app/build/outputs/apk/debug/app-debug.apk
```

CI 日志末行 `Verification successful`，6 个 `.so` 全部 `(OK)`。原有的 python ZIP 偏移检查保留，两者交叉验证。

第 4 次运行 [run 45952694106](https://github.com/lxyhyh/netdisk-hub/actions/runs/45952694106) 14 步全 success；
最终断言输出：

```
PASS: 6 个原生库通过 清单 / ELF / PT_LOAD 16 KB 对齐 / GNU_RELRO 官方条件 / ZIP 页对齐 / 符号 检查
```

**未完成（保留，不标完成）**

- 运行时端到端：`probeMagic()` 的返回值未在设备/模拟器上实际执行过（属 T-003）。
- `test` workflow（单元测试）与 `quality-gate` workflow（CRAP / 变异）仍未建立。
- `docs/evidence/ci-setup.md` 的第 3 轮记录见 §11。

**证据归档**：`docs/evidence/ci-setup.md` §11（本轮完整实测记录）。

---

## T-003（阶段1前置）：UniFFI 构建链最小化集成验证

- 阶段：1 前置
- 状态：待启动
- 类型：构建验证（**不做业务逻辑**）

### 目标

验证 UniFFI 0.32.2 + cargo-ndk 4.1.2 + Rust 1.100.0 + NDK r30 + Gradle 9.7.0 的完整协同。

本任务所有验证在 GitHub Actions 完成，本地不编译。CI 运行结果作为唯一验证依据。

### 逐条验证

1. `uniffi-bindgen` 生成 Kotlin 绑定
2. Kotlin 2.4.21 编译生成代码
3. Rust 1.100.0 交叉编译到 Android ABI
4. NDK r30 原生库生成
5. Gradle 9.7.0 自动执行绑定生成和 Rust 编译任务
6. APK/AAB 中各 ABI 的 `.so` 正确打包
7. Android 设备实际加载并调用 UniFFI 接口

### 约束

- 最小化工程，不做业务逻辑。
- 不得声称"本地已编译通过"
- 每完成一步即 push，等 CI 结果再继续
- CI 失败时先读日志定位，不盲目重试
- 依赖 T-002e（CI 基础设施）完成

### 验收

1-7 项验证全部在 CI 上完成，每项有 CI 运行日志或 artifact 作为证据。
CI 全绿视为通过；CI 失败项需记录失败原因和回退方案。
不得以"本地已验证"替代 CI 验证。

### 失败回退

先查第三方插件 / UniFFI 任务配置 / Gradle API；**确认为 Gradle 版本问题**才回退到 9.6.1。

---

## T-004（阶段1前置）：CRAP / 变异测试工具兼容性复核

- 阶段：1 前置
- 状态：待启动
- 类型：工具验证（**不写业务代码**）

### 目标

确认 CRAP 和变异测试工具在 AGP 9.3.1 + Gradle 9.7.0 下可用。

本任务所有工具兼容性验证在 GitHub Actions 完成，本地不编译。CI 运行结果作为唯一验证依据。

### 逐条验证

1. Kotlin 侧 CRAP 工具选型（ADR-001 候选）
2. `cargo-crap 0.10.0` 在 Rust 侧可用
3. `pl.droidsonroids.pitest 0.3.0` 与 AGP 9.3.1 兼容
4. `cargo-mutants 27.1.0` 在 Rust 1.100.0 下可用
5. 所有工具的 CI 退出码和阈值配置可落地

### 约束

- 只做工具验证，不写业务代码。
- 不得声称"本地已验证兼容"
- 依赖 T-002e 完成
- 工具接入 CI 后必须实际跑一次，留下日志

### 验收

1-5 项验证全部在 CI 上完成，每项有 CI 运行日志或 artifact 作为证据。
不兼容项写 ADR 并给替代方案，替代方案同样需在 CI 验证。
不得以"本地已验证"替代 CI 验证。

---

## T-005 Go/No-Go 决策与 MVP 网盘锁定

**阶段**：阶段0
**依赖**：T-002c（登录可行性判定完成）
**状态**：**已完成（2026-11-28），结论 No-Go**（完成情况见本节末尾）

**决策标准（现在定死，不因结果调整）**：

1. **硬准入条件**：原生登录可行性 = 可行
   - 需攻关：可进候选，但必须先完成攻关任务
   - 不可行 / 未知：出 MVP

2. **候选排序维度**（在可行集合内排序）：
   - 协议差异度（认证方式、上传协议差异越大越好，能验证深接口）
   - 客户端校验强度（低优先）
   - 静态资料完整度（高优先）

3. **MVP 锁定 2 个网盘**，一个偏“简单可跑通”，一个偏“有差异能验证抽象”

4. **Go/No-Go 判定**：
   - Go：可行集合 ≥ 2 个
   - No-Go：可行集合 < 2 个，需回头补资料或调整 MVP 范围

**输出（待 T-002c 完成后执行）**：
1. MVP 网盘锁定结论 + 理由
2. 若 Go，进入阶段1（先完成 T-002e CI 基础设施，再并行启动 T-003 / T-004）
3. 若 No-Go，给出补救方案

**约束**：
- 不得为凑数把“需攻关”或“不可行”的网盘拉进 MVP
- 不得因偏好某网盘而放宽准入条件

**完成情况（2026-11-28）**

| 项 | 内容 |
|---|---|
| 决策文件 | `docs/evidence/mvp-go-no-go.md`（340 行 / 21703 B，12 个二级节） |
| 决策结果 | **No-Go** |
| 硬准入数量 | **1**（坚果云，原生登录可行） |
| Go 门槛 | **2** |
| MVP 锁定 | **未锁定任何网盘**，不启动阶段 2 |
| 同步文档 | `docs/PROVIDERS.md` v0.6（§3 / §4 / §6.6 收口）、`docs/RISKS.md` v0.5（R22 关闭 + R27 / R28）、`CHANGELOG.md` |
| 未做 | 未开发 Provider、未改登录协议、未登录真实账号、未发任何真实请求、未抓流量、未修改 Gradle / Rust / CI |

**九家网盘准入结论**

| 网盘 | T-002c 结论 | T-005 处理 | 计入 Go |
|---|---|---|---|
| 坚果云 | 原生登录可行（静态可行，动态未验证） | 唯一符合硬准入 | **是（1）** |
| 百度网盘 | 需攻关 | 不计入，转 T-006 攻关 | 否 |
| 夸克网盘 | 需攻关 | 不计入，攻关优先级 2 | 否 |
| 阿里云盘 | 需攻关 | 不计入，攻关优先级 3 | 否 |
| 天翼云盘 | 未知（需动态分析） | 不进阶段 2 | 否 |
| 123云盘 | 未知（需动态分析） | 不进阶段 2 | 否 |
| 蓝奏云 / UC / 迅雷 | 不可行（资料缺失） | 从 MVP 移除 | 否 |

**为什么不能凑两个**

1. 规则层面：硬准入写死为「可行」，把「需攻关」或「未知」算进分子就是修改规则。
2. 工程层面：三家「需攻关」的阻断点集中在同一类东西——闭源 native 安全组件（夸克 / 阿里为阿里聚安全，百度为 `passport_core` 与 `di` 指纹）。
3. 项目层面：`AGENTS.md` 第 9 条禁止 OAuth / WebView / cookie 降级，而凑数最省事的做法恰恰是降级。

**坚果云验证码风险的处置**

静态核查结论（`v2/ui/signup/TencentCaptchaView.java:29/:73`、APK 内 `assets/TencentCaptchaH5.html` 1522 B）：

- 验证码是**服务端条件触发**的分支（`isNeedRealCaptcha()`，`v2/ui/login/f.java:29-35`），不是每次登录必经；
- 控件是纯客户端代码（`FrameLayout` + `WebView` + 本地 H5），**不依赖官方 App 进程**，但需要腾讯验证码服务（`turing.captcha.qcloud.com/TJCaptcha.js`）生成 `ticket` / `randstr`；
- 因此性质是「第三方闭源 **Web** 服务」，不是「闭源 native SDK」；凭证仍由本项目原生协议（`/mcmd/loginV2`）直接交换，**不构成授权降级**；
- 分类**维持「原生登录可行」**，同时写死降级触发条件（见 `mvp-go-no-go.md` §5.3），`RISKS.md` 新增 R27；
- UA 校验、验证码触发阈值、`tfPasscode` 开关、token 有效期 **全部未验证**（待确认）。

**攻关优先级（只给顺序，不估工时）**

| 优先级 | 网盘 | 主要理由 |
|---|---|---|
| 1 | 百度网盘 | 三通道协议链路已全部找到；RSA / AES / MD5 全在 Java 层可移植；`Sdata` 不在 sapi2 登录链路（0 命中）；剩余问题属可继续挖的静态问题 |
| 2 | 夸克网盘 | 登录链路已读完，唯一卡点是请求体 M9 加密 + `security_sign` 均依赖阿里聚安全 native，单点但硬 |
| 3 | 阿里云盘 | 除聚安全 native 外，`AccountRpc` 所在 `classes18.dex` 未反编译、Havana 二维码字段未读，静态资料尚未读完 |

**11 项验收核对**

| # | 项目 | 结果 |
|---|---|---|
| 1 | 明确判定 Go/No-Go | 通过：**No-Go** |
| 2 | 当前只有 1 家达到硬准入 | 通过：仅坚果云 |
| 3 | 不选择、不锁定两个 Provider | 通过：明确未锁定 |
| 4 | 三个攻关候选比较完整 | 通过：`mvp-go-no-go.md` §6，8 维度比较表 |
| 5 | 建立 T-006 | 通过：见下节，状态待启动 |
| 6 | 坚果云条件验证码与动态未验证事项明确记录 | 通过：§5 四问逐条 + 降级触发条件 + §8 全局边界 |
| 7 | R22 收口关闭 | 通过：`docs/RISKS.md` R22 已关闭，依据本文 |
| 8 | TASKS 同步（T-005 完成 / T-006 待启动 / 底部索引） | 通过 |
| 9 | PROVIDERS 与正式决策一致 | 通过：§6.6 新增 |
| 10 | RISKS 状态与内容同步 | 通过：R22 关闭、R27 / R28 新增、R10 更新 |
| 11 | 未提前启动阶段 1 或伪造 CI 完成 | 通过：T-002e / T-003 / T-004 状态原样保留 |

---

## T-006（阶段0）：百度网盘原生登录攻关与第二家准入验证

**阶段**：阶段0
**依赖**：T-005（已完成，结论 No-Go）
**状态**：**已完成**（2026-11-28）
**类型**：静态取证 + 可行性判定（**不写生产代码、不发真实请求**）

### 目标

把百度网盘从「需攻关」推到「原生登录可行」或明确「不可行」，使 T-005 的 Go 判据「可行集合 ≥ 2」有机会成立。

本任务**不是**实现登录，只是把静态证据链补完整并给出四档结论之一。

### 工作范围（八项，逐条可验收）

1. 继续追 `tpl` / `appSignKey` 的配置来源与注入过程（起点 `SapiConfiguration.java:403/523/857/1071`、`setProductLineInfo` 的调用方）。
2. 追 `SapiDeviceInfo.buildDeviceTokens`（`:82-184`）的 40 个字段，**逐字段分类**为必需 / 条件（受 `diExceptIndex` 或灰度开关影响）/ 可选，并给出分类依据。
3. 查明 `passport_core`（`PassCoreVHelper.java:4-19` 的 6 个 native）与其它 native 安全组件在**选定登录路径**（账号密码 / 短信 / 扫码分别看）中的实际作用。
4. 判断哪些部分能由 Rust 独立复刻，哪些依赖闭源 native；**逐项给结论**，不许写「大概可以」。
5. 明确账号密码路径能否独立实现；**短信与扫码路径分别记录，不得混为一谈**。
6. 检查是否必须移植官方 native 库，或是否需要新增依赖、改变架构。
7. 若涉及新增依赖、引入闭源库或架构变更，**先提 ADR 等批准**，不得擅自实施。
8. 形成完整证据链与明确结论（四档之一）。

### 输出

1. `docs/evidence/login-feasibility.md` §3.2 更新（百度一节补全，结论按实际证据改写）
2. 新增 `docs/evidence/baidu-login-deep-dive.md`：逐项证据表（文件:行号 + 可信度）
3. `docs/API_FACTS.md` 百度登录流程字段按新证据修正
4. `docs/PROVIDERS.md` §3 百度一行同步
5. 结论若涉及依赖变更，另出 ADR 草案（**不实现**）

### 约束

- **不得**直接把百度改成「原生登录可行」——必须有逐项证据支撑。
- **不得**使用真实账号发送登录请求，**不得**抓真实流量。
- **不得**复用官方 App 的私有会话或 cookie。
- **不得**把闭源库移植视为已获批准。
- **不得**在没有证据时宣称 native 依赖可以被替代。
- 可信度只用「已验证 / 推测 / 待确认」；「已验证」只表示静态代码存在，不代表服务端行为已验证。
- App 私有的混淆类名、`p000` 改名规则、`--single-class` 补出流程沿用 T-002d 的口径。

### 验收

1. `tpl` / `appSignKey` 给出取值来源结论（定位到具体调用点，或明确写「静态不可得 + 原因」）。
2. 40 个设备指纹字段完成三类划分，每类有依据。
3. 账号密码 / 短信 / 扫码三条路径**分别**给出「可独立实现 / 需 native / 不可行」结论。
4. 每条结论带 `文件:行号`，且抽查可复现。
5. 四档结论明确（原生登录可行 / 需攻关 / 不可行 / 未知），并说明相对 T-002c 是否变化、为什么。
6. 若结论涉及依赖或架构变更，ADR 草案已提交并经用户批准前不落地。
7. 未新增任何真实请求、未登录真实账号、未把 `reverse/` 纳入 Git。

### 打回条件

- 未读 `passport_core` 的调用链就直接断言「不影响」。
- 把「静态未找到」写成「运行期不需要」。
- 把账号密码、短信、扫码混成一条结论。
- 为达成「转 Go」而放宽判定标准。
- 提交 ADR 之外擅自引入依赖或修改架构。

### 完成后

- 重新评估 T-005 的 Go/No-Go（**由用户拍板**，本任务只给证据与结论建议）。
- 若转 Go，候选组合优先评估「坚果云 + 百度」，但**仍不锁定**，需用户确认。
### 完成情况（2026-11-28）

**状态：已完成。** 结论：**百度整体判为「原生登录可行（静态可行，动态未验证）」**，三条路径分别判为「可独立实现」。相较 T-002c 的「需攻关」**发生变化**，变化依据见下表。

#### 产物

| 文件 | 规模 | 说明 |
|---|---|---|
| `docs/evidence/baidu-login-deep-dive.md` | **新建，857 行 / 74746 B** | 14 节 + 附录 A（勘误登记）+ 附录 B（可信度分布）；每条事实带 `文件:行号` 与三档可信度 |
| `docs/evidence/login-feasibility.md` | 728 → **825 行**，50552 → 60613 B | §3.2 整节改写（含新 3.2.6 复核表）；§2 总表、§4、§5、§6、§7、§8、附录 A/B 同步，全文前后无矛盾 |
| `docs/API_FACTS.md` | 492 → **511 行**，35759 → 39755 B | 百度节 12 项字段更新（第 3、9、10、12 项重写）；总表与全局状态同步；**其他八家一字未改** |
| `docs/PROVIDERS.md` | 267 → **300 行**，21008 → 26378 B | 百度行（能力矩阵 + 登录矩阵）同步；§4 后果段更新；新增 §6.7 |
| `docs/TASKS.md` | 1152 → 本节新增后 | 索引表 + 状态行 + 本节 + 阶段 0 列表 |
| `docs/RISKS.md` | 64 → **更新** | R28 拆分（三家 → 两家）、R5 更新、新增 R29；**未关闭任何未运行时验证的服务端风控风险** |
| `CHANGELOG.md` | 448 → **追加 T-006 条目** | |
| ADR | **无新增** | 本次不需要引入依赖 / 闭源 native / 架构变更（见 deep-dive §10.4） |

#### 八项工作逐条结果

| # | 工作 | 结果 |
|---|---|---|
| 1 | `tpl` / `appSignKey` 来源 | **APK 硬编码明文常量**（`kd/______.java:11` appId / `:14` 国内 appSignKey / `:15` tpl / `:17` 国际 appSignKey）；注入链 13 跳全部落行号；`setProductLineInfo` 全仓 4 处调用方已列全；`DebugSetKey` 覆盖仅 debug 生效（`kernel/debug/C_.java:82-88`）→ release 无运行期来源 |
| 2 | `di` 40 字段 | **40 项逐个定性**：必需 22 / 条件 2 / 可选 16（合计 40）；每行带 `文件:行号`、数据来源、条件逻辑、个人信息风险、Rust 复刻结论；服务端必需性整体标「待确认」 |
| 3 | `passport_core` 六 native | 逐个记录签名 / 参数 / JNI 符号地址 / 调用方 / 三路径关联；全 dex **7 处调用（5 文件）**，**三主链 0 处**；`nm -D` 恰 6 个 `Java_` 导出、无 `JNI_OnLoad`、`RegisterNatives` 0 命中；缺失时有 `catch (Throwable)` 兜底 → **不构成阻断**；另列检索范围 / 命令 / 结果 / 反射与动态注册排查 |
| 4 | 其他 native 安全库 | 8 个库逐个给加载入口 / 调用链 / 关联路径 / 证据 / 影响判断，**均与三条登录路径无调用关系**；另记录一条静态盲区（`libsofiresec.so` 运行时动态下载，不在 APK 内） |
| 5 | 三条路径分别审查 | A 8 跳、B 7 跳、C 4 步，各自完整字段表 + 加密实现 + 阻断依赖逐项检查；统一比较表 11 行 |
| 6 | Rust 可复刻能力 | 10 行清单，五种情形区分；`di` 需 Kotlin 提供的**最小通用输入 10 项**已列出（全为跨网盘通用系统信息，无百度专有语义） |
| 7 | native 移植必要性 | 三条路径**均不需要移植**；逐库给不可替代功能 / 依赖链 / ABI 风险；**不破坏「新增网盘只加 Rust Adapter」**；**不产生 ADR** |
| 8 | 整体四档结论 | **原生登录可行（静态可行，动态未验证）**；相对 T-002c 有 3 处修正 + 1 处勘误登记 |

#### 三处修正 + 一处勘误登记

| # | T-002c 记录 | T-006 实测 |
|---|---|---|
| 1 | `tpl`/`appSignKey` 运行期注入、静态未定位 | **APK 硬编码常量** |
| 2 | `passport_core` 待确认是否影响登录 | **三主链 0 调用 + `catch (Throwable)` 兜底** |
| 3 | `di` 40 字段受灰度裁剪、未分类 | **40 项逐项分类**；`mPrivacyParamesRegulation=false` 使 `dt_params` 实际不生效 |
| 勘误 | `docs/evidence/mvp-go-no-go.md:239` 写「`tpl`/`appSignKey` 静态不可得」 | **已被推翻**。该文件属 T-005 历史产物，**本次不改写**，只在 deep-dive 附录 A 登记 |

#### 验收核对（14 项）

| # | 验收项 | 结果 |
|---|---|---|
| 1 | tpl / appSignKey 找到赋值来源及调用链，或明确静态不可得原因 | **通过**：找到常量位置 + 13 跳注入链 + 4 处调用方 |
| 2 | di 40 字段逐个分类，有数据来源和判断依据 | **通过**：40 行全表，统计 22/2/16 = 40 |
| 3 | passport_core 六个 native 逐个追踪调用链 | **通过**：6 个方法 × 7 处调用点全列，附 JNI 符号地址 |
| 4 | 其他 native 安全库识别与三条路径的实际关联 | **通过**：8 个库 |
| 5 | 账号密码单路径结论 | **通过：可独立实现** |
| 6 | 短信单路径结论（不混） | **通过：可独立实现** |
| 7 | 扫码单路径结论（不混） | **通过：可独立实现** |
| 8 | Rust 可复刻性逐项判断 | **通过**：10 行清单 + 五种情形 |
| 9 | native 移植是否必要、原因和架构影响 | **通过**：不必要，不破坏架构，无需 ADR |
| 10 | 整体四档结论 + 相较 T-002c 变化 | **通过：原生登录可行**，3 处修正 |
| 11 | `baidu-login-deep-dive.md` 存在且每项可定位 | **通过**：857 行 / 74746 B，14 节 |
| 12 | 五份文档同步一致 | **通过**：login-feasibility / API_FACTS / PROVIDERS / TASKS / CHANGELOG（+ RISKS） |
| 13 | ADR 需要时仅提交草案 | **通过（不适用）**：本次无依赖/架构变更 |
| 14 | 安全：无真实凭据、无真实请求、reverse/ 未入 Git | **通过**：`git status` 无 reverse/（`.gitignore:2`）；文档无密钥值 |
| 15 | Git：diff 检查通过，提交与 push 状态明确 | **通过**：见本节末尾 |

> 注：任务卡「验收」小节原列 7 条，T-006 执行指令中另列 15 条验收标准；本节按用户 15 条逐条核对（其中第 6、7 条即卡内「三条路径分别结论」，第 14 条即卡内第 7 条）。

#### 遗留问题（详见 deep-dive §13 与 login-feasibility §8 第 8–12 行）

1. 服务端是否接受本项目构造的 `sig` / `di` —— **动态验证**，静态不可得（本任务禁止发请求）。
2. `diExceptIndex`（`sapi_options`）线上实际下发值 —— **动态验证**。
3. `BAIDUID` cookie 是否必须非空 —— **动态验证**。
4. **百度的「获取短信验证码」接口本次未取证** —— T-006 范围只到「已拿到验证码之后」的登录步骤；实现短信用 B 路径前需补一个小任务。
5. `AccountUtils.O/P/Q`（sofire 配置）是否被其它类赋值 —— **推测**，不构成登录阻断。
6. `libbaiduprotect_sec.so` 的 Java 侧调用链 —— 本次未追，不排除也不作为阻断证据。

#### 对后续决策的影响（只陈述，不做决定）

- 若按本结论计入，硬准入「原生登录可行」数量由 **1（坚果云）** 变为 **2（坚果云 + 百度）**，形式上满足 T-005 的 Go 判据「≥ 2」。
- **但**：①T-005 的历史 No-Go 结论**不改写**；②是否重跑 Go/No-Go 由用户决定；③即使转 Go 也不等于已锁定 MVP 组合；④两个「可行」的静态环境不同质（坚果云依赖腾讯闭源 Web 服务做人机验证；百度核心认证算法不依赖闭源 native，但公共参数 `zid` 依赖 `libfire` 与服务端数据、只能取兜底值），重跑前应先统一口径。

#### 提交

- commit：见 `CHANGELOG.md` T-006 条目与最终报告。


---

---

### T-006 补充收口（2026-11-28）

#### 任务性质

静态证据补充 + 结论校正 + 文档同步。**不写生产代码、不登录、不发请求、不抓包、不引入依赖、不移植闭源 native。** 基线 `0119b1f`，执行前后 `master` 无其它提交。

本轮只做 T-006 遗留的四处缺口，不重复 `tpl`/`appSignKey`/40 项 `di`/`passport_core` 六方法取证。

#### 四项补充结论

| # | 缺口 | 结果 | 结论 |
|---|---|---|---|
| 1 | `libbaiduprotect_sec.so` 加载与调用链 | **已闭环** | APK 内只有 `libbaiduprotect_sec.so`（133168 B）与 `libbaiduprotect_sec_jni_facect.so`（137264 B）；**不存在任何 `libbaidusec*` 文件**；无库 `NEEDED` sec.so、无 dlopen 字面量；两条加载入口（`WBAESCipherImplement.java:17` 静态 + `kh0/____.java:21` 动态插件，后者在本 APK 内缺 4 个依赖 so 走不通）；**仅服务人脸 WBAES、不参与请求构造、不阻断 A/B/C** |
| 2 | 短信发码接口 | **已取证到接口级**，但完整短信登录**仍不成立** | 发码 `POST {wap}/wp/api/login/sms`（前置 `POST passport.baidu.com/cap/init`，失败不阻断）；提交 `POST {wap}/wp/api/login` + `sms=1`+`smsvc`（**修正 T-006 记的 `/v2/sapi/login`**，那是旧 `dynamicPwdLogin` 链）；**阻断点 = `pass-machine.html` 的 WebView 人机验证**，静态不可复刻且 AGENTS.md 第 9 条禁止 WebView 登录 → **B 路径降级为「需攻关」** |
| 3 | 扫码 `hostDeviceId` 生成规则 | **已闭环且结论反转** | 官方客户端**从不赋值**（全仓 `.hostDeviceId =` 赋值 0 命中），恒为 `null`；`pass/http/a.java:333-353` 只拼非空 key/value → 参数**根本不出现**；服务端也不返回该字段。→ 本项目按官方缺省不传即可，**原「任意稳定随机串推测可用」作废** |
| 4 | `cuid` 生成算法 | **已定位官方算法** | `MD5("com.baidu" + ANDROID_ID)` 转大写；黑名单/空值 → `UUID.randomUUID()` + `FUUID` 前缀替换；机型指纹 `MD5(Build.MODEL)[3..15]`；派生字段 `i()` = 4 算子自定义摘要（含 xxHash32 + 反射 PRNG）+ 自定义 Base32，**纯 Java、无 native**；持久化在 `dataDir/.cesium/libbh.so`（数据文件非 so）。**`i()` 当前无法精确复刻**（需移植 `cesium.c.a.*` 且本机无 Java 运行时可产测试向量） |

#### 结论校正（以证据为准，不为维持原判放宽）

| 路径 | T-006 原判 | 补充收口后 | 变化原因 |
|---|---|---|---|
| A 账号密码 | 可独立实现 | **可独立实现** | 证据补齐，判定不变（前置 `cap/init` 失败不阻断，不构成阻断项） |
| B 短信验证码 | 可独立实现 | **需攻关** | 前置人机验证是 WebView 页面，静态不可复刻 |
| C 扫码 | 可独立实现 | **可独立实现（带动态前提）** | `hostDeviceId` 已闭环为「不生成」；残留前提是服务端是否接受该参数缺席 |
| 整体四档 | 原生登录可行 | **原生登录可行（不变）** | 存在 A、C 两条不含静态不可复刻环节的路径 |

另外两处表述校正：

- **扫码加 `sig`**：T-006 记「扫码不加 sig」，实测三个扫码请求都经 `HttpClientWrap.buildParamDTO` → `doSign`（全仓唯一调用点 `HttpClientWrap.java:86-88`）；扫码**加 sig、不发 `di`、不带 cookie**。
- **`di` 分类表述**：40 个位置**始终构造**，被 `diExceptIndex` 排除时**填空字符串而非删除字段**（`SapiDeviceInfo.java:67-187` 共 80 处 `add(`，其中 12 处显式 `add("")`，其余 68 处为三元表达式）；「可选」只描述官方客户端取值来源，**不表示服务端允许删除这些字段**；服务端对 40 项的校验强度**全部待动态验证**。

#### 原 T-006 验收项复核：哪些应判、哪些原不应判通过

| 原验收项 | 原判定 | 补充收口复核 | 说明 |
|---|---|---|---|
| 三条路径均判「可独立实现」 | 通过 | **改为部分不通过** | A、C 成立；**B 不应判「可独立实现」**，应判「需攻关」 |
| 短信路径取证完整 | 通过 | **原不应判通过** | T-006 只取到「已拿到验证码之后」的提交段，发码接口未取证 |
| 扫码路径取证完整 | 通过 | **原不应判通过（已补齐）** | `hostDeviceId` 当时只记「客户端生成」，未定位生成规则；本轮已闭环 |
| native 依赖已排除 | 通过 | **不变** | 本轮补 `libbaiduprotect_sec.so`，结论仍为不阻断 |
| 六类风控库全部定位 | 通过 | **不变** | 本轮另补 `libbaidusec*` 的对应关系（APK 内无此命名文件） |
| `di` 40 字段定性 | 通过 | **不变，但表述需收紧** | 字段与统计有效；须写明「填空串非删除」「服务端必需性未验证」 |
| 提交 commit 与 CI 记录 | 通过 | **不变** | — |

#### 文档同步清单（本轮）

| 文件 | 变化 |
|---|---|
| `docs/evidence/baidu-login-deep-dive.md` | 新增 **§15 T-006 补充收口**（15.0–15.9，含 11 行表 / 6 列步骤表 / 9 行 `hostDeviceId` 表 / `cuid` 逐跳表 / 五类复刻结论 / 更正登记表）；§1.2、§1.3、§5、§13、§13.1、附录 A、附录 B 加修正指引与关闭标记；**T-006 原文一律保留不删** |
| `docs/evidence/login-feasibility.md` | §2 总表百度行、§3.2.3 C 表与「不加 sig」结论、§3.2.6 复核表（新增第 6/7/8 行）、§3.2.7 三路径结论、**新增 §3.2.8 补充收口摘要**、§3.2.3 B 段加旧链纠偏、附录 A 增第 13 项修正与第 15–19 项命令、附录 B 增边界声明 |
| `docs/API_FACTS.md` | 全局状态加百度口径细化；百度第 3 项（短信端点）、第 9 项（分路径结论）、第 12 项（发码已取证 / 新增扫码动态前提）、待确认问题（关闭两项、新增两项）、T-006 增补②（扫码加 sig）、汇总表百度行 |
| `docs/PROVIDERS.md` | §3 百度行（路径细分 + B 阻断点）；**新增 §6.8 补充收口后的准入变化**（含与坚果云的口径对比表） |
| `docs/RISKS.md` | 升 **v0.7**；R29 扩充（扫码动态前提 + `cuid` 派生字段）；**新增 R30**（短信 WebView 人机验证）、**R31**（`hostDeviceId` 服务端接受度）；服务端风控类风险**一律不关闭** |
| `docs/TASKS.md` | 本小节 |
| `CHANGELOG.md` | 本轮条目 |

#### 未解决（保持开放）

1. 服务端是否接受本项目构造的 `sig` / `di`（R29）。
2. `diExceptIndex` 线上实际下发值（R29）。
3. `BAIDUID` 是否必须非空（R29）。
4. 短信人机验证在新设备上是否必然触发（R30）。
5. 服务端是否接受扫码 `hostDeviceId` 缺席（R31）。
6. `cuid` 派生字段 `i()` 的精确复刻（需移植 + 测试向量）（R29）。
7. `AccountUtils.O/P/Q` 运行时是否为空（推测，不阻断）。

#### 边界

本轮**不重跑 T-005、不锁定 MVP、不启动阶段 2**；`docs/evidence/mvp-go-no-go.md` 的历史记录未改写，勘误只在 deep-dive §1.2 指引与本节登记。

---

### T-006 最终一致性校正（2026-11-28）

#### 任务性质

静态证据矛盾校正 + 登录参数闭环 + 文档同步。**不写生产代码、不登录、不发请求、不抓包、不新增依赖、不移植闭源 native；不重新反编译 APK、不重跑 T-006。** 基线 `3af84933ae7b4716a66ad1b01ee8e451d98df94f`。

触发原因：复核发现四处口径不自洽——①`t_cuid` 与 `t_appname` 的字段来源被写反 ②`cuid` 的派生字段 `i()` 是否进入最终请求未证明 ③`sf_zid` 的 `"NoZidYet"` 与十六进制兜底值没有分支关系 ④短信人机验证 WebView 与坚果云 R27 判定口径不一致。

#### 四项闭环结论

| # | 缺口 | 结果 | 关键证据 |
|---|---|---|---|
| 1 | `t_cuid` / `t_appname` 来源 | **已闭环（旧文写反，已更正）** | `SapiDeviceInfo.java:53` 索引 **38 = `t_cuid`**、**39 = `t_appname`**；`:171` 索引 38 ← `confignation.mTPLCuid`，`:181-183` 索引 39 ← `confignation.mTPLAppName`；业务侧唯一调用点 `C_.java:199` 传 `("wangpan_drive", _____(), false)` |
| 2 | `i()` 是否进入最终请求 | **是，已逐跳闭环** | `C_.java:143` → `startup/task/l.java:50` → `DeviceId.java:98-100` → `cesium/h.java:168-186`（`__ == "V"` 时拼 `___ = i(f797_)`）→ 组合串 `"<cuid>|V<i(cuid)>"` → `di` 索引 38 → AES → 请求体。**不复刻则必然不一致** |
| 3 | `sf_zid` 取值分支 | **七分支 + 五种取值已闭环** | `sofire/d/b.java:645-679`；取值 = `"NoZidYet"` / 空串 / 静态缓存 / SP 文件解密值 / **62 位** hex 常量（旧文「56 位」错误）；公共参数 `zid`（`httpwrap/Utils.java:17`）与 `di` 索引 34 同源同值，但 `diExceptIndex` 排除 34 时 `di` 侧为空串 |
| 4 | 验证码 WebView 判定口径 | **与坚果云 R27 同口径** | `ValidationActivity.java:317-323` 只加载 `pass-machine.html`；JS→原生走 `onJsPrompt`（`:28-55`，**非 `addJavascriptInterface`**）；回传仅 `errno`/`msg`/`ds`/`tk`（`ValidationResult.java:4-21`）；`com/baidu/validation` 下 `grep CookieManager|bduss|stoken|ptoken` **0 命中**；认证仍由原生 `POST {wap}/wp/api/login` 完成 |

#### 另两项校正

| 项 | 原表述 | 校正后 |
|---|---|---|
| 人脸 native 对请求的影响 | 「不影响认证或请求构造：否」 | **不影响认证、签名、设备指纹**；但 `SapiAccountManager.java:149-153` 的 catch 会把 `supportFaceLogin` 置假，使 `SapiAccountRepository` 五处（`:801-803`/`:1549-1551`/`:1652-1654`/`:1795-1797`/`:2885-2887`）**少发三个可选能力参数** `liveAbility`/`scanface`/`supFaceLogin` |
| A 密码路径是否存在验证页面分支 | 旧文只记「前置 `cap/init` 失败仍继续」 | **存在**：`NewLoginUtil.java:2107-2145` 在错误码 **50052** 时走 `ValidationService` 人机挑战（`ds`/`tk` 写回 → `onAgainRequestPwdLogin` 重试）；另有整页网页登录兜底错误码清单 |

#### 结论变化（A/B/C 与整体）

| 路径 | 校正前 | 校正后 | 变化原因 |
|---|---|---|---|
| A 账号密码 | 可独立实现 | **可独立实现** | 档位不变；新增「带与 R27 同级的条件性人机验证依赖」说明 |
| B 短信验证码 | 需攻关 | **需攻关** | 档位不变；**理由改写**为「挑战通过条件静态不可复刻」+「是否接受内嵌同一 H5 待拍板」，不再写「违反 AGENTS.md 第 9 条」 |
| C 扫码 | 可独立实现（带动态前提） | **不变** | — |
| 整体四档 | 原生登录可行 | **原生登录可行（不变）** | 成立路径仍为 A / C |

> **一处需用户拍板**：若把坚果云 R27 的口径严格套到百度 B 短信上，B 与 A 同级；两者差别只在「百度官方对短信路径另有网页短信登录兜底」。本报告按保守侧维持「需攻关」，不自行上调。

#### 文档同步清单（本轮）

| 文件 | 变化 |
|---|---|
| `docs/evidence/baidu-login-deep-dive.md` | **新增 §16（16.0–16.6，299 行）作为当前有效结论唯一入口**；改写 §15.4.4（38/39 位）、§15.4.6 行③（`__` = `"V"` 非 `"O"`）与行⑤（`i()` 进请求）、§15.4.6 边界句、§3.3 表行 34（62 位）、§4.5（62 位）、§15.1.6 末行（人脸可选参数）、§15.7.2（降级理由）、§1.2（加再校正指引）；全文日期 2026-11-28 → **2026-11-28** |
| `docs/evidence/login-feasibility.md` | **新增 §3.2.9 最终一致性校正**（7 小节 + 差异表）；改 §3.2.7 第 3 层（B 理由）、§2 总表百度行、§6 风险表（新增 A 路径人机验证依赖行）、§7 注意二（口径差收窄）、§8 未解决问题第 11/12 项（关闭）、头部状态 v1.1 → **v1.3**；日期统一 |
| `docs/API_FACTS.md` | 全局口径细化段改写；百度第 9 项分路径（A 加条件性依赖）、第 12 项（`i()`/`t_cuid`/`zid` 三项新待确认）、待确认问题（新增 2026-11-28 字段口径校正行）、侦察进度表百度行；日期统一 |
| `docs/PROVIDERS.md` | §3 百度行（A 亦带人机验证依赖、B 理由改写）；**新增 §6.9 最终一致性校正之后的准入变化**（含「与 R27 对齐的禁止形态口径表」）；§6.8 与 §6.7 口径段补同质化说明；日期统一 |
| `docs/RISKS.md` | 升 **v0.8**；R29 补 `i()` 已进请求 + `sf_zid` 五种取值与 62 位；**R30 整条重写**（与 R27 同口径 + A 路径也有该分支）；**新增 R32**（整页网页登录兜底分支）；日期统一 |
| `docs/TASKS.md` | 本小节 + 头部「最后更新」 |
| `CHANGELOG.md` | 本轮条目 |

#### 未解决（保持开放）

1. 服务端是否接受本项目构造的 `sig` / `di`（R29）。
2. `diExceptIndex` 线上实际下发值（R29）。
3. `BAIDUID` 是否必须非空（R29）。
4. 服务端是否校验 `cuid` 派生字段 `i()`（R29）。
5. 服务端是否接受 `zid` 兜底值（62 位常量 / 空串 / `"NoZidYet"`）（R29）。
6. 人机验证（A 的 50052、B 的发码与提交）是否在新设备上必然触发（R30）。
7. 服务端是否接受扫码 `hostDeviceId` 缺席（R31）。
8. `B` 档位是否因口径统一而由「需攻关」上调——**需用户拍板**。

#### 边界

本轮**不重跑 T-005、不锁定 MVP、不启动阶段 2**；`docs/evidence/mvp-go-no-go.md` 的历史记录未改写。

### T-006 zid 的 native 依赖与二次查询收口（2026-11-28）

#### 任务性质

现有证据复核 + 文档一致性修正。**不写生产代码、不登录、不发请求、不抓包、不改依赖、不改架构、不移植闭源 native；不重跑 T-006，不重复 CUID/`i()`/人脸/人机验证取证。** 基线 `ba080ea30a3a5af53f86383f4b32b063256eb985`。

触发原因：深度报告 §16 已明确真实 `zid` 存在 native 依赖，但多份摘要仍写「百度三条登录路径均不依赖闭源 native」，口径不一致；且 `SapiDeviceInfo.java:242-249` 的第二次 `getZidAndCheckSafe(...,120)` 是否会改变最终 `di` 与公共 `zid` 未闭环。

#### 工作一：zid 的七条来源与 native 依赖

| 来源 | 需 native | 无 native 时 |
|---|---|---|
| 静态缓存 `sofire/d/b.a` | 读取否、写入依赖 | 停在初值空串或常量 |
| SP `xytk` / `xytk_m` | 读取否、写入由服务引擎 | 恒空串（`grep putString("xytk"` 全仓仅 sofire 包装类） |
| **SP `xyusec` 解密** | **是**（`sofire/l/b.java:257-266` → `l.e.a` → `Asc.dc`） | 空串 |
| **SP `xyus` 加密** | **是**（`sofire/l/d.java:12-62` → `l.e.b` → `Asc.ac`） | `byte[0]`，写不进去 |
| 服务端下发数据 | **是**（经 sofire 服务进程 / `k/c.java` 的 ContentProvider） | 拿不到 |
| 62 位硬编码常量 | 否 | **可取，仅兜底** |
| 空串 / `"NoZidYet"` | 否 | 可取（`"NoZidYet"` 由上层合成） |

**`F.getInstance().re` 直接依赖 native**：`sofire/ac/F.java:248-284` → 静态字段 `com.baidu.sofire.l.e.a` → `Asc.ar(bArr, bArr2)`，无 Java 回退；`sofire/jni/Asc.java:5-8` 静态块 `System.loadLibrary("fire")`。

**四问回答**：①不能稳定生成官方实际存在的真实 zid，只能得到兜底分支的值；②只能是 62 位常量或空串，没有第三种（UUID 分支产出的是未加密种子，无 native 时写不进 SP）；③没有其他静态可得值（六条通道已穷举，`loadLibrary("sofire")`/`loadLibrary("sofiresec")` 全仓 0 命中）；④服务端是否接受——**待动态验证（R29/R33）**，不推测。

#### 工作二：di 二次查询的完整时序

| 阶段 | 公共参数 `zid` | `di` 索引 34 | 已加密 | 已签名 |
|---|---|---|---|---|
| `HttpHashMapWrap()` 构造（`httpwrap/Utils.java:17`） | 事件 id **1** 写入 map | 未生成 | 否 | 否 |
| A `:484-535` / B `:2710-2723` put 字段 | 不变 | 未生成（**都不 `put("di")`**） | 否 | 否 |
| `buildNaCookie` 求值（`ParamsUtil.java:75` → `SapiDeviceInfo.java:67`） | 不变 | 事件 id 1 生成，`:229-254` 末尾加密 | 是 | 否 |
| `sf_zid == "NoZidYet"` 重查（`:242-249`） | **不回写 map** | 事件 id **120** 覆盖 `sf_zid`，加密**前** | 是 | 否 |
| `doSign`（`HttpClientWrap.java:84-105`） | **参与签名** | 不参与（在 cookie `DVIF` 里） | — | 是 |

**五项判断**：①两次取值**可能不同**（事件 id 不同、中间可能有缓存写入）；②A/B 上公共参数与 `di` **互不覆盖**，但取值可能不同；③二次查询在加密**前**；④`zid` **参与签名**，`DVIF` 不参与；⑤A 与 B 同构，**C 扫码 `cookie = null`，不触发 `DVIF` 与二次查询**，公共参数仍照发并签名。

#### 工作三：文档口径修正

统一表述（所有摘要一律沿用）：**百度核心认证算法（RSA 密码加密、`sig` 签名、`di` 的 AES 加密）不依赖闭源 native，但公共参数 `zid` 的真实值存在 native（`libfire`）与服务端下发数据依赖。本项目可尝试复刻官方纯 Java 回退路径，该路径只能产出 62 位硬编码常量或空串，服务端是否接受兜底值尚待动态验证。**

| 文件 | 修改点 |
|---|---|
| `docs/evidence/baidu-login-deep-dive.md` | §1.2、§12.2 第 2 行、§15.9 表首行改写；新增 §17（17.0–17.10） |
| `docs/evidence/login-feasibility.md` | §3.2.7 第 3 层、§7 注意二改写；新增 §3.2.10（6 小节） |
| `docs/API_FACTS.md` | 全局说明、百度第 9 项依据强度、汇总注改写 |
| `docs/PROVIDERS.md` | §3 百度行、§6.7 注意段改写；§6.7/§6.8 标注已被取代；新增 §6.10 |
| `docs/RISKS.md` | R29 补 zid native 依证据；新增 **R33** |
| `docs/TASKS.md` | 本小节 |
| `CHANGELOG.md` | 本条目 |

#### 边界与未解决

A 可独立实现（带条件性人机验证依赖）/ B 需攻关（等用户决定是否接受 H5）/ C 可独立实现（`hostDeviceId` 缺席待验证）—— **三档均未改动**。`i()` 已进入 `t_cuid`，必须复刻；真实 `zid` 当前不能精确复刻，兜底值接受度待动态验证（R29/R33）。本轮**不重跑 T-005、不锁定 MVP、不启动阶段 2**，`docs/evidence/mvp-go-no-go.md` 未改写。

未解决：①服务端是否接受 62 位常量 / 空串 / `"NoZidYet"` 作为 `zid`（R33）②两次 zid 查询在真实环境是否确实返回不同值（需抓包，本轮不做）③`xytk` 的实际来源与刷新频率（服务端行为）④`libfire.so` 的 10 个 native 算法是否可等价重写（本轮不做，需 ADR）。
---
## T-101（阶段1）：定义 UniFFI 深接口

- 阶段：1
- 状态：**已完成**
- 依赖：T-002e（CI 构建链已具备）
- 产出：`rust/core/src/{model,error,provider,ffi}.rs` + `app/src/main/kotlin/com/netdiskhub/app/ProviderProbe.kt`

### 目标

把「一个 Android 客户端管理多家网盘」的**通用**能力固化成 Rust 侧统一接口，
并通过 UniFFI 暴露给 Kotlin。Kotlin 只能看到通用概念，不得接触任何网盘专有字段。

### 交付内容

| 项 | 落地位置 | 说明 |
|---|---|---|
| 统一基础模型 | `rust/core/src/model.rs` | `RemotePath`（自带规范化，拒绝 `.`/`..`/控制字符）、`RemoteFile`、`ListOptions`、`Page`、`Quota`、`TransferOpts`、`TransferReport`、`ProviderInfo` |
| 统一认证输入 | `rust/core/src/model.rs` | `AuthInput` 三态：`Password` / `SmsCode` / `ScanCode`，与 AGENTS.md 第 9 条一致（无 OAuth / WebView / cookie） |
| 会话抽象 | `rust/core/src/provider.rs` + `ffi.rs` | `SessionContext` 内部持有不透明 id；导出层 `Session` 是 `uniffi::Object`，**Kotlin 拿不到任何字符串** |
| 统一错误 | `rust/core/src/error.rs` | `NetdiskError` 十类：AuthFailed / HumanVerificationRequired / RateLimited / Network / InvalidArgument / Unsupported / NotFound / AlreadyExists / InvalidSession / Internal |
| Provider 操作 | `rust/core/src/provider.rs` | `ProviderBackend` 九方法：`info`/`connect`/`list`/`download`/`upload`/`remove`/`rename`/`mkdir`/`quota` |
| UniFFI 导出 | `rust/core/src/ffi.rs` | `NetdiskClient`（句柄对象）+ `providerIds()`；**不导出 trait**，方法逐个转发 |
| Kotlin 编译期验证 | `app/src/main/kotlin/com/netdiskhub/app/ProviderProbe.kt` | 引用全部通用类型与方法，类型不符即编译失败 |

### 关键设计决定

1. **不导出 Rust trait**：UniFFI 0.32.2 支持 `#[uniffi::export] trait`，但导出 trait 会把实现自由度写进对外契约。这里只用 `#[derive(uniffi::Object)]` 句柄 + 普通方法，内部 `Arc<dyn ProviderBackend>` 保持不变。
2. **会话不透明**：`Session` 不暴露 `id`、不暴露 token。Kotlin 只能把 `Session` 原样传回，明文凭据永远不跨边界。
3. **路径在 Rust 侧收口**：`RemotePath::parse` 是唯一入口，拒绝 `..` 与 `..` 段、折叠 `/`，避免每个 Adapter 各写一套拼接导致行为分叉。
4. **探针保留**：`netdiskhub_probe` 与 `probe_magic` 原样保留（CI 的 ELF 符号检查与绑定检查都用它们）。

### 验收

- [x] 统一模型、认证输入、会话、错误、九项操作全部落地，无网盘专有字段
- [x] UniFFI 绑定生成成功，Kotlin 侧引用全部通用类型并通过编译（CI 证明）
- [x] 绑定内无 `token` / `cookie` / `bduss` / `stoken` 等凭据字段（CI 断言）
- [x] 未新增未经批准的依赖；未实现真实网盘请求

### 完成情况（2026-11-28）

- 静态自检：`cargo test --workspace --locked` 在宿主目标通过，12 个测试全绿
- 绑定自检：本地生成 `netdiskhub_core.kt` 2985 行，`providerIds` / `NetdiskClient` / `Session` / `NetdiskException` / `RemotePath` / `quota` / `mkdir` / `stat` 全部命中；凭据字段 0 命中
- 运行时（Kotlin 编译 + 打包 + 设备执行）由 GitHub Actions 证明，见 T-102 完成情况

---

## T-102（阶段1）：实现 MockProvider（两个假 Adapter 验证 seam）

- 阶段：1
- 状态：**已完成**
- 依赖：T-101
- 产出：`rust/core/src/{mock_common,mock_alpha,mock_beta}.rs`

### 目标

用两个**行为不同**的假 Adapter 证明统一接口不是为某一家特制的（对应 ARCHITECTURE.md §2 的真 seam 判定：至少两个实现）。

### 两个实现的行为差异

| 维度 | MockAlpha | MockBeta |
|---|---|---|
| Provider id | `mock-alpha` | `mock-beta` |
| 认证方式 | 密码 / 短信 / 扫码 三种全支持 | 只有密码 / 短信，**扫码返回 `Unsupported`** |
| 默认页长 | 2 | 3 |
| 单页上限 | 无额外限制 | `page_size > 100` 返回 `InvalidArgument` |
| 虚拟根目录 | `documents/`、`notes.txt`、`photo.png`、`readme.md` | `a.txt`、`b.txt`、`shared/` |
| 测试数据 | `documents/archive/`、`documents/report.txt` | `shared/team/`、`shared/memo.txt` |
| 总容量 | 1 GiB | 1 TiB |

两者共享 `mock_common.rs` 的内存虚拟文件系统（目录树、上传/下载/续传/重命名/删除/配额语义），登录与服务差异留在各自 Provider 里——这样「文件操作语义一致」与「登录能力可不同」两件事互不干扰。

### 契约测试

`provider.rs` 里的 `run_conformance()` 是**唯一**一套断言，两个 Mock 各自跑一遍，覆盖：

- 认证：错误密码 → `AuthFailed`；正确密码 / 短信 → 签发会话；扫码按 Provider 能力分别断言成功或 `Unsupported`
- 列目录：根目录含各自种子目录；`page_size=2` 逐页取完后条目集合与一次性取完**完全一致**（用 `assert_eq!(seen, expected)` 比对，不只是比数量）
- 配额：`totals` 非空、`used <= total`
- 目录：`mkdir` 成功；重复 `mkdir` → `AlreadyExists`
- 传输：上传字节数等于本地文件长度；同名未覆盖 → `AlreadyExists`；`overwrite=true` 可覆盖；下载内容逐字节相等；`resume_from=6` 时只写剩余字节且结果完整
- 重命名：改名后在新目录能列出
- 错误：删除不存在 → `NotFound`；下载不存在 → `NotFound`；`..` 路径 → `InvalidArgument`；父目录不存在上传 → `InvalidArgument`；伪造会话 → `InvalidSession`

### 验收

- [x] 两个 Provider 实现同一 trait，各自独立虚拟目录与数据
- [x] 登录成功与失败场景均有测试
- [x] 列目录 / 分页 / 配额 / 上传 / 下载 / 删除 / 重命名 / 新建文件夹全部覆盖
- [x] 无效路径、无效会话、重复文件名、目标不存在均返回明确错误分类
- [x] **同一套契约测试**对两个实现都通过
- [x] 测试数据全部虚构（`example.test` 域名、`1300000000x` 手机号、编造凭证）

### 完成情况（2026-11-28）

- 本地 `cargo test --workspace --locked`：12 个测试全绿，其中 2 个是契约测试（Alpha / Beta 各一）
- CI：`build.yml` 与 `rust.yml` 均新增 `cargo test --workspace --locked` 步骤；Android 构建、双 ABI 检查、UniFFI 绑定检查、APK 原生库检查全部保留
- 踩坑记录：两个 Mock 的契约测试并行执行时会争用同名临时文件（`Os { code: 2, kind: NotFound }`），已改为按 Provider id 分文件

## 阶段 0：控制面与接口侦察

- T-001 建立项目控制面与接口盘点（进行中）
- **T-002a APK 清单合并（已完成）**
- **T-002b 反编译 1 个网盘跑通索引格式（已完成）**
- **T-002c 各网盘登录流程侦察与可行性判定（已完成）**
- **T-002d 反编译百度 + Flutter 网盘壳确认（已完成）**
- **T-005 Go/No-Go 决策与 MVP 网盘锁定（已完成：No-Go，未锁定 MVP）**
- **T-006 百度网盘原生登录攻关与第二家准入验证（已完成：百度改判「原生登录可行」，硬准入数量 1→2，T-005 结论未重跑）**

## 阶段 1 前置

- **T-002e GitHub Actions CI 基础设施搭建（进行中：第 1、2、3 轮完成，test / quality-gate 待做，优先级最高）**
- **T-003 UniFFI 构建链最小化集成验证（待启动，依赖 T-002e；构建链部分已由第 3 轮覆盖，剩余为运行时端到端）**
- **T-004 CRAP / 变异测试工具兼容性复核（待启动，依赖 T-002e）**

## 阶段 1：Provider 抽象 + MockProvider

- **T-101 定义 UniFFI 深接口（已完成：通用模型 / 认证输入 / 不透明会话 / 统一错误 / 9 个 Provider 操作 / UniFFI 句柄导出）**
- **T-102 实现 MockProvider（已完成：MockAlpha + MockBeta 两个实现，同一套契约测试全部通过）**
- T-103 Kotlin UseCase 层与契约测试
- T-104 CRAP ≤6 与 lint/typecheck/test 门禁接入

## 阶段 2：单网盘纵向切片

- T-201 首个网盘登录（原生协议）
- T-202 列目录 / 下载
- T-203 上传（含分片）
- T-204 删除 / 重命名 / 新建文件夹
- T-205 变异测试 ≥90%

## 阶段 3：第二个网盘验证抽象

- T-301 第二个网盘接入（验证 seam 是否真成立）
- T-302 接口不一致处收敛到 Rust 内部
- T-303 Provider 核心变异目标 100%

## 阶段 4：UI / 多账号 / 任务队列

- T-401 账号管理页（P1）
- T-402 统一文件列表（P2）
- T-403 文件操作（P3）
- T-404 任务队列（P4）
- T-405 设置（P5）

## 阶段 5：硬化与接口变更降级

- T-501 接口变更检测与降级策略
- T-502 错误诊断能力
- T-503 安全审计