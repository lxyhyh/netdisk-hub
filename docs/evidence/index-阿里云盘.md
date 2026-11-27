# index-阿里云盘 —— 阿里云盘 Android 客户端静态侦察索引（壳确认模式）

- 对应任务：T-002d（B 项：Flutter 网盘壳确认）
- 生成日期：2026-11-27
- **任务范围**：只做壳确认，**不深挖 `libapp.so`、不研究 Dart AOT、不还原接口**
- 样本：附件区 `alicloud_shujubao_android_36176427979800.apk`
- 可信度取值：**已验证** / **推测** / **待确认**

> **⚠ 本文对阿里云盘协议的覆盖为 0。**
> 阿里云盘是 `Flutter 主体`（且 dex 体量大、含 jsbridge/H5 混合架构），业务逻辑主要在 `libapp.so`（Dart AOT）。
> 本文只记录壳形态、dex 构成与加固线索，**不产出任何接口清单**。

---

## 1. APK 基本信息

| 项 | 值 | 来源 |
|---|---|---|
| 文件 | `alicloud_shujubao_android_36176427979800.apk`（附件区 `69/6955e9d2cd9d654b816e8cacf15fe9a5c903c33e8209a545ca97a8c543d67389/`） | `ls` |
| 大小 | **163545725 B**（约 156 MiB） | `stat -c %s` |
| SHA256 | `6955e9d2cd9d654b816e8cacf15fe9a5c903c33e8209a545ca97a8c543d67389` | `sha256sum` |
| 包名 | `com.alicloud.databox` | `aapt dump badging` |
| versionName / versionCode | `6.21.0` / `281` | `aapt dump badging` |
| compileSdkVersion | 33 | `aapt dump badging` |
| minSdkVersion | 22 | `aapt dump badging` |
| targetSdkVersion | 33 | `aapt dump badging` |
| dex 数量 | **25 个**，`class_defs` 合计 **70881** | 纯 python 解析 `class_defs` |
| **【新增①·必填】主体技术栈判定** | **`B. Flutter 主体`（混合：原生壳 + jsbridge + Flutter）** | 见 §1.1 |
| 是否加固 | **未检出整包加固壳；但含阿里聚安全（sgmain + sgsecuritybody）8 个文件** | 见 §9 |
| 签名 | DN `CN=databox, OU=alicloud, O=alicloud, L=hangzhou, ST=zhejiang, C=86`；SHA-256 `9829ab3ff912e3cc33d519da65af54f0c5ae8dc0b5fe6b77ce87700019288884` | `apksigner verify --print-certs` |
| 原生库 | `lib/` 下 **107 条**，**仅 `arm64-v8a`** | `unzip -l` |
| Manifest 明文流量 | `android:usesCleartextTraffic="true"`；**无** `networkSecurityConfig` 属性；但包内**有** `res/xml/network_security_config.xml`（672 B） | `aapt dump xmltree` + `unzip -l` |

### 1.1 主体技术栈判定依据

| 判据 | 实测 | 结论 |
|---|---|---|
| `lib/<abi>/libflutter.so` | **有**：arm64-v8a **10265304 B**（sha256 `c9f036c37d16371ab3639d6438e132c40f4ffad22d8b4019e891c5555da84290`） | Flutter 引擎 |
| `lib/<abi>/libapp.so` | **有**：arm64-v8a **16794520 B**（sha256 `82f8b331ec368eb9e4eb94cf88541e5a75c2c2fa1018c6c6c8ecf7b0f1933f5c`） | **Dart AOT 业务快照** |
| `assets/flutter_assets/` | **有**：仅 **46 个文件 / 861825 B**（很薄，说明 Flutter 只承载少量页面） | Flutter 资源 |
| dex 内业务类 | **有**：`com/alicloud` **15828** 类；命中 `Lcom/alicloud/databox/jsbridge/request/OpenApiRequest;`、`Lcom/alicloud/databox/jsbridge/handler/biz/OpenApiCreateHandler;`、`Lcom/alicloud/databox/webview/windvane/ALCloudDriveBridge;` | **原生壳 + jsbridge + WebView 混合** |

→ **判定 `B. Flutter 主体`（混合架构）。** 与天翼/123 的纯 Flutter 壳不同：阿里有 15828 个 `com/alicloud` dex 类与 jsbridge 桥接层，**但本任务按范围不做协议还原**（不深挖 dex 业务，也不动 libapp.so）。

**dex 包分布（前 12）**：com/alicloud 15828、com/alibaba 3959、com/taobao 3913、com/google 3416、org/bouncycastle 3243、androidx/media3 2322、kotlin/reflect 1956、com/youku 1821、io/reactivex 1669、cn/hutool 1506、com/ali 1443、com/amap 1399。

---

## 2. 反编译工具及版本

| 项 | 值 |
|---|---|
| 工具 | jadx 1.5.7（复用，未新下载） |
| **是否执行完整反编译** | **未执行**，按 T-002d B 项范围不适用 |
| 实际执行 | `unzip -l` / `aapt dump badging` / `aapt dump xmltree` / `apksigner verify` / 纯 python 解析 25 个 dex 的 `class_defs` / `file` / `readelf -lW` |
| 采集脚本 | `reverse/collect_flutter_facts.py`、`reverse/flutter-dex-scan.py`、`reverse/shell-dex.py`、`reverse/check-flutter-libs.py` |

### 2.1 【新增②·必填】反编译覆盖率

| 项 | 值 |
|---|---|
| dex 内类定义总数 | **70881**（25 个 dex） |
| 覆盖率 | **未执行完整 jadx 反编译，按任务范围不适用** |
| 缺失类是否影响核心链路 | **不适用**（本任务不产出协议结论；协议层主要在 `libapp.so`） |

---

## 3. HTTP 客户端相关类/方法

> **【降级⑥】** 本节仅对 `Java/Kotlin 主体` 适用。**阿里云盘是 Flutter 主体（混合），dex 侧的 HTTP 栈服务于原生壳/WebView 桥，不是网盘文件协议。**

- dex 内可见 `io/reactivex`（1669）、`androidx/media3`（2322）、`org/bouncycastle`（3243）等通用库；
- 搜索方式：未执行反编译，改用 `strings` 扫描 dex 字符串表 → 命中大量三方域名，**未验证属于网盘文件 API**；
- → 结论：**dex 内无足够证据确认网盘文件协议 HTTP 实现**。

---

## 4. 认证相关类/方法

> 当前只做壳检查。**未还原 Dart AOT，无法确认登录接口。**

| 项 | 值 |
|---|---|
| dex 内登录相关类 | 存在阿里统一登录体系类（`com/alibaba`、`com/taobao` 包内），**未验证**是否适用于云盘登录 → **待确认** |
| 登录可行性结论 | **未知**（属 T-002c 范围，本任务不判定） |
| 攻关点（待动态分析） | 登录域名、字段、签名、设备指纹（阿里聚安全参与度）均不可得 |

---

## 5. 文件列表相关类/方法

**无足够证据确认接口。** dex 内有 jsbridge 桥接类（`OpenApiRequest` / `OpenApiCreateHandler` / `ALCloudDriveBridge`），**推测**是 H5/原生互相调用的通道，**未验证**其与文件列表的关系 → 后续需动态分析。分页方式 **待确认**。

---

## 6. 上传相关类/方法

**无足够证据确认接口。** Body 形态 **待确认**；是否分片 **待确认**。

---

## 7. 下载相关类/方法

**无足够证据确认接口。** 是否支持 Range **待确认**。

---

## 8. 关键字符串常量

### 8.1 域名

**未确认到网盘文件 API 域名。** dex 内域名常量众多（CDN/埋点/播放/三方 SDK），按任务约束**不把未验证域名写成结论**。

### 8.2 接口清单

**本任务不产出接口清单**（无静态证据，禁止编造）。

---

## 9. 加固 / 混淆 / 证书绑定 / 设备指纹

| 检查项 | 结果 | 依据 |
|---|---|---|
| 整包加固壳 | **未检出**（无 `libDexHelper`、无 `ijiami.*`、无 `libjiagu*`）；25 个 dex 均可正常解析 | `unzip -l` + 解析 `class_defs` |
| 风控 SDK | **有：阿里聚安全 8 个文件** | 见 §9.1 |
| 证书绑定 | **dex 内未确认**；native 侧 → **待确认** | — |
| 网络安全配置 | 包内 `res/xml/network_security_config.xml`（672 B）存在，但 Manifest **未引用**（无 `networkSecurityConfig` 属性）→ **推测**该 xml 由某 SDK 自带、可能不生效 → **待确认** | `aapt dump xmltree` + `unzip -l` |
| 反调试 | **未检出明显特征** | `unzip -l` |
| 设备指纹 | **推测**由阿里聚安全提供（`libsgsecuritybody`），**未验证**具体采集项 | 文件存在 |
| 混淆 | 有（R8），但 `com/alicloud` 包名与类名大量保留可读 | 包分布 |

### 9.1 【新增③·必填】加固 / 风控 SDK 清单

| SDK | 文件或包名 | 数量 | 影响 |
|---|---|---|---|
| 阿里聚安全 sgmain | `libsgmain.so`（39922 B）、`libsgmainso-6.6.240404.so`（**1873964 B**）、`libsgmainso.ucb.so`（112 B）、`libsgmainso.version.so`（10 B） | 4 | **风控/设备指纹，可能参与请求签名 → 待确认** |
| 阿里聚安全 sgsecuritybody | `libsgsecuritybody.so`（45136 B）、`libsgsecuritybodyso-6.6.240404.so`（**649484 B**）、`libsgsecuritybodyso.ucb.so`（96 B）、`libsgsecuritybodyso.version.so`（10 B） | 4 | 安全体（采集/校验），与登录风控可能相关 |
| Flutter 引擎 | `libflutter.so`（10265304 B） | 1 | 业务快照容器 |
| Dart AOT 业务 | `libapp.so`（16794520 B） | 1 | **协议主体在这里，静态不可读** |
| 其他 | 107 条 `lib/` 中的其余项（播放器/图像/网络/加解密）未逐条分类 | 95+ | 与协议关系**待确认** |

**判定：** 阿里云盘有完整的阿里聚安全风控体系（8 个文件、两个 SDK），即使能读到 Dart 协议，签名/设备指纹环节仍可能挡在 native 层。

---

## 10. 线索可信度汇总

| 结论 | 可信度 |
|---|---|
| 阿里云盘是 `Flutter 主体`（混合架构） | 已验证 |
| `libapp.so` 16794520 B、`libflutter.so` 10265304 B（仅 arm64-v8a） | 已验证 |
| `flutter_assets` 仅 46 文件 / 861825 B | 已验证 |
| dex 25 个、70881 类，`com/alicloud` 15828 | 已验证 |
| 存在 jsbridge / WindVane 桥接类 | 已验证（类名命中） |
| 阿里聚安全 8 个文件（sgmain 4 + sgsecuritybody 4） | 已验证 |
| 无整包加固壳 | 已验证（搜索方式已写） |
| `network_security_config.xml` 是否生效 | 待确认 |
| 登录/列表/上传/下载协议 | **待确认（静态不可得，需动态分析）** |
| 阿里云盘原生登录可行性 | **不在本文范围**（属 T-002c） |

---

## 11. 后续需动态分析

1. **抓包**（先确认 `network_security_config.xml` 是否生效、是否信任用户 CA）；
2. **Dart AOT 逆向**（`libapp.so` 16.8 MB，成本高）；
3. **jsbridge 通道分析**（`OpenApiRequest` / `OpenApiCreateHandler` / `ALCloudDriveBridge` 是否暴露文件操作能力，需动态验证）；
4. **阿里聚安全参与度**（哪些签名参数由 sgmain/sgsecuritybody 生成）。

---

## 附录 A 非 API 域名 / 链接

**未确认**（见 §8.1）。

## 附录 B 反编译失败类清单

**不适用**（未执行完整反编译）。

## 附录 C 三条不变项速查

| 项 | 本文件的做法 | 反例 |
|---|---|---|
| 【不变⑦】行号定位 | 每个结论给 sha256 / 字节数 / 文件路径 | 写「有风控」就完事 |
| 【不变⑧】可信度三档 | jsbridge 与风控参与度明确写「推测/待确认」 | 把桥接类说成文件 API |
| 【不变⑨】未发现写搜索方式 | §3、§8.1、§9 都写了扫描方式 | 只写「未发现」 |
