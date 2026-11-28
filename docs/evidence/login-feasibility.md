# 登录可行性判定（login-feasibility）

- 状态：T-002c 产物 v1.3（百度小节经 T-006 复核改写，并经 2026-11-28 最终一致性校正）
- 产出任务：T-002c 各网盘登录流程侦察与可行性判定
- 依据提交：6ac999dc13c47f0eb92db0464d13bf16e960394d
- 最后更新：2026-11-28（T-006 复核）
- 判定对象：本项目唯一允许的登录方式——由本项目 Rust Adapter 通过官方客户端登录协议主动换取 token。账号密码 / 短信验证码 / 扫码三类界面均认可，但必须走官方客户端协议，不得依赖 OAuth、WebView 提取 cookie、手动粘贴 token、网页会话复用或任何间接授权。

---

## 0. 本文边界与读法

**本文全部结论来自 APK 静态代码阅读，没有发过任何真实网络请求，也没有登录过任何真实账号。**

因此：

- **静态代码发现** = 在反编译产物里读到的类、方法、字符串、调用关系（带 文件:行号）。
- **运行时验证** = 真实请求打到服务端后服务端的实际响应。**本文一项都没有做。**
- 「已验证」在本文里只表示**代码里确实这么写**，不表示**服务端确实这么响应**。
- 服务端反爬 / 风控 / 版本校验 / 灰度开关是否放行，本文一律不背书。

纪律（与 AGENTS.md 一致）：

- 没有证据的不得写成事实；推测必须显式标注；可信度只用「已验证 / 推测 / 待确认」三档。
- 「未发现」必须写明搜索方式，否则不算证据。
- 不写真实账号、密码、手机号、cookie、token、密钥值。
- 反编译产物在 reverse/ 下，**不进 Git**（.gitignore:2）。本文只保留路径与行号，不复制源码。

---

## 1. 判定标准

### 1.1 四类结论

| 结论 | 判定条件 |
|---|---|
| **原生登录可行** | 有完整证据证明可通过官方客户端协议，在本项目自己的 APP 内主动换取会话，不依赖 OAuth、WebView、网页 cookie 等不合格路径 |
| **需攻关** | 已确认存在符合要求的官方客户端原生认证路径，但签名、设备指纹、版本校验、native 注入等关键环节仍需解决 |
| **不可行** | 已有证据证明官方客户端协议不满足项目登录约束，或命中项目已批准的资料缺失兜底规则（docs/SCOPE.md 第 6 节） |
| **未知** | 当前资料不足，不能证明可行也不能证明不可行；必须明确缺少什么资料 |

**「需攻关」不是「原生登录可行」的同义词**，**「未知」不是「不可行」**。

### 1.2 三层区分（不得从第 1 层跳到第 3 层）

1. **界面存在**：APK 里存在原生登录页面。
2. **协议存在**：能追踪到实际的官方客户端认证请求与会话换取。
3. **独立实现可行**：本项目能在不使用不合格登录方式的前提下实现该协议。

本文逐网盘标注落在哪一层。

### 1.3 一条贯穿全篇的事实

官方客户端「自己能在 App 内登录」**不构成**充分条件。判定标准是：该登录流程是否可由本项目**独立复刻**，而不是依赖不可复用的网页会话、外部授权或 native 闭源组件。

---

## 2. 总表

| 网盘 | 登录类型 | 原生登录可行性 | 主要证据 | 缺失环节 / 攻关点 | 是否进入 MVP 候选 |
|---|---|---|---|---|---|
| 坚果云 | 账号密码（含验证码/二次验证分支） | **原生登录可行**（静态可行，动态未验证） | POST /mcmd/loginV2；密码仅 Base64；Authorization 注入链完整；无 OAuth/WebView | 服务端对非官方 UA 的校验策略、验证码触发条件 → 动态验证 | **是**（T-006 前唯一满足硬准入） |
| 百度网盘 | 账号密码 / 短信验证码 / 扫码（三通道均有客户端协议） | **原生登录可行**（静态可行，动态未验证；**仅 A/C 两条路径成立，B 为需攻关**，见 3.2、§3.2.9 与 T-006 补充收口） | 账号密码 `POST {wap}/wp/api/login`（RSA 加密）；短信发码 `POST {wap}/wp/api/login/sms`、提交 `POST {wap}/wp/api/login`（`sms=1`+`smsvc`）；扫码 `GET /v2/api/getqrcode` → `GET /channel/unicast` → `GET /v2/api/bdusslogin`（或 `/v3/login/main/qrbdusslogin`）；bduss/ptoken/stoken 落 sapi_system | 服务端是否接受本项目构造的 sig / di、diExceptIndex 实际下发值、额外风控挑战；短信路径另有**前置 WebView 人机验证**（`pass-machine.html`）静态不可复刻 → 动态验证 | **是**（硬准入满足；B 路径不计入「可独立实现」） |
| 夸克网盘 | 手机号短信验证码（第三方 SSO 属不合格路径，不计入） | **需攻关** | cas.sendSmsCodeForLogin / cas.loginWithSmsCode / cas.login；POST {base}/acs.wsg_access；返回 service_ticket | 请求体 M9 加密依赖 libencrypt.so；签名依赖聚安全 libsgmainso*.so（SecurityGuardManager）→ 这两个 native 静态不可复刻 | **条件候选**（攻关成本高） |
| 阿里云盘 | 手机号短信 / 账号密码 / 扫码（Havana 账号 SDK） | **需攻关** | mtop.taobao.mloginService.smsSend / smsLogin、mtop.taobao.alibabaMLoginService.login；GET_WAP_LOGIN_COOKIES；Havana 二维码接口 | 每个登录请求注入 riskControlInfo（聚安全 native）；MTOP 签名（umt/wua）依赖 libsgmainso*.so / libsgsecuritybody*.so | **条件候选**（攻关成本高） |
| 天翼云盘 | 未知 | **未知（需动态分析）** | dex 仅 3 个壳桩类、0 条登录线索；业务在 libapp.so + 爱加密载荷 | 需动态抓包 + Dart AOT 分析；libsecurity.so / libexecmain.so 职责未验证 | **否**（未知不进阶段 2） |
| 123云盘 | 未知 | **未知（需动态分析）** | dex 仅 4 个壳桩类、0 条登录线索；业务在 libapp.so | 需动态抓包；无已知加固壳但 Flutter AOT 静态成本高 | **否**（未知不进阶段 2） |
| 蓝奏云 | 资料缺失 | **不可行（资料缺失）** | 仅有第三方客户端 com.tooyoung.lanzou，非官方协议样本 | 不攻关（docs/SCOPE.md 第 6 节兜底） | **否** |
| UC 网盘 | 资料缺失 | **不可行（资料缺失）** | 仅有 UC 浏览器 com.UCMobile，网盘为浏览器内 H5 | 不攻关（同上） | **否** |
| 迅雷云盘 | 资料缺失 | **不可行（资料缺失）** | 迅雷下载器 com.xunlei.downloadprovider，云盘为子模块，且带 libDexHelper.so | 不攻关（同上） | **否** |

> 「是否进入 MVP 候选」列只表示**是否符合 T-005 的准入资格**，不表示已选定。**MVP 最终选哪 2 个由 T-005 决定，本文不作选择。**

---

## 3. 逐网盘判定

### 3.1 坚果云

**结论：原生登录可行（静态可行，动态未验证）** — 落在第 3 层（独立实现可行）。

#### 3.1.1 APK / 资料来源

- nutstore.android 4.26.5（versionCode 174），60329002 B，compileSdk 34 / minSdk 22 / targetSdk 34，2 dex，**未加壳**。
- 反编译：jadx 1.5.7，覆盖率 2752/2753（100.0%，口径见 docs/evidence/index-坚果云.md 第 2.1.1 节）。
- 索引：docs/evidence/index-坚果云.md 第 4 节。

#### 3.1.2 登录调用链（完整闭环，每跳都有行号）

| 跳 | 位置 | 动作 |
|---|---|---|
| 1 | reverse/nutstore/sources/sources/nutstore/android/v2/ui/login/LoginActivity.java:306 | 界面回调 this.B.h(str, str2, null, null, null)（B 是 presenter 接口 v2/ui/login/l） |
| 2 | 同文件 :375 | 验证码事件 this.B.h(this.E.toString(), this.I.toString(), nVar.D(), null, null)（带 nVar.D() 作为验证码票据） |
| 3 | 同文件 :103 | presenter 注入 setPresenter(new nutstore.android.v2.ui.login.m(this, nutstore.android.v2.g.D(), teamTrialRequest)) |
| 4 | .../v2/ui/login/m.java:21 | public void h(String str, String str2, String str3, String str4, String str5) |
| 5 | 同文件 :22-25 | str4 非空 → 直接带票据走 :64；否则 :43 走 D(str, str2, str3) |
| 6 | 同文件 :58-60 | D(str,str2,str3) 内 rx.Observable.create(new nutstore.android.v2.ui.login.y(this, str)) → **先预取验证码** |
| 7 | .../v2/ui/login/f.java:29-35 | onNext(CaptchaResponse)：isNeedRealCaptcha() 为真 → 弹人机验证；否则 this.I.h(this.e, this.G, this.J, null, null) 继续登录 |
| 8 | reverse/nutstore/sources-recovered/nutstore/android/v2/ui/login/o.java:35 | **真正发请求**：nutstore.android.connection.i.D(this.H, this.c, this.J, this.I, this.A, this.E, this.d, this.m, this.G)（9 参） |
| 9 | 同文件 :36 | 成功回调 nutstore.android.utils.ja.D(resp.username, this.H, resp.token, null) |
| 10 | .../connection/i.java:38-43 | D(9 参) → D(URI, 10 参) → m2445D(w.D(uri, ...))（构造请求对象） |
| 11 | 同文件 :48 | nutstore.android.connection.o.D(...) 发 HTTP |
| 12 | 同文件 :51-58 | 解析成 NutstoreRequestHelper$LoginSignupResponse |

#### 3.1.3 认证请求及字段

- 方法 / 路径：**POST /mcmd/loginV2**（常量 reverse/nutstore/sources/sources/nutstore/android/connection/w.java:11），**无 Query 参数**。
- body 为 JSON，9 个字段（构造见 w.java:171-196）：

| 字段 | 来源 | 可信度 |
|---|---|---|
| userName | nutstore.android.utils.y.H(str)（Base64） | 已验证 |
| password | nutstore.android.utils.y.H(str2) —— **仅 Base64，无 MD5/SHA 加盐**（.../utils/y.java:124-126） | 已验证 |
| tfPasscode | 二次验证码，非空才写 | 已验证 |
| custom_ticket / exp / sig / reusable | 来自 GET /mcmd/getLoginCaptchaV2 的 CaptchaResponse | 已验证 |
| ticket / randstr | 人机验证（腾讯验证码）回填 | 已验证 |

- 基址：new URI("https","app.jianguoyun.com",null,null,null)（.../v2/g.java:79-90），可被自定义 server_uri 覆盖（zl.m3377D()）。
- 请求头（.../v2/data/remote/api/HttpConfig.java:28-34）：Accept-Language（Locale.getDefault().toString()）、User-Agent = NutstoreApp-Android-<Build.VERSION.RELEASE>-<Build.MODEL>-<versionName>。

#### 3.1.4 token 换取与存储

- 响应字段：NutstoreRequestHelper$LoginSignupResponse.java:5-7 的 token / uid / username，取值 :12-14（hVar.m2815h("token") 等）。
- 落库：.../utils/ja.java:30-60 D(userName, loginName, token, uri)
  - :39-41 写 dao.x.E（值 "login_name"）
  - :42-43 写 dao.x.d（值 "user_name"）
  - :44-49 machine_name 为空则生成 UUID.randomUUID()
  - :50-51 写 "token"
  - :52-59 server_uri（仅自定义服务器时）
- 后续请求注入：HttpConfig.java:36-43 String.format("%s-%s:%s", Base64(token), Base64(uid), userName) 作为 Authorization。
- **刷新机制：无。** 主 token 在客户端没有任何 refresh 接口或定时刷新逻辑，失效由 AccountExpired 错误码触发跳登录页。
- **例外（必须区分）**：工作区（workspace.jianguoyun.com）另有一套独立 token，见 reverse/nutstore/sources-recovered/nutstore/android/v2/data/WorkspaceRepository.java：
  - :141 GET .../ticket-list/summary，header x-ns-mobile-auth: <accessToken>
  - :199 GET .../access-token/apply（带 CookieJar）刷新
  - :235-236 读 payload.data.access_token / payload.data.expires_in
  - :251 **客户端把有效期写死 System.currentTimeMillis() + 1209600000 ＝ 14 天**
  - :145-152 401 视为失效并清库（dao.d.m2482D(dao.x.i)）
  → 该 token 只用于工作区功能，**与网盘文件接口的 Authorization 不是同一套**。

#### 3.1.5 OAuth / WebView / cookie 排除情况（已验证）

- 搜索方式：grep -rn 'WebView|loadUrl|oauth|OAuth' reverse/nutstore/sources/sources/nutstore/android/v2/ui/login/ → **0 命中**。
- 登录全程无 WebView、无 OAuth、无 cookie 复用。
- 唯一的第三方介入是**腾讯人机验证控件**（LoginActivity.java:152-155 tencentCaptchaView），且仅在 isNeedRealCaptcha() 为真时出现。它不是授权降级，但**是「需攻关」级别的交互环节**（本项目需自行实现等价的人机验证交互）。
- 网络层无 CookieJar（全包搜 Cookie 只命中 WebView CookieManager 清空与 mupdf 的 Cookie 类）。

#### 3.1.6 最终结论与理由

- **第 1 层（界面存在）**：有原生登录页 LoginActivity。
- **第 2 层（协议存在）**：有完整请求构造 + 响应解析 + 凭据落库 + 请求注入，四段闭环。
- **第 3 层（独立实现可行）**：**满足**。判定依据：无 OAuth、无 WebView、无 native 签名、无设备指纹、密码处理只有 Base64、token 就是普通字符串。App 能在自己进程内做到的加密，本项目用 Rust 同样能做。

#### 3.1.7 未解决问题及后续工作

| 项 | 内容 | 可信度 |
|---|---|---|
| UA 是否硬校验 | 服务端是否校验 NutstoreApp-Android-* 四段格式 | 待确认（需真实请求） |
| 验证码触发阈值 | isNeedRealCaptcha() 由服务端返回，触发条件未知 | 待确认 |
| tfPasscode 启用条件 | 二次验证码的业务开关 | 待确认 |
| token 服务端有效期 | 客户端无过期字段 | 待确认 |
| 人机验证实现 | 腾讯验证码的 ticket/randstr 换取方式 | 待确认 |

---

### 3.2 百度网盘

**结论：原生登录可行（静态可行，动态未验证）** — 落在第 3 层（独立实现可行）。

> 本小节已于 T-006 复核改写。原 T-002c 判定为「需攻关」，改动依据与逐项证据见 `docs/evidence/baidu-login-deep-dive.md`（857 行，14 节 + 2 附录）。**改动只涉及百度一家，其余八家结论不变。**

#### 3.2.1 APK / 资料来源

- com.baidu.drive.app 13.11.15（versionCode 3711），base.apk 251854857 B，20 dex 合计 182513748 B，未加壳。
- 反编译覆盖率 63259/63259（100.0000%），索引：docs/evidence/index-百度.md 第 4 节。
- T-006 期间补充读取：`classes15/sources/kd/______.java`、`classes15/sources/com/baidu/netdisk/account/AccountUtils.java`、`classes4/.../httpwrap/{HttpClientWrap,HttpHashMapWrap,Utils}.java`、`classes12/.../sofire/**`、`classes2/.../{SapiCache,SapiOptions,SapiSafeFacade}.java`，以及 `split_config.arm64_v8a.apk` 内的 `libpassport_core.so` 等 7 个 so。

#### 3.2.2 登录调用链（业务侧实际入口）

**结论先行：网盘主登录页不直接调 PassportSDK.startLogin，真正的入口是 startNewNALogin（焕新原生登录）。**

| 跳 | 位置 | 动作 |
|---|---|---|
| 1 | reverse/baidu/sources/classes13/sources/com/baidu/netdisk/ui/account/i0.java:352-376 | public final void m()：构造 new com.baidu.sapi2.dto.SapiNALoginDTO() |
| 2 | 同文件 :322-350 | k(dto) 注入登录类型：ACCOUNT_PASSWORD(:339)、SMS(:340)、ONEKEY(:341)、HISTORY(:342)、SHARE(:343)、WECHAT(:347)、GOOGLE_CN(:349) |
| 3 | 同文件 :369 | **passportSDK.startNewNALogin(this.f692_, new i0.C_(), sapiNALoginDTO)** |
| 4 | classes2/sources/com/baidu/sapi2/PassportSDK.java:192-194 | startNewNALogin → com.baidu.sapi2.newlogin.NewLoginService |
| 5 | classes4/sources/com/baidu/sapi2/newlogin/NALoginRouter.java:139-163 | 清缓存 → 挂回调 → setInNaLoginState(true) → :162 startNewLoginActivity(...) |
| 6 | 同文件 :38-49 | new Intent(context, z ? NewLoginHalfActivity.class : NewLoginActivity.class) |
| 7 | classes4/sources/com/baidu/sapi2/newlogin/activity/NewLoginActivity.java、BaseNewLoginActivity.java、helper/NewLoginHelper.java | 搜索 WebView / loadUrl → **三处均 0 命中**（原生页面） |
| 8 | classes4/sources/com/baidu/sapi2/newlogin/logintype/NewLoginAccountAndPassWordTypeBaseView.java:421 | 密码表单提交：getAccountService().accountAndPwdLogin(str, str2, a(str), this.m, map) |
| 9 | classes2/sources/com/baidu/sapi2/SapiAccountService.java:135（转 :68/:78/:82） | → SapiAccountRepository.accountAndPwdLogin(...) |
| 10 | classes2/sources/com/baidu/sapi2/SapiAccountRepository.java:484-535 | :519-521 RSA 加密 username / password；:523-532 map 键 ds→s、tk→k；:534 → wapLogin(2, ...) |
| 11 | 同文件 :2873-2949 | wapLogin：:2875-2877 追加 alg=v3/adapter=3/cv=170601/isEncrypted=1；:2893-2895 time/t/session_id；:2897-2900 nalogin=1/needNaNextUrl=1/enableExternalWeb=0；:2901 **POST** |
| 12 | 同文件 :1126-1128 | 端点 = getWap() + "/wp/api/login" |
| 13 | 同文件 :2935-2937 | 响应 handleAccountAndPwdLoginResult |

**T-006 补充的链尾（第 14–17 跳，此前未记录）**：

| 跳 | 位置 | 动作 |
|---|---|---|
| 14 | classes2/sources/com/baidu/sapi2/BaseWapLoginResult.java:23-32 → SapiAccountService.java:542 | 分派到 handleAccountParseResult(cb, data.optString("xml"), PWD) |
| 15 | classes2/sources/com/baidu/sapi2/SapiCoreUtil.java:68-77 | 正则取 `<client>…</client>` 片段 → XmlPullParser 解析出 bduss / ptoken / username / errmsg / incomplete_user / actiontype（**纯 XML，无 WebView**） |
| 16 | classes2/sources/com/baidu/sapi2/SapiAccountManager.java:798-812 | 先 getDeleteAppStatus(bduss, cb)（SapiAccountRepository.java:1342-1348，URL :1397-1399 `/v3/api/user/getdeleteappstatus`，超时 2000ms，code!=110000 直接放行） |
| 17 | SapiAccountManager.java:823-836 → SapiContext.java:978-1016 | validate 落库：本地 AES 加密 JSON 存 SharedPreferences + SapiUtils.webLogin 写 BDUSS/PTOKEN 到 CookieManager |

**另有两条 WebView 容器旧路径**（记录但**不作为可用路径**）：classes13/.../ui/account/d0.java:504-506 与 SMSLoginRegisterActivity.java:267 调 PassportSDK.startLogin(...) → CoreViewRouter.java:1356-1357 → com.baidu.sapi2.activity.LoginActivity / YouthStyleLoginActivity（WebView 容器）。

#### 3.2.3 认证请求及字段

##### 三条路径共用的签名机制（T-006 新增，已验证）

- **`tpl` / `appId` / `appSignKey` 是 APK 内硬编码明文常量**，不是运行期注入、不是远程下发：`kd/______.java:11`（appId）、`:14`（国内 appSignKey）、`:15`（tpl）、`:17`（国际 appSignKey），经 `kd/______.java:33-37` → `AccountUtils.E()/K()`（`AccountUtils.java:153-164`）→ `application/___.java:11-19` → `C_.java:196-197` `setProductLineInfo(...)` → `SapiConfiguration.java:854-858`。本文不记录常量取值。
- `DebugSetKey` 覆盖（`kernel/debug/C_.java:82-88`）在 `!NetDiskLog.isDebug()` 时恒返回 null → **release 包无任何覆盖来源**。
- 签名在子类：基类 `com/baidu/pass/http/HttpHashMap.java:7-8` 的 `doSign(String)` **是空方法**；真正实现是 `classes4/.../httpwrap/HttpHashMapWrap.java:9-15`（`put("sig", SapiUtils.calculateSig(getMap(), str))`），唯一调用点 `HttpClientWrap.java:86-88`。
- 算法：`classes12/.../utils/SapiUtils.java:135-160` `calculateSig(Map,String)` = 移除 sig → key 字典序 → 非空值拼 `key=URLEncoder.encode(v)&` → 末尾 `sign_key=`+密钥 → **MD5 hex 小写**。**纯 Java，无 native。**
- 参与签名的公共参数：`classes4/.../httpwrap/Utils.java:5-25`（client / cuid / clientid / clientfrom=native / zid / clientip / appid / tpl / app_version / sdkversion）。

##### A. 账号密码：POST {wap}/wp/api/login（SapiAccountRepository.java:2873-2901）

| 字段 | 说明 | 可信度 |
|---|---|---|
| username | PwdRSAEncryptor.encryptString(true, str)：先 Base64 再 RSA | 已验证 |
| password | encryptString(false, str2 + str3)：**不做 Base64**，直接 RSA | 已验证 |
| alg / adapter / cv / isEncrypted | v3 / 3 / 170601 / 1 | 已验证 |
| time / t | 秒 / 毫秒时间戳 | 已验证 |
| session_id | h81.___._____((UUID + currentTimeMillis).getBytes(), false) —— 纯 Java 摘要（classes15/sources/h81/_____.java 用 MessageDigest.getInstance("MD5")） | 已验证 |
| nalogin / needNaNextUrl / enableExternalWeb | 1 / 1 / 0 | 已验证 |
| lang / liveAbility / scanface / supFaceLogin / support_photo / logLoginType | 条件注入（:2880-2892） | 已验证 |
| sig | calculateSig(公共参数 + body, appSignKey) | 已验证 |
| Cookie | ParamsUtil.buildNaCookie(...)：**cuid / DVIF(=加密后的 di) / "0" / BAIDUID / ploganondeg**（classes12/.../ParamsUtil.java:60-87）；body 里**没有** di，di 只经 DVIF cookie 传 | 已验证 |
| UA | "tpl:" + configuration.tpl + ";android_sapi_v" + VERSION_NAME（AbstractService.java:43-45） | 已验证（tpl 为常量，见上） |

**RSA 参数（可静态复刻）**：classes2/sources/com/baidu/sapi2/PwdRSAEncryptor.java:7-8，指数 "10001"（十六进制），模数为 256 个十六进制字符 ＝ **1024 bit**；:27-60 按 modPow 分段拼接，偶数长度补前导 0。**无 padding，纯 BigInteger。**

##### B. 短信验证码：POST /v2/sapi/login（SapiAccountRepository.java:263-298）

> **2026-11-28 补充收口纠偏**：这是**旧链** `dynamicPwdLogin` → `realDynamicPwdLogin`。网盘 App 现用登录 UI（`NewLoginSmsLayout`）走的是 `LoginSmsWapLoginHelper` → `smsWapLogin` → **`POST {wap}/wp/api/login`**，带 `sms=1` + `smsvc=<验证码>`。发码侧为 `POST {wap}/wp/api/login/sms`。两段都已到接口级；本节保留原文以便对照，详见 `docs/evidence/baidu-login-deep-dive.md` §15.2。

| 字段 | 说明 |
|---|---|
| crypttype | 6 |
| di | SapiDeviceInfo.getDeviceInfo("/v2/sapi/login")，**本路径 di 在 body 里**（:267-270） |
| cert_id | 证书 ID（动态路径取服务端下发；内嵌兜底 SapiDataEncryptor$Cert1.CERT_ID = 1） |
| isdpass | 1 |
| userinfo | RSA（PKCS1）分段加密的 JSON：username / isphone=1 / password / login_type=3 / key=<AESKey> / sdk_version=2 / pinfo=<URLEncoder(Build.BRAND)> |

**前置（T-006 新增）**：`dynamicPwdLogin`（:861-893）先 `GET /sslcrypt/get_last_cert`（常量见 :104-106）拿证书与 cert_id，**无需登录态**。

AES 密钥：SapiDataEncryptor.java:12-19 createAESKey() 本地随机 16 位 [a-z0-9]，**随请求一起明文放进 key 字段**（:278）。RSA 分段长度 116（UploadContract.UploadTasks.ki，UploadContract.java:39）。→ **全部 JCA 标准算法，可复刻。**

##### C. 扫码：三步客户端协议

| 步骤 | 位置 | 接口 | 关键参数 |
|---|---|---|---|
| 取码 | classes2/sources/com/baidu/sapi2/QrCodeService.java:314-326 | GET /v2/api/getqrcode | lp(默认 pc) / apiver=v3 / tt / openPlatformId / encuid / needQrCodeContent / hostDeviceId，**cookie 传 null** |
| 轮询 | 同文件 :387-397 | GET /channel/unicast | apiver=v3 / callback=cb / channel_id / tt（超时 40000 ms），响应是 JSONP，需 substring(3, len-2) 剥壳（:146） |
| 换取 | 同文件 :354-364 | GET /v3/login/main/qrbdusslogin 或 /v2/api/bdusslogin | display=pcsdk / qrcode=1 / bduss(可空) / tt / hostDeviceId（恒 null，见下） |
| 被扫端确认 | 同文件 :367-384 | POST /v2/sapi/qrlogin?lp=app | sign / cmd（**本项目作为扫码端不需要**） |

→ **扫码登录的 token 来源是客户端协议（/v2/api/bdusslogin），不是 WebView 抓取。**

**T-006 补充的三点（重要）**：

1. `channel_id` 来自取码响应的 `sign` 字段（`QrCodeService.java:49`）。
2. `getQrLoginResult` 的 `bduss` 参数**允许为空**（只有「已有账号」场景才填），因此**不依赖任何既有登录态**。
3. `QrCodeService.java` 全文**不含** `HttpHashMapWrap`（`grep` 0 命中），但三个请求都经 `HttpClientWrap` → `buildParamDTO` → `httpHashMap.doSign(appSignKey)`（`HttpClientWrap.java:86-88`，全仓唯一 `doSign` 调用点）→ **扫码加 sig**；全文也**不含** `getDeviceInfo`，且三处 `get(...)` 的 cookie 参数都传 `null` → **扫码不发 di、不带 cookie**。<br>**2026-11-28 补充收口修正**：原文「扫码不加 sig」有误，`HttpHashMapWrap` 只是签名实现类的选择，不是是否签名的开关。

#### 3.2.4 token 换取与存储

- 会话字段：classes12/sources/com/baidu/sapi2/shell/response/SapiAccountResponse.java:14-20 的 bduss / ptoken / stoken / displayname / username / email / uid；SapiAccountRepository.java:2603-2613 逐字段搬运。
- 落库：classes2/sources/com/baidu/sapi2/SapiContext.java:61-65 getSharedPreferences("sapi_system", 0)；键名见 classes2/sources/com/baidu/sapi2/SapiAccount.java:5-18。
- **刷新机制：无主动刷新。** SapiAccountManager.java:436 有 onBdussExpired(GetUserInfoResult) 回调；SapiAccountRepository.java:2994-2997 用错误码 400021 表示 bduss 过期。
- **oauth() 的方向必须注意**：SapiAccountRepository.java:2519-2530 oauth(...) POST /v2/sapi/bdussexchangeaccesstoken（body openPlatformId / bduss / scope）——**是用 bduss 换 access token，不是用 OAuth 换登录态**。

#### 3.2.5 OAuth / WebView / cookie 排除情况

**A. OAuth 常量定性（关键，已验证）**

classes2/sources/com/baidu/sapi2/SapiAccountRepository.java:1111-1121 generateSsoHash(...) 的 AsyncTask 里 execute("https://openapi.baidu.com/oauth/2.0/token")，**只用来 openConnection() 取 getDate()/1000 当时间戳**（:1102 return Long.valueOf(uRLConnectionOpenDate / 1000)），再喂给 Security().encryptSsoHash(...)。

→ **该 URL 不参与登录鉴权，不是 OAuth 授权入口。** 不能因为发现这个常量就判「百度只能 OAuth」。

补充搜索：grep -rn '.oauth(' classes*/sources/com/baidu/netdisk/ → **0 命中**。

**B. WebView 路径存在但不在主链**

web2NativeLogin（SapiAccountRepository.java:2951-3010）从 **cookie 读 BDUSS/PTOKEN**（:2965-2966）再换会话；调用方是 SapiJsInterpreters.java:1212 与 AccountCenterActivity.java:35。这是官方 App 内部的账号中心/分享场景，**与「本项目原生独立登录」无关，判定时不计为可用路径**。

**C. Sdata 签名不影响登录（重要，已验证）**

Sdata 头只在**自研网络栈**加：classes6/sources/com/baidu/netdisk/network/c.java:126、classes6/sources/p20/b.java:38、classes6/sources/o20/___.java:166。且 c.java:105-130 的条件是 if (!zContains && !whitelist.contains(path)) —— **zContains 即 isHttps，只有非 https 且不在白名单时才加**。

登录请求走的是 sapi2 的 HttpClientWrap → PassHttpClient → com/baidu/pass/http/a.java，该链路只用 setRequestProperty("User-Agent", ...) 与 Content-Type（a.java:116/125/139），**不注入 Sdata**（在 classes2/sources/com/baidu/sapi2 与 classes12/sources/com/baidu/sapi2 搜 Sdata → 0 命中）。

**D. Sdata 实现仍是缺口（但只影响文件类接口）**

- 使用点：hk.__._.b(___.____(), str2)。
- 真正实现在 classes7/sources/hk/__.java:469-475：Function2 function2 = G; if (function2 != null) return (String) function2.invoke(str, str2); return null; —— **纯转发，无算法**。
- 注入 setter：hk/__.java:665-667 public final void n0(Function2...) { G = function2; }。
- **未找到 .n0( 的静态调用方**：grep -rn '..n0.(' classes6/sources --include=*.java 命中 7 处全是无关类；grep -rn 'n0(new kotlin.jvm.functions.Function2' classes*/sources → **0 命中**。
→ **Sdata 签名函数由运行期注入，静态不可复刻**。但它只作用于明文 http 的文件接口，**不影响登录**。

#### 3.2.6 T-006 复核：原 5 项攻关点的处理结果

| # | 原攻关点（T-002c） | T-006 复核结论 | 状态 | 证据 |
|---|---|---|---|---|
| 1 | di 设备指纹 40 字段受风控开关裁剪，服务端是否校验未知 | **40 个字段已逐项定性**（必需 22 / 条件 2 / 可选 16）；加密算法与 16 字符 AES key 均在 Java 侧（`SapiDeviceInfo.java:8`、`:189-203`）；`mPrivacyParamesRegulation=false`（`C_.java:199` 实参）使 `dt_params` 灰度裁剪**实际不生效**；唯一仍动态的是服务端下发的 `diExceptIndex`（默认空） | **静态已解**；服务端校验强度待动态 | deep-dive §3 |
| 2 | tpl / appSignKey 取值（记「运行期注入，静态未定位」） | **推翻旧说法**：三者是 APK 硬编码明文常量（`kd/______.java:11/:14/:15/:17`），`DebugSetKey` 覆盖仅 debug 生效 | **已解** | deep-dive §2 |
| 3 | passport_core 六个 native | 全 dex **仅 7 处调用**（5 文件），**三条主链 0 处**；缺失时 `SapiConfiguration.java:553-575` 有 `catch (Throwable)` 兜底；`nm -D` 只有 6 个 `Java_` 导出、无 `JNI_OnLoad`、`RegisterNatives` 0 命中 | **已解：不阻断** | deep-dive §4 |
| 4 | 服务端风控放行 + 6 类 native 风控库 | 6 个库的加载入口与唯一调用点全部落行号，**均不经过三条登录路径**（分别属业务 API 加签 / APK 签名校验 / P2P 下载 / MSA OAID / sofire 旁路） | **静态已解**；服务端风控仍动态 | deep-dive §5 |
| 5 | 文件类接口的 Sdata | 本次未变（登录之外的缺口） | **仍开放**（不影响登录判定） | 本文 3.2.5 D |
| 6 | `libbaiduprotect_sec.so` / `libbaidusec*` 是否阻断登录（**2026-11-28 补充**） | 已完整追踪：APK 内只有 `libbaiduprotect_sec.so`（133168 B）与 `libbaiduprotect_sec_jni_facect.so`（137264 B），**不存在任何 `libbaidusec*` 文件**；无库 `NEEDED` sec.so、无 dlopen 字面量；唯一 Java 消费方是 `WbEncryptUtil.java:53`（人脸 WBAES），A/B/C 三链均不触及 | **已解：不阻断** | `docs/evidence/baidu-login-deep-dive.md` §15.1 |
| 7 | 短信发码接口与完整短信登录（**2026-11-28 补充**） | 发码 = `POST {wap}/wp/api/login/sms`（前置 `cap/init`）；提交 = `POST {wap}/wp/api/login` + `sms=1`+`smsvc`；**但前置人机验证是 `pass-machine.html` 的 WebView 页面，静态不可复刻** | **B 路径降级为「需攻关」** | 同文件 §15.2、§15.7.2 |
| 8 | 扫码 `hostDeviceId`（**2026-11-28 补充**） | **官方客户端从不赋值，恒 `null`，`null` 参数被 `a.java:333-353` 丢弃**，本项目按缺省不传即可 | **已解**（原「客户端生成」表述作废） | 同文件 §15.3 |

#### 3.2.7 结论与理由

- 第 1 层：有 5 个原生登录 Activity（newlogin/activity/ 下）。
- 第 2 层：**满足**。账号密码、短信、扫码三条通道都追到了实际 HTTP 请求与会话字段回填。
- 第 3 层：**A 与 C 满足；B 不满足**。三处此前的阻断点（tpl/appSignKey 取值、passport_core、di 裁剪）已分别落到常量、无调用点 + Throwable 兜底、逐字段分类；三条路径**均不依赖任何官方闭源 native**。但 **B 路径的前置人机验证是 `pass-machine.html` 的 WebView 页面**（`ValidationActivity.java:319-321`、`:179`），其通过条件由服务端与页面 JS 决定，**静态不可复刻**，故 B 不能判「可独立实现」。
> **2026-11-28 校正**：原文「又触碰 AGENTS.md 第 9 条禁止用 WebView 完成登录」**表述不当**——该 WebView 只承载条件触发的人机挑战、不承载凭据、不读 cookie、回传仅 `ds`/`tk`，与坚果云 R27 同口径，不构成禁止形态。档位维持，理由改写，详见 §3.2.9 (5)。

**三条路径分项结论（四档）**：

| 路径 | 结论 | 关键原因 |
|---|---|---|
| A 账号密码 | **可独立实现** | RSA 1024 + MD5 签名 + di 全部在 Java 侧；响应是 XML 解析；前置 `cap/init` 失败不阻断 |
| B 短信验证码 | **需攻关**（2026-11-28 由「可独立实现」降级） | 发码与提交两段都已到接口级，但**前置人机验证是 WebView + 服务端 JS**，静态不可复刻且本项目禁用 WebView 登录 |
| C 扫码 | **可独立实现（带动态前提）** | **不发 di、不带 cookie**（但**加 sig**）；`hostDeviceId` 官方客户端恒 `null`、参数被丢弃；不依赖既有登录态。残留前提：服务端接受该参数缺席 |

→ **整体：原生登录可行（静态可行，动态未验证）**。

**仍然只是静态证据、必须动态验证的事项**：服务端是否接受本项目构造的 `sig` 与 `di`；`diExceptIndex` 线上实际下发值；`BAIDUID` 是否必须非空；是否有额外风控挑战（验证码 / 设备信任 / 异地登录）。详见 deep-dive §12.4。

**明确不成立的三个误判（保留）**：
1. 不能因 openapi.baidu.com/oauth/2.0/token 常量存在就判「只能 OAuth」——它只用于取时间戳（3.2.5 A）。
2. 不能因存在原生登录 Activity 就判「可行」——第 1 层≠第 3 层（本次判「可行」依据的是第 3 层证据，不是第 1 层）。
3. Sdata 未定位 ≠ Sdata 已还原（3.2.5 D）。

**一个必须同时读的限制**：本结论只说明「协议可由本项目独立实现」，**不说明「登录一定成功」**。服务端风控是黑盒，见 `docs/RISKS.md` R5 / R28。

---

#### 3.2.8 T-006 补充收口摘要（2026-11-28）

本节把补充收口的四项结论并入上面的判定，**不删除任何 T-006 原文**。详细证据见 `docs/evidence/baidu-login-deep-dive.md` §15。

| 项 | 结论 | 对判定的影响 |
|---|---|---|
| `libbaiduprotect_sec.so`（及 `*_jni_facect.so`） | 仅服务人脸 WBAES 加密；无库 `NEEDED` sec.so、无 dlopen 字面量、无 `Java_` 导出（sec.so）、唯一 Java 消费方是 `WbEncryptUtil.java:53` | **不影响**；「`libbaidusec*`」实为这两个库，APK 内无同名文件 |
| 短信发码链路 | `LoginSendSmsHelper` → `SapiAccountService.doCapInitAndGetLoginSms` → `POST {wap}/wp/api/login/sms`（前置 `POST passport.baidu.com/cap/init`） | **B 降级为「需攻关」**：真正的人机墙是 `pass-machine.html` WebView |
| 短信提交端点 | `NewLoginSmsLayout` → `LoginSmsWapLoginHelper` → `smsWapLogin` → `POST {wap}/wp/api/login`（`sms=1`+`smsvc`） | 修正原记录（原写 `/v2/sapi/login`，那是旧 `dynamicPwdLogin` 链） |
| 扫码 `hostDeviceId` | 官方客户端**从不赋值**，恒 `null`；`a.java:333-353` 丢弃空值参数 | **已闭环**，原「客户端生成、需保持一致」作废；本项目不传即可 |
| 扫码是否加 `sig` | **加**（经 `HttpClientWrap.buildParamDTO` → `doSign`）；但**不发 `di`、不带 cookie** | 修正原「不加 sig」 |
| `cuid` 算法 | `MD5("com.baidu" + ANDROID_ID)` 转大写；黑名单/空值 → `UUID` + `FUUID` 前缀；派生字段 `i()` 为 4 算子自定义摘要 + 自定义 Base32 | 主值可复刻；`i()` **当前无法精确复刻**（需移植且无测试向量） |

**三条路径修正后**：A 可独立实现 / **B 需攻关** / C 可独立实现（带「服务端接受 `hostDeviceId` 缺席」这一动态前提）。**百度整体仍判「原生登录可行」**，理由是 A、C 两条路径不含静态不可复刻环节；B 不计入。

**Cookie 边界（必须与结论一起读）**：`cuid` 与 `DVIF` 是**本项目按官方算法自行构造的设备信息 Cookie**；`BAIDUID` 由服务端 `Set-Cookie` 下发（本项目只能读、读不到发空串）；`BDUSS`/`STOKEN`/`PTOKEN` 是登录要换的认证凭据。**本项目不复用浏览器或官方 App 的私有认证 Cookie。**

---
#### 3.2.9 最终一致性校正（2026-11-28，当前有效入口）

> 本节是百度小节的**当前有效结论入口**。§3.2.1–§3.2.8 的取证历史保留不删；凡与本节冲突以本节为准，逐条差异见末尾小表与 `docs/evidence/baidu-login-deep-dive.md` §16。

**(1) `t_cuid` / `t_appname` 的字段来源（旧文写反，已更正）**

- `SapiDeviceInfo.java:53` 的 `getCookiesKeyList()` **第 38 位是 `t_cuid`、第 39 位是 `t_appname`**。
- `buildDeviceTokens`（同文件 `:168-184`）：索引 38 ← `confignation.mTPLCuid`（`:171`）；索引 39 ← `confignation.mTPLAppName`（`:181-183`）。
- 业务侧唯一调用点 `classes15/sources/com/baidu/netdisk/application/C_.java:199` 传的是 `("wangpan_drive", _____(), false)`：`t_appname` = 常量 `"wangpan_drive"`；`t_cuid` = `_____()` 返回的**设备侧组合串**。

**(2) `i()` 确实进入最终登录请求**

`C_.java:143` `cuid = ew._.j` → `startup/task/l.java:50` `ew._.j = DeviceId.getCUID(ctx)` → `DeviceId.java:98-100` `getCUID = b(ctx).v()` → `cesium/h.java:168-186` `v()` 在 `__ == "V"` 时把 `___ = i(f797_)` 拼进组合串，形如 `"<cuid>|V<i(cuid)>"`。

→ **`i()` 进入 `di` 索引 38 → AES 加密 → 请求体**。Rust 若不复刻 `i()`，该段**必然**与官方客户端不同（不是「可能不同」）。
→ `t_appname` **不需要** `i()`（常量，仅当 `mTPLAppName` 为 null 才回落 `getClientId`，业务侧不会为 null）。
→ `__` 的正确取值是 **`"V"`**（新建 `h.java:398`、JSON 读回 `h.java:214` 默认值）；`InterfaceC0042a.__ = "O"` 只是常量声明，未被这两条路径使用。

**(3) `sf_zid` 不是恒为 `"NoZidYet"`**

七分支（`sofire/d/b.java:645-679`）与五种取值（`"NoZidYet"` / 空串 / 静态缓存 / SP 文件解密值 / **62 位**十六进制常量 `74FFB5E6…BA03`）。**旧文两处「56 位」错误**。
公共参数 `zid`（`httpwrap/Utils.java:17`）与 `di` 索引 34 **同源同值**（都取 `getCurrentZid`），但 `diExceptIndex` 排除 34 时 `di` 侧为空串而公共参数侧仍有值。

**(4) 人脸 native：不影响认证，但影响三个可选能力参数**

`SapiAccountManager.java:149-153` 的 `Class.forName` + catch 分支把 `supportFaceLogin` 置假；`SapiAccountRepository` 五处（`:801-803`/`:1549-1551`/`:1652-1654`/`:1795-1797`/`:2885-2887`）据此决定是否发 `liveAbility`/`scanface`/`supFaceLogin`。
→ 认证、`sig` 签名、`di` 指纹**均不依赖**该库；但「完全不影响请求构造」的旧表述**过强**，已改为「影响三个可选能力参数是否出现」。

**(5) 验证码 WebView 与坚果云 R27 同口径（B 降级理由改写）**

`ValidationActivity` 只加载 `pass-machine.html` 人机挑战页，JS→原生走 `onJsPrompt`（`ValidationActivity.java:28-55`，**不是 `addJavascriptInterface`**），回传仅 `errno`/`msg`/`ds`/`tk`（`result/ValidationResult.java:4-21`）；`grep CookieManager|bduss|stoken|ptoken` 在 `com/baidu/validation` 下 **0 命中**；认证仍由原生 `POST {wap}/wp/api/login` 完成。
→ **不构成 AGENTS.md 第 9 条禁止的形态**，与 R27 同口径，**不得仅因存在 WebView 就降级**。
→ B 档位维持「需攻关」，理由改写为：①挑战通过条件由服务端与页面 JS 决定，静态不可复刻 ②是否接受内嵌同一 H5 作为人工验证环节尚未拍板。
→ 另需记录**整页网页登录兜底分支**（A：400101/400023/400031/400408/400035/400041/500072/340002/500001/500002/500020/500021/400605/402008/402013；B：72200/402012/400704/400706/400418/400425/500072/402008/402013/400605）——本项目对这些场景只能**显式失败**。

**(6) 账号密码路径同样存在人机验证分支（旧文未写）**

`newlogin/helper/NewLoginUtil.java:2107-2145`：错误码 **50052** 时若 `ValidationServiceUtil.isAvailable()` 为假走整页网页登录，否则 `startValidation` 回调写回 `ds`/`tk` 并 `onAgainRequestPwdLogin` 重试原生登录。
→ A 的档位不变（仍「可独立实现」），但必须写明它带**与 R27 同级的条件性第三方验证依赖**，属条件触发、非每次必经。

**(7) 与 §3.2.7 / §3.2.8 的差异**

| 原位置 | 原文摘要 | 校正后 |
|---|---|---|
| §3.2.7 第 3 层 | 「B 触碰 AGENTS.md 第 9 条禁止用 WebView 完成登录」 | 该 WebView 仅人机挑战，**不构成禁止形态**；降级理由已改写 |
| §3.2.8 表「短信发码链路」 | 「真正的人机墙是 `pass-machine.html` WebView」 | 保留，但补「只出票据、认证仍原生」 |
| §3.2.8 表「`cuid` 算法」 | 「`i()` 当前无法精确复刻（需移植且无测试向量）」 | 保留，但补「`i()` 确实进入最终请求，不复刻则必然不一致」 |
| §3.2.8 末「三条路径修正后」 | A 可独立实现 / B 需攻关 / C 可独立实现 | **不变**，B 的理由改写 |
| §3.2.1–§3.2.8 全文日期 | 「2026-11-28」 | 统一改 **2026-11-28** |
| §3.2 全节 | 只记前置 `cap/init` 失败仍继续 | **新增 50052 人机挑战分支**（见上 (6)） |
### 3.3 夸克网盘

**结论：需攻关** — 落在第 2 层，第 3 层被两项 native 闭源组件阻断。

> **与 T-002d 结论的差异**：T-002d 把夸克记为「Flutter 主体，协议需动态分析」。T-002c 定向读取 dex 后确认：**网盘文件业务协议确实在 Dart 侧，但账号登录体系完整存在于 dex 的 Java 层**。本节只判定登录，不涉及文件协议。详见第 4 节。

#### 3.3.1 APK / 资料来源

- com.quark.clouddrive 10.21.5.144（versionCode 144），120924033 B，12 dex / 115475 类，仅 arm64-v8a。
- 定向反编译（T-002c 新增）：classes7.dex 与 classes10.dex → reverse/t002c/src/quark/classes{7,10}/sources/（10505 个 .java）。
- 登录类分布：classes10 的 com/ucpro/feature/personal/login 159 类、classes7 的 com/uc/base/account 24 类、classes2 的 com/ucweb/login 7 类。
- 桥接判断：自家包 MethodChannel 仅 2 处（ASR/TTS）、Plugin 73 处集中在 webwindow/nezha、flutter 287 处集中在阅读中心/播放器/搜索/学习 → **账号登录不在 Flutter 侧**。

#### 3.3.2 登录调用链

| 跳 | 位置 | 动作 |
|---|---|---|
| 1 | reverse/t002c/src/quark/classes10/sources/com/ucpro/feature/personal/login/PersonalLoginPresenter.java:188-208 | c7(ValueCallback) 手机号登录入口：走 com.ucpro.feature.account.AccountManager.L().O(callback)；打点 logintype=phone |
| 2 | .../classes7/sources/com/uc/base/account/service/account/login/UCLoginServiceImpl.java:232-280 | g(...) 发短信：method cas.sendSmsCodeForLogin，字段 mobile / captcha_id / captcha_code |
| 3 | 同文件 :190-230 | e(...) 短信登录：method cas.loginWithSmsCode，字段 mobile / sms_code / 可选 cooling_off_filter=1；成功判据 getContentStatus()/20000 == 1 |
| 4 | 同文件 :333 | cas.login（账号密码变体） |
| 5 | .../classes7/sources/rn0/a.java:6-59 | a(TreeMap) 真正发 HTTP：new URL(vn0.a.a().concat("/acs.wsg_access")) |
| 6 | .../classes7/sources/vn0/a.java:10-15 | base URL 三选一：https://n-access.open.uc.cn / **https://access-open.quark.cn（网盘侧）** / https://access.open.uc.cn（默认） |

#### 3.3.3 认证请求及字段

- 方法 / 路径：**POST {base}/acs.wsg_access**，Content-type: text/plain（rn0/a.java:19）。
- 公共参数（rn0/a.java:61-163）：t、request_id(UUID)、client_id="636"、device_id、sdk_v="1.0"、server_v="1.2"、os_version、fr(系统版本)、locale；
- client_info 长串（:73-132）：ip: / imei: / imsi: / sn: / machine:<MODEL> / app_name:qk_clouddrive / os:Android / idfa: / utdid: / version: / net_type: / net_id: / client_identity:；
- **security_sign**（:142-146）＝ com.uc.base.account.service.account.c.b().b(拼接串)。
- **请求体加密**：:27 反射调 com.uc.encrypt.EncryptHelper.encrypt(String)；响应体 :49 反射调 decrypt。
- 业务 method 名：cas.login、cas.loginWithSmsCode、cas.sendSmsCodeForLogin、cas.getCaptcha、cas.getUserBasicInfoByServiceTicket、cas.loginWithThirdPartyToken、cas.loginWithThirdPartyAccount、cas.logout。

#### 3.3.4 token 换取与存储

- 返回字段：.../account/service/account/login/e.java:24 service_ticket（另有 uid / nickname / expiresIn，见 :39/:60）。
- 落库：e.java:43 写偏好键 service_ticket；:222 写 "number"（手机号）。

#### 3.3.5 OAuth / WebView 排除情况

- **第三方登录是不合格路径，明确排除**：PersonalLoginPresenter.java:227 微信、:232-249 淘宝、:252+ 支付宝，均为 SSO。本文**不把它们计入登录类型**。
- PersonalLoginPresenter.java:167-170 与 :178-181 只是打开隐私/用户协议 WebView（terms.alicdn.com），**与登录凭证无关**。
- 手机号短信登录本身走 cas.sendSmsCodeForLogin / cas.loginWithSmsCode 客户端协议，**不经 WebView**。

#### 3.3.6 需攻关的具体问题

| # | 攻关点 | 证据 | 可信度 |
|---|---|---|---|
| 1 | **请求体 M9 加密是 native** | com/uc/encrypt/EncryptHelper.java:314-325 System.loadLibrary("encrypt")；:327-359 SecurityGuardManager.getInitializer().initialize(context)；M9EncryptionHandler.java:16/18 private native byte[] decodeM9(byte[]) / encodeM9(byte[]) | 已验证（native 依赖存在） |
| 2 | **security_sign 是聚安全 native** | .../account/c.java:12-14 → new tn0.a(k.a.a.a())；.../classes7/sources/tn0/a.java:29-43 取 SecurityGuardManager.getInstance(...).getSecureSignatureComp()，map.put("INPUT", str + "3dsa33@213!!!22")，requestType=3，返回 a(appKey) + strSignRequest | 已验证 |
| 3 | 服务端对 client_info 各字段的校验强度 | — | 待确认 |

→ **两项 native 均为阿里聚安全 SecurityGuard（libsgmainso-6.7.251102.so 2990668 B 已在包内），静态不可复刻**。这不是「资料不足」，而是「已有证据证明依赖闭源 native」。

#### 3.3.7 结论与理由

- 第 1 层：有原生登录页（PersonalLoginWindow / LoginDialog 等）。
- 第 2 层：**满足**。短信收发、cas.loginWithSmsCode、/acs.wsg_access、service_ticket 回填全链路可读。
- 第 3 层：**未满足**。请求体加密与签名都落到 native，本项目 Rust Adapter 无法调用该 so 的私有接口。

→ **需攻关**。

#### 3.3.8 未解决问题及后续工作

- 若不破解聚安全，可行替代是把该 so 一起打包进本项目并由 Rust 通过 JNI 调用 → 但那是「移植官方闭源组件」，属架构与合规决策，**须先写 ADR 并经用户批准**，本文不预设。
- client_id="636"、app_name=qk_clouddrive 是否为硬校验 → 待确认。
- service_ticket 的服务端有效期（expiresIn 的实际语义）→ 待确认。

---

### 3.4 阿里云盘

**结论：需攻关** — 落在第 2 层，第 3 层被聚安全风控数据与 MTOP 签名阻断。

> **与 T-002d 结论的差异**：T-002d 记为「Flutter 主体（混合：壳 + jsbridge + Flutter），协议需动态分析」。T-002c 定向读取 dex 后确认：**账号登录体系（淘宝 login4android / Havana）完整在 Java 层，且每个登录请求都注入聚安全风控数据**。详见第 4 节。

#### 3.4.1 APK / 资料来源

- com.alicloud.databox 6.21.0（versionCode 281），163545725 B，25 dex / 70881 类，仅 arm64-v8a。
- 定向反编译（T-002c 新增）：classes9.dex（com/ali/user/mobile 246 类）与 classes11.dex（com/alicloud/databox 94 类）→ reverse/t002c/src/ali/classes{9,11}/sources/（4050 个 .java）。
- 登录类分布：com/ali/user/mobile/login/ui 113、com/alicloud/databox/account/service 94、account/user 68、account/authentication 53、com/ali/user/mobile/login/presenter 53、login/tasks 51、com/taobao/login4android/ui 46、account/login/havana 41、account/login 40。
- 桥接判断：自家包 MethodChannel 仅 1 处、Plugin 2 处、jsbridge 262 处（ADriveBridgeWebView / FlutterBridgeActivity）→ **账号登录在原生 Java/Kotlin 层，不在 Flutter 层**。

#### 3.4.2 登录调用链

| 跳 | 位置 | 动作 |
|---|---|---|
| 1 | manifest 声明 | com.alicloud.databox.account.login.LoginActivity / TransparentLoginActivity / ScanCodeFailedActivity；com.ali.user.mobile.login.ui.UserLoginActivity / HalfTransparentUserLoginActivity；com.taobao.login4android.membercenter.account.* |
| 2 | reverse/t002c/src/ali/classes11/sources/com/alicloud/databox/account/login/LoginViewModel.java（912 行） | :156-157 注册登录广播 + LoginController.getInstance().isLoginSDKInited()；:162 监听 HavanaInitEvent；方法 handleLogin / onLoginSuccess / recheckLoginToken / refreshInfo / mergeDeviceId / exitLoginState |
| 3 | .../classes9/sources/com/ali/user/mobile/login/LoginApi.java:66-174 | 静态门面：pwdLogin(:88) / smsSend(:92) / smsLogin(:100) / nickSmsSend(:96) / nickSmsLogin(:104) / simLogin(:108) / tokenLogin(:116) / faceLogin(:120) / autoLogin(:124) / snsLogin(:165) |
| 4 | .../classes9/sources/com/ali/user/mobile/login/service/impl/UserLoginServiceImpl.java:19-34 | EASY_LOGIN 构造：PasswordLoginRequest{loginId, password, site, loginType} → rpcRequest.addParam("loginInfo", ...) → RpcService.post(...) |
| 5 | 同文件 :37-83 | buildPwdLoginRequest：按开关选 OCEAN_PW_LOGIN(:65) 或 PW_LOGIN(:73) |
| 6 | .../LoginDataRepository.java:26/87/109/113-116 | simLogin / TOURIST_LOGIN / BIOMETRIC_LOGIN / tokenLogin（tokenInfo） |

#### 3.4.3 认证请求及字段

MTOP 接口名（明文常量，.../classes9/sources/com/ali/user/mobile/rpc/ApiConstants.java:66-120）：

| 用途 | API_NAME | 行号 |
|---|---|---|
| 短信登录 | mtop.taobao.mloginService.smsLogin | :80 |
| 发短信 | mtop.taobao.mloginService.smsSend | :73 |
| 账号密码 | com.taobao.mtop.mloginService.login / mtop.taobao.alibabaMLoginService.login | :119 / :115 |
| 一键登录 | com.taobao.mtop.mEasyLoginService.login | :92 |
| 自动登录 | com.taobao.mtop.mLoginUnitService.autoLogin | :86 |
| 扫码取码 / 确认 | HAVANA_GEN_QR_CODE / HAVANA_LOGIN_BY_QR_CODE / COMMON_SCAN_QRCODE / COMMON_CONFIRM_QRCODE / COMMON_CANCEL_QRCODE | :105 / :106 / :91 / :90 / :89 |
| 支付宝 SSO | mtop.taobao.havana.mlogin.alipayaso | :67 |
| 登出 | mtop.taobao.havana.mlogin.logout | :110 |

请求体关键参数：

- loginInfo（JSON）：loginId / password / site / loginType（ICBU / YOUKU / TAOBAO）。
- ext（JSON）：UserLoginServiceImpl.java:81。
- tokenInfo：LoginDataRepository.java:116。
- **riskControlInfo（必现）**：com/ali/user/mobile/data/LoginComponent.java:127 与 :441 都是 rpcRequest.addParam("riskControlInfo", JSON.toJSONString(SecurityGuardManagerWraper.buildWSecurityData()))；注册链路 RegisterComponent.java:59/116/176/196 同样注入 ApiField.RISK_INFO。

#### 3.4.4 token 换取与存储

- 会话：Havana / login4android 会话模型；com/ali/user/mobile/base/helper/LoginDataHelper.java:214 putSessionModelToFile、:187-191 putLoginHistory/saveHistoryOnly/updateLoginHistoryIndex、:398 getFingerValue(str)。
- 刷新：LoginViewModel.java 有 recheckLoginToken / refreshInfo 方法（Kotlin 协程）；GET_WAP_LOGIN_COOKIES（ApiConstants.java:104）与 GET_ALIPAY_COOKIES(:99) 属 cookie 换取通道。→ 具体刷新周期 **待确认**。

#### 3.4.5 OAuth / WebView 排除情况

- **明确排除**：ALIPAY_SSO_LOGIN（支付宝）、snsLogin、APPLY_SSO_LOGIN、APPLY_SSO_TOKEN_V2 均为第三方 SSO，不计入。
- **GET_WAP_LOGIN_COOKIES / GET_ALIPAY_COOKIES 属 cookie 通道，本项目不得使用**（AGENTS.md 第 9 条）。
- 短信 / 账号密码 / Havana 扫码三条是客户端协议路径，本身不经 WebView。但注意：MainModuleApiImpl.java:1495 存在 https://cs-center.alipan.com/xcustomer/bus/index?...&browser_info=WebView，属客服页，与登录无关。

#### 3.4.6 需攻关的具体问题

| # | 攻关点 | 证据 | 可信度 |
|---|---|---|---|
| 1 | **riskControlInfo 由聚安全 native 生成** | LoginComponent.java:127/441；LoginDataHelper.java:301 com.taobao.wireless.security.sdk.SecurityGuardManager.getInstance(context)；包内 8 个聚安全 so（libsgmain.so / libsgmainso-6.6.240404.so 1873964 / libsgsecuritybody* 等） | 已验证 |
| 2 | MTOP 签名（umt / wua） | anet/channel/security/SecurityGuardImpl.java:61-69 取 getSecureSignatureComp().signRequest(...) | 已验证（依赖 native） |
| 3 | AccountRpc 主类 | 位于 classes18.dex（1 类），**本次未反编译该 dex** | 待补 |
| 4 | 服务端对 riskControlInfo 的校验强度 | — | 待确认 |

#### 3.4.7 结论与理由

- 第 1 层：有多个原生登录 Activity。
- 第 2 层：**满足**。MTOP 接口名、参数结构、风控注入点全部可读。
- 第 3 层：**未满足**。每个登录请求都带 native 生成的风控数据；MTOP 签名同样依赖聚安全 so。

→ **需攻关**。攻关成本与夸克同量级（同一套聚安全）。

#### 3.4.8 未解决问题

- classes18.dex 的 AccountRpc 未反编译 → 待补（下一步最小工作）。
- Havana 二维码流程的完整字段（HAVANA_GEN_QR_CODE 的入参/出参）→ 未读，待确认。
- 阿里云盘自己的 https://auth.alipan.com/ 与淘宝 MTOP 的关系（是否两条并行通道）→ 待确认。

---

### 3.5 天翼云盘

**结论：未知（需动态分析）** — 落在第 1 层下方（连界面都不可静态确认）。

#### 3.5.1 APK / 资料来源

- com.cn21.ecloud 11.1.2（**versionCode 638**，非 apk-inventory 早期记的 1112），387095539 B，compileSdk 34 / minSdk 24 / targetSdk 30。
- 索引：docs/evidence/index-天翼.md（壳确认模式，协议覆盖 0）。

#### 3.5.2 dex 是否存在登录协议线索

**不存在。**

- classes.dex 仅 10136 B，**class_defs 共 3 个**：Ls/h/e/l/l/C;、Ls/h/e/l/l/N;、Ls/h/e/l/l/S;（壳桩类）。
- 扫描结果：login/auth 类名命中 **0**，URL 字符串命中 **0**（脚本 reverse/login-clues.py，遍历 string_ids/type_ids/class_defs）。
- 加固：**爱加密**——assets/ijiami.dat 22378961 B + assets/ijiami.ajm 10890680 B + libijm-emulator.so 166360 B + libijmDataEncryption.so 433744 B。真实业务 dex 被加密，运行时才解密。

#### 3.5.3 登录类型能否确认

**不能。**

#### 3.5.4 当前能否确认官方客户端登录协议

**不能。**

#### 3.5.5 缺失的字段 / 签名 / 会话流程

- 登录接口路径与字段（在 libapp.so Dart AOT 内，静态不可读）
- 请求签名机制（libsecurity.so 21528 B 职责未验证）
- 会话 token 结构、刷新与过期
- 客户端环境校验（UA / 版本 / 设备指纹 / TLS pin）
- 爱加密载荷的运行期解密流程（libexecmain.so 215640 B / libexec.so 765648 B 职责未验证）

#### 3.5.6 后续必须通过什么动态分析

1. 动态抓包（需绕过 network_security_config 与可能的 pinning；@0x7f160004 指向的明文 xml 未能解出 → 待确认）。
2. 爱加密载荷脱壳（ijiami.dat + ijiami.ajm）。
3. libapp.so 的 Dart AOT 分析（或在脱壳后读 dart 快照的字符串表）。

#### 3.5.7 结论

**未知（需动态分析）**。不是「不可行」——没有任何证据证明它必须走 OAuth/WebView；也不是「可行」——连界面都不可静态确认。

#### 3.5.8 本网盘独有的阻塞点

**爱加密整包载荷**：dex 被加密成 3 个壳桩类，必须先脱壳才能开始任何静态工作。这与 123（无加固壳）的阻塞点完全不同。

---

### 3.6 123云盘

**结论：未知（需动态分析）** — 落在第 1 层下方。

#### 3.6.1 APK / 资料来源

- com.mfcloudcalculate.networkdisk 3.3.4（versionCode 3244），114861135 B，minSdk 24 / targetSdk 34，1 dex，仅 arm64-v8a。
- 索引：docs/evidence/index-123云盘.md（壳确认模式，协议覆盖 0）。

#### 3.6.2 dex 是否存在登录协议线索

**不存在。**

- classes.dex 仅 13916 B，**class_defs 共 4 个**：Ls/h/e/l/l/A;、C、N、S（壳桩类）。
- 扫描结果：login/auth 类名命中 **0**，URL 字符串命中 **0**。
- **未检出已知整包加固壳**：无 ijiami.*、无 libDexHelper.so、无 libjiagu*。另有 libijm-emulator.so 166360 B（**爱加密模拟器检测组件**），来源与是否参与脱壳 → 待确认。

#### 3.6.3–3.6.5 登录类型 / 协议 / 缺失环节

- 登录类型：**未知**。
- 官方客户端登录协议：**不能确认**。
- 缺失：登录接口路径与字段（libapp.so arm64 18219952 B 内）、签名机制、token 结构与刷新、UA/版本/指纹校验。

#### 3.6.6 后续必须通过什么动态分析

1. 动态抓包。
2. libapp.so 的 Dart AOT 分析。

#### 3.6.7 结论

**未知（需动态分析）**。

#### 3.6.8 本网盘独有的阻塞点

与天翼不同：**没有已知整包加固壳**，dex 之所以只有 4 个类，是因为业务写在 Flutter AOT 里，而不是因为被加密。因此本网盘的攻关入口是「Flutter 侧」而不是「脱壳」，只需动态抓包即可开始；但 libijm-emulator.so 的存在提示可能有运行期反调试/反模拟器，会干扰抓包。

---

### 3.7 蓝奏云

**结论：不可行（资料缺失）**

#### 3.7.1 APK / 资料来源

- com.tooyoung.lanzou 1.3.4.8，4765515 B，1 dex，无加固（libluajava.so、libmd5.so）。
- **这是第三方客户端，不是蓝奏云官方客户端。**

#### 3.7.2 为什么不可行

第三方客户端的登录协议不等同于官方客户端协议。按 docs/SCOPE.md 第 6 节「资料缺失的兜底策略」与 docs/PROVIDERS.md 第 4 节准入规则：

1. 不硬凑——不得拿第三方协议冒充官方协议；
2. 不降级——不得改走 OAuth / WebView / 手动粘贴 cookie；
3. 可重开——若日后取得官方 APK，可重开任务重新判定。

#### 3.7.3 结论

**不可行（资料缺失）**，不进 MVP 候选。

---

### 3.8 UC 网盘

**结论：不可行（资料缺失）**

#### 3.8.1 资料来源

- com.UCMobile 19.2.8.1564，136173692 B，11 dex，Flutter 主体（libdalvikhack.so / libdexloaderuc.so / libfakedexuc.so）。
- **这是 UC 浏览器，不是 UC 网盘客户端。** UC 网盘功能在浏览器内以 H5 页面提供。

#### 3.8.2 为什么不可行

- 不存在独立的客户端登录协议样本；H5 页面意味着登录走网页会话，而网页 cookie 复用是项目明令禁止的路径。
- 即便想逆向浏览器，其网盘部分是 H5 而非原生协议，方向本身不合格。

#### 3.8.3 结论

**不可行（资料缺失）**，不进 MVP 候选。**不得**因「浏览器里能登录」而改判可行。

---

### 3.9 迅雷云盘

**结论：不可行（资料缺失）**

#### 3.9.1 资料来源

- com.xunlei.downloadprovider 25.2.91.280，199853782 B，1 dex，非 Flutter，含加固壳 libDexHelper.so。
- **这是迅雷下载器，云盘只是其中一个子模块。**

#### 3.9.2 为什么不可行

- 云盘模块嵌在下载器内，未提供独立客户端；模块边界与协议归属无法在现有资料下确认。
- dex 被 libDexHelper.so 保护，静态拿不到业务逻辑。

#### 3.9.3 结论

**不可行（资料缺失）**，不进 MVP 候选。

---

## 4. 与 T-002d 结论的差异说明（必须记录）

T-002d 对四个 Flutter 网盘统一写「协议需动态分析」。T-002c 定向读取 dex 后，事实需要分成两组：

| 网盘 | T-002d 写法 | T-002c 实测 | 是否修正 |
|---|---|---|---|
| 天翼 | Flutter 主体，协议需动态分析 | dex 仅 3 个壳桩类，**0 条登录线索** | **不变**（确认） |
| 123 | Flutter 主体，协议需动态分析 | dex 仅 4 个壳桩类，**0 条登录线索** | **不变**（确认） |
| 夸克 | Flutter 主体，协议需动态分析 | **账号登录体系完整在 dex 的 Java 层**（UCLoginServiceImpl + MTOP over /acs.wsg_access）；网盘**文件**业务协议仍在 Dart 侧 | **必须修正**：登录「协议存在」，只是被 native 签名阻断 |
| 阿里 | Flutter 主体（混合），协议需动态分析 | **账号登录体系完整在 dex**（com/ali/user/mobile + Havana/login4android）；**文件**协议在 Dart / jsbridge 侧 | **必须修正**：同上 |

**修正的性质**：不是推翻 T-002d，而是补一层——「网盘文件业务协议」与「账号登录协议」在夸克/阿里身上归属不同层。T-002d 判的是文件业务，结论仍然成立；T-002c 判的是登录，落点在 Java 层。

**这直接改变了 T-005 的输入**：夸克与阿里从「未知（无资料）」变为「需攻关（有路径、缺 native）」，攻关成本可估算；天翼/123 仍是「未知（无资料）」。

**T-006 追加的第三条差异（2026-11-28）**：百度的结论由「需攻关」改为「原生登录可行」，依据是 `docs/evidence/baidu-login-deep-dive.md`。这与 T-002d 的百度索引无冲突（T-002d 判的是文件与接口覆盖面，T-006 判的是登录可行性），但**与 T-005 的 No-Go 输入冲突**——T-005 当时只承认坚果云 1 个「可行」。是否因此重跑 Go/No-Go 由用户决定，本文不改写 T-005 的历史结论。

---

## 5. 禁止路径排除记录

| 禁止路径 | 坚果云 | 百度 | 夸克 | 阿里 | 天翼/123 | 蓝奏/UC/迅雷 |
|---|---|---|---|---|---|---|
| OAuth 第三方授权 | 未发现（login/ 目录搜 oauth 0 命中） | 有 oauth/2.0/token 常量但**只取时间戳**；.oauth( 在 com/baidu/netdisk 0 命中 | 有第三方 SSO（微信/淘宝/支付宝），**已排除不计入** | 有支付宝 SSO 与 snsLogin，**已排除不计入** | 未知 | 不适用 |
| WebView 提取 cookie | 无 | 旧 startLogin 路径是 WebView 容器，**主链不用**；web2NativeLogin 从 cookie 读 BDUSS 属官方内部通道，**不计为可用**；T-006 复核：账号密码响应是 XML 解析（SapiCoreUtil.java:68-77），扫码三请求 cookie 传 null（QrCodeService.java:326/384） | 仅协议页 WebView，与凭证无关 | GET_WAP_LOGIN_COOKIES 属 cookie 通道，**不得使用** | 未知 | 不适用 |
| 手动粘贴 token | 不采用 | 不采用 | 不采用 | 不采用 | 不采用 | 不采用 |
| 网页会话复用 | 不采用 | 不采用 | 不采用 | 不采用 | 不采用 | **本身就是 H5，直接判定不合格** |

**结论：本次判定没有把任何一个网盘降级为 OAuth / WebView / cookie / 手动 token 方案。**

---

## 6. 可信度与剩余风险

| 风险 | 说明 | 影响 |
|---|---|---|
| 全部结论未经运行时验证 | 本文没有发任何真实请求 | 服务端风控/版本校验可能拒绝非官方客户端，静态可行不等于跑得通 |
| 坚果云「可行」是静态可行 | 密码仅 Base64、无签名、无指纹；但 UA 与验证码阈值未知 | 若服务端强校验 UA 或频繁触发人机验证，实际体验下降但协议仍成立 |
| 百度改判「可行」仍是静态结论 | T-006 已把 40 字段逐项定性、确认三条路径无 native 依赖；但服务端是否接受本项目构造的 sig / di 未验证 | 静态可行 ≠ 跑得通；风控挑战（验证码/设备信任/异地登录）可能要求人工介入 |
| 百度 A 路径也带条件性人机验证依赖 | 错误码 50052 会走与坚果云 R27 同级的 `ValidationService` 人机挑战并重试（`NewLoginUtil.java:2107-2145`），触发频率静态未知 | 若常规设备高频触发，A 与 B 的实际可达性会一起下降；此风险与 R27 同源，不改变静态档位 |
| 夸克/阿里依赖聚安全 native | 同一套闭源组件，两个网盘共用 | 若不移植 so，两条路径都走不通；移植则需 ADR 与合规评估 |
| 天翼/123 结论可能永久停在「未知」 | 需脱壳/Dart AOT，成本高 | T-005 不应把它们算作 Go 的分子 |
| reverse/ 产物不在 Git | 换机器需重新反编译 | 不影响已归档的 文件:行号 证据，但复现需重跑 |

---

## 7. 对 T-005 的输入（候选集合，不含选择）

按 T-005 决策标准（docs/TASKS.md「T-005」小节）：

- **硬准入 = 原生登录可行性「可行」** → **2 个：坚果云、百度**（百度为 T-006 复核后改判，此前 T-005 时的快照只有坚果云 1 个）。
- **「需攻关」可进候选但须先完成攻关** → **夸克、阿里** 2 个（百度已从这一档移出）。
- **「未知」不进阶段 2** → **天翼、123** 2 个。
- **「不可行」出 MVP** → **蓝奏、UC、迅雷** 3 个。

**注意一（口径）**：T-005 的 Go 判据是「可行集合 ≥ 2」。按本文当前结论为 **2 个**，形式上满足判据；但 T-005 的历史 No-Go 结论**不改写**，是否重跑 Go/No-Go 由用户决定。

**注意二（不同质）**：两个「可行」的静态环境并不相同——坚果云的验证码分支依赖腾讯闭源 Web 服务（`docs/RISKS.md` R27），百度则已确认三条路径**完全不依赖闭源 native**。**2026-11-28 补充**：百度 A 路径与坚果云默认路径**都带同级的条件性人机验证依赖**（分别见 §3.2.9 (6) 与 R27），差异只剩「触发频率」与「百度官方另有网页登录兜底」。重跑 Go/No-Go 时应把这层口径也写进判据，否则两家不是用同一把尺子量出来的。

本文**不选 MVP 的两个网盘**，这是 T-005 的决定。

---

## 8. 未解决问题与后续工作

| # | 问题 | 归属 | 下一步 |
|---|---|---|---|
| 1 | 百度的 tpl 与 appSignKey 常量值 | **已关闭（T-006）** | 已定位为 APK 硬编码常量 kd/______.java:11/:14/:15/:17；见 deep-dive §2 |
| 2 | 阿里 classes18.dex 的 AccountRpc 未反编译 | T-002c 遗留 | 单 dex 定向反编译（约 1 类，成本低） |
| 3 | 阿里 Havana 二维码流程字段 | T-002c 遗留 | 读 com/alicloud/databox/account/login/havana/* |
| 4 | 坚果云 UA / 验证码阈值的服务端行为 | 阶段 2 动态验证 | 真实抓包（需用户授权） |
| 5 | 天翼脱壳、123 Dart AOT | 未排期 | 需新任务卡 |
| 6 | 聚安全 so 是否移植 | 架构决策 | 需 ADR |
| 7 | 9 个网盘的 token 服务端有效期 | 阶段 2 | 真实请求后观测 |
| 8 | 百度服务端是否接受本项目构造的 sig / di | **T-006 新增，动态验证** | 用测试账号发 1 次请求看返回码（本任务禁止发请求，未做） |
| 9 | 百度 diExceptIndex（sapi_options）线上实际下发值 | **T-006 新增，动态验证** | 抓一次 options 下发响应；当前按默认空串实现 |
| 10 | 百度 BAIDUID cookie 是否必须非空 | **T-006 新增，动态验证** | ParamsUtil.java:81 允许空串，服务端是否接受未知 |
| 11 | 百度的「获取短信验证码」接口 | **已关闭（T-006 补充收口）** | 发码 = `POST {wap}/wp/api/login/sms`（前置 `cap/init`）；详见 deep-dive §15.2 |
| 12 | 百度扫码 hostDeviceId 的生成规则 | **已关闭（T-006 补充收口）** | 官方客户端从不赋值、恒 `null`、参数被丢弃（`a.java:333-353`）；本项目按缺省不传。原「推测任意稳定随机串可用」作废 |

---

## 附录 A：本次用到的检索命令（可复现）

~~~bash
# 0) 通用前提：所有命令在项目根目录执行，reverse/ 需先完成对应反编译（不进 Git）

# 1) 坚果云：登录目录排除 OAuth / WebView
grep -rnE 'WebView|loadUrl|oauth|OAuth' \
  reverse/nutstore/sources/sources/nutstore/android/v2/ui/login/
# 预期：0 命中

# 2) 百度：三个原生登录页是否含 WebView
for f in activity/NewLoginActivity activity/BaseNewLoginActivity helper/NewLoginHelper; do
  grep -nE 'WebView|loadUrl' \
    "reverse/baidu/sources/classes4/sources/com/baidu/sapi2/newlogin/$f.java" || echo "0 命中: $f"
done

# 3) 百度：Sdata 是否出现在 sapi2 登录链路
grep -rn 'Sdata' reverse/baidu/sources/classes2/sources/com/baidu/sapi2 \
                reverse/baidu/sources/classes12/sources/com/baidu/sapi2
# 预期：0 命中（说明 Sdata 不属于登录链路）

# 4) 百度：Sdata 实现由运行期注入的证据
grep -rnF '.n0(' reverse/baidu/sources/classes6/sources --include=*.java
grep -rnF 'n0(new kotlin.jvm.functions.Function2' 'reverse/baidu/sources'
# 预期：前者命中的全是无关类；后者 0 命中

# 5) 百度：OAuth 常量是否真的被用于鉴权
grep -rn 'openapi.baidu.com/oauth' 'reverse/baidu/sources'
grep -rnF '.oauth(' reverse/baidu/sources/classes*/sources/com/baidu/netdisk/
# 预期：前者只命中 SapiAccountRepository.generateSsoHash；后者 0 命中

# 6) 夸克：聚安全 native 依赖
grep -nE 'loadLibrary|SecurityGuardManager|native' \
  reverse/t002c/src/quark/classes7/sources/com/uc/encrypt/EncryptHelper.java
grep -nE 'native|SecurityGuardManager' \
  reverse/t002c/src/quark/classes7/sources/com/uc/encrypt/M9EncryptionHandler.java

# 7) 阿里：登录请求的风控注入点
grep -rn 'riskControlInfo' 'reverse/t002c/src/ali/classes9/sources/com/ali/user/mobile/data/'
grep -rn 'SecurityGuardManager' 'reverse/t002c/src/ali/classes9/sources/com/ali/user/mobile/base/helper/'

# 8) 阿里 / 夸克：登录实现是否在 dex 的 Java 层（而非 Dart）
python3 reverse/bridge-scan.py
# 脚本会统计自家包内 MethodChannel / Plugin / flutter / Bridge 的数量

# ===== T-006 追加（百度深度取证，详见 docs/evidence/baidu-login-deep-dive.md 第 14 节）=====

# 9) 百度：tpl / appSignKey 常量位置与注入链
grep -n 'public static String' reverse/baidu/sources/classes15/sources/kd/______.java
grep -rn 'setProductLineInfo' reverse/baidu/sources --include=*.java
# 预期：常量在 kd/______.java:11/:14/:15/:17；setProductLineInfo 全仓 4 处（1 定义 + 1 异常消息 + 1 人脸SDK转发 + 1 真实注入 C_.java:197）

# 10) 百度：签名真正在子类（基类 doSign 是空方法）
grep -rnF '.doSign(' reverse/baidu/sources --include=*.java
# 预期：唯一命中 HttpClientWrap.java:87
baksmali d reverse/baidu/dex/classes15.dex -o /tmp/hm/out && grep -A4 'doSign' /tmp/hm/out/com/baidu/pass/http/HttpHashMap.smali
# 预期：方法体只有 return-void

# 11) 百度：di 40 字段与灰度裁剪
sed -n '10,65p' reverse/baidu/sources/classes12/sources/com/baidu/sapi2/utils/SapiDeviceInfo.java
sed -n '67,210p' reverse/baidu/sources/classes12/sources/com/baidu/sapi2/utils/SapiDeviceInfo.java
sed -n '89,160p' reverse/baidu/sources/classes12/sources/com/baidu/sapi2/utils/ParamsUtil.java

# 12) 百度：passport_core 六个 native 的调用点与导出符号
grep -rn 'PassCoreVHelper' reverse/baidu/sources --include=*.java
# 预期：5 个文件 7 处
nm -D <解包出的>/lib/arm64-v8a/libpassport_core.so | grep -c 'Java_'
# 预期：6
nm -D <解包出的>/lib/arm64-v8a/libpassport_core.so | grep -c 'JNI_OnLoad'
# 预期：0
strings <解包出的>/lib/arm64-v8a/libpassport_core.so | grep -c RegisterNatives
# 预期：0
readelf -dW <解包出的>/lib/arm64-v8a/libpassport_core.so | grep NEEDED
# 预期：仅 liblog/libm/libstdc++/libdl/libc

# 13) 百度：扫码路径不发 di、不带 cookie，但**加 sig**（2026-11-28 修正）
grep -c 'HttpHashMapWrap' reverse/baidu/sources/classes2/sources/com/baidu/sapi2/QrCodeService.java
# 预期：0（只是没直接选签名实现类，不等于不签名）
grep -c 'getDeviceInfo' reverse/baidu/sources/classes2/sources/com/baidu/sapi2/QrCodeService.java
# 预期：0（不发 di）
grep -n 'doSign' reverse/baidu/sources/classes4/sources/com/baidu/sapi2/httpwrap/HttpClientWrap.java
# 预期：:87 —— 全仓唯一 doSign 调用点，扫码与账号密码共用该链路 → 扫码也带 sig

# 14) 百度：账号密码响应是 XML 解析（无 WebView）
sed -n '60,80p' reverse/baidu/sources/classes2/sources/com/baidu/sapi2/SapiCoreUtil.java
# 预期：正则 <client>...</client> + XmlPullParser

# ===== T-006 补充收口追加（2026-11-28）=====

# 15) 百度：libbaiduprotect_sec 的全部 Java 加载点
grep -rn 'baiduprotect' reverse/baidu/sources --include=*.java
# 预期：WBAESCipherImplement.java:17 的 loadLibrary + kh0/____.java:21 的常量（动态插件路径）
grep -rn 'libbaidusec' reverse/baidu/sources --include=*.java
# 预期：0 命中（APK 内不存在该命名的库）

# 16) 百度：短信发码与提交端点
grep -n 'wp/api/login' \
  reverse/baidu/sources/classes2/sources/com/baidu/sapi2/SapiAccountRepository.java
# 预期：:1126-1128 getAccountAndPwdUrl(/wp/api/login)、:1632-1634 getLoginSmsUrl(/wp/api/login/sms)

# 17) 百度：hostDeviceId 是否被赋值（结论：从未赋值，恒 null）
grep -rn '\.hostDeviceId *=' reverse/baidu/sources --include=*.java
# 预期：0 命中
sed -n '333,353p' reverse/baidu/sources/classes15/sources/com/baidu/pass/http/a.java
# 预期：只拼接 non-empty 的 key/value → null 参数被丢弃

# 18) 百度：cuid 是否依赖 native（结论：纯 Java）
grep -rnE 'loadLibrary|System\.load|" native "' \
  reverse/baidu/sources/classes3/sources/com/baidu/cesium
# 预期：0 命中

# 19) 百度：短信前置人机验证是 WebView 页面
grep -n 'pass-machine' reverse/baidu/sources/classes18/sources/com/baidu/validation/ValidationManager.java
grep -n 'loadUrl\|machine_sdk' \
  reverse/baidu/sources/classes18/sources/com/baidu/validation/activity/ValidationActivity.java
# 预期：:179 machine_sdk.show()、:319-321 loadUrl(...)
~~~

> 说明：`reverse/` 内的产物不进 Git，上述命令需先在本地完成对应 APK 的反编译（坚果云 jadx 全量、百度按 dex 分批、夸克 classes7/10、阿里 classes9/11）。
> `bridge-scan.py` 等扫描脚本位于 `reverse/`（同样不进 Git），只在本地复现用。

## 附录 B：本次未做的事（边界声明）

- 没有登录任何真实账号，没有发送任何真实请求，没有抓任何流量。
- 没有对天翼/123 的 libapp.so 做 Dart AOT 逆向。
- 没有破解任何加固壳（爱加密 / libDexHelper.so / 7z 容器）。
- 没有尝试调用或破解聚安全 native。
- 没有修改 Gradle / Rust / CI，没有新增依赖。
- 没有执行 T-005，没有选定 MVP 网盘。

**T-006 追加（2026-11-28）**：

- 没有为百度发任何真实请求，没有登录百度账号，没有抓包。
- 没有移植、复制或集成任何官方闭源 so；没有新增 Gradle / Cargo 依赖；没有改架构。
- 没有逆向 `libfire.so` / `libsofiresec.so` 的内部算法（只判关联）。
- ~~没有取证百度的「获取短信验证码」接口（T-006 范围只到「已拿到验证码之后」的登录步骤）。~~ **已于 2026-11-28 补充收口取证**（`/wp/api/login/sms`），但完整短信登录仍判「需攻关」。
- 没有改写 T-005 的历史 No-Go 结论（`docs/evidence/mvp-go-no-go.md` 原文保留）。

**T-006 补充收口追加（2026-11-28）**：

- 没有登录百度账号，没有发送任何真实登录或短信请求，没有抓包。
- 没有把 APK 硬编码密钥（`appSignKey` / `di` 的 AES key / 人脸 `pass-key`）的实际值写入文档或日志。
- 没有复制、提交或集成任何官方闭源 native so（本轮只做 `readelf`/`nm`/`strings` 静态观察）。
- 没有新增 Gradle / Cargo 依赖，没有修改 Android / Kotlin / Rust / UniFFI 架构。
- 没有执行 T-005，没有锁定 MVP 网盘，没有启动阶段 2。
