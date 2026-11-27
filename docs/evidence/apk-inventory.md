# APK 清单（apk-inventory）

- 产出任务：T-001（初版）/ T-002a（合并附件区完整清单）/ T-002b（更新反编译状态）/ **T-002d（更新百度 Flutter 判定与天翼 versionCode）**
- 盘点时间：2026-11-26
- 盘点方式：`aapt dump badging` / `unzip -l` / `sha256sum`，**仅静态读取，未反编译**
- 反编译状态：**部分完成** —— 坚果云（T-002b）与百度网盘（T-002d）已完整反编译并产出索引；天翼 / 123 / 夸克 / 阿里已完成壳确认（T-002d，协议覆盖 0）；蓝奏 / UC / 迅雷资料缺失不反编译

## 0. 汇总

| 项 | 数量 |
|---|---|
| 工作区根目录 APK | 1（天翼云盘） |
| 附件区文件总数 | 12 |
| ├─ 与网盘相关 | 8（其中 1 个为 `.apks` 分包） |
| └─ 与项目无关 | 4 |
| 已覆盖网盘 | 9 / 9（但 3 个资料性质不合格，见 §5） |
| 网页端资料 | **0** |

---

## 1. 工作区根目录

| 文件 | 大小(字节) | 包名 | 版本 | minSdk/targetSdk | 对应网盘 |
|---|---|---|---|---|---|
| `cloud189_v11.1.2.apk` | 387095539 | `com.cn21.ecloud` | 11.1.2（vc **638**） | **24 / 30** | 天翼云盘 |

sha256：`9491addb8c1a6471e4efc10c10df069978a7e780ba39d2f7a82996c6be1412fa`

**根目录只有 1 个 APK。**

---

## 2. 附件只读区（完整）

路径：`/root/.dsh/attachments/v1/files/<sha256>/`

### 2.1 与网盘相关（8 个）

| # | 文件名 | 大小(字节) | 包名 | 版本 | minSdk/targetSdk | 对应网盘 | sha256(前16) |
|---|---|---|---|---|---|---|---|
| 1 | `百度网盘_13.11.15.apks` | 189318289 | `com.baidu.drive.app` | 13.11.15（vc 3711） | 23 / 35 | 百度网盘 | `4caf9d3ced25e248` |
| 2 | `QuarkCloudDrive_V10.21.5.144_android_pf3300_(zh-cn)_release_(Build261119184713-arm64).apk` | 120924033 | `com.quark.clouddrive` | 10.21.5.144（vc 144） | 23 / 30 | 夸克网盘 | `2e0f175fd78f1bd2` |
| 3 | `坚果云 - v4.26.5.apk` | 60329002 | `nutstore.android` | 4.26.5（vc 174） | 22 / 34 | 坚果云 | `6a4ab9c7a48dbfac` |
| 4 | `蓝云 - v1.3.4.8.apk` | 4765515 | `com.tooyoung.lanzou` | 1.3.4.8（vc 211） | 21 / 33 | 蓝奏云（**第三方**） | `e977e6f66bd67686` |
| 5 | `123云盘_3.3.4.apk` | 114861135 | `com.mfcloudcalculate.networkdisk` | 3.3.4（vc 3244） | 24 / 34 | 123云盘 | `f22f940d905ec183` |
| 6 | `UCBrowser_V19.2.8.1564_android_pf145_(zh-cn)_ucrelease_(Build261116085652-arm64).apk` | 136173692 | `com.UCMobile` | 19.2.8.1564（vc 11564） | 23 / — | UC 网盘（**实为浏览器**） | `7b836da6aedcd18f` |
| 7 | `alicloud_shujubao_android_36176427979800.apk` | 163545725 | `com.alicloud.databox` | 6.21.0（vc 281） | 22 / 33 | 阿里云盘 | `6955e9d2cd9d654b` |
| 8 | `迅雷_25.2.91.280.apk` | 199853782 | `com.xunlei.downloadprovider` | 25.2.91.280（vc 18280） | 24 / 30 | 迅雷云盘（**下载器子模块**） | `43b088d6c63aba23` |

> 天翼云盘在根目录（§1），不在附件区。8 + 1 = 9 个网盘全部有对应文件。

### 2.2 与项目无关（4 个，仅登记）

| 文件名 | 大小(字节) | 识别结果 | 处置 |
|---|---|---|---|
| `base (1).apk` | 3373888 | 包名 `r2ixzp.toolkit` 1.1.1 | 无关，不纳入 |
| `thanox_9.6-prc.3404474.apk` | 21320481 | `github.tornaco.android.thanos` 9.6-prc（Thanox 系统工具） | 无关，不纳入 |
| `美团_12.63.474.apks` | 110924628 | 美团 .apks 分包（内部 4 个 apk） | 无关，不纳入 |
| `pandrive-src-20261121.zip` | 802827 | `pandrive-src/` 源码包（docs/、pandrive/） | 无关，不纳入 |

> 因此附件区 **12 个文件中只有 8 个与网盘相关**（第 9 个网盘天翼云盘在根目录）。

### 2.3 百度 .apks 内部分包明细

| 内部分包 | 大小(字节) |
|---|---|
| `base.apk` | 251854857 |
| `split_config.arm64_v8a.apk` | 156505408 |
| `split_config.xxhdpi.apk` | 727022 |
| `split_config.zh.apk` | 586715 |
| 合计 | 409674002 |

base.apk 的 compileSdkVersion=35，targetSdkVersion=35，minSdkVersion=23。

---

## 3. 结构特征（静态探测）

| 网盘 | dex 数 | Flutter（libapp.so + libflutter.so） | 加固 / 风控迹象 | 备注 |
|---|---|---|---|---|
| 天翼云盘 | 1 | ✅ | `assets/ijm_libexec.so`、`libexecmain.so`、`libsecurity.so` | Flutter 主体 |
| 123云盘 | 1 | ✅ | `assets/ijm_libexec.so`、`assets/ijm_libexecmain.so`、`libijm-emulator.so` | Flutter 主体；含大量阿里/阿里妈妈 SDK |
| 夸克网盘 | 12 | ✅ | `libdexloaderuc.so`、`libfakedexuc.so`、`liblibloaderuc.so`、`libquarkshell.so`、`libcycloneuc.so`、`libsgmainso-6.7.251102.so` | Flutter 主体；UC 系壳 + 阿里系安全 SDK |
| 阿里云盘 | 25 | ✅ | `sgmain`×4、`sgsecurity`×4（文件名字符串命中） | Flutter 主体；阿里系风控 SDK；含播放器全家桶 |
| UC浏览器 | 11 | ✅ | `libdalvikhack.so`、`libdexloaderuc.so`、`libfakedexuc.so`、`libbasshash.so` | 浏览器，非网盘 App |
| 迅雷 | 1 | ❌ | `libDexHelper.so`×2（娜迦/加固壳） | 下载器，云盘是子模块 |
| 坚果云 | 2 | ❌ | **无加固迹象**（仅 `libBugly_Native.so`、`libEncryptorP.so`、`libmupdf_java.so`） | 最小、最易侦察 |
| 蓝奏云 | 1 | ❌ | **无加固迹象**（仅 `libluajava.so`、`libmd5.so`） | 第三方客户端 |
| 百度网盘 | 20 | ❌ | 未见常见加固壳特征 | `.apks` 分包；**注意**：base.apk 内无 `lib/`，Flutter 运行时在 `split_config.arm64_v8a.apk` 内（见 §3.1） |

### 3.1 Flutter 判定含义（重要）

天翼云盘、123云盘、夸克网盘、阿里云盘、UC浏览器 **都是 Flutter 应用**，
业务逻辑编译进 `libapp.so`（Dart AOT 快照）。

**jadx / apktool 反编译 dex 拿不到网盘协议。** 静态反编译对这些网盘只能得到：
入口壳、原生桥接方法名、插件注册表、资源文件。

坚果云、蓝奏云、迅雷 **是 Java/Kotlin 主体**，静态反编译有实际价值。

**T-002d 实测修正（2026-11-27）**

| 网盘 | 原判定 | 实测 | 修正后 |
|---|---|---|---|
| 百度网盘 | Java/Kotlin 主体（非 Flutter） | base.apk 内 `lib/` **0 个**，但 `split_config.arm64_v8a.apk` 内有 `libflutter.so` 11036664 B + `libapp.so` 20972464 B，且 `assets/flutter_assets/` 1510 个文件 | **Java/Kotlin 主体 + Flutter 模块共存**（dex 侧仍是协议主战场） |
| 天翼云盘 | vc 1112 / minSdk 21 | `aapt dump badging` 实测 | **vc 638 / minSdk 24** |
| 天翼云盘 | 「Flutter 主体，有壳」 | 解包实测 | Flutter 主体 + **爱加密**（`assets/ijiami.dat` 22378961 B + `ijiami.ajm` 10890680 B），dex 仅 10136 B / **3 个壳桩类** |
| 123云盘 | 「Flutter 主体，有壳」 | 解包实测 | Flutter 主体，dex 仅 13916 B / **4 个壳桩类**；**未检出** `ijiami.*` 载荷 |
| 夸克 | 「Flutter 主体」 | 解包实测 | Flutter 主体，但 `libapp.so` 与 `libflutter.so` **均为 1 字节占位**；真载荷 `libkernelu4_uc_7z.so` 15938463 B 是 **7-zip 容器** |
| 阿里云盘 | 「Flutter 主体」 | 解包实测 | Flutter 主体 **混合架构**（原生壳 + jsbridge + Flutter），dex 25 个 / 70881 类，`com/alicloud` 15828 |

> 结论：Flutter 判定本身没错，但「百度非 Flutter」不完整、「天翼 vc 1112」是错的。
> 详细的壳确认证据见 `docs/evidence/index-天翼.md` / `index-123云盘.md` / `index-夸克.md` / `index-阿里云盘.md`。

---

## 4. 工具链现状

| 工具 | 状态 | 版本 |
|---|---|---|
| `apktool` | ✅ | 2.7.0 |
| `baksmali` / `smali` | ✅ | 可用 |
| `aapt` | ✅ | 可用 |
| `unzip` / `sha256sum` / `stat` | ✅ | 可用 |
| JDK | ✅ | 21.0.13.1（另有 17） |
| python3 | ✅ | 3.12.15 |
| `jadx` | ✅ **已安装** | 1.5.7（`reverse/_tools/jadx-1.5.7/`，74766896 B，详见 `docs/evidence/tooling.md`） |

---

## 5. 资料缺口与结论

### 5.1 资料缺失网盘（按 T-001 拍板结论）

| 网盘 | 现有资料 | 结论 |
|---|---|---|
| **蓝奏云** | 仅第三方客户端 `com.tooyoung.lanzou` 1.3.4.8，**非官方** | **资料缺失；原生登录不可行**；不硬凑 |
| **UC 网盘** | 仅 UC 浏览器 `com.UCMobile`，网盘功能是浏览器内 H5 | **资料缺失；原生登录不可行**；不硬凑 |
| **迅雷云盘** | 迅雷下载器 `com.xunlei.downloadprovider`，云盘为子模块 | **资料缺失；原生登录不可行**；不硬凑 |
| **全部 9 个** | 网页端资料 **0 个** | 网页端资料全部缺失 |

### 5.2 有可用资料的网盘（6 个）

百度网盘、天翼云盘、夸克网盘、坚果云、123云盘、阿里云盘。

其中 4 个是 Flutter（天翼、123、夸克、阿里），**静态反编译产出有限**。

### 5.3 反编译价值排序（供 T-002b 选靶）

| 排序 | 网盘 | Java/Kotlin 主体 | 加固 | 静态反编译价值 |
|---|---|---|---|---|
| 1 | **坚果云** | ✅ | 无 | **高**（2 dex，最易出结果） |
| 2 | 百度网盘 | ✅ | 无 | 高（20 dex，量大） |
| 3 | 天翼云盘 | ❌ Flutter | 有壳 | 低（仅入口） |
| 4 | 123云盘 | ❌ Flutter | 有壳 | 低 |
| 5 | 夸克网盘 | ❌ Flutter | UC 壳 | 低 |
| 6 | 阿里云盘 | ❌ Flutter | 阿里风控 | 低 |

---

## 6. 反编译状态

| 任务 | 状态 | 说明 |
|---|---|---|
| T-002a（本文件） | ✅ 完成 | 附件区 8 个 APK + 根目录 1 个 APK 全部登记 |
| T-002b | ✅ 完成（2026-11-26） | 只反了坚果云 1 个，跑通索引格式；见 `docs/evidence/index-坚果云.md` |
| T-002d（百度完整反编译） | ✅ 完成（2026-11-27） | 覆盖率 **100.0000%**（63259/63259）；见 `docs/evidence/index-百度.md` |
| T-002d（四个 Flutter 网盘壳确认） | ✅ 完成（2026-11-27） | 天翼 / 123 / 夸克 / 阿里，协议覆盖 **0**（技术栈决定） |
| 其余 3 个 | ⏸ 不反编译 | 蓝奏云 / UC 网盘 / 迅雷云盘，已判定资料缺失 |

**已反编译的网盘**：

- 坚果云 `nutstore.android` v4.26.5 → `reverse/nutstore/sources/`（8006 个 .java，约 67 MB）
  - 覆盖率：nutstore.android 包 2753 个类定义，覆盖 2752 个 = **100.0%**（2026-11-26 复核，口径 v2；旧的「2019/1912/缺 107」为统计口径错误，见 `index-坚果云.md` §2.1.1）
  - 工具：jadx 1.5.7（见 `docs/evidence/tooling.md`）
  - 加固：无。证书绑定：无。字符串 XOR 混淆：155 个解密器 / 4804 处调用

**已完整反编译的网盘（2 个）**：

- 坚果云 `nutstore.android` v4.26.5 → `reverse/nutstore/sources/`（8006 个 .java，约 67 MB），覆盖率 100.0%；
- 百度网盘 `com.baidu.drive.app` 13.11.15 → `reverse/baidu/sources/`（约 1.2 GB，按 dex 分批产出），覆盖率 **100.0000%**（63259/63259）。

**已做壳确认的网盘（4 个）**：天翼云盘、123云盘、夸克网盘、阿里云盘 —— 均为 Flutter 主体，协议覆盖 0，
  详见 `docs/evidence/index-天翼.md`、`index-123云盘.md`、`index-夸克.md`、`index-阿里云盘.md`。

**不做反编译的网盘（3 个）**：蓝奏云、UC 网盘、迅雷云盘（已判定资料缺失）。

产物统一放 `reverse/<网盘名>/`（已在 .gitignore）
