# index-坚果云 —— 坚果云 Android 客户端反编译索引

- 对应任务：T-002b（反编译 1 个 APK 跑通索引格式）
- 生成日期：2026-11-26
- 反编译产物：reverse/nutstore/（不进 Git）
- 可信度取值：**已验证**（代码/资源中直接读到并核对）、**推测**（有间接证据但未闭环）、**待确认**（只看到线索，未证实）

> 本文只记录线索，不确认完整接口能力。所有"已验证"均指"在该版本 APK 的反编译产物中读到了这段代码/这个常量"，不代表服务端行为已被验证。

---

## 1. APK 基本信息

| 项 | 值 | 来源 |
|---|---|---|
| 文件名 | 坚果云 - v4.26.5.apk | 附件区 /root/.dsh/attachments/v1/files/6a/6a4ab9c7.../ |
| 大小 | 60329002 字节 | stat |
| SHA256 | 6a4ab9c7a48dbfac0410d36059ff679c8e2676dafa82e0ab389355ed3d81304c | sha256sum |
| 包名 | nutstore.android | aapt dump badging |
| versionName / versionCode | 4.26.5 / 174 | aapt dump badging；与 BuildConfig rn.java: e="4.26.5"、A=174 一致 |
| compileSdkVersion | 34 | aapt（platformBuildVersionCode=34, codename=14） |
| minSdkVersion | 22 | aapt |
| targetSdkVersion | 34 | aapt |
| dex 数量 | 2（classes.dex 8845776 B / classes2.dex 7654112 B） | unzip -l |
| 是否 Flutter | 否。APK 内无 libflutter.so、无 libapp.so | unzip -l |
| 是否加固 | 否。无 DexClassLoader / System.loadLibrary(dex) / PackageInstaller / Security.setProperty 调用点；dex 明文可读 | 在 /tmp/dec 全量 grep，0 匹配 |
| 签名 | v1（JAR 签名）。CN=Nutstore, OU=Nutstore, O=Nutstore, L=Shanghai, ST=Shanghai, C=86；证书 SHA-256 f548c330633c1ba30da4c61378247b1f96ca3280ebc3c1e2e41a569a96250d6d | apksigner verify --print-certs |
| 原生库 | lib/arm64-v8a/{libBugly_Native.so(199272), libEncryptorP.so(75744), libmupdf_java.so(6742824)}；另 assets/webkit 下腾讯 TBS 组件自带 .so | unzip -l |

权限中值得注意的是 READ_PHONE_STATE（见 §9 设备指纹）。

---

## 2. 反编译工具及版本

| 项 | 值 |
|---|---|
| 工具 | jadx 1.5.7 |
| 来源 | https://github.com/skylot/jadx/releases/download/v1.5.7/jadx-1.5.7.zip |
| 包大小 / SHA256 | 74766896 B / 1755e4242ade71a077438723689f404079592ec7764a3a0572f98fef94ebcbf3（本地自算，官方未提供校验值，标待确认） |
| 本地路径 | reverse/_tools/jadx-1.5.7/，入口 bin/jadx |
| 运行环境 | OpenJDK 21.0.13.1，Ubuntu 24.04 aarch64（proot） |
| 命令 | `./reverse/_tools/jadx-1.5.7/bin/jadx --no-res --no-imports -j 4 -d reverse/nutstore/sources "坚果云 - v4.26.5.apk"` |
| 产物 | reverse/nutstore/sources/sources/ 共 8006 个 .java，约 67 MB；reverse/nutstore/sources/resources/ 为资源 |

### 2.1 反编译覆盖率（必填）

> 本节数字为 **口径 v2**（2026-11-26 复核后）。旧版「2019 类 / 产出 1912 / 缺 107（5.3%）」已作废，
> 作废原因与重算依据见 **§2.1.1 口径说明**。复现脚本 `reverse/nutstore/coverage-check.py`。

| 项 | 值 | 来源 |
|---|---|---|
| dex 内 `nutstore.android` 包类定义总数 | **2753**（含 720 个含 `$` 的匿名/内部类） | coverage-check.py，ground truth = dex `class_defs` 表 |
| jadx 主产物独立文件 | 1966 | `reverse/nutstore/sources/sources/` |
| 二次补出独立文件 | 78 | `reverse/nutstore/sources-recovered/`（`--single-class` 补出） |
| 内联在外层文件中 | 591 | 产物内 `// from class: a.b.C.1` 注释命中 |
| 外层文件中的成员类声明 | 97 | 如 `R$anim` → `R.java:6` 的 `public static final class anim` |
| synthetic 空类 | 20 | `ACC_SYNTHETIC` 且方法数 0，jadx 出空文件是正确行为 |
| **仍未定位** | **1** | `nutstore.android.v2.data.WorkspaceRepository$Companion` |
| **覆盖率** | **2752 / 2753 = 100.0%** | 前几项相加校验 = 2753 ✓ |
| **缺失类是否影响核心链路** | **否** | 唯一未定位类零业务逻辑，见 §2.1.1 第 4 问 |
| 旧口径（已作废） | 2019 类 / 1912 产出 / 缺 107 = 5.3% | 见 §2.1.1 |
| 复现脚本 | `reverse/nutstore/coverage-check.py`（不进 Git，`.gitignore:2 reverse/`） | 纯 python 解析 dex，不依赖 jadx/baksmali |
| 缺失类清单 | `reverse/nutstore/jadx-missing.log`（107 条，不进 Git） | 旧口径遗留，已逐条定性，见 §2.1.1 第 4 问 |

### 2.1.1 口径说明（为什么旧数字不是 100%，而新数字是）

**第 1 问：原口径是什么？**

原口径（2026-11-26 首次统计，脚本未保留）：

- 分母：dex 内 `nutstore.android` 包**去掉匿名内部类**后的类数 = **2019**
- 分子：jadx 主产物里能按类名对上的 `.java` 文件数 = **1912**
- 缺失 = 2019 − 1912 = **107**，占比 **5.3%**
- 结论曾写成「缺失类**影响**核心链路」，因为 107 条里含下载核心类 `RemoteFileDownloader`

**第 2 问：「内联」具体指什么？**

jadx 默认开启内联（inline）优化：把匿名内部类、lambda、枚举 switch 映射类的**方法体直接展开进宿主方法**，
不再为它们单独生成 `.java` 文件，只在 `new ...() {` 那一行行尾留注释 `// from class: <点号形式的类名>`。
因此这些类在产物里「没有独立文件」，但**代码在**。三种落地情形：

| 情形 | 说明 | 实例 |
|---|---|---|
| ① 宿主有独立文件，匿名类内联进宿主 | 最常见 | `sources-recovered/nutstore/android/delegate/RemoteFileDownloader.java:122` 的 `// from class: nutstore.android.delegate.RemoteFileDownloader.1`（`.2` 在 `:240`）；`sources/sources/nutstore/android/connection/o.java:21/:24/:30` 是 `NutstoreHttpClient$1`、`$1.1`、`$1.2` |
| ② 宿主被 R8 改名包壳，文件头注释留着原 Kotlin 类名 | Kotlin `ViewModel`/`Presenter` 常见 | `NutstoreImageGalleryViewModel` → `sources/sources/nutstore/android/v2/ui/albumgallery/d.java:3` 的 `/* JADX INFO: compiled from: NutstoreImageGalleryViewModel.kt */`；`NotificationSettingsViewModel` → `v2/ui/newpreference/d.java` |
| ③ `R$xxx` 这类成员类被并入宿主文件 | 资源 ID 类 | `nutstore/android/R$anim` → `sources/sources/nutstore/android/R.java:6` 的 `public static final class anim {`（R.java 内共 17 个成员类） |

注意第 ① 类注释里用的是**点号** `.1`、`.2`，不是 `$1`；按类名做字符串比对时若不归一化点号，
就会把这批类误判成「缺失」——这是旧口径产生 107 的主要来源之一。

**第 3 问：重算依据用什么工具/命令？**

不依赖 jadx 也不依赖 baksmali（在手机上对 60 MB APK 反复跑全量会打满 CPU，T-002b 已踩过）：

```bash
# 1. 从 APK 解出 dex
unzip -o "坚果云 - v4.26.5.apk" 'classes*.dex' -d /tmp/dexcheck/

# 2. 纯 python 解析 dex 头三张表 + uleb128 读 class_data，逐类定形态
python3 reverse/nutstore/coverage-check.py /tmp/dexcheck
```

脚本读 `string_ids`（偏移 56/60）、`type_ids`（64/68）、`class_defs`（96/100），
逐条 class_def 取 `class_idx` 与 `access_flags`，再用 `class_data_off` 的 uleb128 前缀读出
`direct_methods`/`virtual_methods` 数量得到方法数。全程只读 dex 字节，约 80 秒完成，CPU 占用可忽略。

判定顺序（命中即归类，不重复计入）：`file-main` → `file-rec` → `nested-decl-in-outer` →
`synthetic-empty` → `inline-anon-comment` → `missing`。

**第 4 问：原缺失的 107 个类，是否确实全部在产物中以其他形式存在？**

逐条核对 `reverse/nutstore/jadx-missing.log` 的 107 条（条目本身已含 `nutstore/android/` 前缀），
结果 **107 = 76 + 19 + 12**，**没有一个是「有业务逻辑却没被反编译出来」的类**：

| 形态 | 数量 | 说明 |
|---|---|---|
| 已补出为独立文件 | **76** | `--single-class` 补出后落在 `reverse/nutstore/sources-recovered/`，有实际内容 |
| `ACC_SYNTHETIC` 且方法数 0 | **19** | dex 里就是空类（无方法），jadx 输出 1 字节空文件是**正确行为**，不是失败 |
| dex `class_defs` 里根本没有这个类定义 | **12** | 只在别处被引用；其中 2 个是被 R8 改名包壳后以别的文件名存在（见下例） |

具体样例（≥5 个，均可回查）：

| 原判「缺失」的类 | 在产物中的实际形态 |
|---|---|
| `nutstore.android.delegate.RemoteFileDownloader` | 补出为独立文件 `reverse/nutstore/sources-recovered/nutstore/android/delegate/RemoteFileDownloader.java`（18546 B），其匿名内部类 `.1`/`.2` 分别内联在该文件 `:122` 与 `:240` |
| `nutstore.android.utils.xa`（NutstoreObjectUtils） | 补出为独立文件 `sources-recovered/nutstore/android/utils/xa.java`（17455 B） |
| `nutstore.android.v2.data.WorkspaceRepository` | 补出为独立文件 `sources-recovered/nutstore/android/v2/data/WorkspaceRepository.java`（19113 B）；其 `$Companion` 是**唯一**未定位类，见下 |
| `nutstore.android.common.a` | dex 里 `ACC_SYNTHETIC` 且方法数 0 → jadx 输出 1 字节空文件 `sources-recovered/nutstore/android/common/a.java`（`ls -l` = 1 B），空文件即正确结果 |
| `nutstore.android.adapter.d` | 同上：synthetic、方法数 0、`sources-recovered/.../adapter/d.java` 为 1 字节 |
| `nutstore.android.databinding.FragmentNewUserGuideDialogBinding` | dex `class_defs` 无此定义（12 个 NOT-IN-DEX 之一），只在代码里被引用，非反编译失败 |
| `nutstore.android.v2.ui.albumgallery.NutstoreImageGalleryViewModel` | dex 无此定义；R8 改名后落在 `sources/sources/nutstore/android/v2/ui/albumgallery/d.java`（`:3` 的 `compiled from: NutstoreImageGalleryViewModel.kt` 可证），其 `loadItems$1` 亦已补出（18196 B） |

**唯一仍未定位的类**：`nutstore.android.v2.data.WorkspaceRepository$Companion`（方法数 2）。
baksmali 反汇编证实：`InnerClass` 标志 `0x19`，Kotlin `@Metadata` 的 `d2` 里只有一个
private static final String `TAG`；两个方法分别是 `private synthetic <init>()V`（只调 Object 构造）
与 `public synthetic <init>(DefaultConstructorMarker;)V`（转调前者），**零业务逻辑**。
它的 `TAG = "WorkspaceRepository"` 已被 jadx 内联进宿主
`sources-recovered/nutstore/android/v2/data/WorkspaceRepository.java:7`，该文件 `:47/:102/:150/:159` 等处均有使用点
→ **无信息损失**。

**为什么旧分母 2019 也不对**：dex 里 `nutstore.android` 包**不含 `$` 的类定义**共 **2033** 个，
不是 2019（差 14）；且旧统计把含 `$` 的 720 个类全部排除在分母外，恰好这批类是 jadx 内联的主要对象。
旧统计脚本未保留，**2019 与 1912 均无法用任何单一过滤条件复现**，故旧数字整体作废，以本节口径为准。

**口径边界（避免误读）**：本节的 100% 是「按 dex `class_defs` 计、把内联/成员类声明/补出文件都算覆盖」这一定义下的结果，
不是「jadx 产出了每一个独立文件」。对协议侦察而言，这个定义才是关心的：**有没有丢代码**。

---

**jadx 的一个行为要记住**：`--single-class` 对某些类会**产出 0 字节或 1 字节文件且 exit code 为 0**，
不报错、不警告。所以补完后必须检查文件大小，不能只看 "done"。

**下次反编译必须做的**（已写进 `reverse/nutstore/jadx-missing.log`）：

```bash
./reverse/_tools/jadx-1.5.7/bin/jadx --no-res --no-imports -j 4 \
  --log-level debug -d reverse/<网盘名>/sources "<apk>" \
  > reverse/<网盘名>/jadx-full.log 2>&1
```

然后 grep `JADX ERROR` / `Failed to decompile`，并逐个 `--single-class` 补、检查文件大小。

---

## 3. HTTP 客户端相关类/方法

**结论：两套 HTTP 链路并存**（已验证）。

### 3.1 旧链路：Apache HttpClient + 自建 NutstoreRequest 构造器

| 元件 | 位置 | 说明 |
|---|---|---|
| 客户端 | reverse/nutstore/sources/sources/nutstore/android/connection/o.java:7 | 静态单例 `DefaultHttpClient`；connectionTimeout=10s；每路由最大 6 连接；`setRedirecting(params, false)` 禁跟随重定向 |
| 请求构造器 | reverse/nutstore/sources/sources/nutstore/android/connection/w.java | 类 NutstoreRequest，字段 h=URL、k=Authorization、E=Accept-Language、ha=X-Client-Signature、B=Content-Type、ba=SoTimeout、aa=HttpEntity、ka=方法 |
| 请求执行 | reverse/nutstore/sources/sources/nutstore/android/connection/i.java:1314-1326 | 逐个 setHeader：Authorization(:1317)、Accept-Language(:1320)、Accept-Encoding(:1326)、X-Client-Signature(:1314-1316 附近) |
| 超时默认 | w.java:86 | `ba = 30s` 默认 |

**URL 拼接**（w.java:95-101、:103-126）：若 zl.m3377D()（用户自定义 server_uri）为 null，则基址写死 `https://app.jianguoyun.com`；否则用自定义 URI 的 scheme+authority。Query 参数由 w.java:128-150 手工 URLEncoder 拼接。

### 3.2 新链路：Retrofit2 + OkHttp3

| 元件 | 位置 | 说明 |
|---|---|---|
| 构建入口 | reverse/nutstore/sources/sources/nutstore/android/v2/data/remote/api/ServiceGenerator.java:30-40 | `getNutstoreApi(HttpUrl, token, acceptLanguage, userAgent)` |
| OkHttp 版本 | reverse/nutstore/sources/sources/okhttp3/internal/Version.java | okhttp/3.14.9（已验证） |
| Converter | ServiceGenerator.java:33 | GsonConverterFactory |
| CallAdapter | ServiceGenerator.java:33 | RxJavaCallAdapterFactory |
| 拦截器 | ServiceGenerator.java:43-56（HeadersInterceptor）、:59-73（UploadFileHeadersInterceptor）、:75+（DownloadFileHeadersInterceptor）、:124-135（AuthenticationInterceptor） | 见 §3.3 |
| 错误拦截 | ServiceGenerator.java（ApiErrorInterceptor） | 非 2xx 时把 body 反序列化为 ErrorDetail 再抛 ServerException |
| baseUrl | reverse/nutstore/sources/sources/nutstore/android/v2/g.java:79-90 | 同样默认 `new URI("https","app.jianguoyun.com",null,null,null)` |

### 3.3 请求头（已验证）

| Header | 值 | 来源 |
|---|---|---|
| Authorization | `String.format("%s-%s:%s", Base64(token), Base64(uid), userName)` | reverse/nutstore/sources/sources/nutstore/android/v2/data/remote/api/HttpConfig.java:36-43 |
| User-Agent | `String.format("%s-%s-%s-%s", "NutstoreApp-Android", Build.VERSION.RELEASE, Build.MODEL, versionName)` | HttpConfig.java:32-34 |
| User-Agent（旧链路） | `"NutstoreApp-Android-" + RELEASE + "-" + MODEL + "-" + versionName` | connection/o.java:19-24 |
| Accept-Language | `Locale.getDefault().toString()`；请求构造时中文固定 zh_CN、否则 en_US | HttpConfig.java:28-30；connection/w.java:111-121 |
| Content-Type | `application/json;charset=utf-8` 与 `application/octet-stream` 各 addHeader 一次（同名头两个值） | ServiceGenerator.java:55、:71 |
| Accept-Encoding | gzip（下载/列表请求） | ServiceGenerator DownloadFileHeadersInterceptor；connection/w.java:407、:418 |
| X-Client-Signature | 仅 campaign `/api/task` 请求设置，普通网盘请求为 null | connection/w.java:598-616 |

OKHttp 链路**未配置 CookieJar**，且全包 CookieManager 只用于 WebView 清理（utils/bb.java:60、v2/ui/previewfile/z/s.java:31），网盘请求不依赖 cookie（已验证）。

---

## 4. 认证相关类/方法

### 4.1 登录接口

| 项 | 值 | 来源 |
|---|---|---|
| 主登录 | POST /mcmd/loginV2 | connection/w.java:11（常量 D）、请求构造 w.java:171-196 |
| 旧登录 | GET /cmd/loginV1 | connection/w.java:47（常量 ca）、w.java:161-169 + 198-211 |
| 注册 | POST /mcmd/signupV2 / POST /cmd/signupV1 | w.java:13（Ea）、:54（g） |
| 登录执行 | connection/i.java:34-46（m2443D / D 重载）→ m2445D(:49-65) | 响应反序列化为 NutstoreRequestHelper$LoginSignupResponse |

**loginV2 body 字段**（已验证，w.java:180-190 逐字段）：
`userName`、`password`（二者均过 utils/y.H() = Base64(UTF-8 明文)）、`tfPasscode`（非空才发）、`custom_ticket`、`exp`、`sig`、`reusable`、`ticket`、`randstr`。

**密码处理**：客户端只做 Base64，**未做 MD5/SHA 哈希**（已验证：utils/y.java:124-126 H() → D(m2909D(str)) → common/za.java Base64Coder）。密码强度依赖 TLS。

**登录成功响应**：NutstoreRequestHelper$LoginSignupResponse 字段 token、userName、uid；JSON key 为 `token`、`userName`、`uid`（已验证，injectJson 读 UserInfo.UID="uid"）。

### 4.2 验证码 / 二次验证

| 接口 | 方法 | 来源 |
|---|---|---|
| /mcmd/getCaptchaV2 | GET，无参 | connection/w.java:57（ia）、:319-323 |
| /mcmd/getLoginCaptchaV2 | GET，query ticket | w.java:43（Z） |
| /mcmd/getFallbackCaptchaV1 | GET，@Deprecated | w.java:73（r）、:325-330 |

响应模型 reverse/nutstore/sources/sources/nutstore/android/connection/CaptchaResponse.java：`custom_ticket`、`exp`(long)、`needRealCaptcha`(boolean)、`reusable`(boolean)、`sig`。可复用票证由 loginV2 回传。

**登录调用顺序**（已验证，reverse/nutstore/sources/sources/nutstore/android/fragment/bi.java:29-30）：先 GET getLoginCaptchaV2 拿 CaptchaResponse，再 POST loginV2 带上 custom_ticket/exp/sig/reusable，ticket 与 randstr 传 null。若首次调用抛 5xx，回退到不带票证的 loginV2。

**登录错误码分支**（已验证，fragment/bi.java:77-115）：`NeedTwoFactorsAuthentication`（短信/微信二步）、`NeedSetupTwoFactorsAuth`、`NeedSmsAuthentication`、`NeedWechatAuthentication`、`TwoFactorsAuthenticationFailed`、`DisabledByTeamAdmin`、`AuthenticationFailed`、`NoSuchUser`、`UnAuthorized`。

### 4.3 手机/邮箱验证

| 接口 | body 字段 | 来源 |
|---|---|---|
| POST /mcmd/verifyPhoneV2 | phone, code | w.java:49（da）、:253-261 |
| POST /mcmd/verifyRegPhoneV2 | phone, code | w.java:53（fa）、:387-398 |
| POST /mcmd/sendVerifyPhoneMsgV2 | phone, ticket, randstr（或 custom_ticket/reusable/exp/sig） | w.java:15（Fa）、:231-251 |
| POST /mcmd/sendVerifyRegPhoneMsgV2 | 同上 | w.java:6（A） |
| POST /cmd/sendTfSms | userName, nickname | w.java:75（t）、:213-229 |
| POST /mcmd/verifyEmailV2 | — | w.java:25（Ka） |

### 4.4 会话持久化

登录成功后凭据落 SQLite（已验证，reverse/nutstore/sources/sources/nutstore/android/zl.java:73-79）：

| 键 | 含义 | 来源常量 |
|---|---|---|
| E="login_name" | 账号 | dao/x.java:13 |
| d="user_name" | 用户名 | dao/x.java:20 |
| a="token" | 会话 token | dao/x.java:17 |
| I="machine_name" | 设备标识，无则 UUID.randomUUID() | dao/x.java:14 |
| b="server_uri" | 自定义服务器（可空） | dao/x.java:19 |

读取 accessor：zl.H()=token、zl.E()=uid、zl.m()=loginName、zl.d()=deviceId、zl.B()=versionName。**token 无刷新接口、无过期时间字段**（已验证：全包 grep refreshToken/expiresIn 无网盘侧实现）→ 失效靠服务端返回 AccountExpired 触发重新登录（RequestException.isAccountExpired()，跳登录页位置 delegate/xa.java:496、fragment/gi.java:347、service/NutstoreTransportService.java:142）。

---

## 5. 文件列表相关类/方法

### 5.1 沙箱（sandbox）前置概念

所有文件接口都必须带 `sndId` 与 `sndMagic` 两个 Query 参数（已验证）：

- 值来源：`HttpConfig.getSndId(long) = Long.toHexString(sandboxId)`、`getSndMagic(long) = Long.toHexString(magic)`（HttpConfig.java:45-51）
- sandboxId / magic 来自 GET /cmd/getUserInfoV3（5xx 回退 /cmd/getUserInfoV2）响应的 `sandboxes[]`（已验证，connection/i.java:659-666、:687-712）
- 单个 sandbox 的 JSON 字段（已验证，reverse/nutstore/sources/sources/nutstore/android/dao/NSSandbox.java:100-136）：`name`、`sandboxId`、`magic`、`owner`、`ownerNick`、`permission`、`pathPerms[]`、`isDefault`、`isPhotoBucket`、`isOwner`、`desc`、`lanSyncCipher`、`channelId`、`caps`、`exclusiveUser`、`usedSpace`

### 5.2 列目录

| 项 | 值 | 来源 |
|---|---|---|
| 接口 | GET /mcmd/getDirV1 | connection/w.java:74（s）、请求构造 w.java:412-422 |
| Query | sndId, sndMagic, path | 同上 |
| 响应头 | Accept-Encoding: gzip；SoTimeout 120s | w.java:418-419 |
| 响应模型 | ObjectMetadataList | reverse/nutstore/sources/sources/nutstore/android/connection/h.java |
| 响应字段 | `etag`(String)、`truncated`(boolean)、`contents`(数组) | h.java:30-47 |

**分页**（已验证，connection/i.java:330-349）：循环调用，marker = 上一页最后一个对象的名称，直到 `truncated=false`；304 视为"未修改"，返回空结果（i.java:277-280）。

**单对象字段**（已验证，reverse/nutstore/sources/sources/nutstore/android/connection/v.java:24-63）：`path`、`size`(long, 缺省 -1)、`mtime`(long, 缺省 -1)、`hash`(String，字面量 "null" 归一为 null)、`isImage`(boolean)、`tblId`(String)、`version`(long)、`objectType`(只接受 "file"/"directory"，其它抛 ServerException)、`auxInfo`(嵌套对象 isLocked/nickName/userName)。常量 v.java:7 `"file"`、:9 `"directory"`、:8 `12288`。

### 5.3 单对象元数据

GET /mcmd/getMetaData，Query：sndId, sndMagic, path, list(true|false), etag(可选), marker(可选)（已验证，w.java:424-453）。Retrofit 版本 `checkFile` 把 `list=false` 写死进 URL（NutstoreApi.java:17-18）。

### 5.4 搜索

GET /cmd/search/run，Query：sandbox 相关 + `dir` + `all=false` + `keys=<关键字>` + `ft=true`（已验证，w.java:841-857）。

---

## 6. 上传相关类/方法

| 项 | 值 | 来源 |
|---|---|---|
| 接口 | POST /mcmd/uploadFile | connection/w.java:23（Ja）、请求构造 w.java:488-503 |
| Query | sndId, sndMagic, path，可选 `pup=1` | w.java:497-501 |
| Body | **裸文件流，不是 multipart** | connection/NSFileEntity.java（extends org.apache.http.entity.FileEntity） |
| Content-Type | application/octet-stream | w.java:55（ga）、NSFileEntity.setContentType |
| 缓冲区 | 4096 字节，边写边回调进度 | NSFileEntity.java:writeTo |
| 超时 | `max(60s, 文件字节数/102400 秒)` | w.java:924 |
| 客户端硬限制 | 524288000 字节（500 MB） | delegate/xa.java:537、service/NutstoreTransportService.java:7 与 :94、v2/service/uploadfiles/UploadFilesPrepareService.java:117 |
| 响应模型（旧链路） | ObjectMetadata（connection/v.java） | i.java:499-521 |
| 响应模型（Retrofit） | MetaData：id/parentId/path/size/mtime/tblId/objectType/version/hash/isImage/auxInfo/sandbox | v2/data/MetaData.java |

**无分片上传**（已验证）：NutstoreApi.java:11-15、:62-69 声明了 /cmd/checkBlockV1、/cmd/checkBlockListV1、/cmd/storeBlockV1、/cmd/storeBlockListV1、/cmd/writeObjectMetaV1 五个接口，但**全仓无调用点**，属未启用的死代码。不能据此推断分片协议。

另有 POST /mcmd/createFile（Query sndId/sndMagic + body），响应 CreateFileResponse{size(long), version(int)}（v2/data/CreateFileResponse.java）。

---

## 7. 下载相关类/方法

| 项 | 值 | 来源 |
|---|---|---|
| 接口 | GET /mcmd/getFile | connection/w.java:76（u）、请求构造 w.java:400-410 |
| Query | sndId, sndMagic, path, version | 同上 |
| 响应头 | Accept-Encoding: gzip；SoTimeout 120s；读到 gzip 头时包 GZIPInputStream | w.java:407-408；delegate/RemoteFileDownloader.java 内层 handler |
| 调用方 | delegate/RemoteFileDownloader.java（`w.D(path, version, token)`）+ delegate/RemoteDirDownloader.java（整目录） | 前者 jadx 主产物缺失，用 --single-class 补出 |
| 缓冲 | 4096 字节循环读，写临时文件后 move | RemoteFileDownloader.java |

**断点续传：客户端未实现**（已验证）：全包只存在错误码常量 `RANGE_NOT_SATISFIED="RangeNotSatisfied"`（common/exceptions/RequestException.java:28），**无任何代码设置 Range 请求头**。服务端是否支持 Range → 待确认。

**分片下载**：未发现（无 Range、无多线程分段下载代码）。

---

## 8. 关键字符串常量

### 8.1 域名（reverse/nutstore/sources/sources/nutstore/android/common/g.java，明文未混淆）

| 常量 | 值 |
|---|---|
| j | app.jianguoyun.com（API 基址） |
| h | www.jianguoyun.com |
| a | https://www.jianguoyun.com/ |
| I | workspace.jianguoyun.com |
| f | https://push.jianguoyun.net.cn |
| Y | https://ns-n8n-prod.jianguoyun.net.cn |
| C | https://inbox.jianguoyun.net.cn/nutstore/teamtrialsurvey.html |
| U | https://help.jianguoyun.com/ |
| W | https://www.jianguoyun.com/d/forget_pwd |
| b | https://www.jianguoyun.com/s/pricing |
| P | https://www.jianguoyun.com/s/helpdesk |
| R | https://www.jianguoyun.com/s/downloads |
| G | feedback@cn.nutstore.net |

推送专用：https://push.jianguoyun.net.cn/push/token（v2/data/PushRepository.java:67、:99）。comet 长连接：http://comet.jianguoyun.com:80/subscribeV2（connection/i.java:731）。

### 8.2 全部接口路径（connection/w.java 常量表，逐个核对明文）

登录注册类：/mcmd/loginV2(:11)、/cmd/loginV1(:47)、/mcmd/signupV2(:13)、/cmd/signupV1(:54)、/mcmd/getCaptchaV2(:57)、/mcmd/getLoginCaptchaV2(:43)、/mcmd/getFallbackCaptchaV1(:73)、/mcmd/verifyPhoneV2(:49)、/mcmd/verifyRegPhoneV2(:53)、/mcmd/sendVerifyPhoneMsgV2(:15)、/mcmd/sendVerifyRegPhoneMsgV2(:6)、/cmd/sendTfSms(:75)、/mcmd/verifyEmailV2(:25)、/mcmd/generateDeviceTokenJwt（NutstoreApi.java:29-30）

文件类：/mcmd/getDirV1(:74)、/mcmd/getMetaData(:32)、/mcmd/getFile(:76)、/mcmd/uploadFile(:23)、/mcmd/createFile、/mcmd/mkdir(:68)、/mcmd/deleteFileV1(:16)、/mcmd/deleteDirV1(:8)、/mcmd/renameObjectV1(:79)、/mcmd/submitMoveV1(:20)、/mcmd/moveProgressV1(:18)、/cmd/submitCopyV1(:78)、/cmd/copyProgressV1(:27)、/cmd/search/run(:80)、/cmd/browseVersionsV1、/cmd/restoreFilesV1、/mcmd/getThumbnail(:63)、/mcmd/submitUrlToFile(:56)、/mcmd/enableInboundEmailV1(:59)、/mcmd/userReport(:24)

沙箱/团队类：/cmd/getSandboxDetailV1(:58)、/cmd/createSandboxV1(:64)、/cmd/deleteSandboxV1(:36)、/cmd/getSandboxPathPerms(:66)、/cmd/getUserInfoV3(:33)、/cmd/getUserInfoV2(:62,@Deprecated)、/cmd/getContactsV1(:9)、/cmd/browseTeamGroupsV1(:14)、/cmd/getPubObjectV2(:40)、/cmd/pubObjectV1(:28)、/cmd/getMetering(:30)、/cmd/getEtpConfV1(:38)、/cmd/officeEditLink(:82)、/cmd/createShortcutV1(:46)、/cmd/getDiscussObjectInfo(:12)、/cmd/getPathAclBySubAdmin(:19)、/cmd/getWechatInvitationLink(:21)、/cmd/updatePhotoCameraV1(:45)、/cmd/getEventsV4(:17)、/cmd/getLatestEventIdV1(:41)、/cmd/getSSOSessionToken(:67，**全仓无调用点**)

运营/推送类：/static/campaign/latest(:37)、/static/campaignV2/latest.json(:65)、/api/task(:70)、/quest/overview(:7)、/notif/channels(:10)

未启用的分片上传：/cmd/checkBlockV1、/cmd/checkBlockListV1、/cmd/storeBlockV1、/cmd/storeBlockListV1、/cmd/writeObjectMetaV1（在 NutstoreApi.java，无调用点）

### 8.3 参数名常量

connection/w.java:26 `L="path"`、:31 `Na="sndId"`、:35 `R="sndMagic"`、:39 `V="gzip"`、:44 `a="deflate"`、:48 `d="application/delta-data"`、:50 `e="text/plain; charset=UTF-8"`、:52 `f="application/json; charset=UTF-8"`、:55 `ga="application/octet-stream"`、:51 `ea="directory"`、:69 `q="file"`、:29 `Ma="GET"`、:81 `y="POST"`

### 8.4 请求体里的业务字段名

| 接口 | body 字段 | 来源 |
|---|---|---|
| deleteFileV1 / deleteDirV1 | **JSON 数组** `[{path, version}]` | w.java:732-746 |
| renameObjectV1 | srcPath, srcVersion, destName, objectType | w.java:748-762 |
| submitCopyV1 / submitMoveV1 | srcSandboxId, srcSandboxMagic, srcPaths[], destDirPath | w.java:859-886 |
| copyProgressV1 / moveProgressV1 | Query: uuid | w.java:888-902 |
| mkdir | PathInternal{path} | v2/data/remote/api/PathInternal.java |
| updatePhotoCameraV1 | deviceId, latestUploadTs | w.java:904-916 |

### 8.5 错误码字符串

common/exceptions/RequestException.java 常量 40+ 个，与登录/文件操作相关的：`AccountExpired`、`AuthenticationFailed`、`ConcurrentUpdate`、`DuplicateName`、`BlockedTemporarily`、`ENUM_CAPTCHA_INVALID`、`NoSuchUser`、`UnAuthorized`、`NeedTwoFactorsAuthentication`、`NeedSetupTwoFactorsAuth`、`NeedSmsAuthentication`、`NeedWechatAuthentication`、`TwoFactorsAuthenticationFailed`、`DisabledByTeamAdmin`、`RangeNotSatisfied`。

---

## 9. 加固 / 混淆 / 证书绑定 / 设备指纹迹象

| 检查项 | 结果 | 搜索方式 |
|---|---|---|
| 加固壳 | **未发现**。无 DexClassLoader、无 System.loadLibrary 加载 dex、无 PackageInstaller、无 Security.setProperty | /tmp/dec 全量 grep |
| 证书绑定 | **未发现**。CertificatePinner / X509TrustManager / sslSocketFactory / checkServerTrusted / TrustKit / CertificateFactory 全部 0 匹配 | 同上 |
| 网络安全配置 | AndroidManifest 声明 `android:networkSecurityConfig=@0x7f110003`，但该资源在 unzip -l 中查不到对应 res/xml 文件（资源名被混淆成 res/-2.xml 这类短名）→ 内容**待确认** | aapt dump xmltree AndroidManifest.xml；unzip -l |
| 反调试/反root/模拟器检测 | **未发现**。FLAG_SECURE / isDebuggerConnected / /system/bin/su / test-keys / isEmulator / ro.kernel.qemu 全部 0 匹配 | /tmp/dec 全量 grep |
| native 方法 | 网盘业务层 **无** `public static native` 声明 | /tmp/dec 全量 grep |
| 字符串混淆 | **有，且很重**。每个类自带 `public static String D(Object)` 做 XOR 解密，全包 155 个解密器、4804 处调用（详见 §2 工具链与 §10 说明） | 自建脚本扫描 |
| 标识符混淆 | **有**。类名/方法名被改成单字母（i、w、v、h、u 等），ProGuard 未保留行号 | jadx 产物 |
| 资源混淆 | **有**。res 文件名被压成 res/-2.xml、res/0D.xml 这类形式，共 2560 个 res/ 条目 | unzip -l |
| 设备指纹 | **基本没有**。TelephonyManager / getDeviceId / ANDROID_ID / getSerial / oaid / IMEI 全部 0 匹配；唯一设备标识是自生成 UUID 存 dao.x.I="machine_name"（zl.java） | /tmp/dec 全量 grep |
| MAC 读取 | utils/m.java:100 读 wifiInfo.getMacAddress()，用于内网 LAN 同步，不参与网盘鉴权 | 同上 |

APK 申请了 READ_PHONE_STATE 权限，但代码里找不到取 IMEI/IMSI 的调用点 → **推测**为友盟/推送 SDK 残留使用，标**待确认**。

---

## 10. 线索可信度汇总

| 结论 | 可信度 |
|---|---|
| 基址 https://app.jianguoyun.com，可被 server_uri 覆盖 | 已验证 |
| Authorization = Base64(token)-Base64(uid):userName | 已验证 |
| 登录 POST /mcmd/loginV2，body 9 个字段，密码只做 Base64 | 已验证 |
| 文件接口必带 sndId/sndMagic，值为 sandboxId/magic 的 hex | 已验证 |
| 列目录 getDirV1 + truncated + marker 末对象名分页 | 已验证 |
| 上传是裸字节流，非 multipart，500MB 上限 | 已验证 |
| 下载 getFile 带 version，无 Range 断点续传 | 已验证（客户端侧） |
| 分片上传接口存在但未启用 | 已验证（无调用点） |
| 无证书绑定、无加固、无强设备指纹 | 已验证（0 匹配） |
| 服务端支持 Range | 待确认（只有错误码常量） |
| networkSecurityConfig 内容 | 待确认（资源被混淆） |
| 107 个类反编译缺失及其失败原因 | **已关闭**（口径问题，见 §2.1.1；真实含业务逻辑缺失 = 0） |
| READ_PHONE_STATE 的实际用途 | 待确认 |

---

## 11. 格式评估（这套索引格式能否直接用于剩余 8 个网盘）

### 11.1 结论

**基本可用，但需要增删 6 处。**这套 10 区块格式在坚果云（Java 主体 + Retrofit + 无加固）上跑通，信息量充足；但剩余 8 个里有 4 个是 Flutter 主体（天翼、123、夸克、阿里），另 3 个资料缺失（蓝奏、UC、迅雷）。直接用会在这两类上产出大量"未发现"，必须补上判定字段才能区分"没找到"和"不存在"。

### 11.2 必须新增的字段

1. **主体技术栈判定**（放在 §1 基本信息）：明确写 `Java/Kotlin 主体` 还是 `Flutter 主体`，给出判定依据（有无 libflutter.so / libapp.so）。坚果云是"否"，Flutter 网盘会是"是"，后者必须在索引开头就写明"dex 内不含业务协议，本索引对协议的覆盖为 0"。
2. **反编译覆盖率**（放 §2）：dex 里的类总数 vs 产出 .java 数 vs 缺失数。坚果云初次统计得 2019 类缺 107 个（5.3%），其中恰好含下载核心类 RemoteFileDownloader；按 dex `class_defs` 重算后排除了内联、synthetic、NOT-IN-DEX 三类，覆盖率实为 2752/2753（见 §2.1.1）。这个字段仍然**必需**——它逼着你把「jadx 没产出独立文件」和「代码真的丢了」分开，否则会把内联误当成缺功能。
3. **加固/风控 SDK 清单**（扩 §9）：不只写"有没有壳"，还要列 APK 里带的第三方安全 SDK 名字与数量。夸克有 libdexloaderuc/libfakedexuc/libsgmainso，阿里有 sgmain×4 + sgsecurity×4，这类会直接决定"照抄客户端协议"的难度。
4. **接口清单的一行式表格**（扩 §8.2）：坚果云这次是逐个列 path 常量。批量时建议统一成 `路径｜HTTP 方法｜Query 参数｜Body 字段｜调用点文件:行` 的表格，便于横向对比 9 个网盘。

### 11.3 建议删除或降级的字段

5. **§8.1 域名清单**可降级为"只列与 API 相关的域"。坚果云列了 13 个，其中 help/pricing/forget_pwd 这类运营页面对协议还原无价值。
6. **§3 里"两套 HTTP 链路"的细节**对 Flutter 网盘会是空白，建议保留但明确写"仅 Java 主体适用"。

### 11.4 不建议改的

- 每条线索带 `文件:行号` 的定位要求必须保留。这次所有关键结论都能回查，是格式能立住的核心。
- 可信度三档（已验证/推测/待确认）必须保留。坚果云里"Range 待确认"和"networkSecurityConfig 待确认"就是这个机制的价值。
- 「未发现」必须写搜索方式的规矩要保留。否则无法区分"搜了没有"和"没搜"。

### 11.5 对 Flutter 网盘的预期

对天翼/123/夸克/阿里，本格式预计 §3-§8 会全部落到"dex 内未发现"，实际有效信息只剩：§1 基本信息、§2 覆盖率、§8.2 里可能残留的纯 Java 侧接口（推送、更新、统计）、§9 加固与风控 SDK。届时应先跑一遍再决定是否继续投入，不要机械填满 10 个区块。

---

> **本索引是 `docs/evidence/index-template.md` v1.0 的来源**（2026-11-26）。
> v2.1 由 v1.0 精简版 + v2.0 详细版合并而成，本节 6 处改动已全部固化进模板；后续网盘索引照模板产出，不再各写一份格式评估。
