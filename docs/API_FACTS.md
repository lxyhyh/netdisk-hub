# 接口事实库（API_FACTS）

- 状态：草案 **v0.5**
- 产出任务：T-001（T-002a 资料定性 / T-002b 坚果云 / T-002d 百度 + 四个 Flutter 网盘壳确认 / **T-002c 九个网盘登录可行性判定**）
- 最后更新：2026-11-28

## 记录格式

每条记录包含：

```
网盘 / 功能 / 来源（APK 类#方法 或 网页 JS 文件:函数）/ 请求 / headers / body /
认证 / 响应 / 错误码 / 可信度（已验证|推测|待确认）/ 证据路径 / 未知问题
```

## 全局说明

**当前状态（T-002c 后）：9 个网盘的原生登录可行性已全部判定——可行 1 个（坚果云）、需攻关 3 个（百度、夸克、阿里）、未知 2 个（天翼、123）、不可行 3 个（蓝奏、UC、迅雷）。**

原因：

- 坚果云（T-002b）与百度（T-002d）是 `Java/Kotlin 主体`，协议在 dex 里，登录链路可完整读出；差别在于坚果云只有 Base64，百度有设备指纹与风控 SDK。
- 夸克、阿里虽然由 Flutter 承载**文件业务**，T-002c 定向反编译确认**账号登录体系完整存在于 dex 的 Java 层**（夸克 `UCLoginServiceImpl` + MTOP；阿里 `com/ali/user/mobile` + Havana/login4android），所以登录协议可读；两者都被阿里聚安全 native 阻断请求签名。
- 天翼、123 的 dex 只有 3/4 个壳桩类、**0 条登录线索**，业务在 `libapp.so`（Dart AOT），天翼另有爱加密整包载荷 → 属技术栈决定的上限，不是遗漏。
- 蓝奏、UC、迅雷按 T-001 拍板记为资料缺失，按 `docs/SCOPE.md` 第 6 节兜底，不攻关。

对照文件：`docs/evidence/login-feasibility.md`（T-002c 判定总表 + 逐网盘证据）、`docs/evidence/index-坚果云.md`、`docs/evidence/index-百度.md`、`docs/evidence/index-天翼.md`、`docs/evidence/index-123云盘.md`、`docs/evidence/index-夸克.md`、`docs/evidence/index-阿里云盘.md`。

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

### 登录流程字段（每个网盘必填 12 项）

每个网盘的「登录流程」小节按下列顺序写全 12 项，**字段不得删除**；确实未知的写「未知」并解释原因。

| # | 字段 | 取值 |
|---|---|---|
| 1 | 登录类型 | 账号密码 / 短信验证码 / 扫码 / 混合 / OAuth / 其他 / 未知 |
| 2 | 是否可在 APP 内原生完成 | 是 / 否 / 未知 |
| 3 | 会话换取方式 | 官方客户端接口 / OAuth / 仅网页端 / 未知 |
| 4 | 是否需要 OAuth 或 WebView 才能完成 | 是 / 否 / 未知 |
| 5 | 扫码登录 token 来源 | 客户端协议 / WebView 抓取 / 未知 |
| 6 | token 刷新机制 | 有 / 无 / 未知 |
| 7 | token 有效期 | 具体时长 / 未知 |
| 8 | 依据 | 类#方法 + 文件:行号 / 官方资料 |
| 9 | 登录可行性结论 | 原生登录可行 / 需攻关 / 不可行 / 未知 |
| 10 | 若需攻关 | 签名 / 设备指纹 / 版本校验 / TLS 指纹 / native 注入 / 其他 |
| 11 | 若不可行 | 原因及证据 |
| 12 | 若未知 | 缺失的关键证据及下一步需要的资料 |

四类结论的边界与判定细则见 `docs/evidence/login-feasibility.md` 第 1 节。

---

## 1. 百度网盘

- 包名 / 版本：`com.baidu.drive.app` / 13.11.15（versionCode 3711）
- 资料：`.apks` 分包（base.apk 251854857 B，20 dex 合计 182513748 B），**未加壳**
- 主体技术栈：**Java/Kotlin 主体 + Flutter 模块共存**（协议在 dex 侧；split 内另有 `libflutter.so` + `libapp.so` 20972464 B）
- 静态侦察状态：**T-002d 已完成**，覆盖率 **100.0000%**（63259/63259）

### 登录流程

1. 登录类型：**账号密码 / 短信验证码 / 扫码**（三条通道均有客户端协议；第三方 SSO 与 WebView 容器路径不计入）
2. 是否可在 APP 内原生完成：**是**。业务侧实际入口 `classes13/sources/com/baidu/netdisk/ui/account/i0.java:369` `passportSDK.startNewNALogin(...)` → `NewLoginActivity` / `NewLoginHalfActivity`（`classes4/sources/com/baidu/sapi2/newlogin/activity/`，搜 WebView/loadUrl 三处均 0 命中）
3. 会话换取方式：**官方客户端接口**。账号密码 `POST {wap}/wp/api/login`（`SapiAccountRepository.java:2873-2901`，端点 `:1126-1128`）；短信 `POST /v2/sapi/login`（同文件 `:263-298`）；扫码 `GET /v2/api/getqrcode` → `GET /channel/unicast` → `POST /v2/sapi/qrlogin?lp=app`（`QrCodeService.java:314-326 / :387-397 / :367-384`）。返回 `bduss` / `ptoken` / `stoken` / `uid`（`SapiAccountResponse.java:14-20`），落 `sapi_system` 偏好（`SapiContext.java:61-65`）
4. 是否需要 OAuth 或 WebView 才能完成：**否**。`https://openapi.baidu.com/oauth/2.0/token` 常量（`SapiAccountRepository.java:1111-1121`）只用于 `openConnection()` 取 `getDate()/1000` 当时间戳（`:1102`），不参与鉴权；`grep -rn '.oauth(' classes*/sources/com/baidu/netdisk/` → 0 命中。`web2NativeLogin`（`:2951-3010`）从 cookie 读 BDUSS，属官方内部账号中心通道，**本项目不计为可用路径**
5. 扫码登录 token 来源：**客户端协议**（`/v2/sapi/qrlogin`，参数 `sign` / `cmd` / `bduss` / `stoken` / `ptoken`），不是 WebView 抓取
6. token 刷新机制：**无主动刷新**。`SapiAccountManager.java:436` 有 `onBdussExpired` 回调，`SapiAccountRepository.java:2994-2997` 用错误码 400021 表示 bduss 过期（已验证）
7. token 有效期：**未知**（客户端无过期时间字段）
8. 依据：`docs/evidence/login-feasibility.md` §3.2（逐跳行号）；`docs/evidence/index-百度.md` §4
9. 登录可行性结论：**需攻关**
10. 若需攻关：① `di` 设备指纹，`SapiDeviceInfo.java:205-209 / :189-191 / :193-203 / :82-184` 采 40 个字段（含 IMEI/MAC/包列表/device_uuid），AES 密钥为明文常量（`:8`），**算法可复刻，但字段裁剪受灰度开关控制**；② `tpl` / `appSignKey` 运行期注入（`getUaInfo()` `:198-200`），取值**待确认**；③ 聚安全 / `passport_core`（`PassCoreVHelper.java:4-19` 六个 native），使用点仅 `SapiConfiguration.java:553/562`、`OneKeyLoginSdkCall.java:131-132`、`ShareStorage.java:259/290/352`，**不在账号密码/短信主链**；④ 服务端风控放行强度**待确认**
11. 若不可行：不适用（无不可行证据）
12. 若未知：不适用（已定位到可枚举的攻关项，非资料不足）

> **修正 T-002d 的两处**：原写「扫码登录未发现独立实现」「换取接口路径未定位」，T-002c 已在 `QrCodeService.java` 定位到三步扫码客户端协议与 `wapLogin` 端点，两处作废。
> **Sdata 不影响登录**：`Sdata` 头只在自研网络栈加（`classes6/sources/com/baidu/netdisk/network/c.java:126` 等 3 处，且 `c.java:105-130` 条件是 `!isHttps && !whitelist`）；sapi2 链路（`com/baidu/pass/http/a.java`）只设 UA 与 Content-Type，在 `classes2`/`classes12` 的 sapi2 目录搜 `Sdata` → 0 命中。Sdata 签名实现仍是**确定缺口**（`hk/__.java:469-475` 纯转发 `Function2.invoke`，`.n0(Function2` 静态调用方 0 命中），但它**不属于登录链路**，本文不写成已还原。

### 待确认问题

- `Sdata` 的 HMAC 实现注入点（`hk/__.G` 的 `Function2` 由谁赋值）？
- `RequestCommonParams` 的 User-Agent / 设备参数真实值？
- token 刷新与过期策略？
- ~~扫码登录是否存在（dex 内未发现）？~~ **已关闭**：T-002c 定位到 `QrCodeService.java` 三步扫码客户端协议（getqrcode → /channel/unicast → /v2/sapi/qrlogin）
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

1. 登录类型：**未知**（dex 仅 3 个壳桩类，login/auth 类名命中 0、URL 命中 0）
2. 是否可在 APP 内原生完成：**未知**
3. 会话换取方式：**未知**
4. 是否需要 OAuth 或 WebView 才能完成：**未知**
5. 扫码登录 token 来源：**未知**
6. token 刷新机制：**未知**
7. token 有效期：**未知**
8. 依据：`docs/evidence/login-feasibility.md` §3.5；`docs/evidence/index-天翼.md`（壳确认，协议覆盖 0）
9. 登录可行性结论：**未知（需动态分析）**
10. 若需攻关：不适用（尚未到可枚举攻关点的阶段）
11. 若不可行：不适用（**无任何证据表明必须走 OAuth/WebView**，不得判不可行）
12. 若未知：缺失 ①登录接口路径与字段（在 `libapp.so` Dart AOT 内）②请求签名机制（`libsecurity.so` 21528 B 职责未验证）③会话 token 结构与刷新 ④客户端环境校验（UA / 版本 / 指纹 / TLS pin）⑤爱加密载荷运行期解密流程（`ijiami.dat` 22378961 B + `ijiami.ajm` 10890680 B + `libexecmain.so` 215640 B / `libexec.so` 765648 B 职责未验证）。下一步需：动态抓包 + 爱加密脱壳 + Dart AOT 分析

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

1. 登录类型：**手机号短信验证码**（另有账号密码 `cas.login`；微信/淘宝/支付宝 SSO 属不合格路径，**不计入**）
2. 是否可在 APP 内原生完成：**是**（登录实现在 dex 的 Java 层：`classes10` 的 `com/ucpro/feature/personal/login` 159 类、`classes7` 的 `com/uc/base/account` 24 类）
3. 会话换取方式：**官方客户端接口**。`POST {base}/acs.wsg_access`（`classes7/sources/rn0/a.java:6-59`），业务 method 为 `cas.sendSmsCodeForLogin`（`UCLoginServiceImpl.java:232-280`）/ `cas.loginWithSmsCode`（`:190-230`）/ `cas.login`（`:333`）；返回 `service_ticket`（`.../account/service/account/login/e.java:24`）。base URL 三选一，网盘侧为 `https://access-open.quark.cn`（`vn0/a.java:10-15`）
4. 是否需要 OAuth 或 WebView 才能完成：**否**（`PersonalLoginPresenter.java:167-181` 的 WebView 只用于打开隐私/用户协议，与凭证无关）
5. 扫码登录 token 来源：**未知**（本轮定向产物中未定位到扫码登录链路）
6. token 刷新机制：**未知**（`e.java:39` 记录 `expiresIn`，语义未验证）
7. token 有效期：**未知**
8. 依据：`docs/evidence/login-feasibility.md` §3.3（逐跳行号）
9. 登录可行性结论：**需攻关**
10. 若需攻关：① **请求体 M9 加密是 native**——`com/uc/encrypt/EncryptHelper.java:314-325` `System.loadLibrary("encrypt")`、`:327-359` `SecurityGuardManager.getInitializer().initialize(context)`、`M9EncryptionHandler.java:16/18` `private native byte[] decodeM9/encodeM9`；② **`security_sign` 是聚安全 native**——`.../account/c.java:12-14` → `tn0/a.java:29-43` 取 `getSecureSignatureComp().signRequest(...)`，INPUT 为 `str + "3dsa33@213!!!22"`，requestType=3；③ `client_id="636"` 与 `app_name:qk_clouddrive` 是否硬校验 → 待确认
11. 若不可行：不适用
12. 若未知：不适用

> **与 T-002d 的差异（必须保留）**：T-002d 记为「Flutter 主体，协议需动态分析」。T-002c 定向反编译 `classes7`/`classes10`（10505 个 .java）后确认：**网盘文件业务协议仍在 Dart 侧，但账号登录体系完整存在于 dex 的 Java 层**。桥接统计：自家包 `MethodChannel` 仅 2 处（ASR/TTS）、`Plugin` 73 处集中在 webwindow/nezha → 登录不在 Flutter 侧。

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

1. 登录类型：**账号密码**（主路径）；另有验证码 / 二次验证（`tfPasscode`）分支
2. 是否可在 APP 内原生完成：**是**。`LoginActivity.java:306` / `:375` → presenter `v2/ui/login/m.java:21` → `v2/ui/login/o.java:35` → `connection/i.java:38-43` → `connection/o.java` 发 HTTP，全链无 WebView
3. 会话换取方式：**官方客户端接口** `POST /mcmd/loginV2`（常量 `connection/w.java:11`，构造 `:171-196`），body 为 JSON 9 字段 `userName` / `password` / `tfPasscode` / `custom_ticket` / `exp` / `sig` / `reusable` / `ticket` / `randstr`；响应 `token` / `uid` / `username`（`NutstoreRequestHelper$LoginSignupResponse.java:5-7`、`:12-14`），落库 `ja.java:30-60`，后续请求带 `Authorization: <B64(token)>-<B64(uid)>:<userName>`（`HttpConfig.java:36-43`）
4. 是否需要 OAuth 或 WebView 才能完成：**否**（`grep -rn 'WebView|loadUrl|oauth|OAuth' .../v2/ui/login/` → 0 命中）
5. 扫码登录 token 来源：**未发现扫码登录**（已搜 `NutstoreApi.java` 全部注解、`w.java` 全部常量、全包 `qr`/`QrCode`/`scan`）
6. token 刷新机制：**无**。无 refresh 接口、无定时刷新；失效由 `AccountExpired` 错误码跳登录页
7. token 有效期：**客户端无过期字段**（服务端有效期 → 未知）。**注意区分**：工作区 token 另有一套，`WorkspaceRepository.java:251` 把有效期**写死 14 天**（`System.currentTimeMillis() + 1209600000`），`:199` 有 `access-token/apply` 刷新通道，401 清库（`:145-152`）——该 token 只用于工作区功能，与网盘文件接口的 Authorization 不是同一套
8. 依据：`docs/evidence/login-feasibility.md` §3.1（逐跳行号）；`docs/evidence/index-坚果云.md` §4
9. 登录可行性结论：**原生登录可行（静态可行，动态未验证）**
10. 若需攻关：无阻断项。遗留待确认：UA 四段格式（`HttpConfig.java:32-34`）是否硬校验；`isNeedRealCaptcha()` 触发阈值；腾讯人机验证的 `ticket`/`randstr` 换取方式
11. 若不可行：不适用（无不可行证据）
12. 若未知：不适用

> **密码处理**：`nutstore.android.utils.y.H()`（`utils/y.java:124-126`）**只做 Base64，无 MD5/SHA 加盐**（已验证，静态）。
> **禁止项排除**：唯一第三方介入是腾讯人机验证控件（`LoginActivity.java:152-155`），仅在 `isNeedRealCaptcha()` 为真时出现，**不是授权降级**。

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

1. 登录类型：**未知**（资料缺失）
2. 是否可在 APP 内原生完成：**未知**（资料缺失）
3. 会话换取方式：**未知**（资料缺失）
4. 是否需要 OAuth 或 WebView 才能完成：**未知**（资料缺失）
5. 扫码登录 token 来源：**未知**（资料缺失）
6. token 刷新机制：**未知**（资料缺失）
7. token 有效期：**未知**（资料缺失）
8. 依据：现有 APK 为第三方客户端 `com.tooyoung.lanzou` 1.3.4.8（4765515 B，1 dex，无加固），**非蓝奏云官方客户端**，其登录协议不构成官方协议证据
9. 登录可行性结论：**不可行（资料缺失）**
10. 若需攻关：不适用 —— 按 `docs/SCOPE.md` 第 6 节兜底策略不硬凑、不攻关
11. 若不可行：第三方客户端协议不等同于官方客户端协议；官方客户端 APK 与网页端资料均未提供。按项目准入规则不进入 MVP
12. 若未知：不适用（已按资料缺失兜底规则定案，不再作为未知候选）

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

1. 登录类型：**未知**（dex 仅 4 个壳桩类，login/auth 类名命中 0、URL 命中 0）
2. 是否可在 APP 内原生完成：**未知**
3. 会话换取方式：**未知**
4. 是否需要 OAuth 或 WebView 才能完成：**未知**
5. 扫码登录 token 来源：**未知**
6. token 刷新机制：**未知**
7. token 有效期：**未知**
8. 依据：`docs/evidence/login-feasibility.md` §3.6；`docs/evidence/index-123云盘.md`（壳确认，协议覆盖 0）
9. 登录可行性结论：**未知（需动态分析）**
10. 若需攻关：不适用（尚未到可枚举攻关点的阶段）
11. 若不可行：不适用（**无任何证据表明必须走 OAuth/WebView**，不得判不可行）
12. 若未知：缺失 ①登录接口路径与字段（在 `libapp.so` arm64 18219952 B 内）②签名机制 ③token 结构与刷新 ④UA / 版本 / 指纹校验。下一步需：动态抓包 + Dart AOT 分析。**与天翼阻塞点不同**：dex 只有 4 类是因为业务写在 Flutter AOT 里、**不是被整包加密**（未检出 `ijiami.*` / `libDexHelper.so` / `libjiagu*`），入口是「Flutter 侧」而非「脱壳」；但 `libijm-emulator.so` 166360 B 提示可能有运行期反调试/反模拟器，会干扰抓包（来源待确认）

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

1. 登录类型：**未知**（资料缺失）
2. 是否可在 APP 内原生完成：**未知**（资料缺失）
3. 会话换取方式：**未知**（资料缺失）
4. 是否需要 OAuth 或 WebView 才能完成：**未知**（资料缺失）
5. 扫码登录 token 来源：**未知**（资料缺失）
6. token 刷新机制：**未知**（资料缺失）
7. token 有效期：**未知**（资料缺失）
8. 依据：现有 APK 为 **UC 浏览器** `com.UCMobile` 19.2.8.1564（136173692 B，11 dex，Flutter 主体），UC 网盘功能在浏览器内以 **H5 页面**提供，不存在独立的客户端登录协议证据
9. 登录可行性结论：**不可行（资料缺失）**
10. 若需攻关：不适用 —— 按 `docs/SCOPE.md` 第 6 节兜底策略不硬凑、不攻关
11. 若不可行：不存在独立客户端登录协议；网盘为 H5，意味着登录走网页会话，而网页 cookie 复用是项目明令禁止的路径。**不得因「浏览器里能登录」而改判可行**
12. 若未知：不适用（已按资料缺失兜底规则定案）

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

1. 登录类型：**手机号短信验证码 / 账号密码 / 扫码**（Havana 账号 SDK + 淘宝 login4android；支付宝 SSO 与 cookie 通道属不合格路径，**不计入**）
2. 是否可在 APP 内原生完成：**是**（登录实现在 dex 的 Java 层：`classes11` 的 `com/alicloud/databox/account/login` 40 类 + `havana` 41 类，`classes9` 的 `com/ali/user/mobile` 246 类）
3. 会话换取方式：**官方客户端接口（MTOP）**。接口名明文在 `classes9/sources/com/ali/user/mobile/rpc/ApiConstants.java:66-120`：短信登录 `mtop.taobao.mloginService.smsLogin`(:80) / 发短信 `smsSend`(:73)；账号密码 `com.taobao.mtop.mloginService.login`(:119) 或 `mtop.taobao.alibabaMLoginService.login`(:115)；扫码 `HAVANA_GEN_QR_CODE`(:105) / `HAVANA_LOGIN_BY_QR_CODE`(:106)；登出(:110)。请求构造见 `UserLoginServiceImpl.java:19-34` / `:37-83`、`LoginDataRepository.java:26/87/109/113-116`
4. 是否需要 OAuth 或 WebView 才能完成：**否**。但 `GET_WAP_LOGIN_COOKIES`(:104) / `GET_ALIPAY_COOKIES`(:99) 属 cookie 通道，**本项目不得使用**
5. 扫码登录 token 来源：**客户端协议**（Havana 二维码接口族），完整字段未读 → 部分待确认
6. token 刷新机制：**有**（`LoginViewModel.java` 的 `recheckLoginToken` / `refreshInfo`），具体周期 → 未知
7. token 有效期：**未知**
8. 依据：`docs/evidence/login-feasibility.md` §3.4（逐跳行号）
9. 登录可行性结论：**需攻关**
10. 若需攻关：① **每个登录请求都注入 `riskControlInfo`**——`com/ali/user/mobile/data/LoginComponent.java:127` 与 `:441` 均 `rpcRequest.addParam("riskControlInfo", JSON.toJSONString(SecurityGuardManagerWraper.buildWSecurityData()))`，注册链路 `RegisterComponent.java:59/116/176/196` 同样注入；② MTOP 签名（umt/wua）依赖聚安全——`anet/channel/security/SecurityGuardImpl.java:61-69` 取 `getSecureSignatureComp().signRequest(...)`，`:19` `Class.forName("com.alibaba.wireless.security.open.SecurityGuardManager")`；包内 8 个聚安全 so（`libsgmainso-6.6.240404.so` 1873964 B、`libsgsecuritybodyso-6.6.240404.so` 649484 B 等）；③ `classes18.dex` 的 `AccountRpc` 主类**本轮未反编译** → 待补
11. 若不可行：不适用
12. 若未知：不适用

> **与 T-002d 的差异（必须保留）**：T-002d 记为「Flutter 主体（混合），协议需动态分析」。T-002c 定向反编译 `classes9`/`classes11`（4050 个 .java）后确认：**账号登录体系完整存在于 dex 的 Java 层**，只有**文件业务协议**在 Dart / jsbridge 侧。桥接统计：自家包 `MethodChannel` 1 处、`Plugin` 2 处、`jsbridge` 262 处 → 登录不在 Flutter 层。

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

1. 登录类型：**未知**（资料缺失）
2. 是否可在 APP 内原生完成：**未知**（资料缺失）
3. 会话换取方式：**未知**（资料缺失）
4. 是否需要 OAuth 或 WebView 才能完成：**未知**（资料缺失）
5. 扫码登录 token 来源：**未知**（资料缺失）
6. token 刷新机制：**未知**（资料缺失）
7. token 有效期：**未知**（资料缺失）
8. 依据：现有 APK 为**迅雷下载器** `com.xunlei.downloadprovider` 25.2.91.280（199853782 B，1 dex，非 Flutter），云盘只是其中一个子模块；dex 被 `libDexHelper.so` 加固壳保护，静态拿不到业务逻辑；未提供独立的迅雷云盘客户端
9. 登录可行性结论：**不可行（资料缺失）**
10. 若需攻关：不适用 —— 按 `docs/SCOPE.md` 第 6 节兜底策略不硬凑、不攻关
11. 若不可行：无独立云盘客户端；模块边界与协议归属在现有资料下无法确认；dex 被加固壳保护。按项目准入规则不进入 MVP
12. 若未知：不适用（已按资料缺失兜底规则定案）

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

| 网盘 | 静态反编译 | 主体技术栈 | 覆盖率 | 登录流程 12 字段 | 文件接口 | **登录可行性（T-002c）** |
|---|---|---|---|---|---|---|
| 百度网盘 | T-002d 已完成 | Java/Kotlin 主体 + Flutter 模块 | 100.0000%（63259/63259） | **已填满** | 静态线索（40+ 接口） | **需攻关** |
| 天翼云盘 | T-002d 壳确认 | Flutter 主体 + 爱加密 | 不适用（dex 仅 3 类壳桩） | **已填满（12 项均为未知）** | 未知 | **未知（需动态分析）** |
| 夸克网盘 | T-002c 定向反编译 classes7/10 | Flutter 承载文件业务；**登录在 Java 层** | 不适用（登录不在 Dart 侧） | **已填满** | 未知（Dart 侧） | **需攻关** |
| 坚果云 | T-002b 已完成 | Java/Kotlin 主体（2 dex，无加固） | 100.0%（2752/2753） | **已填满** | 静态线索 | **原生登录可行**（静态可行，动态未验证） |
| 蓝奏云 | 不需要 | — | — | **已填满（资料缺失）** | 资料缺失 | **不可行（资料缺失）** |
| 123云盘 | T-002d 壳确认 | Flutter 主体 | 不适用（dex 仅 4 类壳桩） | **已填满（12 项均为未知）** | 未知 | **未知（需动态分析）** |
| UC 网盘 | 不需要 | — | — | **已填满（资料缺失）** | 资料缺失 | **不可行（资料缺失）** |
| 阿里云盘 | T-002c 定向反编译 classes9/11 | Flutter + jsbridge 承载文件业务；**登录在 Java 层** | 不适用（登录不在 Dart 侧） | **已填满** | 未知（Dart/jsbridge 侧） | **需攻关** |
| 迅雷云盘 | 不需要 | — | — | **已填满（资料缺失）** | 资料缺失 | **不可行（资料缺失）** |

> 「登录流程 12 字段」指 `docs/evidence/login-feasibility.md` 第 2 节总表与本节各条目是否写全 12 项，与「可行性结论」是两件事。
> **夸克与阿里的登录实现不在 Dart 侧**——这一点修正了 T-002d 把四家统一记为「协议需动态分析」的写法。文件业务协议仍在 Dart/jsbridge 侧，T-002d 的结论对文件协议仍成立。
> 4 个 Flutter 网盘中，天翼/123 的「未知」是技术栈决定的结果，不是本任务遗漏；夸克/阿里的「需攻关」是**已有证据证明依赖闭源 native**，不是资料不足。

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
