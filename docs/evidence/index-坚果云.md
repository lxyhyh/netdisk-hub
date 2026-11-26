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

| 项 | 值 | 来源 |
|---|---|---|
| dex 内 nutstore.android 包类总数 | **2019**（去掉匿名内部类） | 自建脚本扫描 classes.dex + classes2.dex 的 `Lnutstore/android/...;` 常量 |
| jadx 产出 .java 覆盖数 | **1912** | 遍历 reverse/nutstore/sources/sources 与 dex 类名比对 |
| 缺失类数 / 占比 | **107 / 5.3%** | 前两项之差 |
| 缺失类清单 | `reverse/nutstore/jadx-missing.log`（不进 Git，本地保留） | 脚本输出 |
| **缺失类是否影响核心链路** | **是** | 见下方逐个分类 |
| 缺失原因 | **待确认** —— 本次反编译未开启日志重定向，jadx 的 JADX ERROR / WARN 输出已随终端丢失 | 见 jadx-missing.log |

**107 个缺失类按影响分类（实测）：**

| 分类 | 数量 | 是否影响核心链路 | 代表类 |
|---|---|---|---|
| `v2/ui/**` UI 层 | 约 50 | 否（UI 层，与协议无关） | `ui/fileinfos/*`、`ui/login/*`、`ui/fileproperties/*` |
| `databinding/**` | 4 | 否 | `FragmentTeamInitBinding` 等 |
| **网络/数据层** | **8** | **是** | `v2/data/local/MediaFilesLocalDataSource`、`v2/data/remote/PubObjectsRemoteDataSource`、`v2/data/WorkspaceRepository`、`delegate/RemoteFileDownloader`、`utils/xa`（NutstoreObjectUtils） |
| `service/**`、`receiver/**` | 4 | 部分（后台服务） | `service/BookmarkService`、`service/w` |
| `dao/**` | 2 | 否（本地 SQLite DAO，0 引用） | `dao/o`、`dao/s` |
| 其他杂项 | 约 39 | 否（29 个零引用） | — |

**已用 `--single-class` 补出并核对的关键类**：

| 缺失类 | 补出结果 | 内容 |
|---|---|---|
| `delegate/RemoteFileDownloader` | ✅ 18546 B | 下载核心类，已在 §7 引用 |
| `utils/xa` | ✅ 有效 | `NutstoreObjectUtils`，负责目录列表落库（调用 `connection.i.D(path, eTag)`） |
| `v2/ui/login/n` | ✅ 809 B（类体极短） | 只是 `LoginPresenter` 的 RxJava `OnSubscribe` 内部类，调用 `connection.i.m2441D()` 取验证码 |
| `common/a` | ⚠️ 输出 **0 字节** | 补出来是空文件（jadx 对某些类写空白文件而不报错） |

**结论：缺失类**确实**影响核心链路（网络/数据层 8 个），但已逐个补出核对；
补出结果确认：真正含协议逻辑的只有 `RemoteFileDownloader` 与 `utils/xa`，其余是 UI/DAO。

**注意一个 jadx 行为**：`--single-class` 对某些类会**产出一个 0 字节文件且 exit code 为 0**，
不报错、不警告。所以补完后必须检查文件大小，不能只看"done"。

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
| 107 个类反编译缺失及其失败原因 | 待确认 |
| READ_PHONE_STATE 的实际用途 | 待确认 |

---

## 11. 格式评估（这套索引格式能否直接用于剩余 8 个网盘）

### 11.1 结论

**基本可用，但需要增删 6 处。**这套 10 区块格式在坚果云（Java 主体 + Retrofit + 无加固）上跑通，信息量充足；但剩余 8 个里有 4 个是 Flutter 主体（天翼、123、夸克、阿里），另 3 个资料缺失（蓝奏、UC、迅雷）。直接用会在这两类上产出大量"未发现"，必须补上判定字段才能区分"没找到"和"不存在"。

### 11.2 必须新增的字段

1. **主体技术栈判定**（放在 §1 基本信息）：明确写 `Java/Kotlin 主体` 还是 `Flutter 主体`，给出判定依据（有无 libflutter.so / libapp.so）。坚果云是"否"，Flutter 网盘会是"是"，后者必须在索引开头就写明"dex 内不含业务协议，本索引对协议的覆盖为 0"。
2. **反编译覆盖率**（放 §2）：dex 里的类总数 vs 产出 .java 数 vs 缺失数。坚果云实测 2019 类缺 107 个（5.3%），其中恰好漏掉下载核心类 RemoteFileDownloader，说明这个字段是**必需的**——不写就会把"jadx 没反出来"误当成"没有这个功能"。
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
