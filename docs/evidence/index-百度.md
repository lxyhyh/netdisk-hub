# index-百度网盘 —— 百度网盘 Android 客户端反编译索引

- 对应任务：T-002d（A 项：百度网盘完整反编译）
- 生成日期：2026-11-27
- 反编译产物：`reverse/baidu/`（不进 Git）
- 可信度取值：**已验证**（代码/资源中直接读到并核对）、**推测**（有间接证据但未闭环）、**待确认**（只看到线索，未证实）

> 本文只记录线索，不确认完整接口能力。所有「已验证」均指「在该版本 APK 的反编译产物中读到了这段代码/这个常量」，**不代表服务端行为已被验证**。
> 任务边界：**不在本任务判定百度原生登录可行/不可行**（属 T-002c）。

---

## 1. APK 基本信息

| 项 | 值 | 来源 |
|---|---|---|
| 文件名 | `百度网盘_13.11.15.apks`（分包）→ 提取 `base.apk` | 附件区 + `unzip -j` |
| 大小 | .apks 189318289 B；`base.apk` **251854857 B** | `stat` |
| SHA256（base.apk） | `3ad2da623e2a32a4924c1a8a3c84887e6994342dbbc497517a7781b3f4d260e4` | `sha256sum` |
| SHA256（.apks 原包） | `4caf9d3ced25e2486f399bbcce9c16c6787130ef03f83be2da5dda51b603d15d` | `sha256sum` |
| 包名 | `com.baidu.drive.app` | `aapt dump badging` |
| versionName / versionCode | `13.11.15` / `3711` | `aapt dump badging` |
| compileSdkVersion | 35（platformBuildVersionName 15 / Code 35） | `aapt dump badging` |
| minSdkVersion | 23 | `aapt dump badging` |
| targetSdkVersion | 35 | `aapt dump badging` |
| dex 数量 | **20 个**，合计 182513748 B | `unzip -l` |
| **【新增①】主体技术栈判定** | **`Java/Kotlin 主体`（另有 Flutter 模块共存）** | 见 §1.1 |
| 判定依据 | base.apk 内 `lib/` 条目 **0 个**，但 `split_config.arm64_v8a.apk` 内有 `libflutter.so` 11036664 B + `libapp.so` 20972464 B，且 base.apk 内 `assets/flutter_assets/` 有 1510 个文件 | `unzip -l` 两个包 |
| 是否加固 | **未加壳**（无 DexHelper/爱加密等壳；有百度自有安全 SDK，见 §9） | 见 §9 |
| 签名 | DN `C=CN, ST=Beijing, L=Haidian, O=Baidu, OU=BaiduNetdiskGroup, CN=android-drive`；cert SHA-256 `a3d8f1b7f69cd4950bc26d2e9d6f16d0bb73c9fb6139c044fcff1f4116011d79` | `apksigner verify --print-certs` |
| Source Stamp 证书 | DN `CN=Android, OU=Android, O=Google Inc., ...`，SHA-256 `3257d599a49d2c961a471ca9843f59d341a405884583fc087df4237b733bbd6d` | 同上 |
| Manifest 特殊项 | `android:requiredSplitTypes="base__abi,base__density"`；残留 `<instrumentation name="android.test.InstrumentationTestRunner" targetPackage="com.baidu.netdisk">` | `aapt dump xmltree AndroidManifest.xml` |
| 原生库 | base.apk 内 0 个；`split_config.arm64_v8a.apk` 内 **65 个 .so**（含 `libnetdisk-security.so`、`libnetdisk-security-rand.so`、`libnetdisk-signature-check.so`、`libbaiduprotect_sec.so`、`libsofiresec.so`、`libmsaoaidsec.so`、`libjni-kernel.so` 34179392 B） | `unzip -l` |

### 1.1 【新增①】主体技术栈判定说明

**判定：`A. Java/Kotlin 主体`。** 理由：

- base.apk 的 20 个 dex 里 `com.baidu.netdisk` 占 **54452 个类定义**（全量 173602），网络层/业务层/传输层类在 dex 内完整可读；
- 原生库全部放在 ABI split 里，base.apk 自身不带 `.so`，所以「base.apk 无 lib/」**不能**作为「非 Flutter」的判定依据；
- `split_config.arm64_v8a.apk` 同时带 `libflutter.so` + `libapp.so` 20.9 MB → 存在 Flutter 模块（部分页面/插件用 Flutter 实现），但**网盘协议不依赖它**。

**结论：本文对网盘协议的覆盖以 dex 侧为准，dex 侧覆盖率 100%（§2.1）。** Flutter 模块的业务内容不在本文范围内（不深挖 libapp.so，按任务约束）。

---

## 2. 反编译工具及版本

| 项 | 值 |
|---|---|
| 工具 | jadx **1.5.7**（复用 `reverse/_tools/jadx-1.5.7/`，未重新下载） |
| 来源 | https://github.com/skylot/jadx/releases/download/v1.5.7/jadx-1.5.7.zip |
| 包大小 / SHA256 | 74766896 B / `1755e4242ade71a077438723689f404079592ec7764a3a0572f98fef94ebcbf3`（**本地自算，官方无校验值 → 待确认**） |
| 本地路径 | `/root/workspace/网盘/reverse/_tools/jadx-1.5.7/` |
| 运行环境 | OpenJDK 21.0.13.1，Ubuntu 24.04 aarch64（proot，无沙箱后端） |
| 版本核对 | `./bin/jadx --version` → `1.5.7`；与 `docs/evidence/tooling.md` 记录一致 |

### 2.0 反编译命令与「单次全量失败」记录

**单次全量反编译在本机失败**（保留为证据，不重试）：

```bash
export JADX_OPTS="-Xmx5g -Xss4m"
nohup ./reverse/_tools/jadx-1.5.7/bin/jadx --no-res --no-imports -j 4 \
  --log-level debug -d reverse/baidu/sources reverse/baidu/input/base.apk \
  > reverse/baidu/jadx-full.log 2>&1 &
```

- 现象：RSS 涨到 5491760 KB（约 5.2 GB），进度 **32746 of 70481 (46%)** 时进程消失，日志停在 20:26:09，已产出 47442 个 .java / 326 MB；
- 判定：**不是 jadx 报错退出，是被系统内存压力终止**（本机 15 GB 总内存、可用仅约 5 GB）；
- 失败日志：`reverse/baidu/jadx-full.log`（1476074 B / 13056 行，68 条 ERROR，多为 `JadxOverflowException` / `Code restructure failed`，属告警级）；
- 处置：`rm -rf reverse/baidu/sources` 清理不完整产物，改**按 dex 分批**。

**实际采用的分批命令（每个 dex 一次）**：

```bash
unzip -o -j reverse/baidu/input/base.apk "classes*.dex" -d reverse/baidu/dex
export JADX_OPTS="-Xmx3g -Xss2m"
for d in reverse/baidu/dex/classes*.dex; do
  b=$(basename "$d" .dex)
  ./reverse/_tools/jadx-1.5.7/bin/jadx --no-res --no-imports -j 2 \
    --log-level debug -d "reverse/baidu/sources/$b" "$d" \
    >> "reverse/baidu/logs/per-dex/$b.log" 2>&1
done
```

- 产物：`reverse/baidu/sources/classesN/sources/`，**1.2 GB**；
- 脚本与日志：`reverse/baidu/decompile-batch1.sh`、`reverse/baidu/logs/per-dex/progress.txt`（逐 dex 的 rc 与文件数）。

### 2.1 【新增②·必填】反编译覆盖率

**口径 v2（照 `index-坚果云.md` §2.1.1 抄），先写口径再写数。**

| 项 | 值 | 来源 |
|---|---|---|
| dex 内目标包类定义总数（含 `$`） | **63259**（`Lcom/baidu/` 前缀；全量 class_defs 173602） | 纯 python 解析 dex `class_defs` |
| jadx 主产物独立文件 | 27424（file-main） | `find sources -name '*.java'` |
| 改名产出独立文件 | 6670（renamed-file，含 `JADX INFO: renamed from:`） | 同上 |
| 二次补出独立文件 | 26（file-rec，`sources-recovered/`） | `--single-class` 补出目录 |
| 外层文件中的成员类声明 | 15630（nested-decl-in-outer） | 如 `R$anim` → `R.java` 的 `class anim` |
| synthetic 空类 | 169（`ACC_SYNTHETIC` 且方法数 0） | dex access_flags |
| 内联在外层文件中 | 13340（inline-anon-comment） | 产物内 `// from class: a.b.C.1` |
| **仍未定位** | **0** | |
| **覆盖率** | **63259/63259 = 100.0000%** | (总数 − 仍未定位) / 总数 |
| **【必填】缺失类是否影响核心链路** | **否** | 见下 |
| 缺失类清单 | `reverse/baidu/jadx-missing.log`（已写「仍未定位 0 个」） | 脚本输出 |
| 缺失原因 | 不适用（最终 0 缺失）；中途 25 个 DONT_GENERATE 的成因见下 | jadx 日志 |
| 复现脚本 | `reverse/baidu/coverage-check.py` | 纯 python 解析 dex |

**统计口径（口径 v2）**

- **分母** = 20 个 dex 的 `class_defs` 里所有 `Lcom/baidu/` 前缀类定义，**含 `$` 匿名/内部类**；
- **判定顺序**（命中即归类，不重复计入）：`file-main` → `file-rec` → `nested-decl-in-outer` → `synthetic-empty` → `inline-anon-comment` → `missing`；
- **必须识别两类改写**，否则会得出假缺失：
  1. **改名**：jadx 对非法 Java 类名（`_`、`__`、`b0`）会改名并在文件头写 `/* JADX INFO: renamed from: com.baidu.x._ */`，全包 **8303 条** rename 映射；
  2. **包名补前缀**：jadx 对非法包名段加 `p000` 前缀（`com.baidu.netdisk.switch` → `com/baidu/netdisk/p000switch/`），比对前必须 norm() 去掉 `p000`；
- **内联的含义**：jadx 默认把 lambda/匿名类展开进宿主，只在 `new ...() {` 行尾留 `// from class: <点号类名>`（点号 `.1` 不是 `$1`），这类「无独立文件但代码在」，必须算覆盖。

**中途 25 个类补出的过程（已验证）**

| 轮次 | 命令 | 结果 |
|---|---|---|
| 第 1 轮 | `--single-class <cls> -d ...` | 只成功 2 个；其余 23 个 **rc=1、产出 0 字节**，日志 `ERROR - Incorrect arguments: Input class can't be saved by current jadx settings (marked as DONT_GENERATE)` |
| 第 2 轮 | 加 `--no-inline-anonymous --no-inline-kotlin-lambda` | 23 个全部成功，产出 300–1100 字节真内容 |

第 2 轮完整命令（`reverse/baidu/recover-missing2.sh`）：

```bash
./reverse/_tools/jadx-1.5.7/bin/jadx --no-res --no-imports \
  --no-inline-anonymous --no-inline-kotlin-lambda \
  --single-class <class> -d /tmp/si_one --log-level info reverse/baidu/dex/<dex>
```

- 结果记录：`reverse/baidu/logs/single/summary2.txt`（格式 `序号|类名|dex|rc=|bytes=|产出文件`，末行 `RECOVERY2DONE`）；
- 补出产物：`reverse/baidu/sources-recovered/`（25 个 .java / 241 K）；
- 抽检结论：补出的**全部是 Kotlin lambda 合成类**（`implements kotlin.jvm.functions.Function1/Function0`，body 只转调宿主方法），例 `selectfile/b0.java`：

```java
public final Object invoke(Object obj) {
    return Boolean.valueOf(SelectFileFragment.n(this.f2660_, (CloudFile) obj));
}
```

→ **零业务协议逻辑**，故「缺失类是否影响核心链路 = 否」。

**辅助脚本**（均在 `reverse/baidu/`）

| 脚本 | 用途 |
|---|---|
| `coverage-check.py` | 口径 v2 覆盖率核算（读 `recovered-names.txt` 做 rename/p000 归一） |
| `find-missing-dex.py` | 定位缺失类所属 dex |
| `dump-missing-flags.py` | 读 access_flags 与方法数 |
| `dump-missing-methods.py` | 读方法名（direct/virtual 分别累加 `method_idx_diff`） |
| `build-recovered-names.py` | 生成 `recovered-names.txt` |
| `dex-scan.py` / `dump-classes.py` | 生成 `reverse/baidu/classes.tsv`（173602 行，全量类名+所属 dex） |

---

## 3. HTTP 客户端相关类/方法

> 本节对 `Java/Kotlin 主体` 适用。百度**同时存在两套链路**，都要还原。

| 项 | 值 | 来源 |
|---|---|---|
| HTTP 客户端（主链路） | **自研**，基于 `java.net.HttpURLConnection` | `reverse/baidu/sources/classes6/sources/com/baidu/netdisk/network/c.java`（NetworkTask，112917 B） |
| HTTP 客户端（次链路） | **OkHttp3 + Retrofit2**（`GsonConverterFactory`，超时 10 s/10 s） | `classes2/sources/com/baidu/netdisk/io/RetrofitProxy.java:5-9` |
| 次链路接口声明 | `@Url` + `@QueryMap`/`@FieldMap`/`@HeaderMap` 的动态 URL 形式，**无硬编码 path** | `classes6/sources/com/baidu/netdisk/network/retrofit/RetrofitServiceApi.java:5-24` |
| 构建入口（主链路） | `md.e` → `md._____`（`NetworkTaskFactory`）→ `network.c` | `classes15/sources/md/e.java:9-18`；`classes15/sources/md/e.java:20-28` |
| 请求参数容器 | `m20.__`（KV 容器）/ `m20.___`（完整请求描述） | `classes15/sources/m20/__.java`、`classes15/sources/m20/___.java` |
| baseUrl / 基址 | `md.__.___()` → `https://pan.baidu.com/api/`；`md.___.______()` → `https://pan.baidu.com/api/` | `classes15/sources/md/__.java:27-29`、`classes15/sources/md/___.java:24` |
| 拦截器 | `md.c`（`IRequestInterceptor`，转 `hk.__._._(req, z)`）；Retrofit 侧另有 `o20.___` + `ReplaceRequestParamsInterceptor` | `classes15/sources/md/c.java:11-19`；`RetrofitProxy.java:5` |
| Cookie 处理 | 有；响应 `Set-Cookie` 解析后存 `xk.p.r()`（键 `PANPSC_KEY` / `SBOXTKN_KEY` / `CACATATKN_KEY`） | `classes6/sources/com/baidu/netdisk/network/c.java:180-200`；键名 `classes6/sources/com/baidu/netdisk/network/___.java:9-15` |

### 3.1 请求头逐个说明

| Header | 值 / 构造方式 | 来源 |
|---|---|---|
| `Host` | 显式设置，取请求 URL 的 host | `network/c.java:114` |
| `Cookie` | `___.____()` 提供；HTTPS 与 HTTP 分支都加 | `network/c.java:128-135` |
| `Sdata` | **HMAC 类签名**：`hk.__._.b(___.____(), path+?+query)`，仅非 HTTPS 且 path 不在白名单时加 | `network/c.java:118-121`；`hk/__.java:469-475` |
| `User-Agent` | 静态 `p20.c.___`（`"User-Agent"`），值由 `RequestCommonParams.e()` 给出 | `network/c.java:145`；`classes15/sources/p20/c.java:14`；`network/request/RequestCommonParams.java:94-100` |
| `Accept-Encoding` | `gzip`（条件 `___.e()`） | `network/c.java:146-148` |
| `Content-Type` | 固定 `application/x-www-form-urlencoded` | `network/c.java:149`；`p20/c.java:17-20` |
| 自定义头 | `m20.___` 内的 header map 逐条 `setRequestProperty` | `network/c.java:156-164` |

**`Sdata` 签名函数来源**：`hk.__.G` 是 `Function2<String,String,String>` 静态字段，由外部注入（`hk/__.java:666` 处赋值）；**具体注入实现未定位 → 待确认**。

**`RequestCommonParams` 是接口式注入**（`RequestCommonParamsCreator` 6 个方法），真实值在 App 层实现，**静态不可得 → 待确认**。

### 3.2 域名常量表

| 常量 | 值 | 用途 | 来源 |
|---|---|---|---|
| `md.__.f3910_` | `CloudFileServerURL`（debug key） | 域名覆盖用的调试键 | `md/__.java:5` |
| `md.___._f3911_` | `CommonServerURL`（debug key） | 同上 | `md/___.java:5` |
| `md.___.______` | `pan.baidu.com` | 主域 | `md/___.java:12` |
| `md.___.b` | `yun.baidu.com` | 旧主域（判断用） | `md/___.java:14` |
| `md.___.c` | `pcs.baidu.com` | PCS 域 | `md/___.java:15` |
| `md.___.e` | `d.pcs.baidu.com` | 下载域 | `md/___.java:17` |
| `md.___.f` | `c.pcs.baidu.com` | 上传/内容域 | `md/___.java:18` |
| `md.___.g` | `update.pan.baidu.com` | 更新 | `md/___.java:19` |
| `md.___.h` | `eopa.baidu.com` | 运营页 | `md/___.java:20` |
| `md.___.l` | `https://pan.baidu.com/api/` | **文件类接口基址** | `md/___.java:24` |
| `md.___.n` | `https://pan.baidu.com/rest/2.0/dss/` | DSS | `md/___.java:26` |
| `md.___.o` | `https://pan.baidu.com/rest/` | REST 根 | `md/___.java:27` |
| `md.__.b` | `https://pan.baidu.com/rest/2.0/xpan/` | xpan | `md/__.java:13` |
| `md.__.______` | `https://pan.baidu.com/sharedir/` | 共享目录 | `md/__.java:11` |
| `md.__.a` | `https://pan.baidu.com/api/image/` | 图片 | `md/__.java:14` |
| `md.C_.__` | `https://pan.baidu.com/rest/2.0/membership` | 会员 | `md/C_.java:12` |
| `op0.a.f4097_` | `https://pan.baidu.com/rest/2.0/pcs/` | **PCS 基址（上传/下载）** | `classes12/sources/op0/a.java:8` |
| `op0.a.______` | `d.pcs.baidu.com` | 下载域常量 | `op0/a.java:12` |
| `op0.a`（`b()`） | `https://d.pcs.baidu.com/rest/2.0/pcs/file` | **下载接口基址** | `op0/a.java:90-94` |
| `SapiHost.DOMAIN_ONLINE_*` | Base64 编码的主机（`passport.baidu.com` / `wappass.baidu.com` / `openapi.baidu.com` / `wappass.bdimg.com`） | 登录域，运行时 Base64 解码 | `classes12/sources/com/baidu/sapi2/utils/SapiHost.java:4-20` |

**注**：几乎所有域都可被 `com.baidu.netdisk.kernel.debug._` 的调试键覆盖（`md/__.java:16-45` 每个 getter 都有 `replace(pan.baidu.com, str)`），所以「线上真实域名」以常量默认值为准。

---

## 4. 认证相关类/方法

> 本节只记录**静态线索**。百度 APP 内原生登录是否可复刻，**由 T-002c 判定**，本文不给结论。

### 4.1 登录接口

| 项 | 值 | 来源 |
|---|---|---|
| 账号 SDK | `com.baidu.sapi2`（百度账号 SAPI），非百度网盘自研 | `classes2/sources/com/baidu/sapi2/SapiAccountManager.java` 等 |
| 网页登录入口 | `{wap}/passport/login` | `classes2/sources/com/baidu/sapi2/SapiAccountRepository.java:2144` |
| Wap 校验 | `{wap}/v3/login/api/check` | `SapiAccountRepository.java:2136` |
| Wap 分享登录 | `{wap}/v3/login/api/login` | `SapiAccountRepository.java:2148` |
| 登出 | `{url}/v3/api/user/logout` | `SapiAccountRepository.java:2128` |
| OAuth token 交换 | `https://openapi.baidu.com/oauth/2.0/token` | `SapiAccountRepository.java:1119` |
| APP 内原生登录页 | **有**：`BaseNewLoginActivity` / `NewLoginActivity` / `NewLoginHalfActivity` / `NewLoginSmsCheckActivity` / `NewLoginUpSmsCheckActivity` | `classes4/sources/com/baidu/sapi2/newlogin/activity/` |
| 登录服务入口 | `NewLoginService.startLogin(Context, NALoginCallback, SapiNALoginDTO)` | `classes4/sources/com/baidu/sapi2/newlogin/NewLoginService.java:25` |
| 加密器 | `SapiDataEncryptor`（`realDynamicPwdLogin(...)` 参数） | `SapiAccountRepository.java:263` |
| 域名枚举 | `Domain.DOMAIN_ONLINE` / `DOMAIN_QA` / `DOMAIN_CAR_ONLINE` | `classes12/sources/com/baidu/sapi2/utils/enums/Domain.java:4-6` |
| 密码处理 | **待确认**（`realDynamicPwdLogin` 走 `SapiDataEncryptor` + RSA/`javax.crypto` 异常声明，具体算法未读通） | `SapiAccountRepository.java:263` |
| 登录成功响应字段 | `bduss` / `uid` / `ptoken` / `stoken` / `username` / `displayname` / `portrait` / `app` | `classes2/sources/com/baidu/sapi2/SapiAccount.java:19-40` |

### 4.2 验证码 / 二次验证

- 结果类存在：`CheckUserInfoResult`、`CheckUserFaceIdResult`（**含人脸核验**）、`CertGuardianResult`、`AccountRealNameResult`；`classes12/sources/com/baidu/sapi2/result/`
- 具体验证码接口路径**未在 sapi2 内以明文常量出现**（大量走 WebView/JS 交互）→ **待确认**。

### 4.3 手机 / 邮箱验证

- 只有 `SmsWapLoginFailHandler.java` 一个失败处理器（`classes12/sources/com/baidu/sapi2/helper/`），短信流程主体在 WebView/JS → **待确认**。

### 4.4 会话持久化

| 项 | 值 | 来源 |
|---|---|---|
| 存储位置 | `SharedPreferences`，文件名 **`sapi_system`**（`context.getSharedPreferences("sapi_system", 0)`） | `classes2/sources/com/baidu/sapi2/SapiContext.java:61-65` |
| 键名 | 账号字段键：`bduss` / `uid` / `ptoken` / `stoken` / `username` / `displayname` / `portrait` / `phone` / `email` / `extra` / `fromType` | `SapiAccount.java:5-18` |
| 网盘侧会话 | 请求靠 `Cookie`（BDUSS/STOKEN/BAIDUID 等），网盘代码里**没有**独立 CookieStore 类 | 见下「未发现」 |
| token 刷新机制 | **待确认**（未定位刷新调用点） | — |
| token 有效期 | **待确认**（客户端无明文有效期字段） | — |
| 扫码登录 | **未发现独立实现**。搜索方式：`grep -rn --include=*.java '"login".*qr\|QrLogin\|qrcode' classes*/sources/com/baidu/sapi2/` → 仅 `utils/enums/QrLoginAction.java`（枚举 `LOGIN("login")`）与 `qrcode/` 目录下 `R.java`+`a.java`（无业务实现） | — |

**网盘侧 Cookie 持久化「未发现」的搜索方式**（按【不变⑨】要求）

```bash
grep -rln --include=*.java -e 'CookieStore' -e 'BDUSS.*SharedPreferences' -e 'persistent.*cookie' \
  classes*/sources/com/baidu/netdisk/     # → 0 命中
```

→ 结论：网盘业务层**没有**独立的 Cookie/Jar 持久化类；Cookie 由 `network.c` 通过 `___.____()` 回填（提供者实现未定位）→ **待确认**。

---

## 5. 文件列表相关类/方法

### 5.1 前置概念

**无 sandbox/sndId 之类前置概念。** 但有两个必需上下文：

1. **`bdstoken`**：所有写操作（上传/删除/重命名）都要带，值 = `MD5(BDUSS)` 的小写十六进制。
   - `classes6/sources/com/baidu/netdisk/network/__.java:8` `BDSTOKEN = "bdstoken"`；
   - `network/__.java:18` `this.mBdstoken = z01.______._____(str, false)`；
   - `classes8/sources/z01/______.java:45-49` `_____(String, boolean)` → `MD5(UTF-8 bytes)`；
   - `classes8/sources/z01/_____.java:29-31` `___(bytes, z)` → `__(bytes, "", z)`；`z=false` 时 `Integer.toHexString` 原样（**小写**）。
2. **`Cookie`**（BDUSS 等）— 见 §3.1 / §4.4。

### 5.2 列目录

| 项 | 值 | 来源 |
|---|---|---|
| 接口 | `GET https://pan.baidu.com/api/list` | `classes15/sources/kg/__.java:641-657`（`md.__.___() + "list"`） |
| Query | `dir`、`start`（偏移）、`limit`（条数）、`order`、`desc`(0/1)、`preset`(0/1)；`folder=1` 可选 | `kg/__.java:642-655`；`rd/__.java:16`(`g="start"`)；`SearchHistoryContract.__.q2="limit"`（`classes*/sources/com/baidu/netdisk/cloudfile/storage/db/SearchHistoryContract.java:40`） |
| 响应模型 | `GetDirectoryListResponse`，唯一字段 `list` → `CloudFile[]` | `classes15/sources/com/baidu/netdisk/cloudfile/io/model/GetDirectoryListResponse.java:10-12` |
| 单对象字段 | `fs_id`、`isdir`、`server_filename`、`path`、`size`、`md5`、`server_ctime`、`server_mtime`、`dlink`、`category`、`cover` 等 | `classes15/sources/com/baidu/netdisk/cloudfile/io/model/CloudFile.java:57/61/72/119/126/160/163/171` |
| 另一处同名实现 | `kg/__.java:626-637` 返回 `Integer`（只要条数），参数同样 `dir/start/limit/order/desc/preset` | `kg/__.java:626-637` |

### 5.3 分页方式

**`offset`**（偏移 + limit）。依据：`kg/__.java:645` 传入 `rd.__.g` = `"start"`（`rd/__.java:16`），`:646` 传 `limit`（`SearchHistoryContract.__.q2`）。响应**不带** cursor/marker 字段（`GetDirectoryListResponse` 只有 `list`）→ 无 `cursor` 分页。

### 5.4 搜索

| 项 | 值 | 来源 |
|---|---|---|
| 接口 | `GET https://pan.baidu.com/api/search` | `classes15/sources/kg/__.java:293-311` |
| Query | `key`、`dir`、`category`、`recursion`（可选） | `kg/__.java:296-304` |
| 相关接口 | `/richsearch/content/diff`（`cursor`）、`/searchintent/querysplit`（`q`、`from=auto`）、`/richsearch/quantity/update` | `kg/__.java:313-326`、`:424` |

### 5.5 其他列表类

| 功能 | 接口 | 来源 |
|---|---|---|
| 容量 | `GET /api/quota`（`checkrecycle`、`needquota`、`extra_appid`、`svcp_stk`） | `kg/__.java:680-681`；`cloudfile/service/f1.java:6-10` |
| 分类列表 | `GET /api/categorylist` | `kg/__.java:609` |
| 目录大小 | `POST /api/dirsize`（body `list`=[{path}]） | `kg/__.java:660-669` |
| 文件元信息 | `POST /api/filemetas`（`fsids` 或 `target`、`dlink=1`、`text=1`、`include_complete_deleted`） | `classes15/sources/kg/C_.java:16-100` |
| 扩展信息 | `POST /api/filext`（`fsids`） | `kg/__.java:674-677` |
| 快速差异 | `POST /api/batch/filediff`、`/api/filediffcheck`、`/api/filediff`、`/api/filefullcheck` | `kg/__.java:203/269/553/570/587` |
| 跨空间容量 | `GET /rest/2.0/dss/crossquota` | `kg/__.java:184` |
| 空间检查 | `GET /api/space/check` | `kg/__.java:462` |

---

## 6. 上传相关类/方法

| 项 | 值 | 来源 |
|---|---|---|
| 主实现类 | `com.baidu.netdisk.transfer.transmitter.UploadTransmitter`（极大，多处分片逻辑重复） | `classes12/sources/com/baidu/netdisk/transfer/transmitter/UploadTransmitter.java` |
| 接口 1：预创建 | `POST https://pan.baidu.com/api/precreate` | `classes12/sources/com/baidu/netdisk/transfer/h.java:128-136` |
| 接口 2：分片上传 | `POST {pcs}/rest/2.0/pcs/superfile2?method=upload&type=tmpfile&path=<path>&partoffset=<offset>&app_id=<app_id>` | `classes12/sources/op0/a.java:37` |
| 接口 3：落盘 | `POST https://pan.baidu.com/api/create?a=commit` | `classes6/sources/ly/C_.java:152/159`；`classes12/sources/com/baidu/netdisk/transfer/h.java:76` |
| precreate Body | `path`、`size`、`isdir=0`、`local_mtime`、`local_ctime`、`block_list`(JSON 数组)、`rtype`、`mode`、`autoinit=1`、`uploadid`/`uploadsign`(断点续传时)、`content-md5`、`slice-md5`、`contentCrc32` | `UploadTransmitter.java:6121-6150`、`:8046-8065` |
| Body 形态 | **分片**（`superfile2` 传 raw part 二进制；`precreate`/`create` 是 form-urlencoded 字段） | 同上 |
| Content-Type | `application/x-www-form-urlencoded`（`precreate`/`create`）；分片请求由 `m20.___` 单独设置 | `network/c.java:149` |
| 分片大小 | **4194304 B = 4 MiB**（`this.k = 4194304L`），并按文件大小动态翻倍（`v0(long)`：`i *= 2` 直到块数 ≤ 1024） | `UploadTransmitter.java:5958`、`:8647-8660` |
| 客户端硬限制 | **4294967296 B = 4 GiB**（`public static final long b1 = 4294967296L`，`if (j <= b1)` 分支） | `UploadTransmitter.java:25`、`:8648` |
| 响应模型 | `PreCreateFileResponse`（`block_list`）、`CreateFileResponse` | `classes12/sources/com/baidu/netdisk/transfer/io/model/PreCreateFileResponse.java:7` |
| **分片上传** | **有**（真实现，非死代码） | 见上三条接口 |
| 秒传 / 哈希预检 | **有**：`precreate` 的 `block_list` + `content-md5` + `slice-md5` + `contentCrc32` | `UploadTransmitter.java:8056-8062` |
| 分片元数据模型 | `PartInfo`：`part_crc32`、`part_id`、`part_md5`、`part_size` | `classes12/sources/com/baidu/netdisk/transfer/io/model/PartInfo.java:7-18` |
| 定位上传 | `GET {pcs}?method=locateupload` | `classes12/sources/com/baidu/netdisk/transfer/h.java:91` |
| 压缩预检 | `POST /api/precompress`（带 `content-length`） | `transfer/h.java:120-126` |

---

## 7. 下载相关类/方法

| 项 | 值 | 来源 |
|---|---|---|
| 取链接口 | `GET https://d.pcs.baidu.com/rest/2.0/pcs/file?method=locatedownload&path=<path>&ver=2.0&dtype=0&esl=1&ehps=<httpsSpecialEnable>&app_id=<app_id>&check_blue=1`（可选 `token` + `timestamp`） | `classes12/sources/uo0/C_.java:55-68`；基址 `op0/a.java:90-94` |
| 响应模型 | `LocateDownloadResponse`：`host`、`path`、`urls`(List<LocateDownloadUrls>)、`server`、`type`、`httpCode`、`error_code`、`error_info`、`sl`、`fsl`、`redo` | `classes12/sources/com/baidu/netdisk/transfer/io/model/LocateDownloadResponse.java:10-32` |
| 调用方 | `uo0.C_`（`____`/`_____` 两个重载） | `uo0/C_.java:55/72` |
| 下载器实现 | `np0.d` / `np0.r` / `np0.s`（分片下载器，按 `tp0.C_` 分块） | `classes12/sources/np0/d.java`、`np0/r.java`、`np0/s.java` |
| **断点续传（Range）** | **客户端有实现**：`np0/d.java:21` `setRequestProperty("Range", str)`（`:181` 构造 `"bytes=" + length + "-"`）；`np0/r.java:208` `setRequestProperty("RANGE", "bytes=" + start + "-" + end)`；`np0/s.java:139` `setRequestProperty("RANGE", "bytes=" + t + "-")` | 同上 |
| **分片下载** | **有**：`VideoThumbAreaImpl.java:320` 用 `range: bytes=i-end`；`np0/r.java` 按 `xVar._____/xVar.a/xVar.______` 三段式分块 | `classes12/sources/com/baidu/netdisk/stardisk/dynamic/player/area/VideoThumbAreaImpl.java:320` |
| 秒传/直链字段 | `CloudFile.dlink` | `classes15/.../CloudFile.java:30` |

**服务端是否真的接受 Range** → **待确认**（静态只能证明客户端发了 Range 头）。

---

## 8. 关键字符串常量

### 8.1 【降级⑤】API 相关域名

见 §3.2 表。**推送/长连接等辅助域名**：

| 常量 | 值 | 用途 | 来源 |
|---|---|---|---|
| `md.___.a` | `https://pan.baidu.com/api/analytics?type=%1$s&t=%2$s` | 埋点 | `md/___.java:13` |
| `md.g.f3920_` | `https://pan.baidu.com/rest/2.0/pcs/adx` | 广告 | `classes15/sources/md/g.java:7` |
| `md.g._____` | `https://eopa.baidu.com/page/HomePage-YW54iXYf?channel=37` | 运营页 | `md/g.java:11` |
| `op0.a.g` | `"/topic/streaming?"` | 话题流媒体 | `op0/a.java:29` |
| `op0.a.e` | `"/share/streaming?"` | 分享流媒体 | `op0/a.java:28` |
| 视频播放模板 | `%1$span.baidu.com/api/streaming?no_report_preview=1&...` | 视频播放 | `op0/a.java:44-56` |

### 8.2 【新增④·必填】接口清单

#### A. 登录注册类

| 路径 | HTTP 方法 | Query 参数 | Body 字段 | 调用点（文件:行） |
|---|---|---|---|---|
| `{wap}/passport/login` | GET（WebView/JS） | — | — | `classes2/sources/com/baidu/sapi2/SapiAccountRepository.java:2144` |
| `{wap}/v3/login/api/check` | — | — | — | `SapiAccountRepository.java:2136` |
| `{wap}/v3/login/api/login` | — | — | — | `SapiAccountRepository.java:2148` |
| `{url}/v3/api/user/logout` | — | — | — | `SapiAccountRepository.java:2128` |
| `https://openapi.baidu.com/oauth/2.0/token` | POST | — | OAuth token 交换 | `SapiAccountRepository.java:1119` |

#### B. 文件类

| 路径 | HTTP 方法 | Query 参数 | Body 字段 | 调用点（文件:行） |
|---|---|---|---|---|
| `/api/list` | GET | `dir`、`start`、`limit`、`order`、`desc`、`preset`、`folder` | — | `classes15/sources/kg/__.java:626`、`:641` |
| `/api/search` | GET | `key`、`dir`、`category`、`recursion` | — | `kg/__.java:293` |
| `/api/quota` | GET | `checkrecycle`、`needquota`、`extra_appid`、`svcp_stk`、`safe_box`、`cal_quota` | — | `kg/__.java:25`、`:680` |
| `/api/filemetas` | POST | `cid`（企业） | `fsids` 或 `target`、`dlink`、`text`、`include_complete_deleted` | `classes15/sources/kg/C_.java:16/24/34/50/63/76` |
| `/api/filext` | POST | — | `fsids` | `kg/__.java:674` |
| `/api/dirsize` | POST | — | `list`=[{path}] | `kg/__.java:660` |
| `/api/categorylist` | GET | — | — | `kg/__.java:609` |
| `/api/dirsize`/`/api/taskquery` | GET | `taskid` | — | `kg/__.java:166`、`:170` |
| `/api/filemanager?opera=rename` | POST | — | `filelist`=[{path,newname}]、`async`、`ondup`、`product_type` | `kg/__.java:243-262` |
| `/api/filemanager?opera=delete` | POST | — | `filelist`[]、`async`、`ondup`、`captchaidentity`、`captchatype`、`product_type` | `kg/__.java:515-536` |
| `/api/filemanager?opera=copy` | POST | — | `filelist`=[{path,dest}]、`async` | `kg/__.java:466` |
| `/api/filemanager?opera=move` | POST | — | `filelist`=[{path,dest}]、`async` | `kg/__.java:88-120` |
| `/api/filemanager?opera=import` | POST | `cid` | `filelist`=[{path,newname,dest,ondup}]、`async` | `kg/__.java:46-72` |
| `/api/create?a=commit` | POST | `norename`（flag）、`cid` | `path`、`size=0`、`isdir=1`、`local_ctime`、`local_mtime`、`block_list`、`product_type` | `kg/__.java:334-355` |
| `/api/task/create` | GET | — | `sign`、`timestamp`、`type` | `kg/__.java:505-512` |
| `/api/task/query` | POST | — | `detail` | `kg/__.java:194-201` |
| `/api/batch/filediff` | GET | `caid`、`ch_device_addr`、`cursor`、`screen_size` | — | `kg/__.java:202-215` |
| `/api/filediffcheck` | POST | — | `cursor`、`count`、`appid`、`deviceid`、`path`、`md5`、`agver` | `kg/__.java:268-291` |
| `/api/filediff` | POST | — | — | `kg/__.java:553`、`:570` |
| `/api/filefullcheck` | POST | — | — | `kg/__.java:587` |
| `/api/space/check` | GET | — | — | `kg/__.java:462` |
| `/api/dir/set_attr` | GET | — | — | `kg/__.java:396` |
| `/api/dir/mark` | GET | — | — | `kg/__.java:414` |
| `/api/fileappeal/add` | POST | — | — | `kg/__.java:443` |
| `/api/addfilecollection?fs_id=` | GET | `fs_id` | — | `kg/__.java:431` |
| `/api/cancelfilecollection?fsid_list=` | GET | `fsid_list` | — | `kg/__.java:458` |
| `/api/precreate` | POST | — | `path`、`size`、`isdir`、`local_mtime`、`local_ctime`、`block_list`、`rtype`、`mode`、`autoinit`、`uploadid`、`uploadsign`、`content-md5`、`slice-md5`、`contentCrc32` | `classes12/sources/com/baidu/netdisk/transfer/h.java:128` |
| `/api/create?a=commit`（上传落盘） | POST | — | 同建目录 + `uploadid` | `transfer/h.java:76`；`classes6/sources/ly/C_.java:152` |
| `/api/precompress` | POST | — | `content-length` | `transfer/h.java:120` |
| `{pcs}/rest/2.0/pcs/superfile2?method=upload&type=tmpfile&path=&partoffset=&app_id=` | POST | `method`、`type`、`path`、`partoffset`、`app_id` | 分片二进制 | `classes12/sources/op0/a.java:37` |
| `{pcs}?method=locateupload` | GET | `method` | — | `transfer/h.java:91` |
| `https://d.pcs.baidu.com/rest/2.0/pcs/file?method=locatedownload&...` | GET | `method`、`path`、`ver`、`dtype`、`esl`、`ehps`、`app_id`、`check_blue`、`token`、`timestamp` | — | `classes12/sources/uo0/C_.java:55-68` |

#### C. 沙箱 / 团队 / 企业空间类

| 路径 | HTTP 方法 | Query 参数 | Body 字段 | 调用点（文件:行） |
|---|---|---|---|---|
| `/rest/2.0/dss/crossquota` | GET | — | — | `classes15/sources/kg/__.java:184` |
| `{api}/multimedia?method=filemanager` | POST | — | `device_id`、`fsid_list`、`opera` | `kg/__.java:122`、`:538` |
| 企业空间参数 | 各接口追加 `cid` | `cid` | — | `classes15/sources/com/baidu/netdisk/account/h.java:11` |

#### D. 运营 / 推送 / 内容类

| 路径 | HTTP 方法 | Query 参数 | Body 字段 | 调用点（文件:行） |
|---|---|---|---|---|
| `/api/underage/list` | GET | `limit`、`cursor`、`category=0` | — | `kg/__.java:34` |
| `/api/richsearch/content/diff` | POST | — | `cursor` | `kg/__.java:313` |
| `/api/searchintent/querysplit` | GET | `q`、`from` | — | `kg/__.java:323` |
| `/api/richsearch/quantity/update` | GET | — | — | `kg/__.java:424` |
| `/api/image/config` | GET | — | — | `kg/__.java:605` |
| `/api/analytics?type=&t=` | GET | `type`、`t` | — | `md/___.java:13` |
| `/rest/2.0/pcs/adx` | — | — | — | `md/g.java:7` |
| `/rest/2.0/membership` | — | — | — | `md/C_.java:12` |

#### E. 未启用 / 死代码

本次**未发现**明确的「有声明无调用点」接口。搜索方式：

```bash
grep -rn --include=*.java 'md\.__\.[a-z_]*() + "' classes15/sources/kg/  # 逐个回溯调用方
```

→ 上述清单里的接口均在 `kg/__.java`/`kg/C_.java` 内有实际调用点。**判定：无死代码接口。**

### 8.3 参数名常量

| 常量 | 值 | 用途 | 来源 |
|---|---|---|---|
| `network/___.f2214_` | `Set-Cookie` | 响应头 | `classes6/.../network/___.java:8` |
| `network/___.__` | `PANPSC` | Cookie 名 | `network/___.java:10` |
| `network/___.___` | `PANPSC_KEY` | 本地存储键 | `network/___.java:11` |
| `network/___._____` | `SBOXTKN` | Cookie 名 | `network/___.java:14` |
| `network/___.______` | `SBOXTKN_KEY` | 本地存储键 | `network/___.java:15` |
| `network/___.b` / `.c` | `CACATATKN` / `CACATATKN_KEY` | 同上 | `network/___.java:16-17` |
| `network/___.o` | `BDUSS` | Cookie 名 | `network/___.java:28` |
| `network/___.q` | `access_token` | 参数名 | `network/___.java:30` |
| `network/__.java:8` | `bdstoken` | 写操作令牌 | `network/__.java:8` |
| `p20.c.___/____/_____/______` | `User-Agent` / `Content-Type` / `application/x-www-form-urlencoded` / `Cookie` | header 名 | `classes15/sources/p20/c.java:14/17/20/23` |
| `rd/__.g` | `start` | 分页偏移参数 | `classes15/sources/rd/__.java:16` |
| `SearchHistoryContract.__.q2` | `limit` | 分页条数参数 | `classes*/sources/com/baidu/netdisk/cloudfile/storage/db/SearchHistoryContract.java:40` |
| `cloudfile/service/f1.r/s/v` | `safe_box` / `cal_quota` / `svcp_stk` | quota 参数 | `classes15/sources/com/baidu/netdisk/cloudfile/service/f1.java:6-10` |

### 8.4 请求体业务字段

| 字段 | 出现接口 | 含义 | 来源 |
|---|---|---|---|
| `path` | list/create/filemanager/delete/rename/precreate | 网盘路径（以 `/` 开头） | 多处 |
| `filelist` | delete/rename/copy/move/import | JSON 数组字符串，元素含 `path`/`newname`/`dest`/`ondup` | `kg/__.java:244-262`、`:516-536` |
| `block_list` | precreate/create | 分片 MD5 列表 JSON 数组 | `UploadTransmitter.java:6132` |
| `isdir` | create?a=commit | `1` 建目录 | `kg/__.java:338` |
| `local_ctime` / `local_mtime` | create / precreate | 毫秒时间戳 | `kg/__.java:340-341` |
| `part_size` / `part_id` / `part_md5` / `part_crc32` | 分片 | 分片元数据 | `PartInfo.java:7-18` |
| `async` | delete/rename/copy/move | `0/1/2`（`i2 > -1 && i2 < 3`） | `kg/__.java:257` |
| `ondup` | delete/rename | 重名策略 | `kg/__.java:260` |
| `product_type` | 各写接口 | `workspace`（仅在 `/_pcs_.workspace` 前缀时） | `kg/__.java:353` |

### 8.5 错误码字符串

| 项 | 值 | 来源 |
|---|---|---|
| 网络异常类 | `com.baidu.netdisk.network.exception.RemoteException` | 多处方法签名 |
| PCS 错误码常量 | `com.baidu.netdisk.transfer.transmitter.PCSTransmitErrorCode`、`constant.OtherErrorCode` | `classes12/sources/com/baidu/netdisk/transfer/transmitter/` |

---

## 9. 加固 / 混淆 / 证书绑定 / 设备指纹

| 检查项 | 结果 | 搜索方式 |
|---|---|---|
| 加固壳 | **未加壳**。无 DexHelper/爱加密/梆梆壳；`assets/` 内无加密 dex 载荷 | `unzip -l base.apk` → 无 `libDexHelper.so`/`ijiami.dat`；dex 均为标准 DEX（magic `dex\n035`） |
| 证书绑定 | **未发现**（百度自家网络栈用默认 TrustManager） | `grep -rln --include=*.java -e CertificatePinner -e X509TrustManager -e setSSLSocketFactory sources/` → 命中全在 androidx/applovin/第三方（classes3 applovin shadow okhttp、classes10 androidx、classes16/18/19 三方）。百度自家：`network/c.java:38` 用内嵌 `HostnameVerifier` 调 `HttpsURLConnection.getDefaultHostnameVerifier()`；`c.java:232` `setSSLSocketFactory(SSLSocketFactory.getDefault())` |
| 网络安全配置 | `res/xml/network_security_config.xml`：`cleartextTrafficPermitted=true`，`trust-anchors` 同时含 `src="system"` **与 `src="user"`** → **信任用户 CA（可用于抓包）** | `aapt dump xmltree base.apk res/xml/network_security_config.xml` |
| 反调试 / 反 root / 模拟器检测 | 有安全 SDK 但未定位具体反调试调用点 → **待确认** | `grep -rn --include=*.java 'isDebuggerConnected\|frida\|ptrace' sources/` → 未在 `com/baidu/netdisk` 内命中 |
| native 方法 | **303 处** `public static native` | `grep -rn 'public static native' sources/ \| wc -l` → 303 |
| 字符串混淆 | **有**：部分常量 Base64 编码（如 `SapiHost` 的 14 个域常量），其余为明文；未见全量字符串加密 | `classes12/sources/com/baidu/sapi2/utils/SapiHost.java:4-20` |
| 标识符混淆 | **有（部分）**：大量类名被 R8 改短（`kg`/`md`/`m20`/`np0`/`op0`/`vo0`/`uo0`/`lg`/`nd`/`xk`/`z01`），jadx 对非法名再加 `_`/`__` 改名并在文件头标 `renamed from` | `reverse/baidu/sources/classes*/sources/` 目录名 |
| 资源混淆 | 有：`res/xml` 资源 ID 化（`networkSecurityConfig=@0x7f160015`） | `aapt dump xmltree` |
| 设备指纹 | 有：`com.baidu.helios.ids`（百度自有 ID 体系）、`com.bun.miitmdid`（MSA OAID SDK）；`com.mars.united.netdisk.middle.platform.App.getCuid()` 产出 `cuid`/`devuid` | `grep -rn --include=*.java -e getDeviceId -e ANDROID_ID -e getSerial -e oaid sources/` 命中共 3 处；`classes14/sources/com/mars/united/netdisk/middle/platform/App.java:59` |

### 9.1 【新增③·必填】加固 / 风控 SDK 清单

| SDK | 文件 / 包名 | 数量 | 影响 |
|---|---|---|---|
| 百度网盘自有安全 | `libnetdisk-security.so`(1082464 B)、`libnetdisk-security-rand.so`(2884608 B)、`libnetdisk-signature-check.so`(18264 B) | 3 | **签名/校验可能在 native 层** → 影响 §3.1 的 `Sdata` 算法还原 |
| 百度卫士 | `libbaiduprotect_sec.so`(133168 B)、`libbaiduprotect_sec_jni_facect.so`(137264 B) | 2 | 人脸/风控，与文件协议无关 |
| 百度 Sofi 安全 | `libsofiresec.so`(1235496 B) | 1 | 风控，**待确认**是否参与请求签名 |
| MSA OAID | `libmsaoaidsec.so`(622592 B)、`libmsaoaidauth.so`、`com.bun.miitmdid` | 2+ | 设备标识，与协议无关但影响 `devuid` |
| 百度 jni-kernel | `libjni-kernel.so`(10017264 B)、`libjni-kservice.so`(34179392 B) | 2 | 百度的 native 运行时，含大量业务下沉 → **协议可能部分在 native** |
| OpenCV | `libopencv_java3.so`(17889264 B) | 1 | 图像处理，与协议无关 |
| Paddle Mobile | `libpaddle-mobile.so`(6295312 B) | 1 | 端侧推理，与协议无关 |
| Flutter | `libflutter.so`(11036664 B) + `libapp.so`(20972464 B) | 2 | 部分页面用 Flutter，业务逻辑在 libapp.so（本文不深挖） |

**判定**：**有风控/安全 SDK（6 类）**。这意味着 §3.1 的 `Sdata` 签名与 `bdstoken` 之外的校验（如 `sign`/`timestamp`/`deviceid`）**可能在 native 层**，纯静态还原可能不完整 → 相关字段在本文标注「待确认」。

---

## 10. 线索可信度汇总

| 结论 | 可信度 |
|---|---|
| 主体技术栈 = `Java/Kotlin 主体` + Flutter 模块共存 | 已验证 |
| dex 覆盖率 100.0000%（63259/63259，0 缺失） | 已验证 |
| 主链路是自研 HttpURLConnection 栈；次链路是 OkHttp3+Retrofit2 | 已验证 |
| 文件类基址 `https://pan.baidu.com/api/` | 已验证 |
| 下载基址 `https://d.pcs.baidu.com/rest/2.0/pcs/file` | 已验证 |
| 列目录分页 = `offset`（`start` + `limit`） | 已验证 |
| 上传 = `precreate` → `superfile2` 分片（4 MiB 起）→ `create?a=commit` | 已验证 |
| 单文件上传上限 4 GiB | 已验证（客户端常量） |
| 客户端实现了 Range 断点续传 | 已验证 |
| `bdstoken` = `MD5(BDUSS)` 小写十六进制 | 已验证（代码路径完整） |
| `Sdata` 签名 = HMAC 类，算法实现未定位 | 待确认 |
| 服务端是否接受 Range | 待确认 |
| 服务端真实域名（是否与常量一致） | 待确认 |
| 密码处理算法 | 待确认 |
| 扫码登录 | 未发现（已写搜索方式） |
| 证书绑定 | 未发现（已写搜索方式） |
| 百度 APP 内原生登录可行性 | **不在本文范围**（属 T-002c） |

---

## 11. 格式评估

本次直接套用 `docs/evidence/index-template.md` v2.1，**不重写格式评估**（模板规定只有首次跑通时写）。

仅记录一条本次踩到的坑，供模板后续维护参考：

- **【建议增补】§2.1 的「统计口径」应显式要求「识别 jadx 改名与 `p000` 包名前缀」**。坚果云那次是 rename 问题，百度这次还额外踩了 `p000` 包名前缀（`com.baidu.netdisk.switch` → `p000switch`）。本次已在 §2.1 口径里补齐这两条。

---

## 附录 A 【降级⑤】非 API 域名 / 链接

| 常量 | 值 | 用途 |
|---|---|---|
| `md.___.h` | `eopa.baidu.com` | 运营页 |
| `md.g._____` | `https://eopa.baidu.com/page/HomePage-YW54iXYf?channel=37` | 首页运营 |
| `SapiHost.DOMAIN_NSCLICK_URL` | `https://nsclick.baidu.com/v.gif` | 埋点 |
| `SapiHost.ACTION_INTERCEPT_URL` | `https://wap.baidu.com/action_intercept_android` | 拦截页 |
| `classes12/.../share/io/ShareApi.java:12` | `https://pan.baidu.com/share/` | 分享页 |
| `classes12/.../stardisk/dynamic/ui/n0.java:25-37` | `https://pan.baidu.com/aipan/star-pan/{home,share,profilecreate,profileedit,profilefieldedit}` | 星盘页 |
| `classes12/.../scan/ui/webview/____.java:8` | `https://pan.baidu.com/simple/inchPhoto` | 证件照 |
| `classes12/.../scan/painting/ui/ScanPaintingDetailActivity.java:618` | `https://pan.baidu.com/simple/doodle/portal` | 涂鸦 |

---

## 附录 B 反编译失败类清单

**无。** 最终覆盖率 100%，`reverse/baidu/jadx-missing.log` 记录「仍未定位 0 个」。

中途 25 个 `DONT_GENERATE` 类已用 `--no-inline-anonymous --no-inline-kotlin-lambda --single-class` 全部补出到 `reverse/baidu/sources-recovered/`，抽检全部为 Kotlin lambda 合成类，**零业务逻辑**。

---

## 附录 C 【不变⑦⑧⑨】三条不变项速查

| 项 | 本文件的做法 | 反例（不合格） |
|---|---|---|
| 【不变⑦】行号定位 | 每条线索写 `classes15/sources/kg/__.java:641` | 写「某处代码里」 |
| 【不变⑧】可信度三档 | 「待确认」明确标注（`Sdata` 签名、服务端行为、密码算法） | 把推测写成「已确认」 |
| 【不变⑨】未发现写搜索方式 | §4.4「Cookie 持久化」、§9「证书绑定」「扫码登录」都写了 grep 命令与范围 | 只写「未发现」 |
