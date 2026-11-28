# 百度网盘原生登录深度取证（T-006）

- 任务：T-006 百度网盘原生登录攻关与第二家准入验证
- 基准提交：`4e36ec7cb11f91448d526df67bfeaa6658c27797`
- 类型：静态取证 + 可行性判定（**未写生产代码、未登录真实账号、未发送任何真实请求、未抓包**）
- 资料：`reverse/baidu/`（base.apk 251854857 B，20 dex，未加壳；覆盖率 100.0000% = 63259/63259）
- 可信度口径：**已验证**＝该静态代码/文件确实存在且本次实测读出（**不代表服务端行为已验证**）；**推测**＝有间接证据但未闭环；**待确认**＝本次未取到证据

---

## 1. 任务边界与结论摘要

### 1.1 边界（与 T-002c / T-005 的关系）

- 本文**只**回答「百度的三条登录路径能否由本项目 Rust Adapter 独立实现」这一静态问题。
- 本文**不**改写 T-005 的历史 No-Go 结论（`docs/evidence/mvp-go-no-go.md` 原文保留不动），**不**自动把百度计入 Go 分子，**不**锁定「坚果云 + 百度」组合。是否重跑 Go/No-Go 由用户决定。
- 本文**不**实现登录、**不**新增任何依赖、**不**移植官方闭源 so。

### 1.2 结论摘要（一句话版）

**百度的账号密码、短信验证码、扫码三条登录路径，在静态证据上都可以由本项目独立实现；核心认证算法（RSA 密码加密、`sig` 签名、`di` 的 AES 加密）均不依赖官方闭源 native 库，但公共请求参数 `zid` 的真实值依赖 `libfire` 与服务端下发数据（§17）。整体四档结论：原生登录可行（静态可行，动态未验证）。**

> **2026-11-28 补充收口后的一处修正**：上述「三条均可独立实现」已被 §15.7 修正为 **A 账号密码可独立实现 / B 短信需攻关（前置人机验证是 WebView + 服务端 JS）/ C 扫码可独立实现（带「服务端接受 `hostDeviceId` 缺席」这一动态前提）**；native 结论不变。**整体四档仍为「原生登录可行」**，因为 A、C 两条路径不含静态不可复刻环节。本节原文保留不改，差异登记见 §15.9。
>
> **再校正（2026-11-28，正文见 §16）**：本轮统一了四处口径——`t_cuid`/`t_appname` 的字段来源写反、`i()` 是否进入请求、`sf_zid` 的分支与 62 位常量、验证码 WebView 与坚果云 R27 的同口径判定；并新增「A 密码路径同样存在 50052 人机挑战分支」。**当前有效结论以 §16.1 为准。**

### 1.3 逐项结论速览

| # | 工作项 | 结论 | 与 T-002c 相比 |
|---|---|---|---|
| 1 | `tpl` / `appSignKey` 来源 | **APK 内硬编码明文常量**，非运行期注入、非远程下发 | **变化**：T-002c 记「静态未定位到常量值」，实际可定位到 `kd/______.java:11/:14/:15/:17` |
| 2 | `di` 40 字段 | 40 项全部逐字段定性；必需 22 / 条件 2 / 可选 16；**加密算法与密钥均在 Java 侧**，无 native | **变化**：T-002c 只记「40 个字段受灰度裁剪」，未分类 |
| 3 | `passport_core` 六 native | 全 dex 仅 7 处调用，**三条主链 0 处**；缺失时有 `catch (Throwable)` 兜底；无 `RegisterNatives` 动态注册 | **变化**：T-002c 记「待确认」，现确认不阻断 |
| 4 | 其他 native 安全库 | 6 个库全部与三条登录路径**无调用关系**（分别属业务 API 加签、APK 签名校验、P2P 下载、设备标识、sofire 旁路） | 与 T-002c 一致，证据补全 |
| 5 | 账号密码路径 | **可独立实现**（静态） | 由「需攻关」拆出单路径结论 |
| 6 | 短信路径 | **可独立实现**（静态） | 同上 |
| 7 | 扫码路径 | **可独立实现**（静态），且**不依赖任何既有登录态** | 同上 |
| 8 | Rust 可复刻性 | RSA/AES/MD5/签名/设备指纹/会话读写**全部可复刻**；无需 Kotlin 传网盘特有参数 | 新结论 |
| 9 | native 移植必要性 | **不需要**；无 ADR 需要提交 | 新结论 |
| 10 | 整体四档 | **原生登录可行（静态可行，动态未验证）** | 由「需攻关」改为「可行」 |

> §1.3 表中第 6 行（短信「可独立实现」）已被 §15.7.2 修正为「需攻关」；第 7 行（扫码）保留「可独立实现」但补动态前提。其余各行不受影响。

### 1.4 必须与结论一起读的三条限制

1. **「可独立实现」是静态结论**：本次没有发送任何请求，服务端是否接受本项目构造的 `di` / `sig` / UA，静态无法证明。
2. **`di` 的字段取值可复刻，但服务端校验强度不可知**：`di` 里 37 项在官方客户端里同样是常量或本机系统信息，只有「服务端怎么用」是黑盒。
3. **服务端风控（账号封禁/验证码挑战）仍属未验证风险**：见 `docs/RISKS.md` R5 / R28，本文不关闭这两条。

---

## 2. `tpl` / `appSignKey` 来源

### 2.1 结论

**`tpl` / `appId` / `appSignKey` 是 APK 内的硬编码明文字符串常量，通过 `AccountUtils` 静态注入到 `SapiConfiguration`，不存在运行期远程下发，也不存在构建期渠道注入。**

> 本文件不记录这三个参数的真实取值。定位见下表的 `文件:行号`，值本身留在 `reverse/baidu/`（不进 Git）。

### 2.2 赋值链（逐跳，已验证）

| 跳 | 文件:行号 | 动作 |
|---|---|---|
| 1 | `reverse/baidu/sources/classes15/sources/kd/______.java:14` | `b` = 普通（国内）分支 `appSignKey` 常量 |
| 2 | 同文件 `:15` | `c` = `tpl` 常量 |
| 3 | 同文件 `:17` | `e` = 国际分支 `appSignKey` 常量 |
| 4 | 同文件 `:33-37` | 按 `vw.___.___(BaseApplication.____())` 二选一：国际走 `AccountUtils.K("netdisk", c, "1", e)`，国内走 `AccountUtils.E("netdisk", "1", b)` |
| 5 | `reverse/baidu/sources/classes15/sources/com/baidu/netdisk/account/AccountUtils.java:153-157` | `E(String,String,String)` 写静态字段 `K`/`M`/`N` |
| 6 | 同文件 `:159-164` | `K(String,String,String,String)` 写 `K`/`M`/`N`/`L` |
| 7 | 同文件 `:181-190` | `o()`→`M`（appId）、`p()`→`N`（appSignKey）、`q()`→`K`（tpl）、`r()`→`L` |
| 8 | `reverse/baidu/sources/classes15/sources/com/baidu/netdisk/application/___.java:11-19` | `m851_()`→appId、`__()`→appSignKey，均先读 `DebugSetKey` 再回落 `AccountUtils` |
| 9 | `reverse/baidu/sources/classes15/sources/com/baidu/netdisk/account/e.java:6-13` | `m130_()`→国内 tpl（先 `DebugSetKey.passport_tpl`，null 则 `AccountUtils.q()`）；`__()`→国际 tpl（`AccountUtils.r()`） |
| 10 | `reverse/baidu/sources/classes15/sources/com/baidu/netdisk/application/C_.java:196-197` | `new SapiConfiguration.Builder(u).setProductLineInfo(str__, ___.m851_(), ___.__())` |
| 11 | `reverse/baidu/sources/classes2/sources/com/baidu/sapi2/SapiConfiguration.java:854-858` | `setProductLineInfo(tpl, appId, appSignKey)` 直接赋值 |
| 12 | 同文件 `:523-524` | 任一为空抛 `IllegalArgumentException`（说明 SDK 侧不做兜底猜值） |
| 13 | 同文件 `:1069-1071` | 构造器把 Builder 值拷进 `SapiConfiguration` |

### 2.3 `setProductLineInfo` 全部调用方（已验证，全仓 4 处）

| 调用方 | 性质 |
|---|---|
| `com/baidu/sapi2/SapiConfiguration.java:854` | 定义本身 |
| `com/baidu/sapi2/SapiConfiguration.java:524` | 仅出现在异常消息字符串里 |
| `classes4/sources/com/baidu/sapi2/outsdk/PassBiometricCall.java:16` | 人脸 SDK 把宿主配置转发一份 |
| **`classes15/sources/com/baidu/netdisk/application/C_.java:197`** | **唯一真实业务注入点** |

检索命令：`grep -rn 'setProductLineInfo' reverse/baidu/sources --include=*.java`（命中 4 处，与上表一致）。

### 2.4 参数表

| 参数 | 定义位置 | 赋值来源 | 覆盖条件 | 实际消费者 | 登录路径 | Rust 可获取性 | 结论 |
|---|---|---|---|---|---|---|---|
| `tpl` | `kd/______.java:15`（国内）/ `c`；国际分支取 `AccountUtils.r()`→`L` | APK 硬编码常量 | 仅 `NetDiskLog.isDebug()` 为真且外部 `netdiskdebugset` 文件存在时才被覆盖（见 2.5） | ① UA：`AbstractService.java:43-45` 拼 `tpl:<tpl>;android_sapi_v<version>`；② 每个 sapi2 请求的 `tpl` 公共参数：`httpwrap/Utils.java:20`；③ `di` 索引 7：`SapiDeviceInfo.java:89` | 三条均用 | **可**（编译期常量） | **静态可得** |
| `appId` | `kd/______.java:11`（`_____`）经 `AccountUtils.o()`→`M` | APK 硬编码常量 | 同上 | 每个 sapi2 请求的 `appid` 公共参数（`Utils.java:19`） | 三条均用 | **可** | **静态可得** |
| `appSignKey`（国内） | `kd/______.java:14`（`b`） | APK 硬编码常量 | 同上 | `HttpClientWrap.java:21` 取入 → `:87` `httpHashMap.doSign(appSignKey)` → `HttpHashMapWrap.java:9-15` 生成 `sig` | **三条均用** | **可** | **静态可得** |
| `appSignKey`（国际） | `kd/______.java:17`（`e`） | APK 硬编码常量 | 国际版分支 | 同上 | 三条均用 | **可** | **静态可得** |
| `mTPLAppName` | `SapiConfiguration.java:1048`，实参 `"wangpan_drive"`（`C_.java:199`） | 硬编码字面量 | 无 | `di` 索引 39 | 三条均用（`di` 整体） | **可** | **静态可得** |
| `mTPLCuid` | `SapiConfiguration.java:1049`，实参 `C_.java:142-147` `_____()` → `ew._.j` 或 `CommonParam.getCUID(u)` | 运行期取本机 cuid | 无 | `di` 索引 38 | 三条均用 | **可**（Rust 侧需 Kotlin 传入 cuid 或自行生成等价 UUID） | **静态可得** |
| `mPrivacyParamesRegulation` | `SapiConfiguration.java:1050`，实参 `false`（`C_.java:199`） | 硬编码 `false` | 无 | 关闭 `params/di` 灰度裁剪（`ParamsUtil.java:114-159` 第二道闸门） | 三条均用 | **可** | **静态可得（false）** |

### 2.5 `DebugSetKey` 覆盖只对 debug 构建生效（已验证）

- `kernel/debug/C_.java:82-88` `__(String)`：**`if (!NetDiskLog.isDebug() || 空串 || 内部 map 为 null) return null;`** → release 包恒返回 `null`。
- `kernel/debug/C_.java:53-72` 构造器只在 `NetDiskLog.isDebug()` 为真时才尝试读外部文件 `netdiskdebugset`。
- 因此 `___.m851_()` / `__()` / `e.m130_()` 的「先读 DebugSetKey」在 release 版是**空操作**，常量值生效。
- 含义：本项目若要复刻，直接用常量即可；**不存在「渠道包 / 服务端下发」这一类静态不可得的来源**。

### 2.6 `appSignKey` 的真实生效范围（重要，已验证）

> 这一条修正了 T-002b/README 风格的常见误读：基类 `HttpHashMap.doSign` 是空方法，真正的签名在子类。

- `reverse/baidu/sources/classes15/sources/com/baidu/pass/http/HttpHashMap.java:7-8` `doSign(String)` **是空方法**（baksmali 反汇编 classes15.dex 确认方法体仅 `return-void`）。
- `reverse/baidu/sources/classes4/sources/com/baidu/sapi2/httpwrap/HttpHashMapWrap.java:5-7` 构造器先 `putAll(Utils.buildCommonParams())`；`:9-15` `doSign(String)`：map 为空或已含 `sig` 则返回，否则 `put("sig", SapiUtils.calculateSig(getMap(), str))`。
- `HttpClientWrap.java:86-88`：**`if (httpHashMap != null) httpHashMap.doSign(this.appSignKey)`** —— 这是全仓唯一的 `.doSign(` 调用点（`grep -rn '\.doSign('` 命中 1 处）。
- 结论：**凡是走 `HttpClientWrap` 的请求都会被加 `sig`**，而三条登录路径全部走 `HttpClientWrap`（见第 6/7/8 节），所以三条路径**都需要 `appSignKey`**，且它是常量 → Rust 可内置。
- 补充：`SapiAccountRepository.java:2407`、`:2455` 另有两次手动 `put("sig", calculateSig(...))`（EAC 登录 / UC 授权登录），`:19-44` 有同算法的私有副本。

### 2.7 签名算法（已验证，纯 Java 可复刻）

`reverse/baidu/sources/classes12/sources/com/baidu/sapi2/utils/SapiUtils.java:135-160` `calculateSig(Map,String)`：

1. `map.remove("sig")`（:136）
2. 取全部 key 排序（:137-142）
3. 逐个：值非空则追加 `key` + `=`（`wm0.___.__`）+ `URLEncoder.encode(value,"UTF-8")` + `&`（:144-156）
4. 追加 `sign_key=` + 密钥（:157-158）
5. `h81.___._____(bytes, false)` → **MD5 十六进制小写**（:159；实现见 `classes15/sources/h81/_____.java`）

→ **纯 MD5 + 字典序拼接，无 native、无服务端交互。**

公共参数（每个 sapi2 请求都会带，并一起参与 `sig`）：`reverse/baidu/sources/classes4/sources/com/baidu/sapi2/httpwrap/Utils.java:5-25`——`client`、`cuid`、`clientid`、`clientfrom=native`、`zid`、`clientip`、`appid`、`tpl`、`app_version`、`sdkversion`、`<PluginPlatformService 常量>`。
---

## 3. `di` 设备指纹 40 字段逐项分析

路径代号：**A** = 账号密码登录，**B** = 短信验证码登录，**C** = 扫码登录。

### 3.1 先确认「是不是 40 个」

**是 40 个，与既有资料一致。** 依据：`reverse/baidu/sources/classes12/sources/com/baidu/sapi2/utils/SapiDeviceInfo.java:13-56` 的 `DeviceInfoCookieManager.getCookiesKeyList()` 返回的 `Arrays.asList(...)` **恰好 40 个元素**（逐个数过，索引 0–39）；`:58-63` 的 `updateCookiesMap` 用同一顺序写入 `cookiesMap`；`buildDeviceTokens` `:67-187` 也按同一索引顺序 `add`。三处顺序严格一一对应，不存在「名单 40 个但发送 39 个」的错位。

### 3.2 条件控制逻辑（先看开关，再看字段）

每个索引统一写法（`:82` 起逐条重复）：

```
arrayList.add( (diExceptIndex.contains(idx) || !ParamsUtil.checkDiUpload(gray, idx)) ? "" : <取值> );
```

即**字段被排除时不是「不发送」，而是发送空串**——40 个位置永远都在，`di` 的总形状固定。这一点很重要：Rust 侧不需要实现「字段存在性」逻辑，只需要实现「哪些位置填空串」。

四个控制源：

| 控制源 | 定义位置 | 取值为排除项时 | 静态可得性 |
|---|---|---|---|
| `diExceptIndex` | `SapiOptions.java:62` 默认空 list；由 `:694-712` `fromJSON` 从服务端 `sapi_options` 填充 | 对应索引填 `""` | **静态不可得**（服务端下发 + `SharedPreferences("sapi_options")` 本地缓存，`SapiContext.java:534-544` 读、`:879-882` 写、`SapiCache.java:107-145` 更新） |
| `dt_params`（灰度裁剪） | `ParamsUtil.java:114-159` `getGrayControlParams()` | 同上 | **静态不可得**，且见下条 |
| `mPrivacyParamesRegulation`（灰度总闸） | `SapiConfiguration.java:1050`，实参 `false`（`C_.java:199`） | **该值为 false → `getGrayControlParams()` 在第二道闸门直接返回空 JSONArray**（`ParamsUtil.java:118` 分支）→ `checkDiUpload` 收到 null → 恒返回 true → **`dt_params` 对 di 完全不生效** | **静态可得：false** |
| `mapping` 灰度模块 + `extraParams.dt_params` | `ParamsUtil.java:121-159` | 受上一条压制，本客户端实际不生效 | 静态可得：不生效 |

**关键推论（已验证）**：因为 `C_.java:199` 传的第三个实参是 `false`，本客户端的 `di` **不会**被 `dt_params` 灰度裁剪；唯一能裁剪它的是 `diExceptIndex`（服务端 `sapi_options` 下发，默认空）。

其余会影响字段值的判断（逐条追过）：

| 判断 | 位置 | 影响 |
|---|---|---|
| SDK/权限/版本判断 | `SapiUtils.java:268-282` `getClientId` | 未初始化或未同意危险权限且非浏览模式 → 返回 null → 索引 6 空串 |
| `getClientId` 空值回落 | `SapiUtils.java:352-366` `getDeviceID` | 异常时回落 `"123456789"+md5(随机数)`（不是空） |
| 登录类型差异 | 无 | `buildDeviceTokens(String)` 只接受一个「用途路径串」（索引 10），**不按登录类型分支** |
| 接口路径差异 | 仅索引 10 | A 路径传 URL 相关串、B 路径传 `"/v2/sapi/login"`（`SapiAccountRepository.java:267`）、`SapiCache.reportDi` 传 `"/static/appsapi/conf/android-conf.txt"`（`SapiCache.java:30`） |
| 白名单 | 无命中 | `grep -rn 'whitelist\|WhiteList' SapiDeviceInfo.java ParamsUtil.java SapiUtils.java` → 0 命中 |

### 3.3 40 字段完整表

> 「构造位置」全部为 `reverse/baidu/sources/classes12/sources/com/baidu/sapi2/utils/SapiDeviceInfo.java` 的行号（除特别标注）。
> 「对应登录路径」：**A+B** 表示同一份 `di` 同时用于账号密码（作 `DVIF` cookie）与短信（作 body 的 `di` 参数）。**扫码（C）不发送 `di`**，见 §8.4。

| # | 字段名 | 数据来源 | 构造位置 | 条件逻辑 | 对应登录路径 | 分类 | 个人信息风险 | Rust 复刻结论 |
|---|---|---|---|---|---|---|---|---|
| 0 | `PackageName` | `context.getPackageName()` | `:82` | 仅 `diExceptIndex` | A+B | **必需** | 低 | 可复刻（Rust 拿宿主包名，或 Kotlin 经 UniFFI 传一个通用字符串） |
| 1 | `AppVersion` | `SapiUtils.getVersionName()`（`SapiUtils.java:577-587`，PackageInfo→versionName，异常回落 `"0"`） | `:83` | 同上 | A+B | **必需** | 低 | 可复刻（本机需传 versionName，属通用系统信息） |
| 2 | `SdkVersion` | `isAccountManager.getVersionName()` | `:84` | 同上 | A+B | **必需** | 低 | 可复刻（同索引 1 的值） |
| 3 | `PhoneModel` | `getOSModel()`→`Build.MODEL`，空则 `"-1"`（`:215-218`） | `:85` | 同上 | A+B | **必需** | 中（设备型号易识别） | 可复刻（`Build.MODEL`，通用系统信息） |
| 4 | `SystemVersion` | `getOSVersion()`→`Build.VERSION.RELEASE`，空则 `"-1"`（`:220-223`） | `:86` | 同上 | A+B | **必需** | 低 | 可复刻（`Build.VERSION.RELEASE`） |
| 5 | `SystemType` | 字面量 `"android"`（`OS_TYPE`，`:5`） | `:87` | 同上 | A+B | **必需** | 无 | 可复刻（常量） |
| 6 | `cuid` | `SapiUtils.getClientId()`（`:268-282`，空则 `getDeviceID`；未同意危险权限且非浏览模式→null） | `:88` | 同上 + 上面那条前置判断 | A+B | **必需** | **高**（长期稳定设备标识） | 可复刻（生成等价 UUID 即可；`com/baidu/cesium/` 下 grep `loadLibrary\|System.load\| native ` **0 命中** → 纯 Java + 本地文件，无 native） |
| 7 | `tpl` | `confignation.tpl`（即 `kd/______.java:15` 常量，见 §2） | `:89` | 同上 | A+B | **必需** | 无 | 可复刻（常量） |
| 8 | `uid_count` | **无取值**，恒 `""` | `:90` | — | A+B | **可选**（永远空串） | 无 | 无需复刻，直接填空串 |
| 9 | `uid_list` | **无取值**，恒 `""` | `:91` | — | A+B | **可选** | 无 | 同上 |
| 10 | `usetype` | `buildDeviceTokens(String str)` 的**入参** | `:92` | 入参为 null 时置空串 | A+B | **必需** | 低 | 可复刻（B 路径传 `"/v2/sapi/login"`，A 路径传 URL 串） |
| 11 | `used_times` | `getDeviceInfoReadTimes()`（`SapiContext.java:380-384`，每次调用 +1 并回写 SharedPreferences） | `:93` | 同上 | A+B | **必需** | 中（可用于关联调用次数） | 可复刻（本地计数器，需持久化） |
| 12 | `cur_uid` | `SapiContext.getCurrentAccount().uid` | `:95` | 账号为 null 时置空串；登录前**通常为空** | A+B | **条件** | 中（账号 ID） | 可复刻（登录前为空串即可） |
| 13 | `net_type` | `getNetworkClass()`（`SapiUtils.java:414-455`，WIFI/5G/4G/3G/2G/UNKNOWN/UNCNCT；异常分支**无 return**→null） | `:96` | 同上 | A+B | **必需** | 低 | 可复刻（需 `ConnectivityManager`，属通用系统信息） |
| 14 | `is_root` | **无取值**，恒 `""` | `:97` | — | A+B | **可选** | 无 | 直接填空串 |
| 15 | `wifi` | `SapiUtils.getWifiInfo()` → **`return ""`（`:589-591`，方法体就是空串）** | `:98` | 同上 | A+B | **可选** | **高（若真取值会含 MAC/BSSID）** | 直接填空串（官方自己也不取） |
| 16 | `imei` | `SapiDeviceUtils.getIMEI()` → **`return ""`（`:74-76`）**；且 base.apk **未声明 `READ_PHONE_STATE`**（`aapt dump permissions` 实测） | `:99` | 同上 | A+B | **可选** | **高（若真取值是 IMEI）** | 直接填空串 |
| 17 | `emulator` | `isEmulator() ? "emulator" : "android"`；`isEmulator` **恒 false**（`SapiUtils.java:646-648`） | `:100-105` | 同上 | A+B | **必需**（值恒为 `"android"`） | 无 | 可复刻（常量 `"android"`） |
| 18 | `mac_address` | **无取值**，恒 `""` | `:106` | — | A+B | **可选** | **高（若真取值是 MAC）** | 直接填空串 |
| 19 | `cpu_info` | `getCpuName()`（`SapiUtils.java:327-342`，读 `/proc/cpuinfo` 首行 `split(":\\s+",2)[1]`，异常 `""`） | `:107` | 同上 | A+B | **必需** | 中（SoC 型号） | 可复刻（读 `/proc/cpuinfo`） |
| 20 | `ram` | `getRamMemorySize()`（`:543-558`，`/proc/meminfo` 首行去 `kB`） | `:108` | 同上 | A+B | **必需** | 低 | 可复刻（读 `/proc/meminfo`） |
| 21 | `internal_memory` | `getInternalMemorySize()`（`:391-399`，`StatFs(getDataDirectory())` blockSize×blocks/1024） | `:109-121` | 同上；此 if/else **两个分支都赋 `i=34; i2=28`** | A+B | **必需** | 低 | 可复刻（`StatFs`） |
| 22 | `internal_avail_memory` | `getInternalAvailableMemorySize()`（`:381-389`） | `:122-127` | 同上 | A+B | **必需** | 低 | 可复刻 |
| 23 | `up_time` | `getTimeSinceBoot()`（`:573-575`，`SystemClock.elapsedRealtime()/1000`） | `:128-133` | 同上 | A+B | **必需** | 低 | 可复刻（`SystemClock.elapsedRealtime()`） |
| 24 | `gps` | **无取值**，恒 `""`（`:134-137` 只调 `checkDiUpload`） | `:134-137` | — | A+B | **可选** | **高（若真取值是位置）** | 直接填空串；且 base.apk **未声明 `ACCESS_FINE_LOCATION`** |
| 25 | `package_list` | `TextUtils.join(",", getPackageList())`（`:467-480` 查 `baidu.intent.action.account.SHARE_SERVICE` 的 `queryIntentServices`） | `:138` | 同上 | A+B | **必需**（值可能为空串） | 中（已装应用列表） | 可复刻（`queryIntentServices` + `Intent` 常量） |
| 26 | `ip` | `SapiUtils.getLocalIpAddress()` → **`return ""`（`:405-407`）** | `:139-143` | 同上 | A+B | **可选** | **高（若真取值是内网 IP）** | 直接填空串 |
| 27 | `device_name` | `getDeviceName()`（`:368-375`，`confignation.deviceName`）；空则 `getBlueToothDeviceName()` **恒 `""`（`:264-266`）** | `:144-151` | 同上 | A+B | **必需**（值可能为空串） | 中（用户设备名） | 可复刻（取 `Build.MODEL` 或蓝牙名，空则空串） |
| 28 | `map_location` | **无取值**，恒 `""` | `:152-155` | — | A+B | **可选** | **高** | 直接填空串 |
| 29 | `device_sn` | **无取值**，恒 `""` | `:156` | — | A+B | **可选** | **高（若真取值是序列号）** | 直接填空串 |
| 30 | `device_uuid` | **无取值**，恒 `""` | `:157` | — | A+B | **可选** | 中 | 直接填空串 |
| 31 | `mtj_cuid` | **无取值**，恒 `""` | `:158` | — | A+B | **可选** | 中 | 直接填空串 |
| 32 | `idfa` | **无取值**，恒 `""` | `:159` | — | A+B | **可选** | 中（iOS 概念，Android 恒空） | 直接填空串 |
| 33 | `baidumap_cuid` | **无取值**，恒 `""` | `:160` | — | A+B | **可选** | 中 | 直接填空串 |
| 34 | `sf_zid` | `SapiSafeFacade.getCurrentZid(context)`（`SapiSafeFacade.java:23-27`→`SafeService.getZidAndCheckSafe`→`FH.gzfi`→libfire；失败返回 **`"NoZidYet"`**） | `:161` | 同上；且 `getDiCookieInfo` 里 `sf_zid=="NoZidYet"` 会二次尝试（`:243`） | A+B | **必需**（失败时值为 `"NoZidYet"`） | 中（设备授权 ID） | **不恒为 `"NoZidYet"`**：共五种可能取值（`"NoZidYet"` / 空串 / 静态缓存 / SP 文件解密值 / **62 位** hex 常量）；`sofire` 链路有纯 Java 兜底（`d/b.java:443-466`），**不会抛异常中断**。分支表见 §16.3.4 |
| 35 | `hostver` | **无取值**，恒 `""` | `:162-165` | — | A+B | **可选** | 无 | 直接填空串 |
| 36 | `iccid` | `SapiUtils.getIccid()` → **`return ""`（`:377-379`）** | `:166` | 同上 | A+B | **可选** | **高（若真取值是 SIM ICCID）** | 直接填空串 |
| 37 | `pass_bio_ver` | **硬编码字面量 `"9.15.0.2026"`** | `:167` | 同上 | A+B | **必需** | 无 | 可复刻（常量） |
| 38 | `t_cuid` | `confignation.mTPLCuid`（实参 `C_.java:142-147` `_____()`→`ew._.j` 或 `CommonParam.getCUID(u)`） | `:168-180` | 为 null 时：`checkDiUpload(gray,6)` 为真则用 `getClientId()`，否则 `""` | A+B | **条件**（依赖 null 分支） | 中 | 可复刻（用与 cuid 同源的值即可，实际等价于取到值而非走 null 分支） |
| 39 | `t_appname` | `confignation.mTPLAppName`（实参字面量 `"wangpan_drive"`，`C_.java:199`） | `:181-184` | 为 null 时回落 `getClientId()` | A+B | **必需** | 无 | 可复刻（常量 `"wangpan_drive"`） |

### 3.4 分类统计

| 分类 | 字段数量 | 备注 |
|---|---|---|
| 必需 | **22** | 其中 3 项值为常量（5 `android`、17 `android`、37 `9.15.0.2026`），2 项可能合法为空串（25 `package_list`、27 `device_name`），1 项失败时有明确兜底值（34 `NoZidYet`） |
| 条件 | **2** | 12 `cur_uid`（登录前为空）、38 `t_cuid`（仅在 `mTPLCuid` 为 null 时才走灰度判断分支） |
| 可选 | **16** | 全部是「官方自己也填空串」的占位位：8/9/14/18/24/28/29/30/31/32/33/35 无取值直接空串；15/16/26/36 的取值方法体就是 `return ""` |
| **合计** | **40** | |

### 3.5 「服务端必需性」标注（不得冒充已验证）

本节所有分类都是**客户端代码层面**的判定：即「这段代码在这些路径上必然构造并发送该字段（或必然填空串）」。**服务端是否会因为某字段为空/异常而拒绝登录，静态代码无法证明。** 因此对上表整体追加一条：

> **服务端必需性：全部 40 项均待确认（需动态验证）。**

另外，「官方客户端恒填空串」不等于「服务端接受空串」——只能说明官方自己发的是空串，服务端必然接受这个官方行为。这一点对我们的复刻是有利的：**只要 Rust 侧与官方发送同样形状的 `di`，就与真机官方客户端在服务端看来一致**。

### 3.6 `di` 的编码与传输（已验证）

`getDeviceInfo`（`:205-209`）→ `encryptDeviceInfo`（`:193-203`）：

1. `plain = TextUtils.join(DELIMITER, tokens)`，`DELIMITER = "\u0001"`（`:7`）
2. `iv = buildIV()`（`:189-191`）= `String.format("%02d", rnd(100))` + `currentTimeMillis()/1000` + `String.format("%03d", 11)` + `"0"` → 固定 16 字符
3. `b64 = Base64( AES/CBC/NoPadding(plain, iv, AES_KEY) )`，`AES_KEY` 为 `:8` 的 16 字符硬编码常量（**明文写在源码里，本次未把值抄出**）
4. `di = iv + "_" + b64 + "_" + md5(iv + "_" + b64 + "_check").substring(0,6)`（分隔符 `bl0/c.java:11` `__="_"`）
5. 异常时 `return ""`

**AES 与 MD5 全部在 Java 侧（`classes15/sources/h81/_____.java`，AES/CBC/NoPadding + Base64 + MD5），无 native。** → Rust 可 1:1 复刻。

**隐私风险点（已验证）**：`:185` `Log.e("privacy_control", "token=" + arrayList)` 与 `:201` `Log.e("privacy_control", "plain=" + str)` 会把**整条明文 di**（含 cuid、已装应用列表、内存、CPU 等）写进 logcat。**本项目不复刻该日志行为。**
---

## 4. `passport_core` 六个 native 方法逐项追踪

### 4.1 声明与库形态（已验证）

`reverse/baidu/sources/classes12/sources/com/baidu/sapi2/utils/PassCoreVHelper.java` 全文 20 行：

- `:5-7` `static { System.loadLibrary("passport_core"); }`
- `:9-19` 六个 `public static native String`，**全部无参数**

| native 方法 | 参数 / 返回值 | JNI 对应符号（`nm -D` 实测） | 地址 |
|---|---|---|---|
| `getAesIv()` | `()` → `String` | `Java_com_baidu_sapi2_utils_PassCoreVHelper_getAesIv` | `0x6f60` |
| `getAesKey()` | `()` → `String` | `..._getAesKey` | `0x70a4` |
| `getOneKeyLoginAppKey()` | `()` → `String` | `..._getOneKeyLoginAppKey` | `0x7470` |
| `getOneKeyLoginAppSecret()` | `()` → `String` | `..._getOneKeyLoginAppSecret` | `0x75b4` |
| `getSofireAppKey()` | `()` → `String` | `..._getSofireAppKey` | `0x71e8` |
| `getSofireSecKey()` | `()` → `String` | `..._getSofireSecKey` | `0x732c` |

库形态（已验证）：

- 文件：`reverse/baidu/input/split/split_config.arm64_v8a.apk` 内 `lib/arm64-v8a/libpassport_core.so`，**182504 B**。
- `nm -D` 导出**恰好 6 个** `Java_com_baidu_sapi2_utils_PassCoreVHelper_*` 符号，**没有其它 `Java_` 导出**。
- **不存在 `JNI_OnLoad`**；`strings` 搜 `RegisterNatives` **0 命中** → **静态注册，无动态注册的隐藏方法**（这一点排除了「六个声明之外还有隐藏 native 入口」的可能）。
- `readelf -dW` 的 `NEEDED` 只有 `liblog` / `libm` / `libstdc++` / `libdl` / `libc` → **不依赖任何其它百度自研 so，自包含**。
- `objdump -d --start-address=0x70a4 --stop-address=0x71e8` 反汇编 `getAesKey` 的结构：`adrp x1, 0x21000` + `add x1, x1, #0xc81` 指向 `.rodata` 常量 → 调内部函数（入口 `0x89e8`）→ 再经 vtable 偏移 `1336` 调一次 → 走 JNI `NewStringUTF` 返回。**即密钥是 `.rodata` 里的硬编码常量，经库内解密函数处理后返回，不是运行时从服务端取。**

### 4.2 全 dex 调用点（已验证，含检索证据）

检索命令与结果：

```
grep -rn 'PassCoreVHelper' reverse/baidu/sources --include=*.java
```

命中 **5 个文件、7 处调用**（第 6 处是 `static` 块里的 `loadLibrary`）：

| 调用点 | 方法 | 上层功能 | 与登录的关系 |
|---|---|---|---|
| `PassCoreVHelper.java:6` | `loadLibrary("passport_core")` | 类初始化 | 三类登录路径**都不触发**该类的静态初始化（见 4.3） |
| `classes2/sources/com/baidu/sapi2/SapiConfiguration.java:553` | `getSofireAppKey()` | `sofireAppKey` 为空时的兜底取值 | **间接**：影响 sofire 初始化，而 sofire 只影响 `di` 索引 34 的 `sf_zid` |
| 同文件 `:562` | `getSofireSecKey()` | `sofireSecKey` 兜底 | 同上 |
| `classes4/sources/com/baidu/sapi2/outsdk/OneKeyLoginSdkCall.java:131-132` | `getOneKeyLoginAppKey()` / `getOneKeyLoginAppSecret()` | **运营商一键登录 SDK**（中国移动/联通/电信） | **无关**：本项目不使用一键登录 |
| `classes4/sources/com/baidu/sapi2/share/ShareStorage.java:259` | `getAesIv()` | 分享数据本地 AES 存储 | 无关 |
| 同文件 `:290`、`:352` | `getAesKey()` / `getAesIv()` | 同上 | 无关 |

**三条主链（账号密码 A / 短信 B / 扫码 C）的主要代码路径上没有任何一处直接调用 `PassCoreVHelper`。**

### 4.3 「没有调用点」不等于「运行时不需要」——`loadLibrary` 的触发条件分析

`PassCoreVHelper` 是**普通类**，它的 `System.loadLibrary("passport_core")` 位于 `static` 块，只有在该类被**首次主动使用**（方法调用、字段访问、`new`、`Class.forName`）时才触发类初始化。因此：

| 可能绕开的途径 | 本次核查结果 | 结论 |
|---|---|---|
| 反射调用（`Class.forName("…PassCoreVHelper")` / `getMethod`） | `grep -rn 'PassCoreVHelper' --include=*.java` 命中的 7 处**全部是直接静态调用**，无一处字符串反射写法 | 未发现 |
| JNI 回调（native 侧 `FindClass` 调回 Java） | 库只导出 6 个 `Java_*`，无 `JNI_OnLoad`、无 `RegisterNatives`；反汇编 `getAesKey` 只有 `.rodata` 取常量 + 内部解密 + `NewStringUTF` | 未发现回调 Java 侧 |
| 动态注册 | 见上，0 命中 | 未发现 |
| SDK 内部间接调用（经 `SapiConfiguration` 兜底路径） | `SapiConfiguration.java:553-563` **只要 `sofireAppKey` 非空就不会调用**；而 `sofireAppKey` 来自 `Builder.sofireSdkConfig(...)`（`C_.java:197` 传 `AccountUtils.O/P/Q`） | **取决于 `AccountUtils.O` 是否为空** |

对最后一条再追一层：`C_.java:197` `.sofireSdkConfig(AccountUtils.O, AccountUtils.P, AccountUtils.Q)` —— `AccountUtils` 的 `O`/`P`/`Q` 字段在 `AccountUtils.java:11-14` 的可见初始化里**只有 `K/L/M/N` 四个被赋值**（`E()`/`K()` 两个 setter），`O`/`P`/`Q` 在本次检索中**未见赋值点**。

> 可信度：**推测**。`AccountUtils.java` 的 O/P/Q 可能由其它类通过反射或未在本次范围内的 setter 赋值；本次只确认「在 `AccountUtils.java` 内无赋值」。若 O/P/Q 在运行时为空，则 `SapiConfiguration` 会走 `catch (Throwable)` 兜底路径（`:553-575` 对 `Exception` 与 `Throwable` 各有一个 catch），**`build()` 仍然成功返回**，只是 `sofireAppKey` 为空。

### 4.4 加载失败时的行为（已验证，这是「不构成阻断」的关键证据）

`SapiConfiguration.java:553-575` 两处写法同构：

```java
if (TextUtils.isEmpty(sofireAppKey)) {
    try { sofireAppKey = PassCoreVHelper.getSofireAppKey(); }
    catch (Exception e) { Log.e(...); }
    catch (Throwable th) { Log.e(...); }
}
```

- `System.loadLibrary` 失败会抛 `UnsatisfiedLinkError`，它继承 `Error` 而非 `Exception`；此处有 **`catch (Throwable)`** 兜住。
- 即便兜住后 `sofireAppKey` 仍为空，`build()` 也**不因它抛异常**（`:523-524` 的空值校验只针对 `tpl`/`appId`/`appSignKey`，不针对 sofire）。
- 结论：**`passport_core` 缺失 → 最多导致 `sofireAppKey` 为空 → 进而导致 `di` 索引 34 取到 `"NoZidYet"` 或 sofire 的纯 Java 兜底常量 → 登录请求照常发出。**

### 4.5 `sf_zid` 的纯 Java 兜底链（已验证，证明「无需 native 也能有值」）

`SapiSafeFacade.java:23-27` → `SafeService.getZidAndCheckSafe`（`SafeService.java:27-30`）→ `FH.gzfi`（`FH.java:49-51`）→ `com/baidu/sofire/d/b.java:645-678`：

1. `!l.k.a(context)` → **直接返回 `e(context)`**（纯 Java 兜底，`:443-466`）
2. 静态字段 `a` 非空 → 直接返回该缓存值
3. `!l.b.a(context,false)` → 返回 `""`
4. 反射调 `gzfi` 失败 → 回落 `b(...)`

`e(context)`（`d/b.java:443-466`）的顺序：读本地缓存 → 读 `l.d` 文件按 `"|"` split 后解密 → **全部失败时返回硬编码常量 `"74FFB5E6…BA03"`（**62 位**十六进制，本次未把完整值抄进文档）**。

> **2026-11-28 校正**：七分支与五种取值的完整关系见 §16.3；此处「一定有非空值」的说法只在命中兜底分支时成立，分支 ③/⑦ 会返回空串并由上层转成 `"NoZidYet"`。

> 含义：**在 `libfire.so` 完全不可用的环境下，`sf_zid` 仍然会得到一个非空值**（缓存/文件/硬编码常量三选一），不会是 `null` 也不会抛异常。

### 4.6 六个方法的登录关联结论

| native 方法 | 账号密码 A | 短信 B | 扫码 C | 实际作用 | 是否构成登录阻断 |
|---|---|---|---|---|---|
| `getAesIv` | 无调用 | 无调用 | 无调用 | 分享存储 AES IV | **否** |
| `getAesKey` | 无调用 | 无调用 | 无调用 | 分享存储 AES Key | **否** |
| `getOneKeyLoginAppKey` | 无调用 | 无调用 | 无调用 | 运营商一键登录 | **否**（本项目不用一键登录） |
| `getOneKeyLoginAppSecret` | 无调用 | 无调用 | 无调用 | 同上 | **否** |
| `getSofireAppKey` | 间接（仅 sofire 初始化，且仅当 `AccountUtils.O` 为空） | 同 | 同 | sofire SDK 初始化 | **否**（有 `catch (Throwable)` + 纯 Java 兜底） |
| `getSofireSecKey` | 同 | 同 | 同 | 同上 | **否** |

### 4.7 检索范围声明（避免「没找到就断言不需要」）

| 项 | 内容 |
|---|---|
| 搜索范围 | `reverse/baidu/sources` 全量（20 个 dex 分批反编译产物，`com/baidu/` 类覆盖率 100.0000%） |
| 检索命令 | `grep -rn 'PassCoreVHelper' --include=*.java`；`grep -rn 'passport_core' --include=*.java`；`grep -rn 'System.load(' --include=*.java`；`grep -rn 'dlopen' --include=*.java`；`nm -D` / `readelf -dW` / `strings \| grep RegisterNatives`（对 .so） |
| 搜索结果 | 7 处调用、1 处 `loadLibrary`、六个 `Java_` 导出、无 `JNI_OnLoad`、无 `RegisterNatives` |
| 反射/回调可能性 | 已分别核查，未发现（见 4.3） |
| 仍存在的静态盲区 | ①sofire 内部 `Engine`/`EngineProxy` 会在运行时**动态下载并加载** `libsofiresec.so`（`sofire/x0/jni/ProtectSoUtil.java:19-40` 明确从下载目录 `System.load(path)`），该路径不受 Java 层静态引用约束；②`libfire.so` 自身的 JNI 头文件与导出符号未做完整逆向（超出本任务范围） |

---

## 5. 其他 native 安全组件与三条登录路径的关联

> 只分析与登录相关的实际关联，不对无关库做逆向。

| 原生库 | 大小 | 加载入口（文件:行号） | 调用链 | 关联登录路径 | 证据 | 影响判断 |
|---|---|---|---|---|---|---|
| `libnetdisk-security.so` | 1082464 B | `classes12/sources/com/baidu/netdisk/security/URLHandler.java:5-9`（`Runtime.loadLibrary("netdisk-security")` + `getSK()` + `getDeviceID()`） | `URLHandler.java:20` `native String handlerURL(Context, String url, String bduss, String stoken)`；唯一调用点 `classes6/sources/ex/n3.java:49` | **无** | `ex/n3.java:24-57` 在 `n3$C_.m1398_(HttpUrl.Builder, Context)` 内给业务 URL 追加 `devuid/pdevuid/clienttype/channel/version/logid/vip/lang/firstlaunchtime/time/cuid/c3_aid/c3_oaid` 后再签名；上游 `ex/m1.java:294` 是 **okhttp interceptor** | **不影响**：只作用于业务 API 网关（`ServerApiKt`/`ex.m1` 拦截器），**登录走 sapi2 的 `HttpClientWrap`，不经过该拦截器** |
| `libnetdisk-signature-check.so` | 18264 B | `classes12/sources/com/baidu/netdisk/sign/SignatureUtil.java:5-11`（`loadLibrary("netdisk-signature-check")` + `native boolean checkSignature(Context)` + `native String getSignature(Object)`） | 唯一业务调用 `classes12/sources/com/baidu/netdisk/sign/CertVerifier.java:38` `if (SignatureUtil.checkSignature(context))` | **无** | `CertVerifier` 的调用方仅 `classes12/sources/ok0/C_.java:77`、`classes2/.../FHFeedFragment.java:954`、`classes2/.../newfeedhome/FHFeedFragment.java:1304` | **不影响**：APK 签名校验 / 信息流页，与应用身份有关，与登录协议无关 |
| `libnetdisk-security-rand.so` | 2884608 B | `classes14/sources/com/mars/united/netdisk/middle/platform/security/RandAlgorithm.java:4-9`（类为 package-private；`native String getRand(Context, String cuid, String version, String time, String skEncode, String bduss, String uid)`） | 唯一调用 `security/RandAlgorithmKtKt.java:16-43` → `addRandParameters`（`CommonParametersInterceptorKt.java:69-84`，uid 非空且 sk 非空时加 `time` + `rand`） | **无** | 拦截器注册在 `classes14/sources/com/mars/united/netdisk/middle/platform/network/ServerApiKt.java:23` | **不影响**：业务 API 网关的 `rand` 参数，登录不走该拦截器 |
| `libsofiresec.so` | 1235496 B | **Java 层无 `System.loadLibrary("sofiresec")`**（`grep` 0 命中）；由 `classes12/sources/com/baidu/sofire/x0/jni/ProtectSoUtil.java:19-40` 运行时**动态下载后 `System.load(path)`**；`Engine.java:141-170` `loadLib(int)` 分流 `loadLibrary`/`System.load` | `SapiAccountManager.java:159-163` `initSofireSDK` → `SafeService.init` → `FH.init` → `d/b.java` → `Engine/EngineProxy` | **间接且不阻断** | `libfire.so`（84416 B）由 `classes12/sources/com/baidu/sofire/d/b.java:78`、`:92` `loadLibrary("fire")` | **不影响**：只服务 `di` 索引 34；有纯 Java 兜底（§4.5） |
| `libmsaoaidsec.so` + `libmsaoaidauth.so` | 622592 / 376832 B | `classes3/sources/com/baidu/helios/ids/oid/a.java:34` | MSA OAID 获取（`com.bun.miitmdid`） | **无** | 与 `di` 的 OAID 相关字段无关（40 字段里无 oaid 位；`c3_oaid` 只出现在业务拦截器 `ex/n3.java`） | **不影响** |
| `libjni-kernel.so` | 10017264 B | `classes3/sources/com/baidu/library/P2P.java:170/:191/:203/:213/:239` | P2P / 下载加速 | **无** | — | **不影响** |
| `libjni-kservice.so` | 34179392 B | `classes2/sources/lp/___.java:12-24`（拼动态插件目录 `p2psdk/jni-kservice_<version>`，实现 `PluginExecutor.SubExecutor`） | 动态插件（下载加速） | **无** | — | **不影响** |
| `libbaiduprotect_sec.so` | 133168 B | 未在本次范围内做调用点定位（不在任务要求的库清单的登录关联判定内） | — | ~~**待确认**~~ **已闭环（§15.1）** | — | 标 **待确认**：本次未追其 Java 侧调用链，不作为登录阻断证据，也不排除。<br>**2026-11-28 补充收口**：已完整追踪，结论为仅服务人脸 WBAES、不参与请求构造、不阻断 A/B/C（见 §15.1.6）；同族 `libbaiduprotect_sec_jni_facect.so` 一并核对，APK 内不存在任何 `libbaidusec*` 文件 |

### 5.1 一个必须记录的静态盲区

`libsofiresec.so` 的加载是**运行时动态下载**（`ProtectSoUtil.java:19-40`：`needDownLoadSo()` 为真则从下载目录 `System.load(path)`）→ 该库**不在 APK 里**，因此「APK 内 `lib/` 没有它」不能推出「运行时没有它」。这与本任务结论无冲突（sofire 只影响 `di` 索引 34），但必须写清楚，避免后续误读。

### 5.2 与 T-002c 的差异

T-002c 只记「其他 native 安全组件待确认」；本次把 6 个库的**调用入口、唯一调用点、上游注册点**全部落到行号，判定「三条登录路径均不经过」。
---

## 6. 账号密码登录路径（A）

### 6.1 完整调用链（已验证，8 跳）

| 跳 | 文件:行号 | 内容 |
|---|---|---|
| 1 | `classes2/sources/com/baidu/sapi2/SapiAccountRepository.java:484` | `accountAndPwdLogin(SapiCallback, String username, String password, String checkcode, String encryptedId, HashMap)` |
| 2 | 同文件 `:489-503` | 参数空 / 无网早退 |
| 3 | 同文件 `:504` | `new HttpHashMapWrap()`（→ 构造器 `putAll(Utils.buildCommonParams())`） |
| 4 | 同文件 `:519-521` | `username = pwdRSAEncryptor.encryptString(true, username)`；`password = encryptString(false, password + checkcode)` |
| 5 | 同文件 `:534` | `wapLogin(2, true, ...)` |
| 6 | 同文件 `:2873-2901` | 组装 13 个 body 字段（下表），最后一次 `post(...)` |
| 7 | 同文件 `:1126-1128` | URL = `environment.getWap() + "/wp/api/login"`（即 `http://wappass.baidu.com/wp/api/login`） |
| 8 | `classes4/.../httpwrap/HttpClientWrap.java:80-100` | `buildParamDTO` → `doSign(appSignKey)` 生成 `sig` → 发出 |

### 6.2 请求字段（已验证，全部落行号）

| 字段 | 值来源 | 行号 |
|---|---|---|
| `username` | RSA(v3) 加密后的用户名 | `:519-520` |
| `password` | RSA(v3) 加密后的「密码+验证码」 | `:521` |
| `alg` | `"v3"` | `:2875` |
| `adapter` | `"3"` | `:2876` |
| `cv` | `"170601"` | `:2877` |
| `isEncrypted` | `"1"`（常量 `PwdRSAEncryptor.REQUEST_KEY_IS_ENCRYPTED`） | `:2878` |
| `lang` | 按系统语言二选一 | `:2879-2883` |
| `liveAbility` / `scanface` / `supFaceLogin` | 仅 `supportFaceLogin` 为真时置 `"1"` | `:2884-2888` |
| `support_photo` | 仅 `supportPhoto` 为真时置 `"1"` | `:2889-2891` |
| `logLoginType` | `"sdk_login"` | `:2892` |
| `time` | 秒级时间戳 | `:2893` |
| `t` | 毫秒级时间戳 | `:2894` |
| `session_id` | `md5(UUID.randomUUID().toString() + currentTimeMillis())` | `:2895` |
| `nalogin` | `"1"`（条件分支内） | `:2896-2898` |
| `needNaNextUrl` | `"1"` | `:2899` |
| `enableExternalWeb` | `"0"` | `:2900` |
| `sig` | `calculateSig(公共参数 + body, appSignKey)` | `HttpHashMapWrap.java:9-15` / `HttpClientWrap.java:87` |
| Cookie + UA | `ParamsUtil.buildNaCookie(wapUrl, configuration)` + `getUaInfo()` | `:2901` |

### 6.3 RSA 实现（已验证，纯 Java）

`classes2/sources/com/baidu/sapi2/PwdRSAEncryptor.java`（69 行）：

- `:7` 指数 `"10001"`；`:8` 1024 bit 的十六进制模数常量（硬编码）；`:9-11` 三个 `BigInteger` 由指数/模数构造。
- `:19-21` 私有 `a(String)` = `Base64(str, 2)`。
- `:23-25` `encryptUsername`；`:27-60` `encryptString(boolean z, String str)`：`z` 为真先 Base64 → 按模数字节长度分组 → `BigInteger.modPow(...)` → `toString(16).toUpperCase()`，奇数长度前补 `"0"`。
- **无 padding、无 native、无 BouncyCastle 依赖**（纯 `java.math.BigInteger`）。

### 6.4 响应与落库（已验证）

| 步 | 文件:行号 | 内容 |
|---|---|---|
| 1 | `SapiAccountRepository.java:2934-2947` | 取 `errInfo`；`no != 0 && no != 400032` 判失败；`data` 非空则转交 |
| 2 | `classes2/.../BaseWapLoginResult.java:23-32` | `handleAccountAndPwdLoginResult` 分派 |
| 3 | `classes2/.../SapiAccountService.java:542` / `:570` | `handleAccountParseResult(cb, data.optString("xml"), PWD)` |
| 4 | `classes2/.../SapiCoreUtil.java:68-77` | `parseAccountXmlToResponse`：正则 `"<client>([\\S\\s]*?)</client>"` 取 XML 片段 → `XmlPullParser` 解析出 `bduss` / `ptoken` / `username` / `errmsg` / `incomplete_user` / `actiontype` |
| 5 | `classes2/.../AbstractService.java:47-57` | `parseAccount`：`uid` / `bduss` / `displayname` / `username` / `stoken` / `ptoken` / `extra` / `app` |
| 6 | `classes2/.../SapiAccountManager.java:798-812` | 先 `getDeleteAppStatus(bduss, cb)`（`SapiAccountRepository.java:1342-1348`，URL `:1397-1399` `/v3/api/user/getdeleteappstatus`，超时 2000ms，`code!=110000` 直接放行），`onNonBlocking` 才落库 |
| 7 | `SapiAccountManager.java:823-836` | `validate`：`ShareAccountAccessor.updatePtoken` → `SapiContext.setCurrentAccount` → `addLoginAccount` → `putEncryptStr(KEY_LAST_LOGIN_UID)` → `ShareStorage.asyncSet(2)` → `onValidateSuccess` |
| 8 | `classes2/.../SapiContext.java:978-1016` | 持久化 = `SapiDataEncryptor.encryptAccountInfo` 加密 JSON 存 SharedPreferences + `SapiUtils.webLogin` 写 `BDUSS`/`PTOKEN` 到 CookieManager |

**响应解析是纯 XML（`XmlPullParser`），不是 WebView；落库是本地 AES + CookieManager，不是网页会话复用。**

### 6.5 本路径的阻断依赖逐项检查

| 依赖 | 是否必需 | 静态可否复刻 | 证据 |
|---|---|---|---|
| RSA 1024 加密 | 是 | **可**（`BigInteger.modPow`，纯 Java） | `PwdRSAEncryptor.java:27-60` |
| `tpl` / `appId` / `appSignKey` | 是 | **可**（APK 常量） | §2 |
| `sig` 签名 | 是 | **可**（MD5 + 排序） | `SapiUtils.java:135-160` |
| `di`（经 `DVIF` cookie） | 是 | **可**（§3） | `ParamsUtil.java:75-79` |
| `BAIDUID` cookie | 是（`buildNaCookie` 必加） | **可**（先 GET `https://baidu.com` 取 `BAIDUID`，或接受空串） | `ParamsUtil.java:81`、`SapiUtils.java:284-300` |
| `passport_core` | **否** | 不需要 | §4 |
| 其他 native 安全库 | **否** | 不需要 | §5 |
| WebView / OAuth / cookie 会话复用 | **否** | 不需要 | §6.4 响应是 XML；无 `WebView.loadUrl` 调用点 |
| 服务端风控 | **未知** | 静态不可判定 | 见 §12.4 |

### 6.6 结论

**A：可独立实现（静态）。** 未发现任何未还原的阻断依赖。

---

## 7. 短信验证码登录路径（B）

### 7.1 完整调用链（已验证，7 跳）

| 跳 | 文件:行号 | 内容 |
|---|---|---|
| 1 | `classes2/sources/com/baidu/sapi2/SapiAccountRepository.java:861-893` | `dynamicPwdLogin`：先发 `GET getLastCert()` 拿证书 |
| 2 | 同文件 `:104-106` | `getLastCert()` = 常量路径 `/sslcrypt/get_last_cert` |
| 3 | 同文件 `:263-298` | `realDynamicPwdLogin(cb, certContent, certId, username, pwd, z, sapiDataEncryptor)` |
| 4 | 同文件 `:267` | `SapiContext.getDeviceInfo("/v2/sapi/login")` → 加密 `di` |
| 5 | 同文件 `:268-270` | 非空则 `put("di", deviceInfo)` |
| 6 | 同文件 `:273-282` | 组 `userinfo` JSON 并加密 |
| 7 | 同文件 `:283` | `post("/v2/sapi/login")` |

### 7.2 请求字段（已验证）

| 字段 | 值 | 行号 |
|---|---|---|
| `crypttype` | `"6"` | `:265` |
| `di` | 加密后的 40 字段指纹（§3） | `:267-270` |
| `cert_id` | 服务端下发证书的 id | `:271` |
| `isdpass` | `"1"` | `:272` |
| `userinfo` | `encrypt(certContent, JSON)` | `:281-282` |

`userinfo` 内部 JSON（`:273-281`）：`username`、`isphone="1"`、`password`、`login_type="3"`、`key=AESKey`、`sdk_version="2"`、`pinfo=URLEncoder(Build.BRAND)`。

### 7.3 加密实现（已验证，纯 JCA）

`classes12/sources/com/baidu/sapi2/utils/SapiDataEncryptor.java`（90 行）：

- `:5` `mAESKey = createAESKey()`；`:12-19` 随机 16 位 `[a-z0-9]`。
- `:7-10` 内嵌 `Cert1.CERT`（**VeriSign 签发、`CN=openapi.baidu.com` 的 X509 明文**）+ `CERT_ID = 1`。
- `:33-37` `RSA/NONE/PKCS1Padding`；`:51-53` 解密；`:55-57` `getAESKey()`。
- `:59-89` `encrypt(String cert, String plain)`：从证书取公钥 → 按 `UploadContract.UploadTasks.ki = 116`（`UploadContract.java:39`）**分段 RSA** → `Base64(JSONArray)` → 整体再 Base64。
- **全部是 `java.security` / `javax.crypto` 标准算法，无 native。**

### 7.4 本路径的阻断依赖逐项检查

| 依赖 | 是否必需 | 静态可否复刻 | 证据 |
|---|---|---|---|
| `di` 完整 40 字段 | 是 | **可**（§3） | `:267-270` |
| 服务端下发证书（`cert` + `cert_id`） | 是 | **可**（多一次 GET `/sslcrypt/get_last_cert`；也可用内嵌 `Cert1.CERT` + `CERT_ID=1` 起步） | `:861-893`、`SapiDataEncryptor.java:7-10` |
| AES 随机 key | 是 | **可** | `SapiDataEncryptor.java:12-19` |
| RSA 分段加密 | 是 | **可**（`RSA/NONE/PKCS1Padding`，Java 标准 Provider 支持 `NONE`） | `:33-37`、`:59-89` |
| `sig` 签名 | 是（走 `HttpClientWrap`） | **可** | `HttpClientWrap.java:87` |
| `passport_core` / 其他 native | **否** | 不需要 | §4、§5 |
| WebView / OAuth / cookie | **否** | 不需要 | 无调用点 |
| 服务端风控 | **未知** | 静态不可判定 | 见 §12.4 |

### 7.5 结论

**B：可独立实现（静态）。** 唯一新增动作是多一次 `GET /sslcrypt/get_last_cert`（无需登录态）。

---

## 8. 扫码登录路径（C）

### 8.1 完整调用链（已验证）

```
① GET  /v2/api/getqrcode        ← 取二维码图片 + sign(=channelId)
② GET  /channel/unicast         ← 长轮询（超时 40s，JSONP 包裹）
③ GET  /v2/api/bdusslogin       ← status==0 时用 channel_v.v 换取
   GET  /v3/login/main/qrbdusslogin   ← 备选（isJoinCodeLogin 分支）
④ POST /v2/sapi/qrlogin?lp=app  ← App 内确认（被扫端，本项目不需要）
```

| 步 | 文件:行号 | 内容 |
|---|---|---|
| ① | `classes2/.../QrCodeService.java:306-327` | `getQrCodeImage`：`lp` 默认 `"pc"`、`apiver="v3"`、`tt`=ms、可选 `openPlatformId`/`encuid`、`needQrCodeContent`、`hostDeviceId`；**cookie 传 `null`** |
| ①回调 | `:9-61` 内部类 `a` | 读 `errno`、`imgurl`（拼 `https://`）、`prompt`、**`:49` `channelId = optString("sign")`**、`qrcontent_url` |
| ② | `:387-398` | `qrLoginStatusCheck`：`apiver="v3"`、`callback="cb"`、`channel_id`、`tt`；超时 **40000ms** |
| ②回调 | `:114-193` 内部类 `c` | `:146` `str.substring(3, str.length()-2)` 剥 JSONP（`cb(...)`）；`:152` `errno==1` 继续轮询；`:165-168` `status==1`（已扫码待确认）→ 继续；`:170-177` `status==0` → 取 `channel_v.v` 调 `getQrLoginResult`；`:183` `status==2` → 用户取消 |
| ③ | `:354-365` | `getQrLoginResult`：`display="pcsdk"`、`qrcode="1"`、`bduss`（无则空）、`tt`、`hostDeviceId`；`z` 真→`/v3/login/main/qrbdusslogin`，否则 `/v2/api/bdusslogin` |
| ③回调 | `:195-242` 内部类 `d` | `:223` 读 `errno`；`:227` **`errno != 0 && errno != 110000` 才判失败**；`:231` `isJoinCodeLogin ? data : res`；`:232` `parseAccount`；`:233` `DispersionCertification.fromJSONObject(...).tplStokenMap`；`:234` `setPreLoginType(QR)`；`:235` `validate(account)` |
| ④ | `:367-385` | `qrAppLogin`：**被扫端（已登录设备）才调用**，本项目不需要 |

### 8.2 字段来源（已验证）

| 字段 | 来源 | 行号 |
|---|---|---|
| `sign` / `channel_id` | ①响应的 `sign` 字段 | `:49` |
| `cmd` | 被扫端二维码 URL 里的 `cmd`（本项目作为扫码端**不需要**） | `:369`、`:374` |
| `hostDeviceId` | 客户端生成，`getQrCodeImage` 与 `getQrLoginResult` 用同一个值 | `:325`、`:360` |
| `bduss` | 若当前已有登录账号才带（**非必需**） | `:375-380`（只在 `qrAppLogin`）、`:358`（`getQrLoginResult` 允许空） |
| `stoken` / `ptoken` | 同上，仅 `qrAppLogin` 且已有账号时 | `:375-380` |
| `bduss`/`ptoken`/`stoken` 的**获得** | ③响应 JSON → `AbstractService.parseAccount`（`AbstractService.java:47-57`） | — |

### 8.3 与 A/B 的关键差异

| 项 | A / B | C |
|---|---|---|
| 是否发 `di` | 是 | **否**（三个请求都不带 `di`，也不带 `HttpHashMapWrap`，见下） |
| 是否带 `sig` | 是 | **否**（`QrCodeService` 的 `get`/`post` 不传 `HttpHashMapWrap`） |
| 是否依赖既有登录态 | 否 | **否**（`getQrCodeLoginWithEnuidImage` 才需要，本项目不用它） |
| 是否需要 WebView/OAuth/cookie | 否 | **否** |

> 关于「是否带 `sig`」：`QrCodeService` 内部通过 `ISAccountService`/`HttpClientWrap` 发请求，但调用时未传 `HttpHashMapWrap` 实例。按 `HttpClientWrap.java:86-88` 的 `if (httpHashMap != null)` 条件，**不传就不加签**。可信度：**已验证**（`QrCodeService.java` 全文无 `new HttpHashMapWrap`，`grep -n 'HttpHashMapWrap' QrCodeService.java` → 0 命中）。

### 8.4 `di` 在扫码路径中的缺席（重要）

`grep -n 'getDeviceInfo\|put("di"' QrCodeService.java` → **0 命中**。扫码路径的三个请求（`/v2/api/getqrcode`、`/channel/unicast`、`/v2/api/bdusslogin`）都不构造 `di`。

**含义（对复刻有利）**：扫码路径是三条里**静态依赖最少**的一条——不需要 `di` 40 字段，不需要 `sig`，只需要常量域名 + `tt` + `hostDeviceId` + JSON/JSONP 解析。

### 8.5 本路径的阻断依赖逐项检查

| 依赖 | 是否必需 | 静态可否复刻 | 证据 |
|---|---|---|---|
| `di` | **否** | — | §8.4 |
| `tpl`/`appSignKey` | **否** | — | 不加签 |
| `passport_core` / 其他 native | **否** | — | 无调用点 |
| WebView / OAuth / cookie 复用 | **否** | — | 无调用点 |
| JSONP 解析（`substring(3, len-2)`） | 是 | **可** | `:146` |
| 40s 长轮询 | 是 | **可** | `:397` |
| 服务端风控 | **未知** | 静态不可判定 | 见 §12.4 |

### 8.6 结论

**C：可独立实现（静态）。** 三条里依赖最少。

### 8.7 三条路径比较表

| 检查项 | 账号密码 A | 短信验证码 B | 扫码 C |
|---|---|---|---|
| 官方客户端协议存在 | 是 | 是 | 是 |
| 请求链路完整 | 是（8 跳） | 是（7 跳） | 是（4 步，含 1 步可跳过） |
| 参数来源明确 | 是 | 是 | 是 |
| `tpl` / `appSignKey` 依赖 | **是** | **是** | **否** |
| `di` 设备指纹依赖 | **是**（`DVIF` cookie） | **是**（body `di`） | **否** |
| `passport_core` 依赖 | 否 | 否 | 否 |
| 其他 native 依赖 | 否 | 否 | 否 |
| WebView / OAuth / cookie 依赖 | 否 | 否 | 否 |
| Rust 可复刻部分 | RSA、MD5 签名、`di` 全部、Cookie 构造、XML 解析 | AES、RSA 分段、`di` 全部、JSON 构造 | JSONP 解析、长轮询、JSON 解析 |
| 无法复刻部分 | **无**（静态） | **无**（静态） | **无**（静态） |
| **单路径结论** | **可独立实现** | **可独立实现** | **可独立实现** |
---

## 9. Rust 独立复刻能力清单

### 9.1 五种情形的划分标准

| 情形 | 含义 |
|---|---|
| **①纯算法可实现** | 算法本身是公开标准，输入输出完整，Rust 侧可 1:1 实现，不需要任何外部秘密 |
| **②算法可实现但缺参数** | 算法简单，但所需密钥/常量在静态资料里取不到 |
| **③需要 Kotlin 提供通用系统信息** | 依赖 Android 系统 API 或系统文件，属「通用设备信息采集」，不是网盘特有逻辑 |
| **④依赖闭源 native** | 必须调用官方 so 才能得到结果 |
| **⑤静态证据不足** | 本次无法判断 |

> 架构边界不变：Kotlin 只负责 UI 与**通用**系统信息采集（经 UniFFI 传入），Rust 负责协议/加密/签名/请求构造，**网盘特有参数不进 Kotlin**。第 ③ 类里传入的都是跨网盘通用的字段（包名、版本号、`Build.MODEL`、内存、CPU、网络类型、开机时长），不含任何百度专有语义。

### 9.2 清单

| 组成部分 | 当前实现位置 | 实现依赖 | Rust 独立实现可能性 | 阻断条件 | 证据 |
|---|---|---|---|---|---|
| **RSA 加密（A 路径）** | `classes2/.../PwdRSAEncryptor.java:27-60` | `java.math.BigInteger.modPow` + 硬编码 1024bit 模数 + 指数 `10001` | **①可**（`num-bigint` 等纯 Rust 大数即可；模数/指数是 APK 常量，可用） | 无 | §6.3 |
| **AES 加密（`di` 编码 / B 路径 userinfo）** | `classes12/.../SapiDeviceInfo.java:193-203`；`classes15/sources/h81/_____.java:10-14`；`SapiDataEncryptor.java:59-89` | AES/CBC/NoPadding + 硬编码 16 字符 key（`di`）；随机 key（B 路径） | **①可**（`aes`/`cbc` crate） | 无（key 是 APK 常量，可用） | §3.6、§7.3 |
| **MD5 / 摘要** | `classes15/sources/h81/_____.java`（`_____`）、`SapiUtils.java:159` | MD5 hex 小写 | **①可**（`md-5`/`md5` crate） | 无 | §2.7 |
| **`session_id`** | `SapiAccountRepository.java:2895` | `md5(UUID.randomUUID() + currentTimeMillis())` | **①可**（UUID v4 + 毫秒 + MD5） | 无 | §6.2 |
| **`tpl` / `appId` / `appSignKey`** | `kd/______.java:11/:14/:15/:17` → `AccountUtils` → `SapiConfiguration` | APK 硬编码常量 | **①可**（内置常量） | **无**（T-002c 曾判「静态不可得」，已被推翻，见 §2） | §2 |
| **设备指纹 `di`（40 字段）** | `SapiDeviceInfo.java:67-187` | 22 必需字段中：3 项常量、2 项可能空、1 项兜底值、其余 16 项为本机系统信息 | **①+③可**（编码/加密/拼接全部纯算法；取值需 Kotlin 传 8 个通用系统信息） | 无阻断；需 Kotlin 提供最小输入集（见 §9.3） | §3 |
| **`passport_core`** | `PassCoreVHelper.java:9-19` | 6 个无参 native | **①不需要**（三条主链均不调用） | 无 | §4 |
| **其他安全 SDK** | 见 §5 表 | 业务 API 网关 / APK 签名校验 / P2P / MSA OAID | **①不需要** | 无 | §5 |
| **token 换取与持久化** | `AbstractService.java:47-57`、`SapiAccountManager.java:823-836`、`SapiContext.java:978-1016` | 本地 AES 加密 JSON + CookieManager | **①+③可**（持久化由 Rust 自己做；写 WebView Cookie 那一步本项目不需要） | 无 | §6.4 |
| **扫码轮询与会话换取** | `QrCodeService.java:306-398` | HTTP GET + JSONP 剥离 + 40s 长轮询 + JSON 解析 | **①可** | 无 | §8 |

### 9.3 `di` 需要 Kotlin 提供的最小通用输入集（只是边界描述，本任务不设计接口）

下表是**唯一**需要跨 UniFFI 边界的字段。全部是跨网盘通用的 Android 系统信息，**不含任何百度专有语义**。

| 输入 | 对应 `di` 索引 | 获取方式（Kotlin 侧） | 是否百度特有 |
|---|---|---|---|
| 宿主包名 | 0 | `context.packageName` | 否 |
| 宿主 versionName | 1、2 | `PackageManager` | 否 |
| `Build.MODEL` | 3、27 | `android.os.Build` | 否 |
| `Build.VERSION.RELEASE` | 4 | 同上 | 否 |
| cuid（设备标识） | 6、38 | `Settings.Secure.ANDROID_ID` 派生或自生成 UUID 持久化 | 否 |
| 网络类型 | 13 | `ConnectivityManager` | 否 |
| CPU 名称 | 19 | 读 `/proc/cpuinfo` | 否 |
| RAM / 内部存储（总/可用） | 20、21、22 | `/proc/meminfo` + `StatFs` | 否 |
| 开机时长 | 23 | `SystemClock.elapsedRealtime()` | 否 |
| 已装应用（`SHARE_SERVICE`） | 25 | `queryIntentServices` | 否（Intent action 常量由 Rust 侧下发或硬编码在 Rust Adapter 内） |

> 其余 14 个「常量或空串」位（索引 5/7/8/9/14/15/16/17/18/24/26/28/29…36/37/39）**完全在 Rust 侧生成**，不跨边界。

### 9.4 必须区分清楚的两件事

- **「Rust 能写 RSA」≠「百度登录能独立实现」。** 本次的证据链是：算法可写（§9.2）+ 参数可得（§2、§3）+ 无 native 依赖（§4、§5）+ 无 WebView/OAuth 依赖（§6.4、§8）**四条同时成立**，才得出「可独立实现」。
- **不把设备信息全交给 Kotlin。** 上表只列 10 个通用输入，`di` 的 40 个位置里有 30 个由 Rust 侧填充常量/空串/算法生成。

---

## 10. native 库移植必要性分析

### 10.1 三条路径分别结论

| 路径 | 不移植 native 能否实现 | 仅移植某个特定库能否解决 | 是否依赖宿主包名/签名/注入环境 | 结论 |
|---|---|---|---|---|
| 账号密码 A | **能** | 不需要 | 否（只有 `di` 的 cuid 与包名，属通用信息） | **不需要移植** |
| 短信 B | **能** | 不需要 | 否 | **不需要移植** |
| 扫码 C | **能** | 不需要 | 否 | **不需要移植** |

### 10.2 逐库必要性

| 库 | 是否必须 | 若移植的不可替代功能 | 依赖链 | ABI / 兼容风险 | 对架构的影响 |
|---|---|---|---|---|---|
| `libpassport_core.so` | **否** | 无（三条主链 0 调用；缺失有 `catch (Throwable)` 兜底） | 自包含（`NEEDED` 仅系统库） | 仅 arm64-v8a | 无 |
| `libfire.so` / `libsofiresec.so` | **否** | 只影响 `di` 索引 34；有纯 Java 兜底链 | `libsofiresec` 运行时动态下载 | 仅 arm64-v8a | 无 |
| `libnetdisk-security.so` | **否** | 业务 API URL 加签（本项目登录不经过） | — | 仅 arm64-v8a | 无 |
| `libnetdisk-security-rand.so` | **否** | 业务 API `rand` 参数（登录不经过） | — | 仅 arm64-v8a | 无 |
| `libnetdisk-signature-check.so` | **否** | APK 签名校验（与登录无关） | — | 仅 arm64-v8a | 无 |
| `libmsaoaidsec.so` 等 | **否** | MSA OAID（`di` 无该位） | — | — | 无 |
| `libjni-kernel.so` / `libjni-kservice.so` | **否** | P2P/下载加速（后续功能可能相关，但不在登录链路上） | — | — | 无 |

### 10.3 是否破坏「新增网盘只增加 Rust Adapter」

**不破坏。** 因为不需要集成任何官方闭源 so；百度的 Adapter 完全是 HTTP + 密码学 + 常量，与「坚果云 Adapter」在同一种形态里。

### 10.4 是否产生 ADR

**不需要新增 ADR。** 依据：T-006 的触发条件是「需要引入新依赖、闭源 native 库或改变架构」，本次三项均不成立：

- 新增依赖：无（Rust 侧若未来实现，所需的 AES/MD5/大数均为常规 crate，属 Provider 实现阶段选型，不在本任务范围内定案）。
- 闭源 native：不引入。
- 架构变更：无（Kotlin 仍只做 UI + 通用系统信息，UniFFI 仍是唯一边界）。

> 说明：Rust 具体 crate 选型（大数/AES/MD5）留到 Provider 实现任务，届时如引入新 crate 需按 AGENTS.md 第 3 条走 ADR。

---

## 11. 三条路径独立结论（汇总）

| 路径 | 结论（四选一） | 关键证据 | 仍需动态验证的事 |
|---|---|---|---|
| A 账号密码 | **可独立实现** | 8 跳链路齐全；RSA/sig/`di` 全在 Java 侧；响应 XML 解析；无 native / 无 WebView | 服务端是否接受本项目构造的 `sig` + `DVIF`；是否有额外风控挑战 |
| B 短信验证码 | **可独立实现** | 7 跳链路齐全；`di` 直接在 body；AES+RSA 全标准算法；证书可从 `/sslcrypt/get_last_cert` 取 | 服务端对 `di` 的校验强度；短信发送接口本身的频控/风控 |
| C 扫码 | **可独立实现** | 4 步链路；**不需要 `di`、不需要 `sig`**；JSONP 解析 + 40s 长轮询；不依赖既有会话 | 二维码有效期、`channel_v.v` 的换取时机是否严格 |

---

## 12. 百度整体可行性结论

### 12.1 四档结论

**原生登录可行（静态可行，动态未验证）。**

### 12.2 逐条回答任务要求说明的七项

| # | 要求 | 回答 |
|---|---|---|
| 1 | 哪条路径能够独立实现 | **三条都能**（A / B / C） |
| 2 | 哪些路径依赖 native | **核心认证算法：没有**（RSA / `sig` / `di` 加密全在 Java 侧）。**公共参数 `zid`：有**——真实值依赖 `libfire`（`Asc.ar/dc/ac`）与服务端下发数据，无 native 时只能取 62 位常量或空串（§17） |
| 3 | 是否存在至少一种符合项目要求的原生登录方式 | **存在**：本项目 Rust Adapter 直接走官方 Android 客户端协议、自行收集输入、自行换 token，符合 AGENTS.md 第 9 条「登录必须 APP 内原生」 |
| 4 | 是否仍依赖未解决的设备指纹/风控/运行时注入 | **设备指纹已解**（§3，40 字段全部定性，加密算法全在 Java 侧）；**运行时注入不存在**（§2.5，`tpl`/`appSignKey` 是常量）；**服务端风控仍未知**（不可静态证明） |
| 5 | 是否需要 ADR | **不需要**（§10.4） |
| 6 | 相较 T-002c 是否变化 | **变化**：`需攻关` → `原生登录可行`。三处具体修正见 12.3 |
| 7 | 哪些结论仍只是静态证据 | 见 12.4 |

### 12.3 相较 T-002c 的三处修正

| 项 | T-002c 记录 | T-006 实测 | 修正依据 |
|---|---|---|---|
| 1 | `tpl` / `appSignKey` 「运行期注入，静态未定位到常量值」 | **APK 硬编码明文常量** | `kd/______.java:11/:14/:15/:17`（§2） |
| 2 | `passport_core` 「待确认是否影响登录」 | **三条主链 0 调用**，缺失有 `catch (Throwable)` 兜底，且无 `RegisterNatives` | §4 |
| 3 | `di` 40 字段「受灰度裁剪，未分类」 | **40 字段逐项分类：必需 22 / 条件 2 / 可选 16**；且 `mPrivacyParamesRegulation=false` 使 `dt_params` 实际不生效 | §3.2、§3.4 |

> 另需登记：`docs/evidence/mvp-go-no-go.md` 第 239 行「`tpl` / `appSignKey` 由运行时 `setProductLineInfo` 注入…静态不可得」这一表述**已被本次实测推翻**。该文件属 T-005 历史产物，**本文不改写它**，只在此登记，供用户决定是否在重跑 Go/No-Go 时一并修订。

### 12.4 仍然只是静态证据、必须动态验证的事项

| # | 事项 | 为什么静态不可得 | 影响 |
|---|---|---|---|
| 1 | 服务端是否接受本项目构造的 `sig` | `sig` 由客户端算法 + 常量生成，服务端校验逻辑在服务端 | 若被拒，A/B 路径需调整参数集 |
| 2 | 服务端对 `di` 的校验强度 | 同上；`di` 是加密串，服务端如何解密/比对未知 | 若严校验，需精确保真 `di` 形状（本报告已给出全部 40 位置） |
| 3 | `diExceptIndex` / `sapi_options` 的实际下发内容 | 服务端下发 + 本地缓存，默认空 | 若线上把关键字段 join 排除项，`di` 形状会变；本项目复刻时按默认空处理 |
| 4 | 账号密码路径是否需要 `BAIDUID` 有效值 | `buildNaCookie` 必加 `BAIDUID` cookie，但允许空串 | 若服务端要求非空，需先 GET `https://baidu.com` 拿一次 |
| 5 | 短信路径的 `/sslcrypt/get_last_cert` 返回是否稳定 | 未发请求 | 若证书轮换，需每次动态获取（代码路径已支持） |
| 6 | 扫码路径二维码有效期与轮询节奏 | `tt` 字段语义与 40s 超时的服务端配合未知 | 影响 UI 交互设计，不影响可行性 |
| 7 | 是否有额外风控（验证码挑战、设备信任、异地登录） | 纯服务端行为 | 可能要求人工介入；属产品可用性问题而非协议不可实现 |

### 12.5 对 T-005 的影响（只陈述，不改决策）

- 若百度按本报告判为「原生登录可行」，则**硬准入可行集合**从 1（坚果云）变为 2（坚果云 + 百度），**满足 T-005 卡里「Go = 可行集合 ≥ 2」的判据**。
- **但**：①T-005 的历史 No-Go 结论不改写；②是否重跑 Go/No-Go 由用户决定；③即使 Go，也不等于用户已选定 MVP 组合（T-005 卡里的排序维度仍需重新走一遍）。
- 另需注意：坚果云的登录路径依赖腾讯闭源 Web 服务做人机验证（`docs/RISKS.md` R27），与百度的「无 native 依赖」不同质，二者能否同时满足「原生登录可行」的同一把尺子，应在重跑 Go/No-Go 时明确口径。
---

## 13. 未解决问题

| # | 未解决问题 | 类型 | 卡在哪 | 需要什么才能推进 |
|---|---|---|---|---|
| 1 | 服务端是否接受本项目构造的 `sig` / `di` | 动态验证 | 静态不可得（服务端逻辑） | 用测试账号发 1 次请求看返回码；**本任务禁止发请求**，故未做 |
| 2 | `diExceptIndex` 线上实际下发值 | 动态验证 | 服务端下发 + 本地缓存 | 抓一次 `sapi_options` 下发响应（**本任务禁止抓包**）；或按默认空串先实现 |
| 3 | `BAIDUID` 是否必须有效 | 动态验证 | `buildNaCookie` 允许空串，但服务端是否接受未知 | 同上 |
| 4 | `AccountUtils.O/P/Q`（sofire 配置）是否被其它类赋值 | 静态未闭环 | 本次只确认 `AccountUtils.java` 内无赋值点 | 全量 grep 反射写法或跨类 setter；**不构成登录阻断**，优先级低 |
| 5 | ~~`libbaiduprotect_sec.so` 的 Java 侧调用链~~ **已关闭（§15.1）** | — | 已于 2026-11-28 补充收口完成 | 结论：仅服务人脸 WBAES，不阻断 A/B/C |
| 6 | `libfire.so` / `libsofiresec.so` 内部算法 | 静态未做 | 超出本任务范围（只判关联，不逆向） | 仅当需要精确复刻 `sf_zid` 到「真实 zid」级别时才有必要；当前用兜底值即可 |
| 7 | ~~短信验证码的发送接口（获取验证码）~~ **已取证（§15.2.1）**，但完整短信登录仍开放 | 静态已到接口级；**阻断在 WebView 人机验证** | 发码 = `POST {wap}/wp/api/login/sms`；前置 `POST passport.baidu.com/cap/init` + `pass-machine.html` | 需动态验证人机验证触发条件；按 AGENTS.md 第 9 条本项目不得用 WebView 完成登录 → B 路径判「需攻关」 |
| 8 | ~~扫码路径的 `hostDeviceId` 生成规则~~ **已关闭且结论反转（§15.3）** | — | 2026-11-28 补充收口完成 | **官方客户端从不赋值，恒为 `null`，参数被丢弃**（全仓 `.hostDeviceId =` 赋值 0 命中）；本项目按官方缺省不传即可。原「任意稳定随机串推测可用」的推测**作废** |

### 13.1 一条必须说清的范围边界

**本任务没有验证「短信验证码怎么发出去」。** `realDynamicPwdLogin` 是「用户已经拿到验证码之后」的那一步。发码接口本次未定位、未追调用链。

> **2026-11-28 补充收口**：发码接口已定位 —— `POST {wap}/wp/api/login/sms`（`SapiAccountRepository.java:1632-1634`），前置 `POST https://passport.baidu.com/cap/init`；提交侧实测走 `smsWapLogin` → `POST {wap}/wp/api/login`（**不是** `/v2/sapi/login`）。完整短信登录仍**不能判可独立实现**，阻断点是 `pass-machine.html` 的 WebView 人机验证。详见 §15.2、§15.7.2。

---

## 14. 检索命令与复现方式

### 14.1 前置资料

反编译产物位于 `reverse/baidu/sources/`（**不进 Git**，`.gitignore:2` 忽略 `reverse/`）：

```
reverse/baidu/
├── input/base.apk                       251854857 B，sha256 3ad2da62…d260e4
├── input/split/split_config.arm64_v8a.apk   含 69 个 .so
├── dex/classes.dex … classes20.dex      20 个，合计 182513748 B
├── sources/classesN/sources/…           逐 dex 反编译，共 8006+ 个 .java
├── sources-recovered/                   25 个补出的合成类
├── coverage-check.py                    覆盖率脚本（class_defs 口径）
└── logs/                                反编译日志
```

### 14.2 关键检索命令（逐条可复现）

| 目的 | 命令 |
|---|---|
| `tpl`/`appSignKey` 常量位置 | `grep -n 'public static String' reverse/baidu/sources/classes15/sources/kd/______.java` |
| `setProductLineInfo` 全部调用方 | `grep -rn 'setProductLineInfo' reverse/baidu/sources --include=*.java` |
| 唯一 `doSign` 调用点 | `grep -rn '\.doSign(' reverse/baidu/sources --include=*.java` |
| 基类 `doSign` 是否空实现 | `baksmali d reverse/baidu/dex/classes15.dex -o /tmp/hm/out && grep -A4 'doSign' /tmp/hm/out/com/baidu/pass/http/HttpHashMap.smali` |
| 签名算法 | `sed -n '130,165p' reverse/baidu/sources/classes12/sources/com/baidu/sapi2/utils/SapiUtils.java` |
| `di` 40 字段名单 | `sed -n '10,65p' reverse/baidu/sources/classes12/sources/com/baidu/sapi2/utils/SapiDeviceInfo.java` |
| `di` 构造 | `sed -n '67,210p' reverse/baidu/sources/classes12/sources/com/baidu/sapi2/utils/SapiDeviceInfo.java` |
| `di` 灰度裁剪 | `sed -n '89,160p' reverse/baidu/sources/classes12/sources/com/baidu/sapi2/utils/ParamsUtil.java` |
| `passport_core` 调用点 | `grep -rn 'PassCoreVHelper' reverse/baidu/sources --include=*.java` |
| `passport_core` 导出符号 | `nm -D /tmp/pc/libpassport_core.so \| grep Java_` |
| 是否有动态注册 | `strings /tmp/pc/libpassport_core.so \| grep -c RegisterNatives`（结果 0） |
| 是否有 `JNI_OnLoad` | `nm -D /tmp/pc/libpassport_core.so \| grep -c JNI_OnLoad`（结果 0） |
| 依赖哪些库 | `readelf -dW /tmp/pc/libpassport_core.so \| grep NEEDED` |
| `getAesKey` 反汇编 | `objdump -d --start-address=0x70a4 --stop-address=0x71e8 /tmp/pc/libpassport_core.so` |
| 其他 6 个安全库的加载入口 | `grep -rn 'loadLibrary("netdisk-security\|loadLibrary("netdisk-signature-check\|loadLibrary("netdisk-security-rand\|loadLibrary("sofiresec\|loadLibrary("msaoaidsec\|loadLibrary("jni-kernel' reverse/baidu/sources --include=*.java` |
| 账号密码路径 | `sed -n '484,540p;2870,2950p' reverse/baidu/sources/classes2/sources/com/baidu/sapi2/SapiAccountRepository.java` |
| 短信路径 | `sed -n '260,300p' reverse/baidu/sources/classes2/sources/com/baidu/sapi2/SapiAccountRepository.java` |
| 扫码路径全文 | `sed -n '1,409p' reverse/baidu/sources/classes2/sources/com/baidu/sapi2/QrCodeService.java` |
| 响应 XML 解析 | `sed -n '60,80p' reverse/baidu/sources/classes2/sources/com/baidu/sapi2/SapiCoreUtil.java` |
| 落库 | `sed -n '790,840p' reverse/baidu/sources/classes2/sources/com/baidu/sapi2/SapiAccountManager.java` |
| Cookie 构造 | `sed -n '58,90p' reverse/baidu/sources/classes12/sources/com/baidu/sapi2/utils/ParamsUtil.java` |
| 域名常量 | `sed -n '10,35p' reverse/baidu/sources/classes12/sources/com/baidu/sapi2/utils/SapiHost.java` |
| 覆盖率复算 | `cd reverse/baidu && python3 coverage-check.py dex sources` |

### 14.3 本次用到的工具与版本

| 工具 | 版本 / 路径 | 用途 |
|---|---|---|
| jadx | 1.5.7（`reverse/_tools/jadx-1.5.7/`） | 逐 dex 反编译，命令带 `--log-level debug` |
| baksmali | 系统 `/usr/bin/baksmali` | 核对 `HttpHashMap.doSign` 是否空方法体 |
| nm / readelf / objdump | 系统 GNU binutils（aarch64） | 导出符号、`NEEDED`、`getAesKey` 反汇编 |
| aapt | `/opt/android-sdk/build-tools/*/aapt` | `dump permissions` / `dump badging` |
| python3 | 3.12.15 | dex `class_defs` 解析、覆盖率统计 |

### 14.4 三条可复现的断言（供后续复查）

1. **`HttpHashMap.doSign` 是空方法**：用 baksmali 反汇编后 `grep -A4 doSign HttpHashMap.smali` 应看到方法体只有 `return-void`。
2. **`libpassport_core.so` 只有 6 个 `Java_` 导出**：`nm -D | grep -c 'Java_'` 应为 6。
3. **`QrCodeService.java` 不含 `HttpHashMapWrap`**：`grep -c 'HttpHashMapWrap' QrCodeService.java` 应为 0。

---

## 15. T-006 补充收口（2026-11-28）

> 本节由「T-006 补充收口」任务追加。**§1–§13 的原文一律保留不改**，凡与本节冲突的表述，以本节为准，并在 §15.9 汇总登记。
> 本节只做静态取证：未登录、未发请求、未抓包、未使用任何真实凭据；`reverse/` 不进 Git。

### 15.0 为什么需要补这一节

T-006 主体取证完成后复核发现四处证据没有闭环，且有三条结论被过度概括：

| # | 缺口 | 本轮处理结果 |
|---|---|---|
| 1 | `libbaiduprotect_sec.so` 只标「待确认」，未追加载与调用链 | **已闭环**（§15.1），结论：仅服务人脸 WBAES，不阻断 A/B/C |
| 2 | 短信只取了「拿到验证码之后」的提交段，发码接口未取证 | **发码已取证到接口级**（§15.2），但完整短信登录**仍不能判可独立实现** |
| 3 | 扫码 `hostDeviceId` 只记「客户端生成」，未定位生成规则 | **已闭环且结论反转**：官方客户端**从不赋值**，恒为 `null`（§15.3） |
| 4 | `cuid` 只写「ANDROID_ID 派生或自生成 UUID」 | **已定位官方算法**（§15.4），但派生字段 `i()` 涉及反射 PRNG，**当前无法精确复刻** |
| 5 | 「三条路径均可独立实现」 | **A 保留可行；B、C 降级为条件性结论**（§15.7） |

### 15.1 `libbaiduprotect_sec.so` 完整追踪

#### 15.1.1 文件与依赖事实（已验证，`readelf` / `sha256sum` / `strings` 实测）

| 文件 | 大小 | SHA-256 | 类型 | `NEEDED` |
|---|---|---|---|---|
| `lib/arm64-v8a/libbaiduprotect_sec.so` | 133168 B | `2e2f74f82136a8d6f9763ac07ebf6feba8aa11bf4131dc8c0ba91a080ba5ec68` | ELF64 aarch64，stripped | `liblog` `libc` `libm` `libstdc++` `libdl` |
| `lib/arm64-v8a/libbaiduprotect_sec_jni_facect.so` | 137264 B | `b4ba82eb554064a0260c6a10dc744362c31a305dd4999f2528f2db7270cc3e5b` | ELF64 aarch64，stripped | `liblog` `libc` `libm` `libstdc++` `libdl` |

- APK 内只有 arm64-v8a 一个 ABI（`split_config.arm64_v8a.apk`，65 个 `.so`）。
- **依赖方向**：把 65 个 `.so` 全部 `readelf -d` 扫一遍，唯一 `NEEDED` 指向 `libbaiduprotect_sec_jni_facect.so` 的是 `libbd_pass_face_sdk.so`（1235136 B）；**没有任何库 `NEEDED` `libbaiduprotect_sec.so`**。
- 全库 `strings` 搜 `libbaiduprotect_sec.so`，只在该库自身的 `SONAME` 出现 → **不存在 dlopen 字面量**。

#### 15.1.2 符号与注册机制（已验证）

| 库 | `dynsym` 导出 | `Java_` 导出 | `JNI_OnLoad` | `RegisterNatives` |
|---|---|---|---|---|
| `libbaiduprotect_sec.so` | 91 个，全部 `_0x` 前缀混淆名（如 `_0x5qgOQ` / `_0xhApNi` / `_0xTDqBS` / `_0xXAvZv`） | **0** | **无** | **无** |
| `libbaiduprotect_sec_jni_facect.so` | 7 个 `Java_` + 同族 `_0x` 集 + `gsecf_vfacect` | **恰 7 个** | 无 | 无 |

- `libbaiduprotect_sec.so` 的符号含 `_ZTS8_0x5XJdUIL9keyLength128EE`、`_ZN8_0xq3YG48_0xAJXCWIL9keyLength128EEEiR10WaesTablesIXT_EEPh` → **WBAES（白盒 AES-128）C++ 实现**，纯 C++ 库，自身不做 JNI 注册。
- `libbaiduprotect_sec_jni_facect.so` 的 7 个导出：`Java_com_baidu_protect_crypto_facect_WBAESCipherImplement_{initKeyDataFile, initKeyDataMemory, encrypt, decrypt, encryptFile, decryptFile, destroyContext}`。
- 两库各自独立编译同源代码（facect 的 `UND` 里 0 个 `_0x` 符号、无任何非 libc 的 `UND`，且不 `NEEDED` sec.so）→ **运行期不互相加载**。

#### 15.1.3 加载入口（两条，第二条为 T-006 遗漏）

**入口 1：静态 `loadLibrary`（全 dex 唯一 `baiduprotect` 的 `loadLibrary`）**

- `reverse/baidu/sources/classes15/sources/com/baidu/protect/crypto/facect/WBAESCipherImplement.java:17` → `System.loadLibrary("baiduprotect_sec_jni_facect")`。

**入口 2：动态插件管理器 + `System.load(绝对路径)`**

- 常量类 `reverse/baidu/sources/classes12/sources/kh0/____.java:21`：`public static final String c = "libbaiduprotect_sec.so"`。
- 该类注释 `SourceDebugExtension` 显示原始来源 `com/baidu/netdisk/service/album/plugin/logic/model/NativeLibExecutor`，`@Tag("NativeLibExecutor-DynamicPluginManager")`。
- 同类常量：`a = libncnn.so`、`b = libface_landmark.so`、`c = libbaiduprotect_sec.so`、`d = libopencv_java4.so`、`e = model_complex`、`f = model_simple`、`______ = libc++_shared.so`。
- `kh0/____.java:126-223` 方法 `c(String)` 按文件名 switch 归类，生成 face-landmark 插件的 `soFileList`。
- 实际加载点 `reverse/baidu/sources/classes14/sources/com/mars/united/intelligent/face/landmark/SDK.java:163-173`：对 `ft1.C_.b(config)` 的每个元素，以 `/` 开头则 `System.load(str)`，否则 `System.loadLibrary(str)`。
- 加载顺序表 `reverse/baidu/sources/classes14/sources/ft1/C_.java:263` 方法 `b(Config)`：`Config` 为 null 或无 `soFileList` 时返回 `{c++_shared, baiduprotect_sec, opencv_java4, ncnn, face_landmark}`；有 `soFileList` 时按名匹配这 5 类，**缺任一抛 `IllegalStateException`**（「so protect file is not exist」）。`ft1/C_.java:27`：`public static final String a = "baiduprotect_sec"`。
- **该动态插件链路在本 APK 内不可用**：`split_config.arm64_v8a.apk` 内只有 `lib/arm64-v8a/libopencv_java3.so`（17889264 B），**没有 `libncnn.so` / `libface_landmark.so` / `libopencv_java4.so` / `libc++_shared.so`**（5 个里 4 个名不匹配）。
- landmark SDK 的调用方只有 3 个类：`com/mars/united/intelligent/face/landmark/{SDK.java, C_.java, FaceLandmarkService.java}`，全仓无其它引用 → 属**相册人脸打点**功能。

#### 15.1.4 登录前初始化阶段是否执行（已验证）

- `reverse/baidu/sources/classes2/sources/com/baidu/sapi2/SapiAccountManager.java:148-154` 用 `try { Class.forName("com.baidu.pass.face.platform.FaceSDKManager"); new PassBiometricCall().initPassBioSDK(sapiConfiguration); } catch (Exception) { Log.e("SDK_INIT", "VIS SDK可能未集成…"); sapiConfiguration2.supportFaceLogin = false; }`。
- 该链最终会加载 `libbd_pass_face_sdk.so`（`NEEDED` 含 `libbaiduprotect_sec_jni_facect.so`）→ **登录 SDK 初始化阶段确实会加载 jni_facect 库**。
- 但它只提供 7 个 WBAES Cipher 方法（`initKey*` / `encrypt*` / `decrypt*` / `destroyContext`），**不修改请求参数、不参与签名、不修改设备环境**。

#### 15.1.5 唯一 Java 消费方（类引用闭包，已验证）

- `reverse/baidu/sources/classes12/sources/com/baidu/sofire/utility/WbEncryptUtil.java:53` → `Cipher.getInstance("WBAES", null)`，内部转调 `WBAESCipherImplement`。
- 调用方只有人脸 SDK：`classes12/sources/com/baidu/sofire/face/a/{b.java:1370, b.java:1521, f.java:1435, f.java:1521, m.java:586, m.java:628, q.java:610, q.java:652}` 与 `classes12/sources/com/baidu/sofire/face/b/b.java:116`，用于 `transcheckdata` 与 face 数据加密。
- 密钥路径 `classes15/sources/com/baidu/pass/face/platform/FaceSDKManager.java:295` → `WbEncryptUtil.setKeyPath(resRootPath + "pass-key.face-android")`。

#### 15.1.6 最终结论表（任务要求的 11 行）

| 项目 | 证据 |
|---|---|
| so 文件位置与 ABI | `lib/arm64-v8a/libbaiduprotect_sec.so`（133168 B，sha256 `2e2f74f8…ec68`）与 `lib/arm64-v8a/libbaiduprotect_sec_jni_facect.so`（137264 B，sha256 `b4ba82eb…3e5b`）；仅 arm64-v8a |
| Java 加载入口 | ①`WBAESCipherImplement.java:17` `System.loadLibrary("baiduprotect_sec_jni_facect")` ②`kh0/____.java:21` 常量 `libbaiduprotect_sec.so` → `face/landmark/SDK.java:163-173` `System.load`（动态插件路径，本 APK 内缺 4/5 个依赖 so，不可用） |
| 类初始化 / SDK 初始化 | `SapiAccountManager.java:148-154` 用 `Class.forName` + `try/catch` 初始化人脸 SDK（失败仅置 `supportFaceLogin=false`，不中断登录） |
| Java 调用方 | 静态库：`com.baidu.protect.crypto.facect.WBAESCipherImplement` ← `com.baidu.sofire.utility.WbEncryptUtil.java:53` ← `com.baidu.sofire.face.*`（9 处）。动态插件：`com.mars.united.intelligent.face.landmark.*`（3 类，相册人脸打点） |
| JNI 方法与 native 导出 | sec.so：91 个 `_0x`，**0 个 `Java_`**、无 `JNI_OnLoad`、无 `RegisterNatives`；jni_facect.so：**恰 7 个 `Java_com_baidu_protect_crypto_facect_WBAESCipherImplement_*`** |
| 是否存在动态加载 | **是**，但只有一条：face-landmark 插件管理器按 `soFileList` 用 `System.load` 加载；该链路在本 APK 内**因缺少 4 个依赖 so 而走不通** |
| 账号密码路径关联 | **无**。A 路径 8 跳均不触及 `WbEncryptUtil` / `WBAESCipherImplement` / `face/landmark/*`（类引用闭包 + `NEEDED` 依赖图双重核对） |
| 短信路径关联 | **无**。B 路径（发码 `/wp/api/login/sms`、提交 `/wp/api/login`）同上 |
| 扫码路径关联 | **无**。C 路径（`/v2/api/getqrcode` → `/channel/unicast` → `/v2/api/bdusslogin`）同上 |
| 是否影响认证或请求构造 | **不影响认证与签名**：该库只做白盒 AES 加解密原语（`Cipher` 实现类），不参与参数构造、不参与签名、不写设备指纹。**2026-11-28 校正**：加载失败会把 `supportFaceLogin` 置假，使请求**少发三个可选人脸能力参数**（`liveAbility`/`scanface`/`supFaceLogin`），故不得再写成「完全不影响请求构造」，详见 §16.4 |
| 最终判断 | **已排除对 A/B/C 的阻断**；保留「静态未知（无运行动态验证）」。理由：结论基于**类引用闭包 + `NEEDED` 依赖图 + 无 dlopen 字面量**三重证据，**不是**「搜不到调用点」 |

#### 15.1.7 「`libbaidusec*`」的覆盖核对

- **APK 内不存在任何名为 `libbaidusec*` 的文件**（`unzip -l` 全表核对）。T-006 报告中所写 `libbaidusec*`，实际对应本节这两个库，已全覆盖。
- 同族需要一并交代的库（均已在 T-006 覆盖，本节不重复取证）：

| 库 | 大小 | 归属 | 与登录的关系 |
|---|---|---|---|
| `libmsaoaidsec.so` | 622592 B | MSA OAID（`classes3/sources/com/baidu/helios/ids/oid/a.java:34` 加载） | 无（`di` 40 字段里无 oaid 位） |
| `libnetdisk-security.so` | 1082464 B | 业务 API 加签（`ex/n3.java:49`，上游拦截器 `ex/m1.java:294`） | 无（登录不走该拦截器） |
| `libnetdisk-security-rand.so` | 2884608 B | 业务 API `rand` 参数（`ServerApiKt.java:23` 注册拦截器） | 无 |
| `libnetdisk-signature-check.so` | 18264 B | APK 签名校验（`CertVerifier.java:38`） | 无 |
| `libsofiresec.so` | 1235496 B | sofire 旁路（`SONAME = libsofire.so`；Java 侧**无** `loadLibrary("sofiresec")`） | 只影响 `di` 索引 34，有纯 Java 兜底 |

### 15.2 短信验证码完整流程

#### 15.2.1 发码链路（已验证到接口级，6 列步骤表）

| 步骤 | 文件:行号 | 接口 / 方法 | 参数来源 | native 依赖 | 证据状态 |
|---|---|---|---|---|---|
| 1 UI「获取验证码」 | `classes4/sources/com/baidu/sapi2/newlogin/widget/NewLoginSmsLayout.java:158-169` | `d(String)`，日志 `sms_code_resend: action=request` | 用户输入手机号 | 无 | 已验证 |
| 2 构造发码 helper | 同上 `:169` | `new LoginSendSmsHelper(context, 1, o, j, str, i, h, !j, p).a(new HashMap<>(), …)` | 同上 | 无 | 已验证 |
| 3 helper 转调 | `classes4/sources/com/baidu/sapi2/newlogin/helper/LoginSendSmsHelper.java:161-169` | `SapiAccountManager.getInstance().getAccountService().doCapInitAndGetLoginSms(b, c, d, map, l)` | map 可带 `ds`/`tk`/`isValidate`/`isOver` | 无 | 已验证 |
| 4 前置人机令牌（可选） | `classes2/sources/com/baidu/sapi2/SapiAccountService.java:173-193` | `doCapInitAndGetLoginSms`：`ds`/`tk` 都空 → `ValidationUtil.capInit(...)`；**失败也继续** | `ValidationUtil.java:8-16` 的 `ak` / `scene="test"` / `tplExtraCode="111"` | 无 | 已验证 |
| 5 人机初始化请求 | `classes18/sources/com/baidu/validation/ValidationManager.java:5`、`:93-104` | `POST https://passport.baidu.com/cap/init`，body `tt`/`refer`/`ak`/`scene`/`ver="2"`，走 `HttpHashMapWrap`（含公共参数与 `sig`）+ `ParamsUtil.buildNaCookie` | `Referer` = `m143_()` 生成的人机页 URL（`:84`） | 无 | 已验证（响应 `code==0` 取 `data.tk`/`as`/`ds`） |
| 6 发码请求 | `classes2/sources/com/baidu/sapi2/SapiAccountRepository.java:1530`，端点 `:1632-1634` | `POST {wap}/wp/api/login/sms` | 参数见下 | 无 | 已验证 |

**第 6 步参数（`SapiAccountRepository.java:1536-1588`，已验证）**：`alg=v3`、`adapter=3`、`vstrswitch=1`、`cv=170601`、`isEncrypted=1`、`lang`、`liveAbility=1` + `scanface=1` + `supFaceLogin=1`（当 `supportFaceLogin`）、`support_photo=1`（当 `supportPhoto`）、`logLoginType=sdk_login`、`time`（秒）、`t`（毫秒）、`session_id = md5(UUID+millis)`、`nalogin=1`、`enableExternalWeb=0`、`username`（RSA 加密）、`countrycode`（非 `0086` 时）、`encryptedId` + `uid` + `encryptedType=living`、`isValidate`/`isOver`（来自 map）、`s`/`k`（来自 map 的 `ds`/`tk`）。

**是否需要 `di` / `sig` / `appSignKey` / `tpl`**：**全部需要**。发码走 `HttpHashMapWrap`（签名唯一调用点 `classes4/sources/com/baidu/sapi2/httpwrap/HttpClientWrap.java:86-88`），且带 `ParamsUtil.buildNaCookie`（含 `cuid` 与 `DVIF` cookie）。

**错误码（`classes4/sources/com/baidu/sapi2/result/GetLoginSmsResult.java`，已验证）**：`50020` / `50021` / `500002` / `50052` 触发人机验证、`50092` 短信阈值/频控、`71100` 触发注册、`400603` 注册、`400800` 注册弹窗、`50051` 安全短信弹窗、`97101`、`-205` / `-501` 解析失败；`data.smsCodeLength` 给验证码位数。

#### 15.2.2 提交链路（**修正 T-006 的端点记录**）

| 步骤 | 文件:行号 | 接口 / 方法 | 说明 | 证据状态 |
|---|---|---|---|---|
| 1 提交入口 | `classes4/sources/com/baidu/sapi2/newlogin/widget/NewLoginSmsLayout.java:178-207` | `LoginSmsWapLoginHelper.doSmsWapLogin(true, b, h, i, str, listener)` | `str` = 用户输入的验证码 | 已验证 |
| 2 helper 转调 | `classes4/sources/com/baidu/sapi2/newlogin/helper/LoginSmsWapLoginHelper.java:97-103` | `getAccountService().smsWapLogin(k, z, c, d, e, code)` | — | 已验证 |
| 3 组装参数 | `classes2/sources/com/baidu/sapi2/SapiAccountRepository.java:2710-2723` | `smsWapLogin`：`encryptedId` / `encryptedType=living` / `username`(RSA) / `sms="1"` / `smsvc=<验证码>` / `countrycode` → `wapLogin(1, z, …)` | — | 已验证 |
| 4 追加公共参数 | 同文件 `:2873-2901` | `wapLogin`：`alg=v3`、`adapter=3`、`cv=170601`、`isEncrypted=1`、`lang`、`liveAbility`/`scanface`/`supFaceLogin`、`support_photo`、`logLoginType=sdk_login`、`time`、`t`、`session_id`、`nalogin`、`needNaNextUrl=1`、`enableExternalWeb=0` | — | 已验证 |
| 5 实际端点 | 同文件 `:1126-1128` | `POST {wap}/wp/api/login` | 与账号密码路径**同一端点**，区别是带 `sms=1` + `smsvc` | 已验证 |
| 6 响应解析 | 同文件 `:2873-2901` → `BaseWapLoginResult.handleAccountAndPwdLoginResult:23-32` → `SapiAccountService.java:542`、`:570` | 读 `errInfo.no`（`!=0 && 400032` 判失败），再 `SapiCoreUtil.parseAccountXmlToResponse` 解析 `<client>` XML 得 `bduss`/`ptoken`/`stoken` | — | 已验证 |

**修正点**：T-006 记「短信走 `/v2/sapi/login`（`realDynamicPwdLogin`）」。实测新版登录 UI 走的是 **`smsWapLogin` → `/wp/api/login`**；`realDynamicPwdLogin` 只由旧 `dynamicPwdLogin` 链使用（`SmsPager.java:347`、`SmsLoginView.java:455`、`SmsLoginAccount.java:381`、`SmsLoginSamsungView.java:381`）。两条链都真实存在，但**网盘 App 现用 UI 走前者**。

**发码返回的 `ds`/`tk` 不流向提交**：`LoginSendSmsHelper.java:120-122`（与 `:49-51`）成功后执行 `finishValidation()` 并 `put("validation_times","0")`、`put("ds","")`、`put("tk","")`。

#### 15.2.3 完整短信登录必须分三段判定

| 段 | 结论 | 依据 |
|---|---|---|
| 发码请求 | **已到接口级**（端点、参数、签名、错误码、前置 `cap/init` 全部静态可读） | §15.2.1 |
| 收到验证码后的提交与会话换取 | **已到接口级**（`/wp/api/login` + `sms`/`smsvc`，响应 XML → `bduss`/`ptoken`/`stoken`） | §15.2.2 |
| **完整短信登录能否独立实现** | **暂不能判为「可独立实现」** | 见下 |

**不能判「可独立实现」的三条理由**：

1. **发码前置人机验证是 WebView 页面**。`passport.baidu.com/cap/init` 只取 `ds`/`tk`/`as` 三个令牌（失败不阻断），真正的人机墙是 `https://wappass.baidu.com/static/activity/pass-machine.html`：`classes18/sources/com/baidu/validation/activity/ValidationActivity.java:319-321` 用 `ValidationWebView` `loadUrl(...)`、`:179` `evaluateJavascript("javascript:window.machine_sdk.show()")`。**通过条件是服务端下发 + 页面内 JS 风控，客户端静态无法复刻**；且本项目按 AGENTS.md 第 9 条禁止用 WebView 完成登录。
2. **触发时机由服务端决定**：错误码 `50020`/`50021`/`500002`/`50052` 才拉起人机页，静态无法预测何时触发。
3. **频控与设备校验是服务端行为**：`50092` 为短信频控错误码，规则不可静态得出。

→ 与坚果云 R27（人机验证依赖腾讯闭源 Web 服务）**同类**，已在 `docs/RISKS.md` 登记。

### 15.3 扫码路径的 `hostDeviceId`（结论反转）

#### 15.3.1 事实（已验证）

| 项目 | 证据 |
|---|---|
| 生成函数 | **不存在**。`classes6/sources/com/baidu/sapi2/dto/GetQrCodeImageDTO.java:9` 与 `QrLoginStstusCheckDTO.java:6` 都只声明 `public String hostDeviceId;`（无初值）；**全仓 `.hostDeviceId =` 赋值 0 命中** |
| 调用链 | 客户端从不赋值 → 恒为 `null` → `HttpHashMap.put(key, null)`（`classes15/sources/com/baidu/pass/http/HttpHashMap.java:24-26`） |
| 数据来源 | **无来源**（不是系统信息、不是 SDK 生成器、不是持久化 ID、不是随机数） |
| 是否持久化 | 否 |
| 是否需要跨请求保持一致 | **不适用**（值恒 `null`，参数根本不出现） |
| 是否属于通用设备信息 | 否 |
| Rust 是否可以独立生成 | **不需要生成**，按官方缺省不传即可 |
| 是否存在 native 依赖 | 否 |

#### 15.3.2 `null` 参数被丢弃（已验证）

- `classes15/sources/com/baidu/pass/http/a.java:333-353` 私有方法 `a(HttpHashMap)` 只拼接 `!TextUtils.isEmpty(key) && !TextUtils.isEmpty(value)` 的项。
- GET 分支 `a.java:164-167`：仅当拼出的串非空才 `url += "?" + str`。
- multipart 分支 `a.java:143-147` 同样跳过空值。
- → `hostDeviceId = null` 时该参数**完全不出现**在 URL/表单里。

#### 15.3.3 服务端不返回该字段

`classes4/sources/com/baidu/sapi2/result/GetQrCodeImageResult.java` 只有 `channelId` / `contentUrl` / `imageUrl` / `prompt` 四个字段，**没有 `hostDeviceId`**。

#### 15.3.4 使用点（仅 4 处，全部是 `put` 透传）

| 位置 | 动作 |
|---|---|
| `classes2/sources/com/baidu/sapi2/QrCodeService.java:325` | `getQrCodeImage` → `put("hostDeviceId", dto.hostDeviceId)` |
| 同文件 `:350` | `getQrCodeLoginWithEnuidImage` → 同上（**被扫端**用途，本项目不用） |
| 同文件 `:360` | `getQrLoginResult` → `put("hostDeviceId", str2)`，`str2` 来自 `:176` 的 `qrLoginStstusCheckDTO.hostDeviceId` |
| 同文件 `:176` | `qrLoginStatusCheck` 回调 → 构造 `QrLoginStstusCheckDTO` 时透传 |

#### 15.3.5 扫码五段各自状态

| 段 | 状态 | 证据 |
|---|---|---|
| ① 二维码获取 | **已明** | `GET /v2/api/getqrcode`，参数 `lp` / `apiver=v3` / `tt` / `openPlatformId` / `encuid` / `needQrCodeContent` / `hostDeviceId`（`QrCodeService.java:306-327`）；`channelId = optString("sign")`（`:49`） |
| ② 长轮询 | **已明** | `GET /channel/unicast`，参数 `apiver=v3` / `callback=cb` / `channel_id` / `tt`，超时 **40000ms**（`:387-397`）；JSONP 剥离 `:146`；`status` 分支 `:152`、`:165-168`、`:170-177`、`:183` |
| ③ `hostDeviceId` 生成 | **已闭环（官方不生成）** | §15.3.1–15.3.2 |
| ④ 会话换取 | **已明** | `GET /v3/login/main/qrbdusslogin`（`isJoinCodeLogin=true`）或 `GET /v2/api/bdusslogin`，参数 `display=pcsdk` / `qrcode=1` / `bduss` / `tt` / `hostDeviceId`（`:354-365`）；回调 `:195-242` → `parseAccount` → `setPreLoginType(QR)` → `validate(account)` |
| ⑤ 认证状态持久化 | **已明（与 A/B 同一链）** | `SapiAccountManager.validate:823-836` → `SapiContext.java:978-1016`（AES 加密 JSON 存 SharedPreferences + `SapiUtils.webLogin` 写 CookieManager） |

**唯一仍未闭环的是「服务端是否接受 `hostDeviceId` 缺席」**，属动态验证，静态无法证明。

#### 15.3.6 顺带修正：扫码**加** `sig`

- T-006 §8.3/§8.7 记「扫码不加 `sig`、不传 `HttpHashMapWrap`」，**与实测不符**。
- 实测：`QrCodeService` 三个扫码请求都经 `HttpClientWrap` → `buildParamDTO` → `httpHashMap.doSign(this.appSignKey)`（`classes4/sources/com/baidu/sapi2/httpwrap/HttpClientWrap.java:86-88`，全仓唯一 `doSign` 调用点）→ **扫码也加 `sig`**。
- 但扫码三处 `get(...)` 的 cookie 参数都传 `(java.util.List) null` → **确实不带 cookie，也不带 `di`**。
- 所以正确表述是：**扫码不发 `di`、不带 cookie，但加 `sig`**。

### 15.4 `cuid` 的真实生成算法

#### 15.4.1 逐跳表（已验证）

| 跳 | 类:行号 | 动作 | 数据来源 | 是否持久化 |
|---|---|---|---|---|
| 1 | `classes12/sources/com/baidu/sapi2/utils/SapiUtils.java:268-282` | `getClientId(Context)`：`ServiceManager`/`isAccountManager`/`confignation` 任一为 null → 返回 null；`!isAgreeDangerousProtocol() && !isSupportBrowseMode()` → 返回 null | 配置 | — |
| 2 | 同文件 `:352-366` | `getDeviceID(Context)`：同两道闸门，否则调 `com.baidu.android.common.util.DeviceId.getDeviceID(ctx)` | — | — |
| 3 | `classes3/sources/com/baidu/android/common/util/DeviceId.java:102-104` | `getDeviceID` = `b(ctx).c()`（返回 `cesium.h.a` 对象） | — | — |
| 4 | 同文件 `:138-153` | `b()` 双检锁单例，缓存 `d = a(context).c()`；构造 `cesium.____ h`、`cesium.h e = new h(ctx, new cesium.e.C_(ctx), h)`、`cesium.a f` | — | — |
| 5 | 同文件 `:37-66` | `c()`：`e.h()` 加锁（`.lock`，100 次 × 100 ms）→ `e()` = `f()`（`e.m999_()` 读 `files/libcuid.so` 解密）否则 `g()`（读 `new File(ctx.getFilesDir(),"libcuid.so")` 经 `cesium.______._____(cesium.f.___.m998_(file))` 解密） | 本地文件 | 读 |
| 6 | 同文件 `c()` 中段 | 判定：已有合法 cuid 且 deviceid 非空且不等于 `h.f()`；`cesium.a.__.m952_(cuid)` 判设备 ID 是否失效 | 内存/文件 | — |
| 7 | 同文件 `c()` 分支 | 都不满足 → `a(null,null)` → `a(null)`（`e.e` 新建）→ `c(aVarA)` 异步落库；否则重建并 `______(aVarE.v())` | — | 写 |
| 8 | `classes3/sources/com/baidu/cesium/h.java:377-400` | `e(String)` 新建：①`Settings.Secure.getString(resolver, "android_id")`（`vm.___.b = "android_id"`）②若 `cesium.a.__.__(str)` 为 false（黑名单 `0000000000000000`）→ `UUID.randomUUID().toString()` 且置 `z=true` | Android 系统 | — |
| 9 | 同文件 | ③`cesium.____.C0037____.__(('com.baidu'+str).getBytes(), true)` = **MD5 大写十六进制**（`____.java:191-206`）④`z` 为真时 `"FUUID" + str__.substring(5)` ⑤写入 `aVar.f797_`（即 cuid），`__ = "O"`，`___ = i(str__)` | 算法 | — |
| 10 | 同文件 `:274-282` | 派生字段 `i(String)` = `new cesium.f.C_("ABCDEFGHIJKLMNOPQRSTUVWXYZ234567=", false, false).__(new cesium.a.C_().m953_(str.getBytes("UTF-8")))` | 算法 | — |
| 11 | 同文件 `:262-272` | 机型指纹 `f()` = `MD5(Build.MODEL)` 小写十六进制 **第 3..15 位共 12 字符**，缓存于静态 `e` | Build.MODEL | 内存缓存 |
| 12 | `classes3/sources/com/baidu/cesium/e/C_.java` | 持久化：目录 `dataDir/.cesium`，文件常量 `d = "libbh.so"`（**数据文件不是 so**）；JSON 键 `dik`/`v270fk`/`cck`/`ek`/`ctk`/`vsk`/`csk`/`pmk`/`hrk`/`ock`/`ifu` ↔ 字段 `f797_`/`__`/`___`/`d`/`____`/`______`/`a`/`c`/`_____`/`b`/`e`；写入经 `com.baidu.cesium.c.b.___().m987_()` 加密 | 文件 | 写 |
| 13 | 同文件 `h.java:139-155` | `v()` 输出 `f797_ + "|" + (__ 空则 "0" 否则 __) + (__=="V" ? ___ ) + (_____ 非空 ? _____)` → 这是 `DeviceId.getCUID()`；实际传输用 `getClientId()` = 纯 `f797_` 值 | — | — |

#### 15.4.2 黑名单与权限分支（已验证）

- `classes3/sources/com/baidu/cesium/a.java:53-64`：`f745_` Base64 解码 = `DED0AFDB1AD0CC4CA974D5EBA0165141` / `C77D5D04D94F5F56C8A0A6DC3DBF240A`；`__` = `0000000000000000`。
- `m952_(str)`：空 → false；等于任一黑名单（忽略大小写）→ false；否则 true（设备 ID 仍有效）。
- `SapiUtils.getClientId` 的两道闸门：`isAgreeDangerousProtocol()` 或 `isSupportBrowseMode()` 之一为真才返回 cuid，否则返回 `null`（→ `di` 第 6/38 位发空串）。

#### 15.4.3 无 native 依赖（已验证）

`com/baidu/cesium` 全目录 54 个 `.java`，`grep loadLibrary|System.load|" native "` **0 命中**；`libbh.so` / `bohrium` 只是文件名/目录名字符串，APK 内**不存在**该 `.so`（`unzip -l` 核对）。

#### 15.4.4 `t_cuid` 与 `cuid` 的关系（已验证）

> **本节已于 2026-11-28 更正**（原表述把 38/39 两位写反，原文见 §16.6 差异表）。正确映射：

- `classes12/sources/com/baidu/sapi2/utils/SapiDeviceInfo.java:53` 的 `getCookiesKeyList()` 第 **38** 位是 `t_cuid`、第 **39** 位是 `t_appname`。
- `buildDeviceTokens`：索引 38 取 `confignation.mTPLCuid`（`:171`），为 null 时按 `checkDiUpload(gray,6)` 决定回落 `getClientId` 还是空串（`:172-178`）；索引 39 取 `confignation.mTPLAppName`（`:181-183`），网盘业务侧传入常量 `"wangpan_drive"`（`classes15/sources/com/baidu/netdisk/application/C_.java:199`）。
- 因此 **`t_cuid` 是设备侧组合串（内含 `i(cuid)`），`t_appname` 才是常量**。完整闭环、`i()` 是否进入请求、以及 Rust 复刻口径见 §16.2。

#### 15.4.5 `i()` 派生字段的算法构成（已验证到类与方法，移植仍需工作量）

- `classes3/sources/com/baidu/cesium/a/C_.java:9`：`public com.baidu.cesium.a.d[] f746_ = {new com.baidu.cesium.a.e(8, 0), new com.baidu.cesium.a.f(0, 1), new com.baidu.cesium.a.f(1, 1), new com.baidu.cesium.a.e(7, 1)};`；`:12-30` `m953_(byte[])` 按这 4 个算子迭代。
- `classes3/sources/com/baidu/cesium/a/g.java` 是 **xxHash32**：常量 `a=16`、`b=13`、`c=-1640531535`、`d=-2048144777`、`e=-1028477379`、`f=668265263`、`g=374761393`；`:27-29` 小端读 4 字节。
- `classes3/sources/com/baidu/cesium/a/f.java:6-17`：`f(int,int)` 设 `f753_=4294967295L`、`__=4`、`___=32`，`__(byte[],int,int)` 直接 `new g().____(...)`。
- `classes3/sources/com/baidu/cesium/a/e.java:8-40`：用 `Class.forName(com.baidu.cesium.a.i.___(com.baidu.cesium.a.a.m956_()), true, Object.class.getClassLoader())` + `i.____(...)` 反射调用；`i.___(byte[])` = `new String(com.baidu.cesium.c.a.__.____(种子, 种子, bArr))`（对称密钥派生）。
- `classes3/sources/com/baidu/cesium/a/a.java`：`m956_()` 返回 32 字节常量，`__()`/`___()` 各 16 字节。

→ **`i()` = 自定义摘要（4 算子，含 xxHash32 与反射 PRNG）+ 自定义 Base32（字母表 `ABCDEFGHIJKLMNOPQRSTUVWXYZ234567=`）**。纯 Java 可移植、无 native，但需移植 `cesium.c.a.*` 的密钥派生与加解密，且本机无 Java 运行时可产测试向量。

#### 15.4.6 `cuid` 的 Rust 复刻结论（五类分开说明）

| 类别 | 内容 | 说明 |
|---|---|---|
| ① Rust 可按官方算法独立生成 | `MD5("com.baidu" + android_id)` 转大写十六进制；`FUUID` 前缀替换；机型指纹 `f()` = `MD5(Build.MODEL)` 小写 hex 第 3..15 位 | 纯算法，输入由 Kotlin 传两个通用字段 |
| ② 必须 Kotlin 经 Android API 取 | `Settings.Secure.getString(resolver, Settings.Secure.ANDROID_ID)`、`Build.MODEL`、`UUID.randomUUID()`（回退分支） | 属通用系统信息，符合架构约束（网盘专有语义仍留 Rust） |
| ③ 必须在本机持久化 | `cuid` 值本身（`f797_`）与伴生字段 `__`（新建与读回均为 `"V"`，见 §16.2.8）、`___`（`i()` 结果），否则每次启动都会生成新设备号 | 官方落在 `dataDir/.cesium/libbh.so`（加密 JSON）；本项目可用自己的存储，不必复刻其文件格式与加密 |
| ④ 只能按分支处理 | `ANDROID_ID` 为 `0000000000000000` 或读不到 → `UUID.randomUUID()` + `FUUID` 前缀；命中黑名单 `DED0AFDB1AD0CC4CA974D5EBA0165141` / `C77D5D04D94F5F56C8A0A6DC3DBF240A` → 判设备 ID 失效并重建；`isAgreeDangerousProtocol()`/`isSupportBrowseMode()` 都为假 → cuid 发空串 | 三种分支行为已逐条落行号 |
| ⑤ 当前无法精确复刻 | 派生字段 `i()`（`cesium.a.C_` 的 4 算子 + `cesium.a.e` 反射 PRNG + `cesium.c.a.*` 密钥派生 + 自定义 Base32） | 纯 Java 可移植但工作量与出错风险大，且**本机无 Java 运行时无法产测试向量**。**2026-11-28 校正**：`i()` 确实进入最终请求（§16.2.5），不复刻则 `t_cuid` 段必然与官方不同；但服务端是否校验该段仍待动态验证 |

> 边界不变：**Rust 负责协议与算法，Kotlin 只提供通用系统信息**；百度专有参数不进 Kotlin。**2026-11-28 更正**：取常量的不是 `t_cuid` 而是 `t_appname`（`"wangpan_drive"`，Rust 侧常量）；`t_cuid` 是设备侧组合串，其主值由 Kotlin 提供 `ANDROID_ID`/`Build.MODEL`、由 Rust 计算。

### 15.5 账号密码路径（A）补充取证

#### 15.5.1 `libbaiduprotect_sec.so` 关联（已闭环）

见 §15.1.6 —— **不影响 A 路径**。

#### 15.5.2 `cuid` 与 `DVIF` cookie 的边界（已验证）

| 项 | 事实 | 证据 |
|---|---|---|
| `cuid` cookie 名 | `"cuid"`（常量 `classes12/sources/com/baidu/netdisk/transfer/e.java:15`） | `ParamsUtil.java:74` |
| `cuid` 值来源 | `SapiUtils.getClientId(context)`（即 §15.4 的 cuid） | `ParamsUtil.java:74` |
| `DVIF` 值来源 | `SapiContext.getDeviceInfo()`（即 40 字段 `di` 的 AES 加密串） | `ParamsUtil.java:79` |
| `DVIF` 性质 | **本项目客户端自行构造的设备信息 Cookie**，不是官方 App 的私有认证 Cookie | 构造点 `SapiDeviceInfo.getDeviceInfo:205-209` |
| `BAIDUID` 值来源 | `SapiUtils.getCookie("https://baidu.com", "BAIDUID")`，即 `android.webkit.CookieManager` 里读；读不到返回 `""`（空串照发） | `SapiUtils.java:284-300`、`ParamsUtil.java:81` |
| `BAIDUID` 的产生 | 服务端 `Set-Cookie` 回写（首次访问 baidu 域时），**不是客户端生成** | 同上 + 待动态验证 |
| 认证 Cookie（BDUSS/STOKEN/PTOKEN） | 服务端在登录响应 XML 的 `<client>` 里下发（`SapiCoreUtil.parseAccountXmlToResponse`），**不是本项目构造** | `SapiAccountService.java:570-602` |

**三类 Cookie 必须分开写，不得混为一谈**：

| 类别 | 举例 | 谁产生 | 本项目可否自行构造 |
|---|---|---|---|
| 设备信息 Cookie | `cuid`、`DVIF` | 客户端按官方算法构造 | **可以**（本项目的实现，值来自 §15.4 的 cuid 与 §15.6 的 `di`） |
| 站点标识 Cookie | `BAIDUID` | 服务端 `Set-Cookie` | **不可以**（只能由服务端下发；空串是否被接受待动态验证） |
| 认证凭据 Cookie | `BDUSS`、`STOKEN`、`PTOKEN` | 服务端登录响应下发 | **不可以**（这就是登录要换的东西本身） |

> 特别说明：本项目**不复用**浏览器或官方 App 的私有认证 Cookie。`DVIF` 虽名为 cookie，但它是本项目按官方算法自行计算的设备指纹载荷，与「复用官方会话」是两件事。

### 15.6 `di` 分类表述校正（不改动已核实的 40 项与统计）

> 原 §3 的字段分类数字（必需 22 / 条件 2 / 可选 16）与逐字段定性**全部保留有效**。本节只校正表达方式，避免被读成「可以随意删字段」。

#### 15.6.1 四条必须写准的说法

| # | 正确表述 | 为什么必须这样写 | 证据 |
|---|---|---|---|
| 1 | **40 个位置始终构造**，被 `diExceptIndex` 排除或灰度过滤时**填空字符串，不是删除字段** | 客户端 `buildDeviceTokens` 用 `TextUtils.join` 按固定长度拼接，缺位会导致后续位错位 | `SapiDeviceInfo.java:67-187` 共 **80 处 `add(`**，其中显式 `add("")` 12 处，其余 68 处是三元表达式 `(diExceptIndex.contains(N) \|\| !checkDiUpload(...)) ? "" : <值>` |
| 2 | 「16 个恒空位」= **官方客户端发送空值**，**不表示服务端允许删除这些字段** | 客户端构造行为与服务端必需性是两个不同命题 | 同上 |
| 3 | **客户端构造行为已完全可读；服务端对 40 项的实际校验强度全部待动态验证** | 本文所有 `di` 结论都只到「客户端怎么发」这一层 | — |
| 4 | 「可选」**只描述官方客户端的取值来源**（常量/空串/条件），**不表示可以从加密载荷里随意删除** | 载荷是定长拼接后整体 AES 加密的，删位即破坏结构 | `getDeviceInfo:205-209`（`TextUtils.join` 后加密） |

#### 15.6.2 一句话对照表

| 说法 | 是否可用 | 原因 |
|---|---|---|
| 「可选字段可以不传」 | **不可用** | 该字段位置始终存在，值为空串 |
| 「可选字段官方客户端发空串」 | 可用 | 与代码一致 |
| 「服务端不需要这 16 项」 | **不可用** | 无任何静态证据，属动态验证范畴 |
| 「40 个位置恒定，值可为空串」 | 可用 | 与代码一致 |

### 15.7 三条登录路径修正后的各自结论

> 判定原则：**以证据为准，不以维持原「可行」结论为目标**。任一路径只要还有「静态无法复刻且可能阻断」的环节，就不能写「可独立实现」。

#### 15.7.1 A 账号密码

| 检查项 | 状态 | 证据 |
|---|---|---|
| 请求链路完整 | 是（8 跳） | §6 |
| RSA + MD5 签名 + `di` + Cookie 构造 | 全部已还原 | §6.3、§3、§15.5.2 |
| 响应解析 | XML `<client>` → `bduss`/`ptoken`/`stoken` | `SapiCoreUtil.parseAccountXmlToResponse` |
| 前置 `cap/init` | **存在**（`SapiAccountService.java:63-93`：`ds`/`tk` 都空时调 `ValidationUtil.capInit`；**失败也继续**） | 只取 `ds`/`tk`/`as` 三个令牌，**失败不阻断**，因此不构成阻断项 |
| `libbaiduprotect_sec.so` 影响 | **无** | §15.1.6 |
| `cuid` 算法 | 主值可复刻；派生字段 `i()` 需移植 | §15.4.6 |
| Cookie 边界 | `cuid`/`DVIF` 自构造；`BAIDUID` 服务端下发（可为空）；认证 Cookie 服务端下发 | §15.5.2 |
| native 阻断 | **无** | §4、§5、§15.1 |
| **结论** | **原生登录可行（静态可行，动态未验证）** —— 三条路径中唯一保持原判的一条 |

#### 15.7.2 B 短信验证码 —— 拆成三段，不得以半段等同整条

| 段 | 结论 | 关键证据 |
|---|---|---|
| 发码请求 | **可独立实现**（端点、参数、签名、错误码全部静态可读） | §15.2.1 |
| 收到验证码后的提交 | **可独立实现**（`/wp/api/login` + `sms=1` + `smsvc`） | §15.2.2 |
| **完整短信登录** | **需攻关** | 前置人机验证是 WebView 页面 `pass-machine.html`（`ValidationActivity.java:319-321`、`:179`），通过条件由服务端与页面 JS 决定，客户端静态无法复刻；本项目按 AGENTS.md 第 9 条又禁止用 WebView 完成登录 |

→ **由「可独立实现」降级为「需攻关」**。降级理由不是资料不足，而是**存在一条静态无法复刻的交互环节**。

> **2026-11-28 校正**：本节的降级理由（「本项目禁用 WebView 登录」）**表述不当**——该 WebView 只承载条件触发的人机挑战，不承载凭据、不读 cookie、回传只有 `ds`/`tk`，与坚果云 R27 同口径，不构成 AGENTS.md 第 9 条禁止的形态。档位暂维持「需攻关」，但理由改写为「挑战通过条件静态不可复刻」+「是否接受内嵌同一 H5 尚未拍板」。详见 §16.5.7。

#### 15.7.3 C 扫码 —— 拆成五段

| 段 | 结论 | 关键证据 |
|---|---|---|
| ① 二维码获取 | 已明 | §15.3.5 |
| ② 长轮询 | 已明 | §15.3.5 |
| ③ `hostDeviceId` 生成 | **已闭环：官方客户端不生成、恒 `null`、参数被丢弃** | §15.3.1–15.3.2 |
| ④ 会话换取 | 已明（`/v3/login/main/qrbdusslogin` 或 `/v2/api/bdusslogin`） | §15.3.5 |
| ⑤ 认证状态持久化 | 已明（与 A/B 同一 `validate` 链） | §15.3.5 |
| **结论** | **原生登录可行（静态可行，动态未验证）** —— 但只在「服务端接受 `hostDeviceId` 缺席」这一条动态前提成立时；该前提待验证 |

→ C 的本地证据已完整（五段全明），残留不确定性**只在服务端一侧**，与 A 同类，因此**保留「可行」但必须带条件句**。

#### 15.7.4 三路径对照（修正后）

| 路径 | 修正前（T-006） | 修正后（本节） | 变化原因 |
|---|---|---|---|
| A 账号密码 | 可独立实现 | **可独立实现** | 证据补齐，判定不变 |
| B 短信验证码 | 可独立实现 | **需攻关** | 前置人机验证是 WebView + 服务端 JS，静态不可复刻 |
| C 扫码 | 可独立实现 | **可独立实现（带动态前提）** | `hostDeviceId` 已闭环为「不生成」，残留仅在服务端 |

### 15.8 百度整体四档结论（修正后）

**原生登录可行（静态可行，动态未验证）——成立条件：走账号密码路径或扫码路径。**

| 判据 | 结果 |
|---|---|
| 是否存在**至少一种**满足 AGENTS.md 第 9 条（APP 内原生、非 OAuth/WebView/cookie 复用）的路径 | **是**（A 与 C） |
| 是否三条路径全部满足 | **否**（B 需攻关） |
| native 是否仍构成阻断 | **否**（`libbaiduprotect_sec` 已排除，其余 6 库此前已排除） |
| 是否有静态无法复刻的环节 | **有 1 处**：短信发码的前置人机 WebView（只影响 B） |
| 整体档位 | **原生登录可行**（理由：存在 A、C 两条不含静态不可复刻环节的路径） |

**不因此放宽的地方**（保持原 §12.4 与 §13）：服务端是否接受本项目构造的 `sig`/`di`、`diExceptIndex` 实际下发值、`BAIDUID` 是否必须非空、是否出现额外风控挑战、扫码服务端是否接受 `hostDeviceId` 缺席 —— **全部仍待动态验证**。

### 15.9 对 §1–§13 的更正登记（原文不删，此处列差异）

| 原位置 | 原文摘要 | 本节实测 | 处理 |
|---|---|---|---|
| §1.2 一句话版 | 「三条登录路径……均不依赖官方闭源 native；唯一未闭环的是服务端风控」 | 「均不依赖 native」过于笼统：核心认证算法成立，公共参数 `zid` 的真实值依赖 native（§17）；另外 B 还有 WebView 人机验证未闭环 | **已被本节与 §17 修正**，以 §15.7 + §17.10 为准 |
| §1.3 表第 6 行 | 「短信路径：可独立实现（静态）」 | 降级为**需攻关** | 同上 |
| §1.3 表第 7 行 | 「扫码路径：可独立实现（静态）」 | 保留，但补「`hostDeviceId` 官方不生成」与「服务端是否接受缺席待验证」 | 同上 |
| §5 表 `libbaiduprotect_sec.so` 行 | 「待确认：本次未追其 Java 侧调用链」 | 已闭环，见 §15.1 | 同上 |
| §7 全节 | 短信 = `realDynamicPwdLogin` → `/v2/sapi/login` | 网盘现用 UI 走 `smsWapLogin` → `/wp/api/login`（`realDynamicPwdLogin` 属旧链） | 同上 |
| §8.2 `hostDeviceId` 行 | 「客户端生成，两个接口用同一个值」 | **客户端从不赋值，恒 `null`，参数被丢弃** | 同上 |
| §8.3 / §8.7 | 「扫码不发 `sig`」 | 扫码**加 `sig`**、不发 `di`、不带 cookie | 同上 |
| §9.3 `cuid` 行 | 「`Settings.Secure.ANDROID_ID` 派生或自生成 UUID」 | 官方是 `MD5("com.baidu" + android_id)` 大写；黑名单/空值才走 `FUUID` + 随机 UUID | 同上 |
| §11 汇总表 B 行 | 可独立实现 | 需攻关 | 同上 |
| §12.1 / §12.2 第 1 条 | 「三条都能」 | **A、C 能；B 需攻关** | 同上 |
| §13 第 5 行 | `libbaiduprotect_sec.so` 静态未做 | 已闭环 | 同上 |
| §13 第 7 行 | 短信发码接口未取证 | 已取证到接口级 | 同上 |
| §13 第 8 行 | `hostDeviceId` 生成规则不完整，推测任意随机串可用 | **官方不生成**，不存在「推测可用」的问题 | 同上 |
| §13.1 范围边界 | 「本任务没有验证短信验证码怎么发出去」 | 本轮已补（§15.2.1） | 同上 |
| 附录 B 可信度分布 | 推测 2 条（其一是 `hostDeviceId` 生成规则） | 该推测已关闭 | 同上 |

> **本节新增的推测条目**：0 条。新增的「待确认」条目 2 条：①服务端是否接受 `hostDeviceId` 缺席 ②短信人机验证在空设备/新设备上是否必然触发。

## 16. T-006 最终一致性校正（2026-11-28）

> **本节是当前有效结论的唯一入口。** §1–§15 的取证历史全部原样保留，凡本节与旧文冲突，**以本节为准**，逐条差异见 §16.6。
> 本节**未重新反编译 APK、未新增依赖、未发任何真实请求、未登录任何账号、未抓包**；全部结论来自既有反编译产物的静态读取。

### 16.0 本轮范围

| 项 | 内容 |
|---|---|
| 触发原因 | 复核发现四处口径不自洽：①`t_cuid` 与 `t_appname` 的来源被写反 ②`cuid` 的派生字段 `i()` 是否进入最终请求未证明 ③`sf_zid` 的 `"NoZidYet"` 与十六进制兜底值没有分支关系 ④百度短信的人机验证 WebView 与坚果云 R27 判定口径不一致 |
| 本轮动作 | 只做静态证据核对与文档校正；不重跑 T-006 的 native 追踪，不重跑 T-005，不锁定 MVP，不启动阶段 2 |
| 结论数量变化 | 三路径档位：A 保持「可独立实现」（新增条件性验证依赖说明）、B 保持「需攻关」但**降级理由改写**、C 保持「可独立实现（带动态前提）」 |

### 16.1 当前有效结论总入口

| # | 问题 | 当前有效结论 | 详见 |
|---|---|---|---|
| 1 | `t_cuid` / `t_appname` 的字段来源 | 索引 **38 = `t_cuid` ← `mTPLCuid`**；索引 **39 = `t_appname` ← `mTPLAppName`**（网盘传入常量 `"wangpan_drive"`）。旧文把两者写反，已更正 | §16.2.1 |
| 2 | `i()` 是否进入最终登录请求 | **是**。它进入 `cuid` 组合串的第三段，再经 `mTPLCuid` → `di` 索引 38 → AES 加密 → 请求体 | §16.2.5 |
| 3 | `sf_zid` 的取值 | **不恒为 `"NoZidYet"`**：共五种可能（`"NoZidYet"` / 空串 / 静态缓存 / SP 文件解密值 / **62 位**十六进制常量）。旧文「56 位」错误 | §16.3 |
| 4 | 公共参数 `zid` 与 `di` 的 `sf_zid` 是否同一个值 | **同源同值**（都取 `getCurrentZid`），但在 `diExceptIndex` 排除索引 34 时 `di` 侧为空串而公共参数侧仍有值 | §16.3.5 |
| 5 | `cuid` / `zid` 能否由 Rust 精确复刻 | `cuid` 主值（MD5(`"com.baidu"+android_id`) 大写）可精确复刻；**`i()` 不可在不移植 `cesium.a.*` 的情况下复刻**（纯 Java 但工作量大且本机无 Java 运行时可产测试向量）；`zid` 的真实值依赖 native `libfire` 与服务端下发文件，**不可静态复刻**，只能走纯 Java 兜底（常量/空串） | §16.2.8、§16.3.6 |
| 6 | 人脸 native 对登录请求的实际影响 | **不影响认证、签名、设备指纹**；但加载失败会把 `supportFaceLogin` 置假，使请求**少发三个可选能力参数**（`liveAbility`/`scanface`/`supFaceLogin`）。旧文「完全不影响请求构造」表述过强，已改 | §16.4 |
| 7 | 短信路径的人机验证 WebView 性质 | **只是条件触发的人机挑战**：WebView 不承载账号/密码/验证码输入，不读不写 cookie，回传只有 `errno`/`msg`/`ds`/`tk`；认证仍由原生 `POST {wap}/wp/api/login` 完成。**与坚果云 R27 同口径，不得仅因 WebView 存在就判不可行** | §16.5 |
| 8 | 账号密码路径是否也存在验证页面分支 | **存在**。错误码 50052 会走同一个 `ValidationService` 人机挑战并重试；另有一组错误码直接退整页网页登录 | §16.5.4 |
| 9 | A / B / C 最终档位 | A **可独立实现**（带与 R27 同级的条件性第三方验证依赖）；B **需攻关**（理由已从「违反第 9 条」改写为「人机挑战通过条件静态不可复刻，且是否接受内嵌 H5 尚未拍板」）；C **可独立实现（带动态前提）** | §16.5.7 |
| 10 | 日期 | 本轮补充收口统一为 **2026-11-28**（此前误写 2026-11-28，已全仓更正） | §16.6 |

### 16.2 `t_cuid` / `t_appname` 与 `i()`

#### 16.2.1 索引定位（已验证，纠正旧文写反）

- 字段名清单：`classes12/sources/com/baidu/sapi2/utils/SapiDeviceInfo.java:13-56` 的 `getCookiesKeyList()`，其中 `:53` 是 `t_cuid`、`:54` 是 `t_appname`（索引从 0 起，共 40 项）。
- 取值构造：同文件 `:168-184`（`buildDeviceTokens` 尾部）：

| 索引 | 代码 | 取值 |
|---|---|---|
| 38 | `:168-179` 先判 `diExceptIndex.contains(38) || !checkDiUpload(gray,38)` → 空串；否则 `:171` `clientId = confignation.mTPLCuid`；为 null 时 `:173-177` 再看 `checkDiUpload(gray,6)` 决定用 `getClientId(context)` 还是空串；`:180` `arrayList.add(clientId)` | **`t_cuid` ← `mTPLCuid`** |
| 39 | `:181-183` `if (!diExceptIndex.contains(39) && checkDiUpload(gray,39) && (clientId2 = confignation.mTPLAppName) == null) clientId2 = getClientId(context)`；`:184` add | **`t_appname` ← `mTPLAppName`** |

> 旧文（§15.4.4 原文）写「第 39 位是 `t_cuid`，取 `mTPLAppName`」——**错**，已在本节更正，§15.4.4 同步改写。

#### 16.2.2 `mTPLCuid` 的完整赋值链（已验证）

| 位置 | 动作 |
|---|---|
| `classes2/sources/com/baidu/sapi2/SapiConfiguration.java:83` | 字段 `private String mTPLCuid` |
| 同文件 `:1044-1052` | 三参 setter `setPrivacyParamesConfig(String tAppname, String tCuid, boolean z)`，`:1048` 设 `mTPLAppName = tAppname`、`:1049` 设 `mTPLCuid = tCuid` |
| 同文件 `:832-843` | 两参 setter `setPrivacyParamesConfig(String, boolean)`，**只设 `mTPLAppName`，不设 `mTPLCuid`** |
| 同文件 `:1156-1157` | 拷贝构造里两个字段都复制 |
| `classes15/sources/com/baidu/netdisk/application/C_.java:199` | **全仓唯一的业务调用点**：`setPrivacyParamesConfig("wangpan_drive", _____(), false)` |
| 同文件 `:142-148` | `_____()`：`String cuid = ew._.j`（`:143`）→ 若为空则 `cuid = CommonParam.getCUID(u)`（`:145`）→ null 则空串 |

> `setPrivacyParamesConfig` 全仓只有 3 处（`:832` 两参、`:1044` 三参、`C_.java:199` 调用），无其它业务侧写入点。

#### 16.2.3 `ew._.j` 是什么（已验证）

- 定义：`classes12/sources/com/baidu/netdisk/ew/_.java` 的静态字段 `j`（同 `com.baidu.netdisk.ew._` 包下的启动标志集合）。
- 赋值点只有一个类：`classes12/sources/com/baidu/netdisk/startup/task/l.java`（CUID3 启动任务）：
  - `:50` `ew._.j = DeviceId.getCUID(this.h)`（快路径）
  - `:51` `ew._.l = DeviceId.getDeviceID(this.h)`
  - `:62-66` 优化路径：先读 SP 键 `m.f2512_`，命中就复用，未命中则调 `DeviceId.getCUID` 并回写 SP
  - `:76`/`:79-80` 对 `ew._.l` 做同构处理
- 调用点：只有 `C_.java:143` 读取它。

→ **`ew._.j` 恒为 `DeviceId.getCUID` 的组合串**（不是纯 cuid）。

#### 16.2.4 四种 ID 的区别（不得混用，已验证）

| 方法 | 位置 | 返回内容 | 闸门 / 异常 | 对应字段 |
|---|---|---|---|---|
| `SapiUtils.getClientId(Context)` | `classes12/sources/com/baidu/sapi2/utils/SapiUtils.java:268-282` | **纯 `cuid`**（内部有 `clientId` 缓存；为空则赋 `getDeviceID`） | `ServiceManager`/`isAccountManager`/`confignation` 任一 null → null；既未同意危险协议又非浏览模式 → null | `di` 索引 6、`cuid` cookie |
| 同文件 `getDeviceID(Context)` | `:351-366` | **纯 `cuid`** | 无上述两道闸门；异常时返回 `"123456789" + md5hex(随机)` | `ew._.l` |
| `DeviceId.getDeviceID` | `classes3/sources/com/baidu/android/common/util/DeviceId.java:102-104` | `b(ctx).c()` = 纯 cuid 对象 | 单例双检锁 | — |
| `DeviceId.getCUID` | 同文件 `:98-100` | `b(ctx).v()` = **组合串**（见 16.2.5） | 同上 | `ew._.j` |
| `CommonParam.getCUID(Context)` | `classes3/sources/com/baidu/android/common/util/CommonParam.java:10-11` | 直接 `return DeviceId.getCUID(context)` | 无 | 备用回退 |

> 关键：**`getClientId()` 与 `getCUID()` 不是同一个方法**，前者返回纯 `cuid`、后者返回带 `i()` 的组合串；`getDeviceID()` 与 `getClientId()` 值相同但闸门不同。

#### 16.2.5 `i()` 是否进入最终请求 —— **是**（已验证，逐跳）

| 跳 | 位置 | 事实 |
|---|---|---|
| 1 | `C_.java:143` | `cuid = ew._.j` |
| 2 | `startup/task/l.java:50` | `ew._.j = DeviceId.getCUID(ctx)` |
| 3 | `DeviceId.java:98-100` | `getCUID = b(ctx).v()` |
| 4 | `classes3/sources/com/baidu/cesium/h.java:168-186` | `v()` 拼接：`f797_ + "|" + (__ 为空则 "0" 否则 __)`，当 `__ == "V"` 时再追加 `___`，最后若 `_____` 非空再追加 |
| 5 | 同文件 `:380-402` | 新建时 `__ = "V"`、`___ = i(f797_)`、`_____ = null` → 组合串形如 **`"<cuid>|V<i(cuid)>"`** |
| 6 | `C_.java:199` | 组合串作为 `tCuid` 传入 `setPrivacyParamesConfig` → `mTPLCuid` |
| 7 | `SapiDeviceInfo.java:171` | `clientId = confignation.mTPLCuid` → `:180` 写入索引 38（`t_cuid`） |
| 8 | `SapiDeviceInfo.getDeviceInfo:205-209` | 40 项 `TextUtils.join` 后整体 AES/CBC 加密（密钥硬编码在 Java 侧，见 §3.6）→ 请求中的 `di` |

→ **`i()` 确实进入最终登录请求的加密载荷**，不是「无关字段」，也不是「可选字段」。
→ **`t_appname` 不需要 `i()`**：它是常量 `"wangpan_drive"`（`C_.java:199` 第一实参），只有 `mTPLAppName` 为 null 时才回落 `getClientId`，而业务侧必然传入常量，故实际恒为该常量。

#### 16.2.6 `mTPLCuid` 是否必然非空（分支已闭环）

| 分支 | 条件 | 索引 38 实际值 |
|---|---|---|
| ① | `diExceptIndex.contains(38)` 或 `!checkDiUpload(grayControlParams, 38)` | `""`（空串，非 null） |
| ② | 未排除，且 `mTPLCuid != null` | 组合串（非空） |
| ③ | 未排除，且 `mTPLCuid == null` | `checkDiUpload(gray,6)` 为真 → `getClientId`（可能是 null）；为假 → `""` |
| ④ | 业务侧 `_____()` 两步都失败 | 传入 `""`（**空串而不是 null**）→ `:172` 的 `== null` 判断不成立 → 不进 ③ 分支 |

> 因此：**`mTPLCuid` 在网盘业务侧实际恒非空**（`ew._.j` 由启动任务保证，兜底 `CommonParam.getCUID`），只有「设备指纹整体生成失败」才可能为空串；而空串与 null 在处理上不同（前者不会再走灰度回退）。
> 唯一「必然空」的场景是服务端下发 `diExceptIndex` 包含 38 或灰度开关关闭——这两者都属**动态下发**，静态不可预知。

#### 16.2.7 输出表（本轮交付要求）

| 值 | 生成位置 | 返回内容 | 持久化 | 最终请求位置 | 是否需要 `i()` |
|---|---|---|---|---|---|
| `DeviceId.getCUID` | `DeviceId.java:98-100` → `cesium/h.java:168-186` | `"<cuid>|V<i(cuid)>"` | `dataDir/.cesium/libbh.so`（加密 JSON，字段 `f797_`/`__`/`___`） | `mTPLCuid` → `di` 索引 38 | **需要**（第三段） |
| `DeviceId.getDeviceID` / `SapiUtils.getClientId` | `DeviceId.java:102-104`、`SapiUtils.java:268-282` | 纯 `cuid`（`f797_`） | 同上 | `di` 索引 6、`cuid` cookie | 不需要 |
| `mTPLAppName` | `C_.java:199` 字面量 | `"wangpan_drive"` | 配置对象 | `di` 索引 39 | 不需要 |
| `i(cuid)` | `cesium/h.java:274-282`（`cesium.a.C_` 4 算子 + `cesium.a.e` 反射 PRNG + 自定义 Base32） | Base32 串 | 随 `___` 落 `libbh.so` | 组合进索引 38 | 自身即 `i()` |
| `zid` / `sf_zid` | `SapiSafeFacade.java:23-27` → `sofire/d/b.java` | 五种分支（见 §16.3.4） | SP / 文件 / 静态缓存 | 公共参数 `zid` + `di` 索引 34 | 不涉及 `i()` |

#### 16.2.8 Rust 复刻口径的收窄（重要）

1. **组合串结构已确定**：新建路径 `__ = "V"`、`_____ = null` → `v()` 输出 `"<cuid>|V<i(cuid)>"`。旧文「伴生字段 `__` 当前恒 `"O"`」**错误**——`InterfaceC0042a.__ = "O"` 只是常量声明，新建（`h.java:398`）与 JSON 读回（`h.java:214` 默认值）两条路径都取 **`"V"`**。
2. **`i()` 不能省略**：既然它进入最终请求，Rust 若不复刻，则 `t_cuid` 段与官方客户端**必然不同**。这比 §15.4.6 ⑤ 的「可先用空/占位」更严格，风险从「可能不一致」上升为「一定不一致」。
3. **仍有可复刻路径**：`i()` 的算法全部在 Java 侧（§15.4.5 已逐类定位），无 native，属「可移植但工作量与出错风险大」；本机无 Java 运行时，**本轮无法产出测试向量**。
4. **不得据此断言服务端必校验**：服务端是否校验 `i()` 段仍属**动态验证**（见 R29），本轮不改变这一判断，也不把它写成「必需」。
5. 边界不变：Rust 负责协议与算法，Kotlin 只提供 `Settings.Secure.ANDROID_ID`、`Build.MODEL` 等通用系统信息；百度专有参数不进 Kotlin。

### 16.3 `sf_zid` / 公共参数 `zid`

#### 16.3.1 调用链（已验证，4 跳）

| 跳 | 位置 | 事实 |
|---|---|---|
| 1 | `classes2/sources/com/baidu/sapi2/SapiSafeFacade.java:23-27` | `getCurrentZid(ctx)`：取 `currentAccount.uid`（无账号则 null）→ `getZidAndCheckSafe(ctx, uid, 1)`；结果为空则返回 `"NoZidYet"` |
| 2 | 同文件 `:35-37` | 转 `SafeService.getInstance().getZidAndCheckSafe` |
| 3 | `classes12/sources/com/baidu/sapi2/utils/SafeService.java:27-30` | `String strGzfi = com.baidu.sofire.ac.FH.gzfi(ctx, str, i)`；`:29` 空则 `"NoZidYet"` |
| 4 | `classes12/sources/com/baidu/sofire/d/b.java:645-679` | `a(ctx,str,i,str2)` 七分支（下表） |

#### 16.3.2 `b.a(...)` 七个分支（已验证，命中即返回）

| # | 条件 | 返回 |
|---|---|---|
| ① | `!l.k.a(context)`（sofire 不可用） | `e(context)`（纯 Java 兜底，`:443-466`） |
| ② | 静态字段 `a` 非空（`:7` 缓存） | 若 `i != 0` 先调 native `a(1,"ice",...)`，随后返回缓存值 |
| ③ | `!l.b.a(context, false)` | `""`（空串） |
| ④ | `l.b.c(context)` 为空 | `l.b.d(pkg)` 为真 → `""`；否则 `b(...)` |
| ⑤ | `l.b.d(strC)` 为真 | 调 native `a(context,0,1,"gzfi",null,str,i,str2)`；`Pair` 为 null 或 `first != 0` → `""`；否则取 `second` |
| ⑥ | 其余 | `b(ctx,str,i,str2)`（`:359-378`） |
| ⑦ | 抛 `Throwable` | `""`（空串） |

#### 16.3.3 `d()` / `e()` 的内部顺序与常量（已验证）

- `d(context)`（`:408-441`）：①`k.a.a(ctx).e()` ②`k.a.a(ctx).c()` ③读 `l.d.a(context)`，按 `"|"` split 成两段，用 `F.getInstance().re(seg[1].getBytes(), seg[0].getBytes())` 解密，成功后 `seg[0] + l.b.a(bytes)` 落 SP 并写静态 `a`；④全部失败 → `a = 62 位十六进制常量`。
- `e(context)`（`:443-466`）：同构，但用 `l.b.d(context) == 0 ? l.d.b(context) : l.d.a(context)`，**不写静态缓存**，末尾统一返回同一常量。
- **常量实测 62 个十六进制字符**：`74FFB5E615AA72E0B057EE43E3D5A23A8BA34AAC1672FC9B56A7106C57BA03`（python `len()` = 62）。旧文两处「56 位」**错误**，已更正。
- native 参与点：`F.getInstance().re` 走 `com.baidu.sofire.l.e.a`（解密需要 `libfire`）；`System.loadLibrary("fire")` 只出现在 `b.java:78` 与 `:92` 的 DEBUG 自检分支；「`libsofiresec.so` 的 SONAME 是 `libsofire.so`，Java 侧无 `loadLibrary("sofiresec")`」。
- `l.k.a(context)`（`classes12/sources/com/baidu/sofire/l/k.java:6-38`）：综合 `l.b.d`、SP 键 `s_a_pl`、标记文件 `files/.tmp_sofire_st/.ffnpp` 判定 sofire 可用性，结果写静态 `a`（1 可用 / 2 不可用）。

#### 16.3.4 输出表（本轮交付要求）

| 环境 / 分支 | `SapiSafeFacade` 返回值 | `di` 的 `sf_zid` | 公共参数 `zid` | 是否需要 native |
|---|---|---|---|---|
| `libfire` 完全不可用（分支 ①） | `e()` 兜底值：缓存 / SP 文件解密值 / **62 位常量** | 同左（除非 `diExceptIndex` 排除 34 → `""`） | 同左 | **否**（走纯 Java 失败路径，代价是拿不到真实值） |
| sofire 可用 + 静态缓存非空（分支 ②） | 缓存值（可能就是真实 zid） | 同上 | 同上 | 视缓存来源，可能否 |
| `l.b.a(ctx,false)` 为假（分支 ③、⑦） | `""` → `SapiSafeFacade` 转 `"NoZidYet"` | `"NoZidYet"` | `"NoZidYet"` | 否 |
| native `gzfi` 调用成功（分支 ⑤） | 真实 zid | 同上 | 同上 | **是** |
| `diExceptIndex` 含 34 或灰度关闭 | 仍返回真实/兜底值 | **`""`（空串）** | 仍有值 | — |

> **重要**：公共参数 `zid` 与 `di` 索引 34 的 `sf_zid` **同源同值**，但**不保证同时非空**——`di` 侧受 `diExceptIndex` 与灰度开关控制，公共参数侧无条件写入。旧文「两者必然一致」的说法需按此修正。

#### 16.3.5 两处取值点（已验证）

- `classes4/sources/com/baidu/sapi2/httpwrap/Utils.java:5-22` `buildCommonParams`：`:13` `clientId = SapiUtils.getClientId(ctx)`、`:17` `map.put("zid", isAccountManager.getCurrentZid(ctx))`。调用方只有 `HttpHashMapWrap.java:6` 与 `MultipartHashMapWrap.java:6`。
- `SapiDeviceInfo.java:161` 索引 34：`(diExceptIndex.contains(34) || !checkDiUpload(gray,34)) ? "" : isAccountManager.getCurrentZid(context)`。
- `SapiDeviceInfo.java:242-249`：若 `sf_zid` 仍是 `"NoZidYet"`，会用参数 **120**（不是 1）再调一次 `getZidAndCheckSafe` 覆盖，说明官方自己也把「拿不到 zid」当异常态处理。

#### 16.3.6 `zid` 的 Rust 复刻结论

| 类别 | 结论 |
|---|---|
| 可复刻 | 七分支的**判定逻辑**与兜底（空串 / 62 位常量）完全可复刻，纯 Java 无 native 依赖 |
| 不可静态复刻 | **真实 zid**：需要 `libfire`（native）+ 服务端下发的 sofire 数据文件；本轮不移植 native 也就拿不到真实值 |
| 不得简化 | **不能把 `"NoZidYet"` 写成唯一取值**，也不能把 62 位常量写成 56 位；实际值取决于环境分支 |
| 不得臆造 | 未找到任何「客户端自造 zid」的实现；本项目只能用兜底值或空串，服务端是否接受属动态验证（R29） |

### 16.4 人脸 native 对登录请求的实际影响（表述校正）

#### 16.4.1 五个问题的直接回答

| # | 问题 | 回答 | 证据 |
|---|---|---|---|
| 1 | 加载 `libbaiduprotect_sec_jni_facect.so` 是否影响 `supportFaceLogin` | **是** | `classes2/sources/com/baidu/sapi2/SapiAccountManager.java:131-154`：`:149` `Class.forName("com.baidu.pass.face.platform.FaceSDKManager")` + `PassBiometricCall().initPassBioSDK`；`:153` catch 分支 `sapiConfiguration2.supportFaceLogin = false` |
| 2 | `supportFaceLogin` 是否影响登录/发码请求字段 | **是，但只影响三个可选能力参数** | `classes2/sources/com/baidu/sapi2/SapiAccountRepository.java` 五组：`:801-803`（checkUserInfoByUid）、`:1549-1551`（getLoginSms 发码）、`:1652-1654`、`:1795-1797`、`:2885-2887`（`wapLogin`，A 与 B 共用）；每组紧接着 `support_photo` 由 `supportPhoto` 单独控制 |
| 3 | native 加载失败是否被 catch 后降级 | **是** | 同 `:149-154`，catch `Exception` 后仅置标志并打日志（文案「VIS SDK可能未集成…」），不中断初始化、不抛给调用方 |
| 4 | 这些字段是否只是可选人脸能力参数 | **是** | `liveAbility`/`scanface`/`supFaceLogin` 只在人脸相关 UI 与能力判定处被读取（`SapiWebView`/`CoreViewRouter`/`SapiUpgradeInterpreters`/`SapiAccountService.java:280`、`:869`）；扫码路径 `QrCodeService.java` 全文无 `supportFaceLogin` 引用 |
| 5 | 该库是否参与账号密码/短信/扫码的核心认证、签名、设备指纹 | **否** | 类引用闭包 + `NEEDED` 依赖图 + 无 `dlopen` 字面量三重核对（§15.1.6）；`doSign` 唯一调用点 `HttpClientWrap.java:87` 与它无关；`di` 40 字段构造（`SapiDeviceInfo.java:67-187`）与它无关 |

#### 16.4.2 校正后的准确表述

- ✅ **核心认证不依赖该 native**：RSA 密码加密、`sig` 签名、`di` 构造、cookie 读写全在 Java 侧。
- ✅ **可选人脸能力参数会受它影响**：加载失败时请求少 `liveAbility`/`scanface`/`supFaceLogin` 三个字段（值为 1 还是缺失，取决于 `supportFaceLogin`）。
- ❌ 旧文「不影响认证或请求构造」（§15.1.6 表末行）**表述过强**，已在本节改为「不影响认证与签名；影响三个可选能力参数是否出现」。
- ❌ 也不得反向夸大：这**不是认证阻断**，缺失这三个参数不阻止任何登录路径（官方自有降级路径，catch 后照常继续）。

### 16.5 验证码 WebView 的统一判定口径

#### 16.5.1 项目规则（本次重申，不新增规则）

| 情形 | 判定 |
|---|---|
| WebView 负责账号/密码登录、会话换取、cookie/token 提取、网页端会话复用 | **禁止**，该路径判不合格 |
| WebView 仅用于条件触发的人机验证，票据返回原生层，之后仍由 Rust Adapter 通过官方客户端协议发起认证 | **不因 WebView 存在而降级**；记为「条件性第三方验证依赖」（先例：坚果云 R27） |

#### 16.5.2 百度短信路径的完整回传链（已验证，8 步）

| 步 | 位置 | 事实 |
|---|---|---|
| 1 | `classes18/sources/com/baidu/validation/ValidationManager.java:139-149` | `openValidation` 只 `new Intent(ctx, ValidationActivity.class)` + `putExtra("source", str)`，**不传任何凭据** |
| 2 | `activity/ValidationActivity.java:317-323` | `:321` `loadUrl(UrlUtils.splicingParams(UrlUtils.splicingCommonParams("https://wappass.baidu.com/static/activity/pass-machine.html"), this.i))`；`this.i` 只含 `ak`/`type`/`scene`/`tk`/`as`/`ds` + extraParams + timestamp（`:280-303`） |
| 3 | 同文件 `:172-181` | `e()` 里 `:179` `evaluateJavascript("javascript:window.machine_sdk.show()", ...)` |
| 4 | 同文件 `:145-152` | 该 `ValueCallback.onReceiveValue(String)` **空实现**（JS 执行结果不参与后续） |
| 5 | 同文件 `:28-55` | JS → 原生走 **`onJsPrompt`**：`:31` `js.__.m149_(str2)` 解析 `{"action":{"name":...,"args":[...]}}`；`:36` 反射实例化 `com.baidu.validation.js.interpreter.<CamelCase>`；`:38` 设回调；`:40` `interpret`；`:54` `jsPromptResult.confirm(...)`。**不是 `addJavascriptInterface`**（`com/baidu/validation` 下该方法 0 命中） |
| 6 | `js/interpreter/ActionValidationResult.java` | `interpret` 取 `args.get(0)` → `ValidationResult.parseFromJson` → `mInterpreterCallback.a(mName, result)` |
| 7 | `ValidationActivity.java:353-376` | switch 三 case：`action_validation_view_setting`（改高度）、`action_load_external_webview`（起外部 WebView）、`action_validation_result` → `ValidationManager.getInstance().mValidationCallback.onFinish(result)` |
| 8 | `ValidationService.java:32-58` + `SapiAccountRepository` | `startValidation` 回调只映射 `ds`/`tk` 回业务；失败码 50052 的处理里把 `ds`/`tk` 写回参数表、`validation_times` 加一，再调 `onAgainRequestPwdLogin` → 之后仍由原生 `POST {wap}/wp/api/login` 完成认证 |

#### 16.5.3 回传内容的边界（已验证，「没有」也要给搜索方式）

| 检查项 | 结果 | 搜索方式 |
|---|---|---|
| 回传是否含账号 / 密码 / 短信验证码 | **否**。`ValidationResult` 字段只有 `errno`、`msg`、`data{ds,tk}`（`result/ValidationResult.java:4-21`） | 读该类全部字段与 `KEY_*` 常量 |
| 是否读写 cookie / BDUSS / STOKEN / PTOKEN | **否** | `grep -rnE 'CookieManager|setCookie|getCookie|bduss|stoken|ptoken' reverse/baidu/sources/classes18/sources/com/baidu/validation` → **0 命中** |
| 是否用 `addJavascriptInterface` 注入桥 | **否**，走 `onJsPrompt` | 同上目录 `grep -rn 'addJavascriptInterface'` → **0 命中**（`classes18` 内仅 `com/baidu/webkit` 的 `ZeusJsBridge` 与 facebook/gms 第三方） |
| WebView 是否自行提交登录表单 | **否**。页面是 `pass-machine.html` 人机挑战页，登录接口调用只出现在原生 `SapiAccountRepository` | 反查 `SapiAccountRepository` 的 `/wp/api/login` 调用点 |
| 挑战通过条件能否静态复刻 | **否**（服务端下发 + 页面内 JS 决定），属动态 | — |

#### 16.5.4 账号密码路径（A）确实存在同一套验证分支（**新发现，旧文未写**）

| 位置 | 事实 |
|---|---|
| `classes4/sources/com/baidu/sapi2/newlogin/helper/NewLoginUtil.java:2001` | 方法 `a(Context, String, String, NewNALoginResult, HashMap<String,String>, IPwdFailListener)`，密码登录失败后的统一分支处理 |
| 同文件 `:2016-2019` | 非 50052 时先 `ValidationServiceUtil.finishValidation()` 并清空 `validation_times`/`ds`/`tk` |
| 同文件 `:2107-2145` | **50052**：若 `ValidationServiceUtil.isAvailable()` 为假 → `a(str, result, ctx)`（整页网页登录）；否则 `:2113` `startValidation`，回调写回 `ds`/`tk`、`validation_times + 1`、`:2130` `onAgainRequestPwdLogin(map)` 重试原生登录 |
| 同文件 `:2049-2051` | 400101 / 400023 / 400031 / 400408 / 400035 → `startWebByUrl`（web 兜底） |
| 同文件 `:2106-2108` | 400041 / 500072 → web 兜底 |
| 同文件 `:2148-2162` | 340002 / 500001 / 500002 / 500020 / 500021 / 400605 / 402008 / 402013 → `CoreViewRouter.startLogin`（**整页网页登录**） |
| 其它错误码 | 400044 / 400012 弹申诉对话框、400036 弹解绑对话框、50028 / 50029 / 400714 仅提示，**都不走 WebView 登录** |

→ 结论：**A 路径也会遇到人机挑战（50052）**，走的是与 B 完全相同的 `ValidationService` 机制；这是「条件触发」，不是「每次登录必经」，静态无法判断触发频率。

#### 16.5.5 与坚果云 R27 的口径对照（统一后）

| 项 | 坚果云（R27，既有判定**不变**） | 百度（本次校正后） |
|---|---|---|
| WebView 承载什么 | 腾讯验证码 H5（`TencentCaptchaView` 加载 `assets/TencentCaptchaH5.html`），只出 `ticket`/`randstr` | `pass-machine.html` 人机挑战页，只出 `ds`/`tk`/`errno`/`msg` |
| 触发条件 | 服务端 `isNeedRealCaptcha()`，**非每次必经** | 服务端错误码（50052 等），**非每次必经** |
| 认证与 token 由谁换 | 仍由原生 `/mcmd/loginV2` | 仍由原生 `POST {wap}/wp/api/login` |
| 是否构成禁止形态 | **否** | **否** |
| 是否因此降级 | 否 | **不应仅因此降级**（旧文「因为用了 WebView 所以 B 必须降级」的理由作废） |
| 额外差异 | 无网页登录兜底记录 | **官方另有整页网页登录/网页短信登录兜底分支**（见下表），本项目对这些场景只能显式失败 |

#### 16.5.6 必须如实记录的例外：整页网页登录兜底

| 路径 | 触发错误码（均为**服务端返回**，非客户端选择） | 本项目处理 |
|---|---|---|
| A 账号密码 | 400101 / 400023 / 400031 / 400408 / 400035 / 400041 / 500072 / 340002 / 500001 / 500002 / 500020 / 500021 / 400605 / 402008 / 402013（以及 50052 且 `ValidationService` 不可用） | **判不支持并明确返回失败**，不得跳网页登录 |
| B 短信 | 72200 / 402012 / 400704 / 400706 / 400418 / 400425 / 500072 / 402008 / 402013 / 400605（以及 50052 且不可用） | 同上 |

> 这些分支的本质是「服务端要求转入网页流程」，属账号级/风控级异常，任何第三方客户端都无法在原生协议内完成；把它们实现为「明确失败 + 可读错误」是正确做法，不是协议缺口。

#### 16.5.7 校正后的 A / B / C 结论

| 路径 | 档位 | 理由（校正后） |
|---|---|---|
| A 账号密码 | **可独立实现** | 常规链 8 跳全在原生协议内；50052 人机挑战与坚果云 R27 同级（条件性第三方验证依赖）；网页登录兜底分支判显式失败。**新增说明，不改档位** |
| B 短信验证码 | **需攻关** | 降级理由**改写**：不再写「违反第 9 条禁止 WebView」，改为 ①人机挑战的**通过条件由服务端与页面 JS 决定，静态不可复刻**（与 R27 同级，但 B 的触发点更多：发码与提交两段都有）②**是否接受内嵌同一 H5 作为人工验证环节尚未由用户拍板**——若接受，B 与 A 同级；若不接受，B 维持需攻关 ③网页短信登录兜底分支判显式失败 |
| C 扫码 | **可独立实现（带动态前提）** | 五段全明；`hostDeviceId` 官方恒 `null`；残留只有「服务端是否接受该参数缺席」这一动态前提。**本轮无变化** |
| 整体 | **原生登录可行（静态可行，动态未验证）** | 成立条件：走 A 或 C |

> **B 的档位需用户拍板**（这是唯一一处不自行定稿的判定）：若把 R27 的口径严格套到 B 上，B 与 A 同级；两者的差别只在「百度官方对短信路径还有网页短信登录兜底」。本报告按保守侧记为「需攻关」，并在 §16.1 与 R30 中同时写明这一分歧点。

### 16.6 对 §15 与 §1–§14 的更正登记（原文不删）

| 原位置 | 原文摘要 | 本节实测 | 处理 |
|---|---|---|---|
| §15.4.4（:1077） | 「`getCookiesKeyList()` 第 39 位是 `t_cuid`；`buildDeviceTokens` 第 39 位取 `mTPLAppName`。→ `t_cuid` 的值是 TPL 子应用名」 | 第 **38** 位才是 `t_cuid`（← `mTPLCuid`），第 **39** 位是 `t_appname`（← `mTPLAppName`） | **已改写 §15.4.4**，以 §16.2.1 为准 |
| §15.4.6 行 ③（:1095） | 伴生字段 `__`（当前恒 `"O"`） | 新建（`h.java:398`）与读回（`h.java:214` 默认值）均为 **`"V"`**；`InterfaceC0042a.__ = "O"` 只是常量声明，未被这两条路径使用 | **已改写** |
| §15.4.6 行 ⑤（:1097） | 派生字段 `i()` 当前无法精确复刻，若服务端不校验可先用为空/占位 | `i()` **确实进入最终请求**（§16.2.5），不复刻则 `t_cuid` 段必与官方不同 | **已改写**（风险升级为「一定不一致」，但仍不据此断言服务端必校验） |
| §15.4.6 边界句（:1099） | 「`t_cuid` 取 TPL 子应用名这条也属 Rust 侧常量」 | 错：`t_appname` 才是常量；`t_cuid` 是设备侧组合串 | **已改写** |
| §3.3 表行 34（:211） | `sf_zid` 失败返回 `"NoZidYet"`；兜底为「硬编码 **56 位** hex 常量」 | 兜底常量实为 **62 位**；取值有五种分支，不恒为 `"NoZidYet"` | **已改写**，分支见 §16.3.4 |
| §4.5（:337） | 「返回硬编码常量（**56 位**十六进制）」 | **62 位** | **已改写** |
| §15.1.6 表末行 | 「是否影响认证或请求构造：**否**」 | 认证/签名：否；**可选人脸能力参数：会受影响** | **已改写**，见 §16.4 |
| §7 / §15.7.2 | B 降级理由为「本项目按 AGENTS.md 第 9 条禁止用 WebView 完成登录」 | WebView 只做条件触发的人机挑战、票据回原生、认证仍原生 → **不构成禁止形态** | **理由已改写**，档位暂不自行提升（待拍板） |
| §6（A 路径） | 只记前置 `cap/init` 失败仍继续，未提其它验证分支 | A 在错误码 50052 下同样走 `ValidationService` 人机挑战并重试 | **新增 §16.5.4** |
| 全文日期 | 多处「2026-11-28」 | 本轮补充收口实际发生在 **2026-11-28** | **全部改为 2026-11-28** |

## 17 zid 的 native 依赖与二次查询闭环（2026-11-28，当前有效入口）

> 本节是 **zid / sf_zid 相关问题的当前有效结论入口**。§1.2、§12.2、§16.3 关于「三条路径均不依赖闭源 native」「zid 可精确复刻」的表述**已被本节取代**（差异见 17.9）。其余章节保留为历史证据，不删。
>
> 本轮**只做静态读码复核**：不发请求、不登录、不抓包、不改生产代码、不移植任何闭源 so。

### 17.0 为什么需要本机复核

§16.3 曾把 `sf_zid` 的七分支与五种取值穷举完，但**没有回答「这些取值在没有官方 native 的情况下能不能拿到」**。§16.3.6 的「Rust 复刻结论」把 `zid` 写成「不能精确复刻，但可用纯 Java 兜底」，却没有证明兜底分支本身不经过 native。本轮把这条缺口补上，并顺带闭环「`di` 二次查询是否会改变最终值」。

### 17.1 zid 的七条数据来源与 native 依赖

| 数据来源 | 是否需要 native | 具体依赖点 | 不存在 native 时的结果 |
|---|---|---|---|
| 静态缓存 `com.baidu.sofire.d.b.a` | **读取不需要，但写入来源依赖** | 静态字段只被 `d()`/`e()`/`c()` 的返回值或 62 位常量赋值（`sofire/d/b.java:401`、`:433`、`:436`、`:465`）；监听同步点 `sofire/d/a.java:57` 的 `onSharedPreferenceChanged` → `b.a = this.b.e()`，取值仍来自 `e()` | 进程内初值为空串（`b.java:7`），无 native 时永远停在初值或常量，拿不到真实值 |
| SP 文件数据 `xytk` / `xytk_m` | **读取不需要，但写入由 sofire 引擎/服务端决定** | `sofire/k/a.java:74-76` 读 `xytk`、`:66-68` 读 `xytk_m`，都是纯 SharedPreferences 读；但 `xytk` 的唯一 Java 写入点是 `androidx/core/hardware/refios/FocusHighlightNp.java:176`（`isEdgeTouched` → `putString("xytk")`），唯一调用方 `sofire/x0/EngineImpl.java:346`，键来自 `:311 sofirePreferences.delete()` 返回的 SP `xytk2`；`k/c.java:31-40` 在服务进程可用时把 SP 读写转成 ContentProvider `sofire` 的 `CallPreferences` 调用 | 恒为空串（纯 Java 侧没有自造点，`grep` 全仓 `putString("xytk"` 只有上述包装类） |
| **SP 文件解密 `xyusec`** | **是，100% 依赖** | `sofire/l/b.java:257-266` `c(String)` = `new String(l.e.a(l.d.a, Base64.decode(str,10), true), "UTF-8")`；`l/e.java` 的 `a(...)` 转调 `com.baidu.sofire.jni.Asc.dc/dcn`；`Asc` 静态块 `System.loadLibrary("fire")` | 返回空串（异常被吞），分支继续往下 |
| **SP 文件加密 `xyus`** | **是，100% 依赖** | `sofire/l/d.java:12-62` 用 `l.e.b(a, str.getBytes("UTF-8"), true)` → `Asc.ac/acn` | 加密失败返回 `byte[0]`，`Base64.encode` 得空串 → 写不进 `xyusec`，下次仍走 UUID 分支 |
| 服务端下发数据 | **是** | 服务端下发的 zid 只能经 sofire 服务进程写入 SP（`k/c.java` 的 ContentProvider 路径）或经 native 解密后写入 `xytk_m`；`xytk` 由 `EngineImpl` 写入，其值来自服务端 | 拿不到，恒空 |
| 62 位硬编码常量 | **否** | `sofire/d/b.java:401`、`:433`、`:436`、`:465` 直接返回字面量 `74FFB5E615AA72E0B057EE43E3D5A23A8BA34AAC1672FC9B56A7106C57BA03` | **可取，但这是兜底值，不是真实 zid** |
| 空串 | **否** | 七分支中的 ③（`!l.b.a(ctx,false)`）、⑤（`Pair` 空或 `first != 0`）、⑦（`catch Throwable`）直接返回 `""` | 可取；到上层由 `SapiSafeFacade.java:23-27` / `SafeService.java:27-30` 合成为 `"NoZidYet"` |
| `"NoZidYet"` | **否** | 不是 sofire 的返回值，是上层对空串的兜底文案 | 可取 |

**结论一**：七条来源里，**SP 文件解密与加密两条硬依赖 native**，服务端下发数据一条**只能经 native 或服务进程**进入本地，静态缓存一条**写入来源仍依赖上述路径**。纯 Java 侧真正能独立产出的只有 **62 位常量** 与 **空串** 两种。

### 17.2 `F.getInstance().re` 是否直接依赖 native

**是，直接且唯一。** 证据链（全部为静态读码）：

1. `reverse/baidu/sources/classes12/sources/com/baidu/sofire/ac/F.java:248-284` `public byte[] re(byte[] bArr, byte[] bArr2)`：取静态字段 `com.baidu.sofire.l.e.a`（类型是 `com.baidu.sofire.jni.Asc`）→ `asc.ar(bArr, bArr2)`；结果为空或抛异常一律 `return new byte[0]`。方法体内**没有任何 Java 解密实现**，也没有第二分支。
2. `reverse/baidu/sources/classes12/sources/com/baidu/sofire/l/e.java`：`public static com.baidu.sofire.jni.Asc a = new com.baidu.sofire.jni.Asc();`；`a(byte[],byte[],boolean)` 走 `asc.dc`/`asc.dcn`，`b(byte[],byte[],boolean)` 走 `asc.ac`/`asc.acn`，失败都返回 `new byte[0]`。
3. `reverse/baidu/sources/classes12/sources/com/baidu/sofire/jni/Asc.java:5-8`：`static { try { System.loadLibrary("fire"); } catch (Throwable) {...} }`，10 个 native 方法 `fai` / `ac` / `acn` / `ar` / `dc` / `dcn` / `df` / `dr` / `it` / `itb`。

**所以 `re()` = 加壳的 native 调用**：`System.loadLibrary("fire")` 失败时静态块吞掉 `Throwable`，之后任何 `ar()` 调用都会抛 `UnsatisfiedLinkError`，被 `re()` 的 catch 转成 `byte[0]`。这不是「纯 Java 兜底」，而是「native 不可用 → 空结果」。

### 17.3 SP 文件解密必然经 native 的直接证据

调用点只有两处，且都直接落到 native：

| 位置 | 代码 | 落到哪 |
|---|---|---|
| `sofire/l/b.java:257-266` `c(String)` | `new String(com.baidu.sofire.l.e.a(com.baidu.sofire.l.d.a, Base64.decode(str, 10), true), "UTF-8")` | `Asc.dc`（native） |
| `sofire/l/d.java:12-62` `a(ctx)` | 读 `xyusec` → `l.b.c(...)`；否则读 `xyus` → `l.e.b(a, str.getBytes("UTF-8"), true)` 加密回写 | `Asc.dc` 与 `Asc.ac`（native） |

`sofire/d/b.java:408-441` `d(context)` 的三段式也说明同一件事：读 `xytk`（空）→ 读 `xytk_m`（空）→ `l.d.a(ctx)` 取 `A|B` 两段 → `F.getInstance().re(seg[1].getBytes(), seg[0].getBytes())` 解密，**成功分支才写 `xytk_m` 并返回**；任一环节失败即落到常量。

### 17.4 关键因果链：passport_core → sofire 引擎 → 纯 Java 兜底

这是本轮最重要的发现，它把「passport_core 在三条主链 0 调用」与「zid 依赖 native」连成一条链，也解释了为什么「passport_core 不阻断登录」与「zid 拿不到真实值」可以同时成立：

```
本项目不使用官方 native
  → System.loadLibrary("passport_core") 失败（PassCoreVHelper.java 静态块）
  → SapiConfiguration.java:550-575 的 getSofireAppKey()/getSofireSecKey() 抛异常被吞
  → sofireAppKey / sofireSecKey 为 null
  → FH.init 拿不到 key，sofire 引擎起不来
  → sofire/l/k.java:6-38 判定「不可用」（SP 键 s_a_pl + files/.tmp_sofire_st/.ffnpp 标记）
  → sofire/d/b.java:645-679 七分支的第 ① 支（!l.k.a(context)）直接走 e(context)
  → e(context) 三段式全部拿不到值 → 返回 62 位常量
  → 上层 getCurrentZid 非空，di 索引 34 与公共参数 zid 都是这个常量
```

**含义**：不使用官方 native 时，百度登录**不会卡在「引擎起不来」而报错**，而是**静默降级到常量**。这既降低了实现难度，也意味着发出去的 `zid` 一定不等于官方真实值。

### 17.5 四问回答

| # | 问题 | 回答 |
|---|---|---|
| 1 | 项目完全不使用官方 native 时，能否稳定生成「官方客户端实际存在」的一种合法 zid 值 | **不能**。能稳定产出的只是官方**兜底分支**的值（62 位常量），或空串（上层转 `NoZidYet`）。二者都是官方代码路径的产物，但都不是官方**正常运行时**实际发出的 zid |
| 2 | 该值是否只能是 62 位常量或空值 | **是**。没有第三种。UUID 分支产出的是 `"<uuid>|0"` 这类**未加密的本地种子**，它随后必须经 `l.e.b(...)` 加密才能写进 `xyusec`，无 native 时加密返回 `byte[0]`，写不进去，所以它不会成为 zid |
| 3 | 是否有其他静态可得的合法值 | **没有**。已穷举 `xytk`、`xytk_m`、`xyus`、`xyusec`、静态缓存、ContentProvider 六条通道，全部要么依赖 native，要么依赖服务进程。全仓 `loadLibrary("sofire")` / `loadLibrary("sofiresec")` **0 命中**，说明 Java 侧没有第二条自造 zid 的路径 |
| 4 | 服务端是否接受这些值 | **待动态验证**（R29）。本轮不发请求，不做任何推测。**不接受「兜底值等价于真实值」的说法** |

### 17.6 di 二次查询的阶段表

`SapiDeviceInfo.java:242-249` 的第二次 `getZidAndCheckSafe(ctx, uid, 120)` 只发生在 `sf_zid` 为 `"NoZidYet"` 时。完整时序（全部静态可读）：

| 阶段 | 公共参数 `zid` | `di` 索引 34（`sf_zid`） | 是否已加密 | 是否已签名 |
|---|---|---|---|---|
| ① `HttpHashMapWrap()` 构造 | `httpwrap/Utils.java:17` 调 `getCurrentZid(ctx)`（**事件 id 1**）写入 map | 尚未生成 | 否 | 否 |
| ② 业务方法 put 各字段（A `SapiAccountRepository.java:484-535` / B `:2710-2723`） | 不变 | 尚未生成（**A/B 都不 `put("di")`**） | 否 | 否 |
| ③ `wapLogin` 求值 `ParamsUtil.buildNaCookie` | 不变 | `ParamsUtil.java:75` → `SapiContext.java:375-378` → `getDiCookieInfo(loginCookieDiKeys)` → `SapiDeviceInfo.java:67` `buildDeviceTokens` 在 `:161` 调 `getCurrentZid`（**事件 id 1**） | 在 `:229-254` 末尾 `z ? encryptDeviceInfo(...)` 处加密 | 否 |
| ④ 若 `sf_zid == "NoZidYet"` 重查 | 不变（**不回写 map**） | `:242-249` 用**事件 id 120** 再取一次并 `jSONObject.put("sf_zid", ...)` 覆盖 | 覆盖发生在**加密前**（同一函数内紧接着就 `encryptDeviceInfo`） | 否 |
| ⑤ `buildNaCookie` 写 cookie `DVIF` | 不变 | 已定稿进 `DVIF` | 已加密 | 否 |
| ⑥ `HttpClientWrap.buildParamDTO` → `doSign` | **参与签名**（`SapiUtils.java:135-160` 遍历整张 map） | 不参与（在 cookie 里，不在 map 里） | — | **此时签名** |

### 17.7 五项重点判断

| # | 判断 | 结论 | 依据 |
|---|---|---|---|
| 1 | 两次 `zid` 获取是否可能得到不同值 | **可能不同**。事件 id 不同（1 与 120），服务端可能按事件返回不同内容；两次之间也可能已有静态缓存 `b.a` 被写入 | `SapiSafeFacade.java:23-27`、`SafeService.java:27-30`、`SapiDeviceInfo.java:242-249` |
| 2 | 公共参数与 `di` 最终是否可能不同 | **A/B 上不会互相覆盖**（第二次只写 `jSONObject` 的 `sf_zid`，不回写 map）；但两者**取值可能不同**，因为第一次是事件 1、第二次是事件 120。C 扫码不生成 `di` | 同上 + `QrCodeService.java:306-334` 传 `cookie = (List) null` |
| 3 | `di` 二次查询是否发生在 AES 加密前 | **是**。`:242-249` 在 `:229-254` 的同一个函数体内，`encryptDeviceInfo` 在函数末尾 | `SapiDeviceInfo.java:229-254` |
| 4 | `zid` 是否参与公共参数签名 | **参与**。`SapiUtils.calculateSig` 对整张 map 排序拼接；`DVIF` 在 cookie 里，**不参与** | `SapiUtils.java:135-160`、`HttpClientWrap.java:84-105` |
| 5 | A/B/C 三条路径是否有差异 | **A 与 B 无差异**（都不 `put("di")`，都走 `buildNaCookie` → `DVIF`）；**C 不同**（`getQrCodeImage`/`getQrLoginResult` 传 `cookie = null`，**完全不触发 DVIF、不触发二次查询**），但公共参数 `zid` 仍取一次并参与签名 | `SapiAccountRepository.java:484-535`、`:2710-2723`、`QrCodeService.java:306-390` |

补充：全仓 `put("di")` 只有 8 处——`AbstractService.java:18`、`OpenBdussService.java:67`、`SapiAccountRepository.java:269`（`realDynamicPwdLogin` 旧链）、`:541`（`buildSapiParams`）、`:3272`、`:3385`、`PassLivenessRecogActivity.java:1850`、`:2089`，**均不在 A/B/C 三条主链上**。另 `SapiCache.java:29-35` 的 `reportDi()` 会在 `/v8/sdkconfig/init` 成功或失败时各触发一次 `di` 生成（含二次查询），**早于任何登录请求**；这是埋点上报，不影响登录请求本身。

### 17.8 A/B/C 三条路径的实际参数差异

| 项 | A 账号密码 | B 短信验证码 | C 扫码 |
|---|---|---|---|
| 公共参数 `zid` | 有（事件 1） | 有（事件 1） | 有（事件 1） |
| 请求体 `di` | **无** | **无** | **无** |
| cookie `DVIF` | 有（含 `di` 二次查询） | 有（含 `di` 二次查询） | **无**（`cookie = null`） |
| 是否参与签名 | 是（含 `zid`） | 是（含 `zid`） | 是（含 `zid`，扫码也 `doSign`） |
| 是否触发人机挑战 | 条件触发（50052） | 发码 + 提交两段均条件触发 | 否 |

→ **对 zid 而言，A 与 B 同构，C 少一条 `DVIF` 通道但公共参数照发**。所以「zid 只能取兜底值」这一限制对三条路径**同等成立**，不因路径不同而缓解。

### 17.9 更正登记（对 §1.2 / §12.2 / §16.3 的修正）

| 原位置 | 原文摘要 | 本节实测 | 处理 |
|---|---|---|---|
| §1.2 一句话版（:21） | 「三条登录路径……均不依赖官方闭源 native 库」 | **不成立**。核心认证算法（RSA/`sig`/`di` 加密）确不依赖 native；但公共参数 `zid` 的真实值依赖 `libfire` 与服务端下发数据 | **已改写** |
| §12.2 表第 2 行（:726） | 「哪些路径依赖 native：没有」 | 同上 | **已改写** |
| §15.9 表首行（:1228） | 「native 结论正确」 | 同上，「结论正确」过于笼统 | **已改写** |
| §16.3.6 Rust 复刻结论 | 「`zid` 不能精确复刻，可用纯 Java 兜底」 | 兜底分支本身不经 native 成立，但**兜底值只有常量/空串两种**，且服务端是否接受未知 | **已由本节取代** |
| §16.3.4 输出表「SP 文件解密值」 | 与「静态缓存」并列列为可选值 | SP 解密**必然经 native**，无 native 时该行恒为空 | **已由本节 17.1 取代** |

### 17.10 当前有效结论（一句话）

**百度核心认证算法（RSA 密码加密、`sig` 签名、`di` 的 AES 加密）不依赖闭源 native，可由本项目静态复刻；但公共参数 `zid`（以及 `di` 索引 34 的 `sf_zid`）的真实值依赖 `libfire`（`System.loadLibrary("fire")` → `Asc.ar/dc/ac`）与服务端下发数据。本项目可复刻的只是官方已确认的纯 Java 回退分支，该分支在没有 native 时只能产出 62 位硬编码常量或空串（上层转 `NoZidYet`），不存在第三种静态可得值。服务端是否接受兜底值——待动态验证（R29），本轮不作推测。**

## 附录 A：与既有文档的关系（勘误登记）

> 本节只登记差异，**不改写任何历史文档的结论**。

| 文档 | 位置 | 原文摘要 | T-006 实测 | 处理 |
|---|---|---|---|---|
| `docs/evidence/mvp-go-no-go.md` | 第 239 行表格「缺失资料」列 | 「`tpl` / `appSignKey` 由运行时 `setProductLineInfo` 注入（`SapiConfiguration.java:523/857/1071`），静态不可得」 | 实为 APK 硬编码常量 `kd/______.java:11/:14/:15/:17` | **不改**（T-005 历史产物），在此登记；是否修订由用户决定 |
| 同上 | 第 252 行 | 「剩下问题是「参数从哪来」（`tpl`/`appSignKey`/`di` 裁剪）」 | `tpl`/`appSignKey` 已解；`di` 裁剪已解（`mPrivacyParamesRegulation=false`）；仅 `diExceptIndex` 仍动态 | 同上 |
| `docs/evidence/login-feasibility.md` | §3.2.6 / §8 | 百度「需攻关」5 项 | 4 项已解，1 项（服务端风控）仍动态 | **已在本同步中更新**（§3.2 与 §2/§6/§7/§8 一并改） |
| `docs/API_FACTS.md` | 百度节第 9 项 | 登录可行性结论「需攻关」 | 「可行」 | **已在本同步中更新** |
| `docs/RISKS.md` R28 | — | 「三家需攻关网盘的阻断点集中在闭源 native」 | 百度一家的阻断点**不在 native** | **已在本同步中更新**（R28 保留另两家） |
| `docs/evidence/index-百度.md` | 全文 | 无 `appSignKey` 字样（`grep` 0 命中） | 签名链路本次才补齐 | **本任务不改索引**（索引是 T-002d 产物，且 T-006 不在其验收范围内） |

## 附录 B：本文的可信度分布

| 可信度 | 条数（约） | 典型内容 |
|---|---|---|
| 已验证 | 绝大多数 | 全部 `文件:行号` 定位、`nm`/`readelf`/`objdump` 实测、`grep` 计数 |
| 推测 | ~~2~~ **1**（2026-11-28 后） | ①`AccountUtils.O/P/Q` 运行时可能为空（§4.3）；~~②`hostDeviceId` 生成规则~~ **已关闭**：官方客户端不赋值（§15.3） |
| 待确认 | 若干 | 服务端行为类全部标待确认（§12.4 七项） |

**「已验证」只代表该静态代码/文件确实存在且本次实测读出，不代表服务端行为已验证。**
