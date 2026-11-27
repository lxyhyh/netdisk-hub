# index-夸克 —— 夸克网盘 Android 客户端静态侦察索引（壳确认模式）

- 对应任务：T-002d（B 项：Flutter 网盘壳确认）
- 生成日期：2026-11-27
- **任务范围**：只做壳确认，**不深挖 `libapp.so`、不研究 Dart AOT、不还原接口**
- 样本：附件区 `QuarkCloudDrive_V10.21.5.144_android_pf3300_(zh-cn)_release_(Build261119184713-arm64).apk`
- 可信度取值：**已验证** / **推测** / **待确认**

> **⚠ 本文对夸克网盘协议的覆盖为 0。**
> 夸克是 `Flutter 主体`，且比另外三个更极端：**`libapp.so` 与 `libflutter.so` 都是 1 字节占位文件**，
> 真正的 Dart 快照被打包进 `libkernelu4_uc_7z.so`（一个 7-zip 容器）。本文只记录壳形态与加固线索，**不产出任何接口清单**。

---

## 1. APK 基本信息

| 项 | 值 | 来源 |
|---|---|---|
| 文件 | `QuarkCloudDrive_V10.21.5.144_...arm64).apk`（附件区 `2e/2e0f175fd78f1bd2872d739fc8be56114730b82fbb2734b69f4ff6aca50f7e05/`） | `ls` |
| 大小 | **120924033 B**（约 115 MiB） | `stat -c %s` |
| SHA256 | `2e0f175fd78f1bd2872d739fc8be56114730b82fbb2734b69f4ff6aca50f7e05` | `sha256sum` |
| 包名 | `com.quark.clouddrive` | `aapt dump badging` |
| versionName / versionCode | `10.21.5.144` / `144` | `aapt dump badging` |
| compileSdkVersion | 34 | `aapt dump badging` |
| minSdkVersion | 23 | `aapt dump badging` |
| targetSdkVersion | 30 | `aapt dump badging` |
| dex 数量 | **12 个**，`class_defs` 合计 **115475** | 纯 python 解析 `class_defs` |
| **【新增①·必填】主体技术栈判定** | **`B. Flutter 主体`（Dart 快照在 7z 容器内）** | 见 §1.1 |
| 是否加固 | **是（多重：7z 容器 + 假 dex APK + sgmain 风控 + dexloader/libloader）** | 见 §9 |
| 签名 | DN `CN=uc, OU=uc, O=uc, L=gz, ST=gd, C=cn`；SHA-256 `bbe2ff269828a0d922498ee87f65afe769c27d62f489d5c19b9cc6c444c80811` | `apksigner verify --print-certs` |
| 原生库 | `lib/` 下 **49 条**，**仅 `arm64-v8a`** | `unzip -l` |
| Manifest 明文流量 | `android:usesCleartextTraffic="true"`；`android:networkSecurityConfig="@0x7f150017"`，且**包内有** `res/xml/network_security_config.xml`（468 B） | `aapt dump xmltree` + `unzip -l` |

### 1.1 主体技术栈判定依据

| 判据 | 实测 | 结论 |
|---|---|---|
| `lib/<abi>/libflutter.so` | 存在但**仅 1 字节**（内容 `0x00`） | **占位/被替换** |
| `lib/<abi>/libapp.so` | 存在但**仅 1 字节**（内容 `0x00`） | **占位/被替换** |
| 真正的 Dart 载荷 | `libkernelu4_uc_7z.so` **15938463 B**，`file` 判定为 **7-zip archive**（magic `37 7a bc af 27 1c`） | **Dart 快照被打包进 7z 容器** |
| 另有「假 dex」 | `libfakedexuc.so` 3125 B，`file` 判定为 **APK**（`PK` 头 + AndroidManifest.xml + APK Signing Block） | 扰动/诱饵 |
| `assets/flutter_assets/` | **有**：3547 个文件 / 14651436 B，含 `packages/clouddrive_shell/` | Flutter 资源 |
| dex 内业务类 | dex 共 115475 类，但主体是 `com/ucpro`(33475)、`com/quark`(9400)、`com/uc`(8489) 等壳/容器/插件/三方 SDK | **dex 不承载协议** |

→ **判定 `B. Flutter 主体`。§3–§8 全部「dex 内未发现协议实现」；且由于 `libapp.so` 是 1 字节占位，连「读 Dart 快照」这条路在静态上也走不通。**

**为什么 dex 有 115475 个类还不能用**：这些类集中在 `com/ucpro`（UC 内核/浏览器容器）、`com/quark`、`com/shuqi`（书旗）、`com/google`、`com/aieditor`、`com/huawei`、`com/amap` 等**平台组件**上，属于「应用外壳 + 三方 SDK」，不是网盘文件协议。已做业务命中统计（`BIZ_HITS`=14253）但**未逐类验证**，按任务约束不写成结论。

---

## 2. 反编译工具及版本

| 项 | 值 |
|---|---|
| 工具 | jadx 1.5.7（复用，未新下载） |
| **是否执行完整反编译** | **未执行**，按 T-002d B 项范围不适用 |
| 实际执行 | `unzip -l` / `aapt dump badging` / `aapt dump xmltree` / `apksigner verify` / 纯 python 解析 12 个 dex 的 `class_defs` / `file` 判类型 / `readelf -lW` |
| 采集脚本 | `reverse/collect_flutter_facts.py`、`reverse/flutter-dex-scan.py`、`reverse/shell-dex.py`、`reverse/check-flutter-libs.py` |

### 2.1 【新增②·必填】反编译覆盖率

| 项 | 值 |
|---|---|
| dex 内类定义总数 | **115475**（12 个 dex） |
| 覆盖率 | **未执行完整 jadx 反编译，按任务范围不适用** |
| 缺失类是否影响核心链路 | **不适用**（协议在 7z 容器内的 Dart 快照，不在 dex） |
| 说明 | 即使 100% 反编译这 115475 个类，也拿不到网盘协议；本任务不投入 |

---

## 3. HTTP 客户端相关类/方法

> **【降级⑥】** 本节仅对 `Java/Kotlin 主体` 适用。**夸克是 Flutter 主体，dex 侧无网盘协议 HTTP 栈。**

- dex 内可检索到 `io/ktor`（1237 类，Kotlin HTTP 客户端）与 `com/uc` 系网络类，但**没有指向夸克网盘接口的调用链**；
- 搜索方式：`grep -rl 'quark.*api\|pan.quark.cn' reverse/quark/*.dex` 未做（未反编译），改用 `strings` 扫描 dex 字符串表 → 仅命中通用域名常量，**未验证是否用于网盘业务**；
- → 结论：**dex 内无足够证据确认 HTTP 协议实现**。

---

## 4. 认证相关类/方法

> 当前只做壳检查。**未还原 Dart AOT（且静态也不可得），无法确认登录接口。**

| 项 | 值 |
|---|---|
| dex 内登录相关类 | **未确认**（`com/ucpro` 内有 UC 账号体系类，但属浏览器内核，**未验证**是否用于夸克网盘） |
| 登录可行性结论 | **未知**（属 T-002c 范围，本任务不判定） |
| 攻关点（待动态分析） | 登录域名、字段、签名、设备指纹均不可得 |

---

## 5. 文件列表相关类/方法

**无足够证据确认接口**，逻辑在 7z 容器内的 Dart 快照 → 后续需动态分析。分页方式 **待确认**。

---

## 6. 上传相关类/方法

**无足够证据确认接口。** Body 形态 **待确认**；是否分片 **待确认**。

---

## 7. 下载相关类/方法

**无足够证据确认接口。** 是否支持 Range **待确认**。

---

## 8. 关键字符串常量

### 8.1 域名

**未确认到网盘 API 域名。** dex 内有大量三方 SDK 域名常量，但按任务约束**不把未验证的域名写成本项目结论**。

### 8.2 接口清单

**本任务不产出接口清单**（无静态证据，禁止编造）。

---

## 9. 加固 / 混淆 / 证书绑定 / 设备指纹

| 检查项 | 结果 | 依据 |
|---|---|---|
| 加固/包裹 | **多重**：① `libapp.so`/`libflutter.so` 被替换为 1 字节占位 ② 真载荷 `libkernelu4_uc_7z.so` 是 7z 容器 ③ `libfakedexuc.so` 是假 APK ④ `libdexloaderuc.so` + `liblibloaderuc.so` 动态加载 ⑤ `libquarkshell.so` 自定义 shell | `file` + `unzip -l` |
| 已知整包壳 | **未检出**爱加密/梆梆/腾讯御安全（无 `ijiami.dat`、无 `libjiagu`、无 `libDexHelper`）；但上述自定义包裹层等效于加固 | `unzip -l` |
| dex 性质 | 12 个 dex 均为标准 DEX（可解析 `class_defs`），但内容是外壳/三方，非协议 | 纯 python 解析 |
| 证书绑定 | **dex 内未确认**；native 侧（`libtnet-3.1.14.so` 是 UC 网络库）**可能有 pin → 待确认** | `libtnet` 存在 |
| 网络安全配置 | 包内 `res/xml/network_security_config.xml` **存在**（468 B）。内容未逐条解析 → **待确认**是否允许用户 CA | `unzip -l` |
| 反调试 / 反模拟器 | 有迹象：`libcycloneuc.so`（UC 安全组件）、`libbasshash.so` | 文件存在 |
| 设备指纹 | **待确认**（`com/uc` 系有 ID 体系，未验证是否用于网盘请求） | — |
| 1 字节占位库 | **15 个**：libandroidlame / libcrashsdk / libdatawings / libdecode100250 / libflutter / libgpuimage / libmarsulog / libnearby_nstackx / libqstream / libquaramera / libturbojpeg / libucffavc / libuncommon_ocr / libunet / libunet_plugin | `stat` 逐个确认 1 B |

### 9.1 【新增③·必填】加固 / 风控 SDK 清单

| SDK | 文件或包名 | 数量 | 影响 |
|---|---|---|---|
| 7z 载荷容器 | `libkernelu4_uc_7z.so`（15938463 B，7-zip archive） | 1 | **Dart 快照/核心代码在此，静态解析成本高** |
| 假 dex APK | `libfakedexuc.so`（3125 B，实为 APK） | 1 | 干扰分析 |
| 动态加载器 | `libdexloaderuc.so`（18280 B）、`liblibloaderuc.so`（43504 B） | 2 | 运行期加载，静态看不到加载内容 |
| UC Shell | `libquarkshell.so`（297600 B，ELF aarch64 stripped） | 1 | 运行期壳 |
| 阿里聚安全（sgmain） | `libsgmain.so`（115083 B）、`libsgmainso-6.7.251102.so`（**2990668 B**）、`libsgmainso.version.so`（16 B） | 3 | **风控 SDK，可能参与请求签名 → 待确认** |
| UC 安全 | `libcycloneuc.so`（61520 B）、`libbasshash.so`（235688 B） | 2 | 风控/哈希，**待确认**用途 |
| UC 网络 | `libtnet-3.1.14.so`（260224 B） | 1 | UC 自有网络库，**可能承载 dex 侧请求**（不是网盘协议） |
| 加密/解析 | `libencrypt.so`（202808 B）、`libpars.so`（2050576 B）、`libsqlite3.so`（1917000 B） | 3 | 通用组件 |

**判定：** 夸克是本批 4 个 Flutter 网盘里**静态侦察收益最低**的一个——三层包裹（占位 so + 7z 容器 + 动态加载器）叠加阿里聚安全，纯静态路线基本走不通。

---

## 10. 线索可信度汇总

| 结论 | 可信度 |
|---|---|
| 夸克是 `Flutter 主体` | 已验证 |
| `libapp.so` 与 `libflutter.so` 均为 1 字节占位 | 已验证（`stat` + `file`） |
| `libkernelu4_uc_7z.so` 是 7-zip 容器（15938463 B） | 已验证（magic `37 7a bc af 27 1c`） |
| `libfakedexuc.so` 是 APK | 已验证（`PK` 头 + Manifest） |
| dex 115475 类，主体为外壳/三方 SDK | 已验证（包分布统计） |
| 存在 sgmain 风控（3 个文件） | 已验证 |
| `network_security_config.xml` 是否信任用户 CA | 待确认 |
| 登录/列表/上传/下载协议 | **待确认（静态不可得，需动态分析）** |
| 夸克原生登录可行性 | **不在本文范围**（属 T-002c） |

---

## 11. 后续需动态分析

1. **抓包**（需用户 CA，先确认 `network_security_config.xml` 内容）；
2. **7z 载荷解包 + Dart 快照分析**（成本极高，且可能仍有二次加密）；
3. **动态加载器行为观察**（`libdexloaderuc` / `liblibloaderuc` 在运行期加载了什么）；
4. **不需要常规脱壳**（没有整包壳，dex 可读但无价值）。

---

## 附录 A 非 API 域名 / 链接

**未确认**（见 §8.1）。

## 附录 B 反编译失败类清单

**不适用**（未执行完整反编译）。

## 附录 C 三条不变项速查

| 项 | 本文件的做法 | 反例 |
|---|---|---|
| 【不变⑦】行号定位 | 给 sha256 / 字节数 / magic / 脚本路径 | 写「有加固」就完事 |
| 【不变⑧】可信度三档 | 大量用「待确认」，不把 UC 内核类当成网盘协议 | 把 `com/ucpro` 说成网盘 SDK |
| 【不变⑨】未发现写搜索方式 | §3、§8.1 写明 strings 扫描做法 | 只写「未发现」 |
