# 接口事实库（API_FACTS）

- 状态：草案 v0.4
- 产出任务：T-001（T-002a 更新资料定性 / T-002b 填入坚果云 / **T-002d 填入百度 + 四个 Flutter 网盘壳确认**）
- 最后更新：2026-11-27

## 记录格式

每条记录包含：

```
网盘 / 功能 / 来源（APK 类#方法 或 网页 JS 文件:函数）/ 请求 / headers / body /
认证 / 响应 / 错误码 / 可信度（已验证|推测|待确认）/ 证据路径 / 未知问题
```

## 全局说明

**当前状态（T-002d 后）：2 个网盘有完整静态反编译线索（坚果云、百度），4 个网盘完成壳确认、协议覆盖为 0（天翼、123、夸克、阿里），3 个网盘资料缺失（蓝奏、UC、迅雷）。**

原因：

- 坚果云（T-002b）与百度（T-002d）是 `Java/Kotlin 主体`，协议在 dex 里，反编译有实际价值；
- 天翼、123、夸克、阿里是 `Flutter 主体`，业务逻辑编译进 `libapp.so`（Dart AOT），静态反编译只能拿到壳，**这是技术栈决定的上限，不是遗漏**；
- 蓝奏、UC、迅雷按 T-001 拍板记为资料缺失。

对照文件：`docs/evidence/index-坚果云.md`、`docs/evidence/index-百度.md`、`docs/evidence/index-天翼.md`、`docs/evidence/index-123云盘.md`、`docs/evidence/index-夸克.md`、`docs/evidence/index-阿里云盘.md`。

**重要区分**：

- 本文件里"静态线索"= 从 APK 反编译读出的代码，**没有发过任何真实请求**
- 可信度「已验证」指"代码里确实是这么写的"，**不指"服务端确实这么响应"**
- 真实请求验证属于阶段 2，本文件不承载

纪律：

- 没有证据的不得写成事实
- 推测必须显式标注
- 可信度只用三档：已验证 / 推测 / 待确认
- 禁止编造字段名、路径、错误码

---

### 登录流程字段（每个网盘必填）

见下方各网盘条目的「登录流程」小节。

---

## 1. 百度网盘

- 包名 / 版本：`com.baidu.drive.app` / 13.11.15（versionCode 3711）
- 资料：`.apks` 分包（base.apk 251854857 B，20 dex 合计 182513748 B），**未加壳**
- 主体技术栈：**Java/Kotlin 主体 + Flutter 模块共存**（协议在 dex 侧；split 内另有 `libflutter.so` + `libapp.so` 20972464 B）
- 静态侦察状态：**T-002d 已完成**，覆盖率 **100.0000%**（63259/63259）

### 登录流程

- 登录类型：**账号密码 / 短信校验**（静态可见 sapi2 SDK 的密码登录与短信校验页面）；**扫码登录未发现独立实现** → 待确认
- 是否可在 APP 内原生完成：**APP 内自带原生登录页**（`classes4/sources/com/baidu/sapi2/newlogin/activity/` 下 5 个 Activity）；能否脱离百度 App 生态独立复刻 → **待确认（属 T-002c）**
- 会话换取方式：静态可见凭据字段 `bduss`/`uid`/`ptoken`/`stoken`（`classes2/sources/com/baidu/sapi2/SapiAccount.java:5-40`），**换取接口路径未定位** → 待确认
- 是否需要 OAuth 或 WebView 才能完成：密码/短信走原生页；`https://openapi.baidu.com/oauth/2.0/token` 常量存在（`SapiAccountRepository.java:1119`）但用途 → 待确认
- 扫码登录 token 来源：**未发现**（`qrcode/` 只有 `R.java` + `a.java`，无业务实现）
- token 刷新机制：待确认
- token 有效期：待确认
- 依据：`docs/evidence/index-百度.md` §4
- 登录可行性结论：**未知（属 T-002c，本条目不作判定）**
- 若需攻关，攻关点：`Sdata` 签名算法（`classes7/sources/hk/__.java:469-475`，实现由注入提供，静态不可得）、`bdstoken = MD5(BDUSS)` 之外的 `sign`/`timestamp` 生成
- 若不可行，原因及证据：**未判定**

### 待确认问题

- `Sdata` 的 HMAC 实现注入点（`hk/__.G` 的 `Function2` 由谁赋值）？
- `RequestCommonParams` 的 User-Agent / 设备参数真实值？
- token 刷新与过期策略？
- 扫码登录是否存在（dex 内未发现）？
- 服务端是否接受 `Range`（客户端已在 `np0/d.java:21`、`np0/r.java:208`、`np0/s.java:139` 发送）？
- `app_id` 常量（`classes2/sources/ew/C_.java:29` 初值为 null，由 `:82` 运行期注入）真实值？
- 是否存在验证码或风控挑战（`libnetdisk-security.so` 等 6 类风控 SDK 的参与度）？

### 已知线索

- 文件类基址 `https://pan.baidu.com/api/`（`classes15/sources/md/___.java:24`）；下载基址 `https://d.pcs.baidu.com/rest/2.0/pcs/file`（`classes12/sources/op0/a.java:90-94`）
- 列目录 `GET /api/list`，分页 **offset**（`classes15/sources/kg/__.java:641-657`；`classes15/sources/rd/__.java:16`）
- 上传 = `precreate` → `superfile2` 分片（4 MiB 起，动态翻倍）→ `create?a=commit`；客户端硬限 4 GiB
- `bdstoken = MD5(BDUSS)` 小写十六进制（`classes6/sources/com/baidu/netdisk/network/__.java:8`、`:18`）
- 完整 40 余条接口清单见 `docs/evidence/index-百度.md` §8.2

### 证据路径

- `docs/evidence/index-百度.md`（43508 B / 590 行）
- 反编译产物 `reverse/baidu/`（不进 Git）

## 2. 天翼云盘

- 包名 / 版本：`com.cn21.ecloud` / 11.1.2（versionCode **638**）
- 资料：根目录 `cloud189_v11.1.2.apk`（387095539 B）
- 主体技术栈：**Flutter 主体**（`libapp.so` arm64 18138032 B；dex 仅 10136 B、3 个壳桩类）
- 加固：**爱加密**（`assets/ijiami.dat` 22378961 B + `ijiami.ajm` 10890680 B）
- 静态侦察状态：**T-002d 壳确认已完成**，协议覆盖 **0**（按任务范围不做完整反编译）

### 登录流程

- 登录类型：**未知**（dex 内 0 条线索）
- 是否可在 APP 内原生完成：**未知**
- 会话换取方式：**未知**
- 是否需要 OAuth 或 WebView 才能完成：**未知**
- 扫码登录 token 来源：**未知**
- token 刷新机制：**未知**
- token 有效期：**未知**
- 依据：`docs/evidence/index-天翼.md`（壳确认，无协议事实）
- 登录可行性结论：**未知（属 T-002c，本条目不作判定）**
- 若需攻关，攻关点：需动态抓包 + Dart AOT 分析；`libsecurity.so`(21528 B)、`libexecmain.so`(215640 B) 职责未验证
- 若不可行，原因及证据：**未判定**

### 待确认问题

- 登录接口路径与字段？（`libapp.so` 内，静态不可读）
- token 结构、刷新与过期策略？
- 客户端环境校验（UA / 版本 / 指纹 / TLS pin）？
- `networkSecurityConfig=@0x7f160004` 指向的实际信任集（是否含用户 CA）？
- 列表 / 上传 / 下载 / 删除的接口形态？分页方式？

### 已知线索

- **dex 内无任何协议线索**（3 个壳桩类、无域名常量、无 HTTP 栈）
- 业务逻辑在 `libapp.so`（Dart AOT），静态不可读

### 证据路径

- `docs/evidence/index-天翼.md`

## 3. 夸克网盘

- 包名 / 版本：`com.quark.clouddrive` / 10.21.5.144（versionCode 144）
- 资料：附件区 `QuarkCloudDrive_V10.21.5.144_...arm64).apk`（120924033 B）
- 主体技术栈：**Flutter 主体**（`libapp.so` 与 `libflutter.so` 均为 **1 字节占位**；真载荷 `libkernelu4_uc_7z.so` 15938463 B 是 7-zip 容器）
- 加固：**三层包裹**（占位 so + 7z 容器 + `libdexloaderuc`/`liblibloaderuc` 动态加载）+ 阿里聚安全 `libsgmainso-6.7.251102.so` 2990668 B
- 静态侦察状态：**T-002d 壳确认已完成**，协议覆盖 **0**

### 登录流程

- 登录类型：**未知**
- 是否可在 APP 内原生完成：**未知**
- 会话换取方式：**未知**
- 是否需要 OAuth 或 WebView 才能完成：**未知**
- 扫码登录 token 来源：**未知**
- token 刷新机制：**未知**
- token 有效期：**未知**
- 依据：`docs/evidence/index-夸克.md`（壳确认，无协议事实）
- 登录可行性结论：**未知（属 T-002c，本条目不作判定）**
- 若需攻关，攻关点：需 7z 载荷解包 + Dart 快照分析 + 聚安全参与度验证，成本为本批最高
- 若不可行，原因及证据：**未判定**

### 待确认问题

- 登录接口路径与字段？（Dart 快照在 7z 容器内，静态不可得）
- 阿里聚安全（sgmain 3 文件）是否参与请求签名？
- `network_security_config.xml`（468 B，包内存在）的实际信任集？
- 列表 / 上传 / 下载 / 删除的接口形态？分页方式？

### 已知线索

- **dex 内无协议线索**。dex 有 115475 个类，但集中在 `com/ucpro`(33475)、`com/quark`(9400)、`com/uc`(8489) 等外壳/三方 SDK，属浏览器内核而非网盘文件协议
- `libfakedexuc.so`(3125 B) 实为 **APK**（PK 头），`libkernelu4_uc_7z.so` 为 **7-zip archive**

### 证据路径

- `docs/evidence/index-夸克.md`

## 4. 坚果云

- 包名 / 版本：`nutstore.android` / 4.26.5
- 资料：60329002 B（57.5 MB），仅 2 dex，无加固 —— 最易侦察
- 侦察状态：**T-002b 已完成静态反编译**（jadx 1.5.7），全部线索见
  `docs/evidence/index-坚果云.md`。**未做真实请求验证，全部结论为静态线索。**

### 登录流程

- 登录类型：**账号密码**（主路径）；另有验证码 / 二次验证分支（可信度：已验证，静态）
- 是否可在 APP 内原生完成：**是**（无可信 OAuth/WebView 依赖，可信度：已验证，静态）
- 会话换取方式：`POST /mcmd/loginV2`（无 Query，JSON body），响应返回 `token`/`userName`/`uid`；
  之后所有请求带 `Authorization: <B64(token)>-<B64(uid)>:<userName>`（可信度：已验证，静态）
- 是否需要 OAuth 或 WebView 才能完成：**否**（可信度：已验证，静态）
- 扫码登录 token 来源：**未发现扫码登录接口**。已搜索 `NutstoreApi.java` 全部 @GET/@POST、
  `connection/w.java` 全部常量、全包 `qr`/`QrCode`/`scan` 关键字 → 无结果（可信度：已验证，静态）
- token 刷新机制：**无**。凭据落 SQLite（`dao/x.java` 的 `token`/`login_name`/`user_name`/`machine_name`），
  无 refresh 接口、无定时刷新逻辑；失效由 `AccountExpired` 错误码触发跳登录页（可信度：已验证，静态）
- token 有效期：**客户端无任何过期时间字段或倒计时**（可信度：已验证，静态）；服务端有效期 → 待确认
- 依据：`docs/evidence/index-坚果云.md` §4（含逐条 文件:行号）
- 登录可行性结论：**静态看可行**（账号密码直连，无 OAuth 门槛）。
  **正式判定留给 T-002c**，本条目不下最终结论
- 若需攻关，攻关点（均为静态推测，待 T-002c 核实）：
  - UA 四段拼接 `NutstoreApp-Android-<release>-<model>-<versionName>`（`HttpConfig.java:32-34`）是否为硬校验 → 待确认
  - `/mcmd/loginV2` 的验证码票证链路（`getLoginCaptchaV2` → `loginV2` 回传 `custom_ticket/exp/sig/reusable`）在异常登录时的触发条件 → 待确认
  - `tfPasscode` 二次验证字段的启用条件 → 待确认
- 若不可行，原因及证据：**当前无"不可行"证据**

### 文件操作接口（T-002b 新增静态线索）

> 全部为静态线索（jadx 反编译），**未经真实请求验证**。定位见 `index-坚果云.md` 对应小节。

| 功能 | 方法 | 路径 | 关键参数 | 定位 |
|---|---|---|---|---|
| 列目录 | GET | `/mcmd/getDirV1` | sndId, sndMagic, path | `connection/w.java:74`（常量）、`:412-422`（构造） |
| 单对象元数据 | GET | `/mcmd/getMetaData` | sndId, sndMagic, path, list, etag, marker | `w.java:32`、`:424-453` |
| 下载 | GET | `/mcmd/getFile` | sndId, sndMagic, path, version | `w.java:76`、`:400-410` |
| 上传 | POST | `/mcmd/uploadFile` | sndId, sndMagic, path, pup（可选） | `w.java:23`、`:488-503` |
| 删除文件 | POST | `/mcmd/deleteFileV1` | body 为 JSON 数组 `[{path, version}]` | `w.java:16`、`:732-746` |
| 删除目录 | POST | `/mcmd/deleteDirV1` | 同上 | `w.java:8` |
| 重命名 | POST | `/mcmd/renameObjectV1` | srcPath, srcVersion, destName, objectType | `w.java:79`、`:748-762` |
| 移动 / 复制 | POST | `/mcmd/submitMoveV1` / `/cmd/submitCopyV1` | srcSandboxId, srcSandboxMagic, srcPaths[], destDirPath | `w.java:20`、`:78`、`:859-886` |
| 建目录 | POST | `/mcmd/mkdir` | PathInternal{path} | `w.java:68` |
| 搜索 | GET | `/cmd/search/run` | sandbox, dir, all, keys, ft | `w.java:80`、`:841-857` |
| 用户信息 / 沙箱 | GET | `/cmd/getUserInfoV3` | （5xx 回退 V2） | `connection/i.java:659-666` |

**核心约定（已验证，静态）**：所有文件类接口都必须带 `@Query("sndId")` 与 `@Query("sndMagic")`，
值分别为 `Long.toHexString(sandboxId)` / `Long.toHexString(magic)`（`HttpConfig.java:45-51`），
数据源是 `getUserInfoV3` 响应里的 `sandboxes[]`。

### 待确认问题

- 服务端是否支持 `Range` 断点续传？（客户端只有错误码常量 `RangeNotSatisfied`，无任何代码设置 Range 头）
- `networkSecurityConfig`（`@0x7f110003`）的实际内容？（资源被混淆，zip 里查不到明文 xml）
- ~~jadx 未产出的 107 个类里是否还有别的关键类？~~ **已关闭**：按 dex `class_defs` 复核，覆盖率 2752/2753（100.0%），107 条 = 76 条已补出 + 19 个 synthetic 空类 + 12 个 dex 中不存在的类，无含业务逻辑的真实缺失（见 `docs/evidence/index-坚果云.md` §2.1.1）
- token 的服务端有效期？
- 上传是否真无分片？（`checkBlock` 系列五个接口在 `NutstoreApi.java` 有声明但全仓无调用点）
- 服务端是否有频率限制？（客户端无限速代码）

### 已知线索

- 域名：`app.jianguoyun.com`（API 基址，`v2/g.java:79-90`）、`www.jianguoyun.com`、
  `push.jianguoyun.net.cn`（推送）、`workspace.jianguoyun.com`（工作区）
- 客户端标识：OkHttp 3.14.9 + Retrofit2 + Gson；**网络层无 `CookieJar`**（全包搜 `Cookie` 只命中 WebView CookieManager 清空与 mupdf 的 Cookie 类，与 HTTP 会话无关）
- 加固：无。证书绑定：无。设备指纹采集：未发现（唯一标识是自生成 UUID 存 `machine_name`）
- 单文件上传硬限 524288000 B（500 MB），`delegate/xa.java:537` 等三处

### 证据路径

- `docs/evidence/index-坚果云.md`（25612 B / 342 行，10 个必填区块 + 格式评估）
- `docs/evidence/tooling.md`（jadx 1.5.7）
- `reverse/nutstore/sources/`（8006 个 .java；不进 Git）

## 5. 蓝奏云

- 包名 / 版本：`com.tooyoung.lanzou` / 1.3.4.8
- 资料：**第三方客户端，非官方**；4765515 字节，1 dex，无加固（`libluajava.so`、`libmd5.so`）

### 登录流程

- 登录类型：**资料缺失**
- 是否可在 APP 内原生完成：**资料缺失**
- 会话换取方式：**资料缺失**
- 是否需要 OAuth 或 WebView 才能完成：**不可判定**
- 扫码登录 token 来源：**资料缺失**
- token 刷新机制：**资料缺失**
- token 有效期：**资料缺失**
- 依据：现有 APK 为第三方客户端 `com.tooyoung.lanzou` 1.3.4.8，非蓝奏云官方客户端，其登录协议不构成官方协议证据；官方客户端 APK 未提供。
- 登录可行性结论：**不可行（资料缺失）**
- 若需攻关，攻关点：不适用 —— 按 T-001 拍板结论不攻关、不硬凑
- 若不可行，原因及证据：现有 APK 为第三方客户端 `com.tooyoung.lanzou` 1.3.4.8，非蓝奏云官方客户端，其登录协议不构成官方协议证据；官方客户端 APK 未提供。

### 待确认问题

- 登录接口路径与签名机制？
- token 刷新与过期策略？
- 客户端环境校验（UA / 版本 / 指纹 / TLS）？
- 是否支持扫码登录？token 由哪一端换取？
- 列表 / 上传 / 下载 / 删除的接口形态？
- 分页方式（offset / cursor / page）？
- 是否存在验证码或风控挑战？

### 已知线索

- 暂无。**未做任何反编译，本条目不含任何接口事实。**

### 证据路径

- 无

## 6. 123云盘

- 包名 / 版本：`com.mfcloudcalculate.networkdisk` / 3.3.4（versionCode 3244）
- 资料：附件区 `123云盘_3.3.4.apk`（114861135 B）
- 主体技术栈：**Flutter 主体**（`libapp.so` arm64 18219952 B；dex 仅 13916 B、4 个壳桩类）
- 加固：**未检出已知整包加固壳**（无 `ijiami.*`、无 `libDexHelper.so`、无 `libjiagu*`）；另有 1 个 `libijm-emulator.so`(166360 B) 来源待确认
- 静态侦察状态：**T-002d 壳确认已完成**，协议覆盖 **0**

### 登录流程

- 登录类型：**未知**
- 是否可在 APP 内原生完成：**未知**
- 会话换取方式：**未知**
- 是否需要 OAuth 或 WebView 才能完成：**未知**
- 扫码登录 token 来源：**未知**
- token 刷新机制：**未知**
- token 有效期：**未知**
- 依据：`docs/evidence/index-123云盘.md`（壳确认，无协议事实）
- 登录可行性结论：**未知（属 T-002c，本条目不作判定）**
- 若需攻关，攻关点：需动态抓包；Dart AOT 静态分析成本高
- 若不可行，原因及证据：**未判定**

### 待确认问题

- 登录接口路径与字段？（`libapp.so` 内，静态不可读）
- token 结构、刷新与过期策略？
- `networkSecurityConfig=@0x7f15001f` 指向的实际信任集？
- 列表 / 上传 / 下载 / 删除的接口形态？分页方式？

### 已知线索

- **dex 内无协议线索**（4 个壳桩类、无域名常量、无 HTTP 栈）
- 业务逻辑在 `libapp.so`（Dart AOT），静态不可读

### 证据路径

- `docs/evidence/index-123云盘.md`

## 7. UC 网盘

- 包名 / 版本：`com.UCMobile` / 19.2.8.1564
- 资料：**实为 UC 浏览器**；136173692 字节，11 dex，Flutter 主体（`libdalvikhack.so`、`libdexloaderuc.so`、`libfakedexuc.so`）

### 登录流程

- 登录类型：**资料缺失**
- 是否可在 APP 内原生完成：**资料缺失**
- 会话换取方式：**资料缺失**
- 是否需要 OAuth 或 WebView 才能完成：**不可判定**
- 扫码登录 token 来源：**资料缺失**
- token 刷新机制：**资料缺失**
- token 有效期：**资料缺失**
- 依据：现有 APK 为 UC 浏览器 `com.UCMobile`，UC 网盘功能在浏览器内以 H5 页面提供，不存在独立的客户端登录协议证据；且 UC 浏览器为 Flutter 主体，业务逻辑在 libapp.so，静态反编译 dex 不可得。
- 登录可行性结论：**不可行（资料缺失）**
- 若需攻关，攻关点：不适用 —— 按 T-001 拍板结论不攻关、不硬凑
- 若不可行，原因及证据：现有 APK 为 UC 浏览器 `com.UCMobile`，UC 网盘功能在浏览器内以 H5 页面提供，不存在独立的客户端登录协议证据；且 UC 浏览器为 Flutter 主体，业务逻辑在 libapp.so，静态反编译 dex 不可得。

### 待确认问题

- 登录接口路径与签名机制？
- token 刷新与过期策略？
- 客户端环境校验（UA / 版本 / 指纹 / TLS）？
- 是否支持扫码登录？token 由哪一端换取？
- 列表 / 上传 / 下载 / 删除的接口形态？
- 分页方式（offset / cursor / page）？
- 是否存在验证码或风控挑战？

### 已知线索

- 暂无。**未做任何反编译，本条目不含任何接口事实。**

### 证据路径

- 无

## 8. 阿里云盘

- 包名 / 版本：`com.alicloud.databox` / 6.21.0（versionCode 281）
- 资料：附件区 `alicloud_shujubao_android_36176427979800.apk`（163545725 B）
- 主体技术栈：**Flutter 主体（混合架构：原生壳 + jsbridge + Flutter）**（`libapp.so` arm64 16794520 B；`flutter_assets` 仅 46 文件 / 861825 B；dex 25 个、70881 类，`com/alicloud` 15828）
- 加固：**未检出整包加固壳**；含**阿里聚安全 8 个文件**（sgmain 4 + sgsecuritybody 4，版本 6.6.240404）
- 静态侦察状态：**T-002d 壳确认已完成**，协议覆盖 **0**

### 登录流程

- 登录类型：**未知**
- 是否可在 APP 内原生完成：**未知**
- 会话换取方式：**未知**
- 是否需要 OAuth 或 WebView 才能完成：**未知**
- 扫码登录 token 来源：**未知**
- token 刷新机制：**未知**
- token 有效期：**未知**
- 依据：`docs/evidence/index-阿里云盘.md`（壳确认，无协议事实）
- 登录可行性结论：**未知（属 T-002c，本条目不作判定）**
- 若需攻关，攻关点：阿里聚安全（sgmain/sgsecuritybody）参与签名的程度；jsbridge 通道是否暴露文件操作
- 若不可行，原因及证据：**未判定**

### 待确认问题

- 登录接口路径与字段？
- jsbridge 桥接类（`OpenApiRequest` / `OpenApiCreateHandler` / `ALCloudDriveBridge`）与文件操作的实际关系？
- `res/xml/network_security_config.xml`（672 B）未在 Manifest 引用，是否生效？
- 阿里聚安全由哪些参数参与请求签名？
- 列表 / 上传 / 下载 / 删除的接口形态？分页方式？

### 已知线索

- **dex 内未确认网盘文件协议**。dex 有 70881 类，主体是阿里/淘宝/优酷等生态组件，类名可读但未验证归属
- 业务逻辑主体在 `libapp.so`（Dart AOT），静态不可读

### 证据路径

- `docs/evidence/index-阿里云盘.md`

## 9. 迅雷云盘

- 包名 / 版本：`com.xunlei.downloadprovider` / 25.2.91.280
- 资料：**迅雷为下载器，云盘是子模块**；199853782 字节，1 dex，非 Flutter，含加固壳 `libDexHelper.so`（2 处命中）

### 登录流程

- 登录类型：**资料缺失**
- 是否可在 APP 内原生完成：**资料缺失**
- 会话换取方式：**资料缺失**
- 是否需要 OAuth 或 WebView 才能完成：**不可判定**
- 扫码登录 token 来源：**资料缺失**
- token 刷新机制：**资料缺失**
- token 有效期：**资料缺失**
- 依据：现有 APK 为迅雷下载器 `com.xunlei.downloadprovider`，云盘只是其中的子模块；且 APK 带 `libDexHelper.so` 加固壳，dex 非 Flutter 但被壳保护，静态反编译拿不到业务逻辑；未提供独立的迅雷云盘客户端。
- 登录可行性结论：**不可行（资料缺失）**
- 若需攻关，攻关点：不适用 —— 按 T-001 拍板结论不攻关、不硬凑
- 若不可行，原因及证据：现有 APK 为迅雷下载器 `com.xunlei.downloadprovider`，云盘只是其中的子模块；且 APK 带 `libDexHelper.so` 加固壳，dex 非 Flutter 但被壳保护，静态反编译拿不到业务逻辑；未提供独立的迅雷云盘客户端。

### 待确认问题

- 登录接口路径与签名机制？
- token 刷新与过期策略？
- 客户端环境校验（UA / 版本 / 指纹 / TLS）？
- 是否支持扫码登录？token 由哪一端换取？
- 列表 / 上传 / 下载 / 删除的接口形态？
- 分页方式（offset / cursor / page）？
- 是否存在验证码或风控挑战？

### 已知线索

- 暂无。**未做任何反编译，本条目不含任何接口事实。**

### 证据路径

- 无

---

## 汇总：侦察进度

| 网盘 | 静态反编译 | 主体技术栈 | 覆盖率 | 登录流程已填 | 文件接口已填 |
|---|---|---|---|---|---|
| 百度网盘 | **T-002d 已完成** | Java/Kotlin 主体 + Flutter 模块 | **100.0000%**（63259/63259） | **静态线索已填（不作可行性判定）** | **静态线索已填（40+ 接口）** |
| 天翼云盘 | **T-002d 壳确认** | **Flutter 主体** + 爱加密 | 不适用（dex 仅 3 类壳桩） | 未知（无静态证据） | 未知（无静态证据） |
| 夸克网盘 | **T-002d 壳确认** | **Flutter 主体**（Dart 在 7z 容器内） | 不适用（协议不在 dex） | 未知（无静态证据） | 未知（无静态证据） |
| 坚果云 | **T-002b 已完成** | Java/Kotlin 主体（2 dex，无加固） | 100.0%（2752/2753） | **静态线索已填** | **静态线索已填** |
| 蓝奏云 | 不需要 | — | — | 资料缺失 | 资料缺失 |
| 123云盘 | **T-002d 壳确认** | **Flutter 主体** | 不适用（dex 仅 4 类壳桩） | 未知（无静态证据） | 未知（无静态证据） |
| UC 网盘 | 不需要 | — | — | 资料缺失 | 资料缺失 |
| 阿里云盘 | **T-002d 壳确认** | **Flutter 主体（混合：壳+jsbridge+Flutter）** | 不适用（协议主要在 libapp.so） | 未知（无静态证据） | 未知（无静态证据） |
| 迅雷云盘 | 不需要 | — | — | 资料缺失 | 资料缺失 |

> 「登录流程已填」只表示**字段是否被静态证据填过**，**不代表可行性已判定**；可行性统一由 T-002c 判定。
> 4 个 Flutter 网盘的「未知（无静态证据）」是技术栈决定的结果，不是本任务遗漏。

## 汇总：资料缺失说明

| 网盘 | 资料情况 |
|---|---|
| 蓝奏云 | APK 为第三方客户端，非官方；官方客户端协议**资料缺失**。结论：**不可行（资料缺失）** |
| UC 网盘 | 只有 UC 浏览器 APK，网盘功能为 H5；**原生登录资料缺失**。结论：**不可行（资料缺失）** |
| 迅雷云盘 | 迅雷 App 内含云盘模块，且带 `libDexHelper.so` 加固；**资料缺失**。结论：**不可行（资料缺失）** |
| 全部 9 个 | **网页端资料全部缺失** |

> 上述三个网盘按 T-001 拍板结论：记为"资料缺失、原生登录不可行"，
> **不硬凑、不降级为 OAuth/WebView/cookie**。若日后补充官方 APK 或网页端资料，
> 可重开任务重新判定。
