# 更新日志（CHANGELOG）

本项目遵循「一个任务一条记录」的粒度。

## [未发布]

### T-002d 百度完整反编译 + 四网盘壳确认（2026-11-27）

#### 新增

- `docs/evidence/index-百度.md`（43508 B / 590 行）：百度网盘完整静态索引，按 `index-template.md` v2.1 的 11 节 + 附录 A/B/C
  - §1 APK 元数据 + 主体技术栈判定（**Java/Kotlin 主体 + Flutter 模块共存**）
  - §2 反编译工具与「单次全量失败」证据；§2.1 覆盖率 **100.0000%**（63259/63259，missing 0）
  - §3 双 HTTP 链路（自研 HttpURLConnection + OkHttp3/Retrofit2）、§3.2 域名常量表
  - §4 认证（sapi2 SDK、5 个原生登录 Activity、`sapi_system` 持久化，**不判定可行性**）
  - §5 文件列表（`/api/list`，分页 = **offset**）、§6 上传（`precreate` → `superfile2` 分片 4 MiB 起 → `create?a=commit`，硬限 4 GiB）
  - §7 下载（`locatedownload`，客户端已实现 Range）、§8.2 **接口清单统一表**（五组 40+ 行，全带调用点行号）
  - §9 加固核查（未加壳、**证书绑定未发现**、`network_security_config` 信任用户 CA、303 处 native）+ §9.1 风控 SDK 6 类
- `docs/evidence/index-天翼.md`（10940 B / 205 行）：壳确认模式，爱加密载荷 + 3 个壳桩类，协议覆盖 **0**
- `docs/evidence/index-123云盘.md`（8069 B / 182 行）：壳确认模式，4 个壳桩类，协议覆盖 **0**
- `docs/evidence/index-夸克.md`（11176 B / 193 行）：壳确认模式，**7z 容器 + 1 字节占位 so + 假 dex APK**，协议覆盖 **0**
- `docs/evidence/index-阿里云盘.md`（10304 B / 187 行）：壳确认模式，混合架构 + 阿里聚安全 8 文件，协议覆盖 **0**

#### 修改

- `docs/API_FACTS.md` v0.3 → **v0.4**：重写百度条目（登录流程 11 字段 + 待确认 + 已知线索 + 证据路径）；
  天翼/夸克/123/阿里四条改为壳确认结论（协议字段一律「未知」并说明原因）；汇总表新增「覆盖率」列，
  并把 Flutter 四行的「登录流程已填/文件接口已填」改为「未知（无静态证据）」
- `docs/PROVIDERS.md` v0.3 → **v0.4**：能力矩阵百度行填满静态线索、四个 Flutter 网盘行改「壳确认完成（协议未知）」；
  登录能力矩阵四行改「未知（待 T-002c）」并加注壳确认状态；§5 资料覆盖说明实测结论；新增 §6.4（T-002d 对切片建议的影响）
- `docs/TASKS.md`：T-002d 状态改**已完成**，索引表同步；新增「完成情况（2026-11-27）」小节（产物表 / 反编译过程 /
  四网盘壳确认对比 / 关键结论 / 17 项验收核对 / T-002c 启动条件核对）；T-002c 状态补「启动条件已满足，等待用户确认」
- `docs/evidence/apk-inventory.md`：补「T-002d 实测修正」表——百度改「Java/Kotlin 主体 + Flutter 模块共存」、
  天翼 versionCode 1112 → **638**、minSdk 21 → **24**；§6 反编译状态改为「部分完成」并列出 2 个完整 + 4 个壳确认 + 3 个不反编译
- `CHANGELOG.md` 本条

#### 说明

- 单次全量 jadx 反编译 182 MB dex **在本机失败**（-Xmx5g，进度 46% 时被系统内存压力终止）；
  改为按 dex 分批（`-Xmx3g -Xss2m`）后才完成。失败日志保留在 `reverse/baidu/jadx-full.log`
- 覆盖率口径新增两条要求（已写入 index-百度 §2.1，并建议回填模板）：识别 jadx 改名注释
  `/* JADX INFO: renamed from: */` 与 `p000` 非法包名前缀；不识别会得出 941 个假缺失
- 中途 25 个类报 `DONT_GENERATE`，需加 `--no-inline-anonymous --no-inline-kotlin-lambda` 才能 `--single-class` 补出
- **未判定任何网盘的登录可行性**（属 T-002c）；未对 Flutter 网盘深挖 `libapp.so`；未做动态分析；
  未新增依赖、未改架构/Gradle/Rust/CI；`reverse/` 未进 Git

---


### T-001 建立项目控制面（2026-11-26）

#### 新增

- `docs/PROJECT_CHARTER.md` 项目章程，含登录方式硬目标与判定表
- `docs/SCOPE.md` 范围冻结、登录非目标、兜底策略、分阶段质量门禁表
- `docs/ARCHITECTURE.md` Android 分层、UniFFI 深接口草图、登录模块归属、工程基线核查
- `docs/PROVIDERS.md` 9 网盘能力矩阵、登录能力矩阵、准入规则、纵向切片建议
- `docs/API_FACTS.md` 接口事实库骨架，每网盘含登录流程字段与待确认问题
- `docs/SECURITY.md` Keystore token 存储、日志脱敏规则、凭据纪律
- `docs/TASKS.md` 阶段 0–5 任务拆分，T-001 任务卡
- `docs/RISKS.md` 风险登记 R1–R13
- `docs/design/UI_DIRECTION.md` UI 方向（Compose + Material 3，P1–P5 页面）
- `docs/evidence/apk-inventory.md` APK 清单与结构特征
- `CHANGELOG.md` 本文件

#### 修改

- `AGENTS.md` 新增硬约束第 9 条：登录必须是 APP 内原生登录

#### 未完成 / 阻塞

- 反编译未开始（等待确认），因此无 `docs/evidence/index-<网盘名>.md`
- `docs/API_FACTS.md` 无任何已验证接口记录
- 工程基线两个依赖坐标待确认：`itos-cc 0.14.3`、`pl.droidsonroids.pitest 0.2.25` 均不存在
- NDK r29 未核实

---

### T-002a APK 清单合并（2026-11-26）

#### 新增

- `docs/ADR/ADR-001-crap-tool.md` 提案：CRAP 工具由 `itos-cc 0.14.3`（不存在）改为 `cargo-crap 0.10.0`
- `docs/ADR/ADR-002-kotlin-mutation.md` 提案：Kotlin 变异插件由 `pl.droidsonroids.pitest 0.2.25`（不存在）改为 `0.3.0`
- `docs/ADR/ADR-003-compilesdk-ndk.md` 提案：compileSdk 写死 36、targetSdk 36；NDK r29 标"待确认"

#### 修改

- `docs/evidence/apk-inventory.md` v0.2：合并附件区 12 个文件完整清单（8 个与网盘相关 + 4 个无关），
  补 minSdk/targetSdk、sha256 全量、百度 `.apks` 分包明细、加固特征表、反编译价值排序
- `docs/ARCHITECTURE.md` v0.2：compileSdk/targetSdk 写死 36；CRAP 与变异工具改为 ADR 提案值；
  待确认项改为表格并标注是否阻塞阶段 1
- `docs/PROVIDERS.md` v0.2：新增"资料缺失"状态；蓝奏云 / UC 网盘 / 迅雷云盘 定性为
  **不可行（资料缺失）**；补充"Flutter 主体"含义说明；纵向切片建议加"待复核"标注
- `docs/API_FACTS.md` v0.2：三个资料缺失网盘的「登录流程」字段全部改为"资料缺失"，
  登录可行性结论写 **不可行（资料缺失）** 并附证据说明
- `docs/SCOPE.md`：兜底策略新增"资料缺失的兜底策略"三条（不硬凑 / 不降级 / 可重开）
- `docs/TASKS.md`：T-002 拆分为 T-002a（已完成）与 T-002b（待启动），补两张任务卡
- `docs/RISKS.md`：R12 关闭；新增 R14（NDK 未核实）、R15（5/6 候选网盘是 Flutter 主体）

#### 结论

- 9 个网盘中：**6 个有可用资料**（百度、天翼、夸克、坚果云、123、阿里），
  **3 个资料缺失**（蓝奏、UC、迅雷）
- 6 个可用资料里 **4 个是 Flutter 主体**（天翼、123、夸克、阿里），静态反编译价值低
- 静态反编译真正有价值的只有 **坚果云 + 百度网盘**

#### 未完成

- T-002b 未启动（等用户确认靶子）
- ADR-001/002/003 均为提案，未批准

---

### T-002b 反编译 1 个 APK 跑通索引格式（2026-11-26）

#### 新增

- `docs/evidence/index-坚果云.md`（25612 B / 342 行）：10 个必填区块 + 「格式评估」小节，
  每条线索带 `文件:行号` 与可信度标注
- `docs/evidence/tooling.md`：jadx v1.5.7 来源 URL、74766896 B、本地自算 SHA256
  `1755e4242ade71a077438723689f404079592ec7764a3a0572f98fef94ebcbf3`、OpenJDK 21.0.13.1
- 反编译产物 `reverse/nutstore/sources/`（8006 个 .java，约 67 MB；已在 `.gitignore`，不进 Git）

#### 修改

- `docs/TASKS.md`：T-002b 状态 进行中 → **已完成**，补「完成情况」小节
  （产物表 / 工具 / 覆盖率 / 关键技术发现 / 8 项验收核对 / 格式评估摘要）
- `docs/PROVIDERS.md` v0.3：
  - 能力矩阵坚果云一行由全「未知」改为**静态线索**（认证、列目录、下载、上传、删除重命名、分页、限速、验证码、加固特征）
  - 登录能力矩阵加注：登录可行性判定属 T-002c，本表暂不填
  - 第 6 节「纵向切片建议」重写为 6.1 原建议（保留原文）+ 6.2 复核结论 + 6.3 候选组合表
- `docs/API_FACTS.md` v0.3：
  - 全局说明改为"仅坚果云有静态线索"，明确「已验证」指代码写法而非服务端响应
  - 坚果云「登录流程」11 个字段由「未知」改为静态线索并标可信度
  - 新增「文件操作接口」表（11 个接口，路径 / 方法 / 参数 / 定位）
  - 「待确认问题」重写为 6 条具体问题；「证据路径」补全
  - 新增「汇总：侦察进度」表（9 个网盘的静态反编译状态 / 主体技术栈 / 已填项）

#### 结论

- **jadx 反编译覆盖率不是 100%**：nutstore.android 包 2019 个非匿名类，jadx 只产出 1912 个，
  缺 107 个（5.3%），其中包括下载核心类 `RemoteFileDownloader`（已用 `--single-class` 补出）。
  失败原因未保留 → 待确认。
- **坚果云无加固、无证书绑定、无设备指纹采集**。字符串用每类自带 XOR 解密器混淆
  （155 个解密器、4804 处调用），已建 keyMap 全量解密。
- **T-002b 实测推翻了 PROVIDERS 第 6 节原建议的一条事实依据**：原文写"天翼已确认非 Flutter 主体"，
  与 `apk-inventory.md` §3/§3.1 的记载矛盾 —— 天翼实为 **Flutter 主体**。已在 6.2 节更正。

#### 格式评估结论（`index-坚果云.md` §11）

- **必增 4 项**：①主体技术栈判定 ②反编译覆盖率（总数/产出/缺失）③加固与风控 SDK 清单
  ④接口清单统一成一行式表格
- **建议降级 2 项**：域名清单只留 API 相关域；HTTP 链路细节注明"仅 Java 主体适用"
- **不建议改**：行号定位、可信度三档、"未发现须写搜索方式"
- 对 Flutter 网盘的预期：§3–§8 会全部落到"dex 内未发现"，届时应先探一轮再决定是否继续投入

#### 未完成

- ADR-001/002/003 均为提案，未批准
- T-002c（登录流程侦察）未启动
- 「格式评估」的 6 处改动**尚未落到模板**（等用户确认）

## 2026-11-26

- 锁定工具链基线：AGP 9.3.1 / Gradle 9.7.0 / Kotlin 2.4.21 / JDK 17 / Build Tools 36.0.0 / NDK r30 / Rust 1.100.0 / cargo-ndk 4.1.2 / UniFFI 0.32.2
- compileSdk / targetSdk 锁定 37
- 移除 Gobley，改用 cargo-ndk + 自定义 Gradle Exec 任务
- 新增 T-003 UniFFI 集成验证、T-004 CRAP/变异工具兼容性复核
- ADR-003 重写，ADR-001/002 追加兼容性复核项

### 任务编号冲突修正（2026-11-26）

- `docs/TASKS.md` 顶部新增**编号索引表**（7 个任务 + 编号规则）
- 旧 T-003「各网盘登录流程侦察与可行性判定」→ **T-002c**
- 旧 T-004「Go/No-Go 决策与 MVP 网盘锁定」→ **T-005**
- 全局扫描 `docs/` + 根目录 md，确认**无重复编号**

### 新增 T-002c / T-005 任务卡（2026-11-26）

- 新增 **T-002c 各网盘登录流程侦察与可行性判定**任务卡（依赖 T-002b，产出 API_FACTS/PROVIDERS 填满 + `docs/evidence/login-feasibility.md` 汇总表）
- 新增 **T-005 Go/No-Go 决策与 MVP 网盘锁定**任务卡，决策标准已定死（执行待 T-002c 完成）


### 索引模板 v1.0 建立 + T-002d 任务卡（2026-11-26）

#### 新增

- `docs/evidence/index-template.md` **索引模板 v1.0**（11 节）。把 T-002b 格式评估的 6 处改动
  固化成正式模板，后续所有 `index-<网盘名>.md` 照此产出：
  - 必增 4 项：主体技术栈判定 / 反编译覆盖率 / 加固风控 SDK 清单 / 接口清单统一表（5 列）
  - 降级 2 项：域名清单只留 API 相关域（其余入附录）；HTTP 链路细节注明"仅 Java 主体适用"
  - 不变项 3 条：行号定位 / 可信度三档 / "未发现"必须写搜索方式
- `docs/TASKS.md` 新增 **T-002d 反编译百度 + Flutter 网盘壳确认**任务卡（第 281 行起）。
  百度完整反编译产出 `index-百度.md`；天翼/123/夸克/阿里只做壳确认，明确标"协议需动态抓包"。
- `docs/TASKS.md` 编号索引表新增 T-002d 一行（状态：待启动）。

#### 修改

- `docs/TASKS.md` **T-002c 依赖改为 T-002d**（原依赖 T-002b）。原因：T-002c 要在 9 个网盘上
  判登录可行性，只有坚果云一个样本会产生大量"未知"，先补百度（Java 主体）与 4 个 Flutter
  网盘壳确认，凑够样本再启动。
- `docs/evidence/index-坚果云.md` §11 末尾加一行来源说明：本索引已作为 index-template.md v1.0 的来源。

#### 说明

- 已产出的 `index-坚果云.md` **不回填新模板**（它本身就是模板的来源），但已标注来源关系。
- 决策依据（用户拍板）：Git 先提交固化里程碑；模板先落地再批量反编译，避免重复踩坑；
  T-002c 暂缓，让 T-002d 抢在它前面。

### 坚果云覆盖率口径复核 + 索引模板合并为 v2.1（2026-11-26）

#### 覆盖率口径复核（先解释，再改数）

- `docs/evidence/index-坚果云.md` §2.1 重写为**口径 v2**，新增 **§2.1.1 口径说明**（回答四个问题）：
  ①原口径是什么 ②「内联」具体指什么 ③重算依据什么工具/命令 ④107 条是否确实全部以其他形式存在（列 7 个样例）
- **新口径结果**：dex `class_defs` 内 `nutstore.android` 包共 **2753** 个类定义（含 720 个含 `$` 的类），
  覆盖 **2752 个 = 100.0%**，唯一未定位类 `WorkspaceRepository$Companion` 零业务逻辑 → **无信息损失**
- 分解：1966 主产物 + 78 二次补出 + 591 内联注释 + 97 成员类声明 + 20 synthetic 空类 + 1 未定位 = 2753
- **107 条旧清单逐条定性**：76 条已 `--single-class` 补出、19 条是 `ACC_SYNTHETIC` 且方法数 0 的空类、
  12 条 dex `class_defs` 里根本不存在 → **含业务逻辑的真实缺失 = 0**
- 旧分母 2019 也复现不出（不含 `$` 的类实为 2033），旧统计脚本未保留 → **旧数字整体作废**
- 新增可复现脚本 `reverse/nutstore/coverage-check.py`（8710 B，纯 python 解析 dex `class_defs`，不依赖 jadx/baksmali）
  与证据 `reverse/nutstore/sources-recovered/`（97 个 .java，946 KB，含 `RemoteFileDownloader` 18546 B、
  `WorkspaceRepository` 19113 B、`utils/xa` 17455 B）；两者均在 `reverse/` 下，不进 Git
- `docs/RISKS.md` **R21 关闭**：标题改为「jadx 反编译覆盖率统计口径易误判」，证据栏写明复核结果，
  状态标「已核实为统计口径问题，非真实缺失」；缓解措施保留三条（按 `class_defs` 算 / `--log-level debug` / 补出后查文件大小）
- 同步修正其余文档的旧数字：`docs/API_FACTS.md`（待确认条目关闭）、`docs/TASKS.md` T-002b 完成情况、
  `docs/evidence/apk-inventory.md` §4；`docs/evidence/index-坚果云.md` §10 汇总表与 §11.2 的旧结论一并改口径

#### 索引模板合并为 v2.1

- `docs/evidence/index-template.md` 升级为 **v2.1**：以 v2.0 详细版（319 行）字段为主体，
  合并 v1.0 精简版（88 行）的 4 增 / 2 降 / 3 不变标注。文件头写明「由 v2.0 详细版 + v1.0 精简版合并而成」
  与「本模板为 T-002d 及后续所有网盘的唯一索引模板」
- 新增 §0 改动清单：**4 项【新增】**（主体技术栈判定 / 反编译覆盖率 / 加固风控 SDK 清单 / 接口清单统一表）、
  **2 项【降级】**（域名只留 API 相关域 / HTTP 链路细节单独一节注明「仅 Java 主体适用」）、
  **3 项【不变】**（行号定位 / 可信度三档 / 「未发现」必须写搜索方式）；并在正文对应位置逐个标注 `【新增①】`…`【不变⑨】`
- 新增附录 C 三条不变项速查（含反例）；§2.1 补「统计口径（照坚果云 §2.1.1 抄）」与 `--log-level debug` 命令模板
- **删除 `docs/evidence/INDEX_TEMPLATE.md`**（v2.0 已被合并进去），`docs/evidence/` 下现只剩一份模板
- `docs/TASKS.md` T-002d 的 3 处模板引用改为 **v2.1**；约束里的「覆盖率低于 95% 按 R21 处理」
  改为「必须按 §2.1 口径 v2 算，不许只写百分比」
- `docs/evidence/index-坚果云.md` §11 末尾来源说明改为：本索引是 v1.0 的来源，v2.1 由 v1.0 + v2.0 合并

#### 说明

- 覆盖率口径复核**先写口径说明再改数字**，不是直接把 5.3% 改成 100%；R21 的关闭有 76/19/12 三条可回查依据。
- 模板合并后行数从 319 → 360，字节 11012 → 15238；v1.0 的「React Native / 混合」措辞未采纳（本项目 9 个样本只出现 Java/Kotlin、Flutter、NestedWeb 三类）。

### CI-first + 公开仓库配置（2026-11-26）

#### 新增

- `README.md`：项目定位（个人自用 / 非官方客户端 / 不分发 APK）、技术栈、状态、构建说明（本地不可编译，走 GitHub Actions）、许可
- `LICENSE`：MIT
- `.github/workflows/` **尚未创建**（属 T-002e 范围），README 指向该目录为后续产物
- `docs/TASKS.md` 新增 **T-002e GitHub Actions CI 基础设施搭建**任务卡（阶段1前置，优先级高于 T-003 / T-004）：
  7 类 workflow（Kotlin 编译 / Rust 交叉编译 / UniFFI 绑定生成 / APK 打包 / 单元测试 / CRAP 门禁 / 变异测试）、
  三分档触发策略（push 跑 build+test、PR/手动跑 quality-gate、docs 改动不触发）、6 条执行步骤、5 条约束、
  3 项输出、6 项验收、5 条打回条件
- `docs/ARCHITECTURE.md` 新增 **§9 CI-first 开发约束**：本地只做代码编写 / 静态分析 / Git / 文档；
  编译构建测试打包全在 CI；CI 基础设施要求（runner 需装 JDK 17、Android SDK compileSdk 37 / Build Tools 36.0.0、
  NDK r30、Rust 1.100.0、cargo-ndk 4.1.2）；CI 验证结果是唯一编译验证依据
- `AGENTS.md` 新增 **第 10、11、12 条**：10 = 编译构建测试打包必须走 CI，不得声称"本地已编译通过"，每批改动即 push 等 CI；
  11 = 涉及验证编译/测试/构建的验收项一律以 CI 结果为准，CI 未跑通不得标"已完成"；
  12 = 公开仓库凭据纪律（不写真实 token/cookie/账号/手机号/密钥值，只写来源定位；commit message 不得写"破解/绕过/封堵"）
- `docs/SECURITY.md` 新增 **§6 公开仓库 CI 安全**：5 条硬约束 + T-002e 搭建时逐条核对的 5 项 workflow 审查清单
- `docs/RISKS.md` 新增 **R23–R26**：本地不可编译 CI 是唯一通道 / runner 环境与本地不一致 /
  公开仓库加速接口封堵 / CI 日志与 artifact 泄露凭据

#### 修改

- `docs/TASKS.md` 编号索引表新增 T-002e 一行，排在 T-003 之前；编号规则补一条「阶段 1 前置任务中 T-002e 优先级最高，T-003 / T-004 均依赖它」
- `docs/TASKS.md` **T-003** 目标栏追加"本任务所有验证在 GitHub Actions 完成，本地不编译"；约束栏从 1 条扩为 5 条（含"依赖 T-002e"）；
  验收栏整段替换为"1-7 项全部在 CI 完成，每项有 CI 日志或 artifact 作为证据 / CI 全绿视为通过 / 不得以本地已验证替代"
- `docs/TASKS.md` **T-004** 同构改写：目标栏追加 CI 说明，约束栏扩为 4 条（含"依赖 T-002e"），验收栏整段替换为 CI 证据要求 + 替代方案同样需 CI 验证
- `docs/TASKS.md` 阶段列表与 T-005 的"进入阶段1"表述同步：先完成 T-002e，再并行 T-003 / T-004

#### 说明

- 仓库为**公开仓库**，从创建时即为 Public，不走 Private → Public 切换，因此不受私有仓库 Actions 2000 分钟限制。
- `reverse/` 继续不进 Git（`.gitignore:2`）；`docs/evidence/` 进 Git 并公开，故索引文件中涉及密钥只写来源定位。
- CI 日志、artifact、索引文件一律按公开内容对待，凭据纪律见 `docs/SECURITY.md` §6。
- 本条目仅落本地文档；**GitHub 仓库创建与首次 push 由用户在本机执行**（需要 GitHub 账号与 remote 凭据），agent 不代跑。
---

## 2026-11-27

### T-002e 第 3 轮：UniFFI 绑定 + Kotlin 集成（2026-11-27）

#### 新增

- `rust/core/uniffi.toml`：`[bindings.kotlin]` + `package_name = "com.netdiskhub.core"`；不写 `cdylib_name`（库模式从 .so 文件名自动推导）
- `rust/uniffi-bindgen/`：workspace 内的绑定生成器 crate（`name = "uniffi-bindgen"` / `edition = "2024"` / `publish = false` / 依赖 `uniffi = { version = "=0.32.2", features = ["cli"] }`），入口 8 行 `fn main() { uniffi::uniffi_bindgen_main(); }`。版本由 `rust/Cargo.lock` 锁定，不装全局命令、不污染 PATH
- `rust/Cargo.lock`：18999 B / 82 个 package / lockfile version 4，含 `netdiskhub-core 0.0.0`、`uniffi-bindgen 0.0.0`、`uniffi 0.32.2`。所有 Cargo 构建改为 `--locked`
- `app/src/main/kotlin/com/netdiskhub/app/NativeProbe.kt`：编译期证明「Kotlin 能调用 Rust 导出函数」（`probeMagic()`），运行时验证留给 T-003
- `scripts/verify-apk-native-libs.py`：纯 python3 无第三方依赖的 APK 原生库校验器。断言 ①`lib/` 条目严格等于 6 条清单（多一条即失败）②每个库 ELFCLASS64 且 `e_machine` 与 ABI 匹配 ③每个 `PT_LOAD` 的 `p_align >= 0x4000` ④`libnetdiskhub_core.so` 动态符号含 3 个必需符号 ⑤ZIP 内均 `ZIP_STORED` 且数据偏移 16 KB 对齐
- `docs/ADR/ADR-004-jna-runtime.md`：引入 JNA 的决策记录，状态**已批准**，明确写「不使用 `tools:overrideLibrary`」

#### 修改

- `rust/Cargo.toml`：`[workspace] resolver = "3"` 的 `members` 从 `["core"]` 扩为 `["core","uniffi-bindgen"]`
- `rust/core/Cargo.toml`：新增 `uniffi = "=0.32.2"` 依赖
- `rust/core/src/lib.rs`：保留 `netdiskhub_probe`（第 2 轮 rust.yml 的符号断言依赖它，删了会让第 2 轮回归变红），新增 `#[uniffi::export] pub fn probe_magic() -> u32` 与 `uniffi::setup_scaffolding!()`。用 proc-macro 路线，不引入 UDL / `build.rs` / `build-dependencies`
- `gradle/libs.versions.toml`：`[versions]` 追加 `jna = "5.19.1"`；`[libraries]` **刻意不声明** jna——版本目录不支持 `@aar` 后缀，声明成普通 module 会解析出不含 `libjnidispatch.so` 的 jar，运行期 `Native.register` 必失败
- `app/build.gradle.kts`：新增 `ndkVersion = "30.0.16248370"`、`defaultConfig.ndk { abiFilters += listOf("arm64-v8a","x86_64") }`、两个抽象任务类型 `CargoNdkBuildTask` 与 `GenerateUniffiBindingsTask`；`androidComponents.onVariants` 里用 `jniLibs.addGeneratedSourceDirectory` 与 `kotlin.addGeneratedSourceDirectory` 接入生成产物；`dependencies` 追加 `net.java.dev.jna:jna:5.19.1@aar`
- `.github/workflows/build.yml`：`runs-on` 由 `ubuntu-latest` 改为 `ubuntu-24.04`；新增 concurrency；SDK 步骤加装 NDK r30 并断言 `Pkg.Revision`、三变量同指一个完整目录；新增装 Rust 1.100.0 + 双 Android target、装 cargo-ndk 4.1.2 `--locked`、校验工具链版本；`assembleDebug` 后新增三步校验（Kotlin 绑定文件内容 / 绑定已编进 dex / APK 原生库）
- `.github/workflows/rust.yml`：交叉编译命令补 `--locked`
- `docs/ARCHITECTURE.md`：新增 **§7.1 支持的 ABI（明确声明）**，写死只支持 `arm64-v8a` 与 `x86_64`、不支持哪 5 个、声明在哪个文件、由哪个脚本校验，并说明 JNA 其余 5 个 ABI 被裁掉
- `docs/TASKS.md`：T-002e 状态改为「进行中（第 1、2、3 轮完成；test / quality-gate workflow 未做）」；新增「第 3 轮完成情况」整节
- `docs/evidence/ci-setup.md`：新增 **§11 第 3 轮运行记录**（11 个子节，含两次失败原文、13 步逐条时间表、工具链实测、绑定生成、dex、APK 原生库 6 条清单、JNA 与 Manifest 合并 4 项验证、Artifact、警告、已验证/未验证）

#### 结论（实测，非推测）

- CI 第 3 次运行 [run 45950571670](https://github.com/lxyhyh/netdisk-hub/actions/runs/45950571670) **13 步全 success**，耗时 3 分 04 秒
- UniFFI 绑定点产出 `app/build/generated/kotlin/generateUniffiBindingsDebug/com/netdiskhub/core/netdiskhub_core.kt`，**40751 B**，含 `package com.netdiskhub.core` 与 ``fun `probeMagic` ``
- 绑定真实编进 dex：`classes3.dex` 与 `classes4.dex` 命中 `Lcom/netdiskhub/core/` 与 `probeMagic`（门面 `Netdiskhub_coreKt`）
- APK 内 `lib/` **恰好 6 条**：两 ABI × {`libandroidx.graphics.path.so`, `libjnidispatch.so`, `libnetdiskhub_core.so`}；全部 ELFCLASS64、`PT_LOAD p_align` 全 `0x4000`、全部 `ZIP_STORED` 且数据偏移 16 KB 对齐
- JNA 5.19.1@aar 可解析、可打包；**Manifest 合并无错、全程未用 `tools:*`**，合并后 `targetSdkVersion=37`（宿主值覆盖 JNA 的 25）

#### 未完成 / 待确认

- **运行时端到端未验证**：`probeMagic()` 的返回值（应为 65538）从未在设备或模拟器执行；`UniffiLib` 能否运行时 `Native.register` 成功未知。属 T-003 范围
- UniFFI contract checksum 期望值 `62777`（绑定第 815 行）只有跑起来才能验证匹配
- `test` workflow 与 `quality-gate` workflow 仍未建立
- `release` 构建（R8/minify、strip 配置）未验证，本轮只跑 `debug`

#### 说明

- 本轮 CI **失败两次才绿**，两次失败原因已原文记入 `docs/evidence/ci-setup.md` §11.2 / §11.3：
  ①Kotlin DSL 里写 `java.io.File(...)` 会被解析成扩展属性（`Unresolved reference: 'io'`）——必须顶部显式 `import java.io.File`；
  ②手动 `set outputDir` 又用字符串塞 `sourceSets.jniLibs.directories`，AGP 看不到依赖，报 `mergeDebugJniLibFolders` 隐式依赖错误——AGP 9 只能走变体 API 的 `addGeneratedSourceDirectory`
- 第 3 轮只做「构建链接通 + APK 静态产物验证」，**不执行任何 Rust 代码**；`T-003` 保持待启动，未标记完成

---

## 2026-11-27（续）

### T-002e 第 3 轮验收加固：GNU_RELRO 官方断言 + zipalign（2026-11-27）

#### 修改

- `scripts/verify-apk-native-libs.py`：新增 `Elf.check_relro()`，按官方 16 KB 页面文档的三个并列条件判定（A 无 `GNU_RELRO` / B 结束地址 `% 0x4000 == 0` / C RELRO 之后同一 LOAD 段内无可写数据）。C 分两档实现：C1 用官方等式 `LOAD.VirtAddr + LOAD.MemSiz == RELRO.VirtAddr + RELRO.MemSiz`；C2 把 RELRO 覆盖范围按 16 KB 取整成页区间，要求区间内 RELRO 之外无可写字节。RELRO 起始地址不落在任何 `PT_LOAD` 内时**直接判失败**，不静默通过。删除原 `relro_note()`（只打印不判定）
- `scripts/verify-apk-native-libs.py`：新增 `--selftest`，用合成 ELF 回归官方 Example 1–4 与一条「RELRO 不在任何 `PT_LOAD` 内」负例；CI 中在检查真 APK 之前执行
- `.github/workflows/build.yml`：`Verify APK native libraries` 步骤加一行 `--selftest`；新增独立步骤 `Verify APK zipalign 16 KB page alignment`，执行 `$ANDROID_HOME/build-tools/36.0.0/zipalign -v -c -P 16 4 "$APK"`，退出码非 0 或缺 `Verification successful` 都令 workflow 失败。原有 python ZIP 偏移检查**保留**
- `docs/evidence/ci-setup.md`：新增 §11.13（为什么要改 / 脚本实现 / 六库 RELRO 实测判定表 / 官方 zipalign 验收 / 第 4 次运行结果 / 本次未改动部分 / 六库完整判定输出）；§11.8 的 RELRO 段改写为「已被 §11.13 取代」并承认旧理由错误；§11.1 补第 4 次提交与运行
- `docs/TASKS.md`：T-002e 第 3 轮完成情况补提交 4、第 4 次运行、六库 RELRO 判定表与 zipalign 验收；状态仍为「进行中」
- `docs/ARCHITECTURE.md`：新增 **§7.1 支持的 ABI（明确声明）**——只支持 `arm64-v8a` 与 `x86_64`，不支持 `armeabi`/`armeabi-v7a`/`mips`/`mips64`/`x86`；声明位置 `app/build.gradle.kts` 的 `abiFilters`；依据 ADR-004；校验脚本 `scripts/verify-apk-native-libs.py`

#### 结论（实测）

- 第 4 次运行 [run 45952694106](https://github.com/lxyhyh/netdisk-hub/actions/runs/45952694106) **14 个业务步骤全 success**，3 分 00 秒
- `--selftest` 输出「selftest 通过：5 个官方示例判定一致」
- 6 个 `.so` 的 RELRO **全部合规、无一需要修复**：`libjnidispatch.so` 两 ABI 靠条件 B；两个 `libnetdiskhub_core.so` 靠条件 C1；两个 `libandroidx.graphics.path.so` 靠条件 C2（按官方规则算出的结论，不是豁免）
- zipalign 输出 `Verification successful`，6 个 `.so` 全部 `(OK)`
- APK Artifact `15177950350`，zip 10770227 B，digest `sha256:a6f6b9e3…2ec1614a`；下载后 30500633 B，SHA256 `725dca77…8077e03ab`
- 最终断言：`PASS: 6 个原生库通过 清单 / ELF / PT_LOAD 16 KB 对齐 / GNU_RELRO 官方条件 / ZIP 页对齐 / 符号 检查`

#### 未完成（不变）

- `probeMagic()` 运行时返回 65538 仍**未验证**（属 T-003）；contract checksum `62777` 是否匹配未知
- `test` workflow 与 `quality-gate` workflow 仍未建立
- `release` 构建（R8/minify、strip）未验证

#### 说明

- 改动前 `PT_GNU_RELRO` 只打印不判定，理由是「Google 自己的 `androidx.graphics.path.so` 也不对齐」；该理由不成立——官方规则是**三个并列条件满足其一即合规**，不对齐只说明要继续走条件 C，不是免检。本次按规则补齐断言
- 按验收要求「不重新设计构建链、不修改已通过的其他部分」，Rust / cargo-ndk / NDK / Gradle / AGP / Kotlin / JNA 版本与 `app/build.gradle.kts` 全部未动
