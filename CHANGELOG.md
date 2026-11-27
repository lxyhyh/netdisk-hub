# 更新日志（CHANGELOG）

本项目遵循「一个任务一条记录」的粒度。

## [未发布]

### T-006 百度网盘原生登录攻关证据与准入复核（2026-11-28）

**结论：百度由「需攻关」改判「原生登录可行（静态可行，动态未验证）」；账号密码 / 短信验证码 / 扫码三条路径分别判为「可独立实现」。硬准入数量 1 → 2，但 T-005 的历史 No-Go 结论不改写、MVP 不锁定、阶段 2 不启动。**

#### 新增

- `docs/evidence/baidu-login-deep-dive.md`（**857 行 / 74746 B**，14 节 + 附录 A/B）：T-006 深度取证报告
  - 1 任务边界与结论摘要　2 `tpl`/`appSignKey` 来源　3 `di` 40 字段逐项分类
  - 4 `passport_core` 六 native　5 其他 native 安全组件　6 账号密码路径　7 短信路径　8 扫码路径
  - 9 Rust 可复刻清单　10 native 移植必要性　11 三路径结论　12 百度整体结论
  - 13 未解决问题　14 检索命令与复现方式　附录 A 勘误登记　附录 B 可信度分布

#### 修改

- `docs/evidence/login-feasibility.md` v1.0 → **v1.1**（728 → **825 行**，50552 → 60613 B）：§3.2 百度整节改写（新增 3.2.3 签名机制、3.2.6 原 5 项攻关点复核表、17 跳链尾），§2 总表、§4、§5、§6、§7、§8、附录 A/B 全部同步；**其余八家一字未改**
- `docs/API_FACTS.md` v0.5 → **v0.6**（492 → **511 行**，35759 → 39755 B）：百度节第 3/9/10/12 项重写（含原 4 项攻关点的复核表），总表与全局状态更新；**其他八家未改**
- `docs/PROVIDERS.md` v0.6 → **v0.7**（267 → **300 行**，21008 → 26378 B）：能力矩阵与登录矩阵百度行同步，§4 后果段更新，新增 **§6.7 T-006 之后的准入变化**
- `docs/TASKS.md`（1152 → **1231 行**，59797 → 67423 B）：索引表 T-006 → **已完成**，状态行、卡末新增「完成情况（2026-11-28）」含产物表 / 八项工作结果 / 三处修正 + 一处勘误 / 15 项验收核对 / 遗留问题 / 对后续决策的影响，阶段 0 列表更新
- `docs/RISKS.md` v0.5 → **v0.6**：**R28 拆分**（三家 → 两家，百度移出）；**R5 更新**（补 T-006 的 `di` 定性结论，状态仍「待动态验证」，未关闭）；新增 **R29**（服务端是否接受构造的 `sig`/`di`，静态不可证明）

#### 关键事实（本任务核查所得，全部带 `文件:行号`，详见 deep-dive）

- **`tpl` / `appId` / `appSignKey` 是 APK 硬编码明文常量**，不是运行期注入、不是远程下发：`reverse/baidu/sources/classes15/sources/kd/______.java:11`（appId）/ `:14`（国内 appSignKey）/ `:15`（tpl）/ `:17`（国际 appSignKey），经 `kd/______.java:33-37` → `AccountUtils.E()/K()`（`AccountUtils.java:153-164`）→ `application/___.java:11-19` → `C_.java:196-197` `setProductLineInfo(...)` → `SapiConfiguration.java:854-858`。`DebugSetKey` 覆盖（`kernel/debug/C_.java:82-88`）在 `!NetDiskLog.isDebug()` 时恒返回 null → **release 包无运行期覆盖来源**。**文档不记录常量取值。**
- **签名真正在子类**：基类 `com/baidu/pass/http/HttpHashMap.java:7-8` 的 `doSign(String)` **是空方法**（baksmali 反汇编确认方法体仅 `return-void`）；实现是 `classes4/.../httpwrap/HttpHashMapWrap.java:9-15`，**全仓唯一调用点** `HttpClientWrap.java:86-88`；算法为 `SapiUtils.java:135-160` 的「排序 + `key=URLEncoder.encode(v)&` 拼接 + `sign_key=` + MD5 hex」，**纯 Java**。
- **`di` 40 字段全部定性**：必需 22 / 条件 2 / 可选 16（合计 40，与 `SapiDeviceInfo.java:13-56` 的 `getCookiesKeyList()` 数量一致）。16 个「可选」位全部是官方自己也填空串的占位（`getWifiInfo`/`getIMEI`/`getIccid`/`getLocalIpAddress` 方法体就是 `return ""`）。`mPrivacyParamesRegulation=false`（`C_.java:199` 实参）使 `getGrayControlParams()` 在第二道闸门即返回空数组（`ParamsUtil.java:114-159`）→ **`dt_params` 灰度裁剪实际不生效**；唯一仍动态的是服务端下发的 `diExceptIndex`（`SapiOptions.java:62` 默认空）。加密为 `AES/CBC/NoPadding` + 16 字符硬编码 key + MD5 校验尾（`SapiDeviceInfo.java:189-203`），**全在 Java 侧**。
- **`passport_core` 不构成阻断**：`PassCoreVHelper.java:5-7` 加载，`:9-19` 六个无参 native；全 dex **仅 7 处调用（5 文件）**，**三条主链 0 处**；`nm -D libpassport_core.so` 恰 6 个 `Java_` 导出、**无 `JNI_OnLoad`**、`RegisterNatives` 0 命中 → 静态注册、无隐藏方法；缺失时 `SapiConfiguration.java:553-575` 有 `catch (Throwable)` 兜底，`build()` 仍成功。`getAesKey` 反汇编显示密钥是 `.rodata` 硬编码常量经库内解密，**不是服务端下发**。
- **其他 6 个安全 native 库均不经过登录路径**：`libnetdisk-security.so`（业务 API URL 加签，唯一调用 `ex/n3.java:49`，上游是 okhttp 拦截器 `ex/m1.java:294`）、`libnetdisk-signature-check.so`（APK 签名校验，`CertVerifier.java:38`）、`libnetdisk-security-rand.so`（业务 API `rand`，`ServerApiKt.java:23` 注册的拦截器）、`libsofiresec.so`（运行时动态下载，只服务 `di` 索引 34，有纯 Java 兜底）、`libmsaoaidsec.so`（MSA OAID）、`libjni-kernel/kservice.so`（P2P / 下载加速）。
- **扫码路径是三条件里依赖最少的**：`QrCodeService.java` 全文 `HttpHashMapWrap` 0 命中（**不加 `sig`**）、`getDeviceInfo` 0 命中（**不发 `di`**）、三个请求 cookie 传 null，且 `getQrLoginResult`（`:354-364`）的 `bduss` **允许为空** → **不依赖任何既有登录态**。
- **账号密码路径的 `di` 不在 body**，只经 `ParamsUtil.buildNaCookie` 的 `DVIF` cookie 传递（`ParamsUtil.java:75-79`）；短信路径的 `di` 才在 body（`SapiAccountRepository.java:267-270`）。
- **账号密码响应是纯 XML 解析**（`SapiCoreUtil.java:68-77` 正则取 `<client>…</client>` + `XmlPullParser`），**不是 WebView**；落库是本地 AES 加密 JSON + CookieManager 写 `BDUSS`/`PTOKEN`（`SapiContext.java:978-1016`）。

#### 与 T-005 结论的关系（只陈述，不改写）

- T-005 判 No-Go 的直接依据是「硬准入数量 1 < 门槛 2」。本任务后数量为 **2**，**形式上满足判据**。
- **但**：①T-005 的历史 No-Go 结论不改写；②是否重跑 Go/No-Go 由用户决定；③即使转 Go 也不等于用户已选定 MVP 组合；④两个「可行」不同质——坚果云的人机验证依赖腾讯闭源 Web 服务（R27），百度不依赖任何闭源 native，重跑前应先统一「原生登录可行」的判定口径。
- `docs/evidence/mvp-go-no-go.md:239` 原写「`tpl`/`appSignKey` 由运行时 `setProductLineInfo` 注入…静态不可得」，**已被本任务推翻**；该文件属 T-005 历史产物，**本次不改写**，只在 deep-dive 附录 A 登记勘误。

#### 本次未做的事（边界）

- 未登录任何真实百度账号，未发送任何真实请求，未抓包。
- 未移植、复制或集成任何官方闭源 so；未新增 Gradle / Cargo 依赖；未改 Android / Kotlin / Rust / UniFFI 架构；**未产生 ADR**（不需要）。
- **未取证「获取短信验证码」的接口**（T-006 范围只到「已拿到验证码之后」的登录步骤）。
- 未逆向 `libfire.so` / `libsofiresec.so` 的内部算法（只判关联）。

### T-005 Go/No-Go 决策与后续网盘攻关计划（2026-11-28）

**结论：No-Go。当前不锁定两个 MVP 网盘，不启动阶段 2 单网盘 Provider 实现。**

#### 决策依据

- 硬准入 = 「原生登录可行」。T-002c 判定结果：可行 **1**（坚果云）/ 需攻关 3（百度、夸克、阿里云盘）/ 未知 2（天翼、123 云盘）/ 不可行（资料缺失）3（蓝奏、UC、迅雷）。
- Go 门槛 = 可行数量 ≥ 2。当前 1 < 2 → **No-Go**。
- 「需攻关」在攻关完成并达到「原生登录可行」后才可计入 Go 分子；「未知」不得进入阶段 2；「不可行（资料缺失）」直接从 MVP 移除；不使用 OAuth / WebView / cookie / 手动 token 作为替代。

#### 新增

- `docs/evidence/mvp-go-no-go.md`（340 行 / 21703 B，12 个二级节）：T-005 决策记录
  - 0 用途与读法　1 判定规则（照抄任务卡，不因结果调整）　2 九家准入表
  - 3 可行数量与 Go 门槛　4 No-Go 决策（含不降低门槛四条）
  - 5 坚果云特殊核对（人机验证的四个问题与动态验证边界）
  - 6 三个「需攻关」候选的 8 维比较　7 补救路径 T-006　8 动态验证未完成的边界
  - 9 与阶段 1 的关系（不得提前启动）　10 关联文件与变更记录　附录 A
- `docs/TASKS.md` 新增 **T-006 百度网盘原生登录攻关与第二家准入验证**（状态：待启动，依赖 T-005，本次只建卡不执行）
- `docs/RISKS.md` 新增 **R27**（坚果云人机验证依赖第三方闭源 Web 服务，含降级触发条件）、**R28**（三家需攻关网盘的阻断点集中在闭源 native 安全组件），新建「登录与风控风险」分节

#### 修改

- `docs/TASKS.md`（1018 → 1152 行 / 51294 → 59797 B）：
  - 索引表 T-005 → **已完成（No-Go）**，新增 T-006 行
  - T-005 卡末新增「完成情况（2026-11-28）」：产物表 / 九家准入表 / 不能凑两个的三条 / 坚果云验证码处置 / 攻关优先级表 / 11 项验收核对
  - 底部阶段 0 汇总更新为 T-002a/b/c/d 已完成 + T-005 已完成 No-Go + T-006 待启动，删除仍把 T-002c 标待启动的过期状态
- `docs/PROVIDERS.md` v0.5 → **v0.6**（233 → 267 行）：新增 §6.6「T-005 决策收口（当前有效结论）」，§6.1 标注为历史建议已被实测取代，§3 矩阵注明 No-Go，明确只有坚果云达硬准入、三个需攻关候选不进入 Go 分子、当前无合格双网盘组合，R22 收口
- `docs/RISKS.md` v0.4 → **v0.5**（59 → 64 行）：**R22 关闭**（历史矛盾收口，保留审计记录）；R5 按 T-002c 实测更新并转「待动态验证」；R10 **关闭**（9 家判定完毕，无一家被判「只能 OAuth/WebView/cookie」）；R11 转「部分缓解」（登录层已绕开，文件层仍待动态分析）

#### 关键事实补充（本任务核查所得）

- 坚果云人机验证：`TencentCaptchaView.java:29/:73` 内嵌 WebView 加载 APK 内 `assets/TencentCaptchaH5.html`（1522 B），该页从 `turing.captcha.qcloud.com` 拉验证脚本生成 `ticket`/`randstr`；触发开关是服务端 `isNeedRealCaptcha()`（`v2/ui/login/f.java:29-35`），**属条件触发分支，不是每次登录必经**；凭证仍由本项目原生 `/mcmd/loginV2` 直接交换，不经 WebView 提取 cookie/token → **维持「原生登录可行」**，但 UA 校验、触发阈值、`tfPasscode`、token 有效期**全部未验证**。
- 三个需攻关候选的攻关顺序：**1 百度 → 2 夸克 → 3 阿里云盘**。百度优先级最高是因为账号密码/短信/扫码三条登录链已全部静态定位、RSA 与参数构造在 Java 层，剩余障碍集中在运行时注入的 `tpl`/`appSignKey` 与设备指纹；夸克、阿里的主要阻断是阿里聚安全 native（M9 请求体加密 / `security_sign` / MTOP 签名 / `riskControlInfo`）。**「百度优先」不等于「百度已可行」。**

#### 未做（明确保留）

- 未修改 `docs/API_FACTS.md`：T-005 没有产生新的接口事实。
- 未把 build CI 全绿写成 T-002e 完成；test workflow 与 quality-gate workflow 仍未建立；T-003、T-004 仍未完成。
- 未启动 T-006 的任何攻关工作，未锁定 MVP 组合，未启动阶段 1 Provider 业务开发。

### T-002c 九个网盘原生登录可行性判定（2026-11-28）

#### 新增

- `docs/evidence/login-feasibility.md`（728 行 / 50552 B，11 个二级节）：T-002c 的汇总证据文件
  - 0 边界与读法（静态发现 vs 运行时验证的区分）
  - 1 判定标准（原生登录可行 / 需攻关 / 不可行 / 未知 四类 + 界面存在→协议存在→独立实现可行 三层）
  - 2 总表（9 网盘 × 登录类型 / 可行性 / 主要证据 / 攻关点 / 是否进 MVP 候选）
  - 3 逐网盘（每个含 资料来源 / 调用链 / 认证请求及字段 / token 换取与存储 / 禁止项排除 / 结论 / 未解决问题）
  - 4 与 T-002d 结论的差异说明　5 禁止路径排除记录　6 可信度与剩余风险　7 对 T-005 的输入　8 未解决问题　附录 A 检索命令 / 附录 B 边界声明
- `reverse/t002c/`（不进 Git）：夸克 classes7/classes10、阿里 classes9/classes11 的定向反编译产物（14555 个 .java）
- `reverse/` 下 T-002c 用到的扫描脚本：`login-clues.py`、`login-clues2.py`、`login-strings.py`、`login-pkgs.py`、`locate-login-dex.py`、`list-login-classes.py`、`bridge-scan.py`

#### 修改

- `docs/API_FACTS.md` v0.4 → **v0.5**：九家「登录流程」全部改写为 12 项编号字段
  - 顶部全局状态换成 T-002c 后的四类分布（可行 1 / 需攻关 3 / 未知 2 / 不可行 3）
  - 「登录流程字段」说明升级为 12 项定义表；各网盘条目填满 12 项
  - 百度两处 T-002d 结论作废（扫码接口已定位、换取路径已定位）；「扫码登录是否存在」待确认项关闭
  - 汇总表新增「登录可行性（T-002c）」列，夸克/阿里两行的主体技术栈改「登录在 Java 层」
- `docs/PROVIDERS.md` v0.4 → **v0.5**：登录能力矩阵扩为 4 列（可行性 / 登录类型 / 登录资料状态 / MVP 准入）；
  列定义改为 T-002c 的四档；§4 准入规则补「需攻关未完成前不计入 Go 分子」；
  §5「有静态反编译价值的网盘」补「按协议分层」表（文件协议 vs 登录协议分别在哪个层）；新增 §6.5（T-002c 对切片建议的影响 + 组合 D）
- `docs/TASKS.md`：T-002c 状态改**已完成**，索引表同步，头部最后更新时间更新；
  新增「完成情况（2026-11-28）」小节（产物 / 九网盘结论 / 两条修正 / T-005 输入 / 11 项验收核对 / 未完成动态分析）；
  原文「但需要用户先明确一件事」改为「执行后的实际情形」记录
- `CHANGELOG.md` 本条

#### 说明

- **九家结论**：坚果云 = 原生登录可行（静态可行，动态未验证）；百度、夸克、阿里 = 需攻关；天翼、123 = 未知（需动态分析）；蓝奏、UC、迅雷 = 不可行（资料缺失）
- **两条必须留档的修正**：
  1. 夸克/阿里的**登录协议在 dex 的 Java 层，不在 Dart 侧**。T-002d 把四家统一记为「协议需动态分析」，对文件业务协议成立，对登录协议不成立。两家登录链路都可静态读出，因此判「需攻关」而非「未知」
  2. 天翼/123 仍是「未知」且**不是遗漏**：dex 侧确实 0 条登录线索（3/4 个壳桩类），是技术栈决定的上限
- **对 T-005 的影响**：硬准入「可行」集合当前**只有坚果云 1 个**，Go 判据「可行集合 ≥ 2」**当前不成立**，需先完成至少一个网盘的攻关。T-002c 不选 MVP
- **百度 OAuth 常量定性**：`https://openapi.baidu.com/oauth/2.0/token` 只用于 `openConnection()` 取 `getDate()/1000` 当时间戳（`SapiAccountRepository.java:1111-1121`、`:1102`），**不参与鉴权**，不能据此判百度只能 OAuth
- **百度 Sdata 定性**：只在非 https 的自研网络栈加（3 处），sapi2 登录链路 0 命中；其实现仍由运行期注入（`.n0(Function2` 静态调用方 0 命中），**未还原、但也不属于登录链路**
- **未做**：未登录真实账号、未发任何真实请求、未抓流量、未做 Dart AOT 逆向、未破加固壳、未执行 T-005
- 对齐 `index-template.md` v2.1 的硬规矩：每条线索带 `文件:行号`，可信度只用三档，「未发现」一律写明搜索方式
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
