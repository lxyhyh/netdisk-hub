# index-天翼云盘 —— 天翼云盘 Android 客户端静态侦察索引（壳确认模式）

- 对应任务：T-002d（B 项：Flutter 网盘壳确认）
- 生成日期：2026-11-27
- **任务范围**：只做壳确认，**不深挖 `libapp.so`、不研究 Dart AOT、不还原接口**
- 样本：`/root/workspace/网盘/cloud189_v11.1.2.apk`
- 可信度取值：**已验证**（直接读文件/解包核对）、**推测**、**待确认**

> **⚠ 本文对天翼云盘协议的覆盖为 0。**
> 天翼云盘是 `Flutter 主体`，业务逻辑编译进 `libapp.so`（Dart AOT 快照），且 dex 侧被爱加密加固成壳桩。
> jadx 反编译 dex 只能得到壳桩类名与少量原生桥接符号，**拿不到登录流程、文件列表、上传、下载任何协议细节**。
> 本文按任务要求只记录「壳长什么样」，不编造任何接口。

---

## 1. APK 基本信息

| 项 | 值 | 来源 |
|---|---|---|
| 文件 | `cloud189_v11.1.2.apk`（工作区根目录） | `ls` |
| 大小 | **387095539 B**（约 369 MiB） | `stat -c %s` |
| SHA256 | `9491addb8c1a6471e4efc10c10df069978a7e780ba39d2f7a82996c6be1412fa` | `sha256sum` |
| 包名 | `com.cn21.ecloud` | `aapt dump badging` |
| versionName / versionCode | `11.1.2` / **`638`** | `aapt dump badging` |
| compileSdkVersion | 34 | `aapt dump badging` |
| minSdkVersion | 24 | `aapt dump badging` |
| targetSdkVersion | 30 | `aapt dump badging` |
| dex 数量 | **1 个，仅 10136 B** | `unzip -l` |
| **【新增①·必填】主体技术栈判定** | **`B. Flutter 主体`** | 见 §1.1 |
| 是否加固 | **是（爱加密壳）** | 见 §9 |
| 签名 | DN `CN=www.21cn.com, OU=世纪龙信息网络有限责任公司, O=世纪龙信息网络有限责任公司, L=GuangZhou, ST=GuangDong, C=CN`；SHA-256 `b122369063be8c5a809c449c3e5655afe17bd8059a8273b498cc9c8b7e1b9ae8` | `apksigner verify --print-certs` |
| 原生库 | `lib/` 下 **156 条**，两个 ABI（`arm64-v8a` + `armeabi-v7a`） | `unzip -l` |
| Manifest 明文流量 | `android:usesCleartextTraffic="true"`（`0xffffffff`）；`android:networkSecurityConfig="@0x7f160004"`（资源 ID 化，`res/xml/` 内**无**同名明文 xml） | `aapt dump xmltree AndroidManifest.xml` |

> **勘误**：`docs/evidence/apk-inventory.md` 早期登记的天翼 versionCode 为 `1112`，实测 `aapt` 输出为 **`638`**（versionName 才是 11.1.2）。已在 T-002d 同步修正。

### 1.1 主体技术栈判定依据

| 判据 | 实测 | 结论 |
|---|---|---|
| 有 `lib/<abi>/libflutter.so` | **有**：arm64-v8a 10464784 B（sha256 `99cf825832a0437f9770d3fc5644daa415419805f71b2ed31786506a2d77bc1d`）；armeabi-v7a 7360020 B（sha256 `01d626b8efc17c9ceeab3d5d9eac9af135257109801fad1bf1ced84434052d1e`） | Flutter 引擎 |
| 有 `lib/<abi>/libapp.so` | **有**：arm64-v8a **18138032 B**（sha256 `e1089f045a0f3bd46775c3a16c3718cc7a15ee786a0b1984e846156c53d3e8c8`）；armeabi-v7a 20365916 B（sha256 `9bb590c611aafc71abf21d58c372fcc0083e26150def4377cd98bbaf9286259`） | **Dart AOT 业务快照** |
| 有 `assets/flutter_assets/` | **有**：1537 个文件 / 53530633 B | Flutter 资源 |
| dex 内是否有业务类 | **无**：dex 仅 10136 B，`class_defs` 只有 **3 个类**（`Ls/h/e/l/l/C;`、`Ls/h/e/l/l/N;`、`Ls/h/e/l/l/S;`） | 壳桩 |

→ **判定 `B. Flutter 主体`。§3–§8 全部为「dex 内未发现」，不是遗漏，是技术栈决定。**

---

## 2. 反编译工具及版本

| 项 | 值 |
|---|---|
| 工具 | jadx 1.5.7（`reverse/_tools/jadx-1.5.7/`，复用，未新下载） |
| **是否执行完整反编译** | **未执行**。按 T-002d 任务范围（B 项只做壳确认），**不适用** |
| 实际执行的操作 | `unzip -l` / `aapt dump badging` / `aapt dump xmltree` / `apksigner verify` / 纯 python 解析 dex `class_defs` / `file` 判类型 / `readelf -lW` |
| 采集脚本 | `reverse/collect_flutter_facts.py`（输出 `reverse/flutter-facts.txt`）、`reverse/flutter-dex-scan.py`（输出 `reverse/flutter-dex-classes.txt`）、`reverse/shell-dex.py` |

### 2.1 【新增②·必填】反编译覆盖率

| 项 | 值 |
|---|---|
| dex 内目标包类定义总数 | 仅 **3** 个（`Ls/h/e/l/l/*` 壳桩） |
| 覆盖率 | **未执行完整 jadx 反编译，按任务范围不适用** |
| 缺失类是否影响核心链路 | **不适用**（dex 内没有业务逻辑可丢） |
| 说明 | 即使 100% 反编译这 3 个壳桩类，也拿不到任何协议信息，故本任务不执行 |

---

## 3. HTTP 客户端相关类/方法

> **【降级⑥】** 本节仅对 `Java/Kotlin 主体` 适用。**天翼云盘是 Flutter 主体，dex 侧 HTTP 栈不存在。**

- dex 内 `class_defs` 仅 3 个类（见 §1.1），**无 okhttp / okio / retrofit / Volley / HttpURLConnection 包装类**；
- 搜索方式：`grep -a -c 'okhttp3\|retrofit2\|HttpURLConnection' <dex 解出的字符串表>` → 0 命中；
- → 结论：**dex 内未发现 HTTP 客户端。真实 HTTP 栈在 `libapp.so`（Dart `dart:io` HttpClient / 第三方 Dart HTTP 包）内。**

---

## 4. 认证相关类/方法

> 当前只做壳检查。**未还原 Dart AOT，无法确认登录接口、加密方式、token 结构。**

| 项 | 值 |
|---|---|
| dex 内登录相关类 | **未发现**（dex 仅壳桩 3 类） |
| 登录页实现层 | **推测**在 Flutter 页面（`assets/flutter_assets/` 内），未验证 |
| 登录可行性结论 | **未知**（属 T-002c 范围；本任务不判定） |
| 攻关点（待动态分析） | 登录请求的字段与签名算法；是否有设备指纹参与 |

**注意**：`libsecurity.so`（21528 B）与 `libexecmain.so`（215640 B）/ `libexec.so`（765648 B）的存在**推测**与安全/签名有关，但**未验证**其具体职责，不写入任何接口结论。

---

## 5. 文件列表相关类/方法

**无足够证据确认接口。** dex 内无文件列表相关类；列表逻辑在 `libapp.so` 内 → **后续需动态分析**。

分页方式：**待确认**（静态不可得）。

---

## 6. 上传相关类/方法

**无足够证据确认接口。**

| 项 | 值 |
|---|---|
| 上传实现层 | 推测在 `libapp.so`（未验证） |
| Body 形态 | **待确认** |
| 是否分片 | **待确认** |

---

## 7. 下载相关类/方法

**无足够证据确认接口。**

| 项 | 值 |
|---|---|
| 下载实现层 | 推测在 `libapp.so`（未验证） |
| 是否支持 Range 断点续传 | **待确认** |

---

## 8. 关键字符串常量

### 8.1 域名

**dex 内未发现任何域名常量**（搜索方式：`strings` 提取 dex 内 UTF-8 串后 grep `https?://` → 0 命中）。

### 8.2 接口清单

**本任务不产出接口清单。** 原因：dex 内 0 条协议线索，任何「接口路径」都只能靠猜，按任务约束**禁止编造**。

---

## 9. 加固 / 混淆 / 证书绑定 / 设备指纹

| 检查项 | 结果 | 依据 |
|---|---|---|
| 加固壳 | **爱加密（ijiami）** | `assets/ijiami.dat` **22378961 B** + `assets/ijiami.ajm` **10890680 B**；`lib/arm64-v8a/libijm-emulator.so` 166360 B、`libijmDataEncryption.so` 433744 B |
| dex 性质 | **壳桩**：仅 3 个类 `Ls/h/e/l/l/{C,N,S};`，无业务实现 | 纯 python 解析 `class_defs` |
| 证书绑定 | **dex 内未发现**（dex 无 SSL 相关类）。native 侧是否 pin → **待确认** | — |
| 反调试 / 反模拟器 | 有迹象：`libijm-emulator.so`（爱加密模拟器检测） | 文件存在 |
| 设备指纹 | **待确认**（dex 侧无实现） | — |

### 9.1 【新增③·必填】加固 / 风控 SDK 清单

| SDK | 文件或包名 | 数量 | 影响 |
|---|---|---|---|
| 爱加密 | `assets/ijiami.dat`（22378961 B）、`assets/ijiami.ajm`（10890680 B）、`libijm-emulator.so`（166360 B）、`libijmDataEncryption.so`（433744 B） | 4 | **dex 被加密，静态反编译无业务价值**；需脱壳才能看 dex，但 dex 里本来也几乎没有业务（业务在 libapp.so） |
| Flutter 引擎 | `libflutter.so`（10464784 / 7360020 B） | 2 | 业务快照容器 |
| Dart AOT 业务 | `libapp.so`（18138032 / 20365916 B） | 2 | **协议全在这里，静态不可读** |
| 自有安全 | `libsecurity.so`（21528 B） | 1 | 职责未验证 → 待确认 |
| 自有执行壳 | `libexec.so`（765648 B）、`libexecmain.so`（215640 B）、`libexec_x86.so`（820272 B）、`libexecmain_x86.so`（215648 B） | 4 | 职责未验证 → 待确认 |
| 其他重型原生库 | `libhippy.so` 16325656 B、`libopencv_java3.so` 18978928 B、`libonnxruntime.so` 16035904 B、`libkjffmpeg.so` 14766896 B、`libtusdk-pulse.so` 14122728 B、`libaplayer_android_1.2.2.408.so` 11395024 B、`libxl_thunder_sdk.so` 9643928 B | 7+ | 与网盘协议无关（播放器/AI/图像） |
| 大体积资源 | `assets/3dspeaker...onnx` 39593765 B、`res/qq6.gif` 8666338 B | 2 | 与协议无关，仅解释 APK 为何 369 MiB |
| 证书/Provider | `META-INF/services/java.security.Provider`、`libsecurity.so` 双 ABI | 3 | 自定义 JCE Provider，**待确认**是否参与请求签名 |

---

## 10. 线索可信度汇总

| 结论 | 可信度 |
|---|---|
| 天翼云盘是 `Flutter 主体` | 已验证 |
| dex 为爱加密壳桩，仅 3 个类 | 已验证 |
| `libapp.so` 双 ABI 存在，arm64 18138032 B | 已验证 |
| `flutter_assets` 1537 文件 / 53530633 B | 已验证 |
| versionCode 实为 638（非 1112） | 已验证 |
| dex 内无 HTTP 栈、无域名、无登录类 | 已验证（搜索方式已写） |
| 登录/列表/上传/下载协议 | **待确认（静态不可得，需动态分析）** |
| `libsecurity.so` / `libexec*.so` 的具体职责 | 待确认 |
| 天翼云盘原生登录可行性 | **不在本文范围**（属 T-002c） |

---

## 11. 后续需动态分析（壳确认后的下一步）

本任务**不执行**以下动作，仅列出后续需要什么：

1. **抓包**：需真实账号与用户 CA（`usesCleartextTraffic=true` 已开，但 `networkSecurityConfig` 为资源 ID，实际信任集**待确认**）；
2. **Dart AOT 逆向**：需 `libapp.so` 符号/快照分析工具，工作量远超本任务；
3. **脱壳**：需先去掉爱加密才能读 dex，但 dex 内业务极少，**优先级最低**；
4. **签名算法定位**：`libsecurity.so`、`ijmDataEncryption` 与请求签名是否相关 → 只能动态验证。

---

## 附录 A 非 API 域名 / 链接

**dex 内未发现**（同 §8.1）。

## 附录 B 反编译失败类清单

**不适用**（未执行完整反编译）。

## 附录 C 三条不变项速查

| 项 | 本文件的做法 | 反例 |
|---|---|---|
| 【不变⑦】行号定位 | 每条结论给 sha256 / 字节数 / 脚本路径 | 写「大概是 Flutter」 |
| 【不变⑧】可信度三档 | 协议一律「待确认」，不因是 Flutter 就猜 | 把「推测在 libapp.so」写成事实 |
| 【不变⑨】未发现写搜索方式 | §3、§8.1 写明 strings+grep 的具体做法 | 只写「未发现」 |
