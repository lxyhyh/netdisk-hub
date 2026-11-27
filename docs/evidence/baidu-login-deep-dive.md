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

**百度的账号密码、短信验证码、扫码三条登录路径，在静态证据上都可以由本项目独立实现，均不依赖官方闭源 native 库；唯一未闭环的是服务端风控是否放行，属动态验证范畴。整体四档结论：原生登录可行（静态可行，动态未验证）。**

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
| 34 | `sf_zid` | `SapiSafeFacade.getCurrentZid(context)`（`SapiSafeFacade.java:23-27`→`SafeService.getZidAndCheckSafe`→`FH.gzfi`→libfire；失败返回 **`"NoZidYet"`**） | `:161` | 同上；且 `getDiCookieInfo` 里 `sf_zid=="NoZidYet"` 会二次尝试（`:243`） | A+B | **必需**（失败时值为 `"NoZidYet"`） | 中（设备授权 ID） | **可复刻为 `"NoZidYet"`**：`sofire` 链路有纯 Java 兜底（`d/b.java:443-466` 读缓存/文件，全失败返回一段硬编码 56 位 hex 常量），**不会抛异常中断** |
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

`e(context)`（`d/b.java:443-466`）的顺序：读本地缓存 → 读 `l.d` 文件按 `"|"` split 后解密 → **全部失败时返回硬编码常量 `"74FFB5E6…BA03"`（56 位十六进制，本次未把完整值抄进文档）**。

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
| `libbaiduprotect_sec.so` | 133168 B | 未在本次范围内做调用点定位（不在任务要求的库清单的登录关联判定内） | — | **待确认** | — | 标 **待确认**：本次未追其 Java 侧调用链，不作为登录阻断证据，也不排除 |

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
| 2 | 哪些路径依赖 native | **没有**（三条均不依赖任何官方闭源 native） |
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
| 5 | `libbaiduprotect_sec.so` 的 Java 侧调用链 | 静态未做 | 不在任务要求的库清单的登录关联判定内 | 若后续发现登录被风控挑战，再回头查 |
| 6 | `libfire.so` / `libsofiresec.so` 内部算法 | 静态未做 | 超出本任务范围（只判关联，不逆向） | 仅当需要精确复刻 `sf_zid` 到「真实 zid」级别时才有必要；当前用兜底值即可 |
| 7 | 短信验证码的发送接口（获取验证码） | **本次未取证** | T-006 只要求审查「登录路径」，`dynamicPwdLogin` 上游的「发短信」接口本次未追 | 下一步若实现 B 路径需补：验证码请求接口 + 频控 |
| 8 | 扫码路径的 `hostDeviceId` 生成规则 | 静态已见生成但不完整 | `getQrCodeImage` 只透传，未见生成实现 | 任意稳定随机串推测可用（**推测**）；需动态验证 |

### 13.1 一条必须说清的范围边界

**本任务没有验证「短信验证码怎么发出去」。** `realDynamicPwdLogin` 是「用户已经拿到验证码之后」的那一步。发码接口（`/v2/api/getvoicecode` 一类）本次未定位、未追调用链。 → 若要实现 B 路径，还需补一个小任务。这是本节最重要的未完成项。

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
| 推测 | 2 | ①`AccountUtils.O/P/Q` 运行时可能为空（§4.3）②`hostDeviceId` 生成规则（§13 #8） |
| 待确认 | 若干 | 服务端行为类全部标待确认（§12.4 七项） |

**「已验证」只代表该静态代码/文件确实存在且本次实测读出，不代表服务端行为已验证。**
