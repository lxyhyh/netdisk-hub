# index-123云盘 —— 123 云盘 Android 客户端静态侦察索引（壳确认模式）

- 对应任务：T-002d（B 项：Flutter 网盘壳确认）
- 生成日期：2026-11-27
- **任务范围**：只做壳确认，**不深挖 `libapp.so`、不研究 Dart AOT、不还原接口**
- 样本：附件区 `123云盘_3.3.4.apk`
- 可信度取值：**已验证** / **推测** / **待确认**

> **⚠ 本文对 123 云盘协议的覆盖为 0。**
> 123 云盘是 `Flutter 主体`，dex 只有壳桩类，业务逻辑在 `libapp.so`（Dart AOT）。
> 本文只记录壳形态与加固线索，**不产出任何接口清单**。

---

## 1. APK 基本信息

| 项 | 值 | 来源 |
|---|---|---|
| 文件 | `123云盘_3.3.4.apk`（附件区 `f2/f22f940d905ec1832fe94aa8dedeef57b9699f326d8daff757d6612083235291/`） | `ls` |
| 大小 | **114861135 B**（约 110 MiB） | `stat -c %s` |
| SHA256 | `f22f940d905ec1832fe94aa8dedeef57b9699f326d8daff757d6612083235291` | `sha256sum` |
| 包名 | `com.mfcloudcalculate.networkdisk` | `aapt dump badging` |
| versionName / versionCode | `3.3.4` / `3244` | `aapt dump badging` |
| compileSdkVersion | 36 | `aapt dump badging` |
| minSdkVersion | 24 | `aapt dump badging` |
| targetSdkVersion | 34 | `aapt dump badging` |
| dex 数量 | **1 个，仅 13916 B** | `unzip -l` |
| **【新增①·必填】主体技术栈判定** | **`B. Flutter 主体`** | 见 §1.1 |
| 是否加固 | **dex 为壳桩（未检出爱加密/梆梆等已知壳特征）** | 见 §9 |
| 签名 | DN `CN=西安一二三云计算有限公司, ... C=cn`；SHA-256 `3818828a144bbf157ab3534f5da4c702b93c274c7bb5e6f7ca01c08c3c27a2be` | `apksigner verify --print-certs` |
| 原生库 | `lib/` 下 **56 条**，**仅 `arm64-v8a`** | `unzip -l` |
| Manifest 明文流量 | `android:usesCleartextTraffic="true"`；`android:networkSecurityConfig="@0x7f15001f"` | `aapt dump xmltree AndroidManifest.xml` |

### 1.1 主体技术栈判定依据

| 判据 | 实测 | 结论 |
|---|---|---|
| `lib/<abi>/libflutter.so` | **有**：arm64-v8a **11107920 B**（sha256 `a23e89413b35b237a4a61a25f5f15ac47d24bebcc81c617a9b6812c9226fac00`） | Flutter 引擎 |
| `lib/<abi>/libapp.so` | **有**：arm64-v8a **18219952 B**（sha256 `93a35149b213fc0dc379c90afb469d19cb6ce3a6fbe944fbfce285f910f76a8a`） | **Dart AOT 业务快照** |
| `assets/flutter_assets/` | **有**：1027 个文件 / 16898862 B | Flutter 资源 |
| dex 内业务类 | **无**：dex 13916 B，`class_defs` 仅 **4 个**（`Ls/h/e/l/l/A;`、`C;`、`N;`、`S;`） | 壳桩 |

→ **判定 `B. Flutter 主体`。§3–§8 全部「dex 内未发现」。**

---

## 2. 反编译工具及版本

| 项 | 值 |
|---|---|
| 工具 | jadx 1.5.7（复用，未新下载） |
| **是否执行完整反编译** | **未执行**，按 T-002d B 项范围不适用 |
| 实际执行 | `unzip -l` / `aapt dump badging` / `aapt dump xmltree` / `apksigner verify` / 纯 python 解析 dex `class_defs` / `file` / `readelf -lW` |
| 采集脚本 | `reverse/collect_flutter_facts.py`、`reverse/flutter-dex-scan.py`、`reverse/shell-dex.py` |

### 2.1 【新增②·必填】反编译覆盖率

| 项 | 值 |
|---|---|
| dex 内类定义总数 | **4**（`Ls/h/e/l/l/*` 壳桩） |
| 覆盖率 | **未执行完整 jadx 反编译，按任务范围不适用** |
| 缺失类是否影响核心链路 | **不适用**（dex 内无业务逻辑） |

---

## 3. HTTP 客户端相关类/方法

> **【降级⑥】** 本节仅对 `Java/Kotlin 主体` 适用。**123 云盘是 Flutter 主体，dex 侧无 HTTP 栈。**

- dex `class_defs` 4 个壳桩类，无任何网络库类；
- 搜索方式：`strings <dex> | grep -E 'okhttp3|retrofit2|HttpURLConnection'` → 0 命中；
- → **dex 内未发现 HTTP 客户端**，真实栈在 `libapp.so`（Dart）内。

---

## 4. 认证相关类/方法

> 当前只做壳检查。**未还原 Dart AOT，无法确认登录接口。**

| 项 | 值 |
|---|---|
| dex 内登录相关类 | **未发现** |
| 登录可行性结论 | **未知**（属 T-002c 范围，本任务不判定） |
| 攻关点（待动态分析） | 登录字段、短信/密码流程、token 结构均不可得 |

---

## 5. 文件列表相关类/方法

**无足够证据确认接口**，逻辑在 `libapp.so` 内 → 后续需动态分析。分页方式 **待确认**。

---

## 6. 上传相关类/方法

**无足够证据确认接口。** Body 形态 **待确认**；是否分片 **待确认**。

---

## 7. 下载相关类/方法

**无足够证据确认接口。** 是否支持 Range **待确认**。

---

## 8. 关键字符串常量

### 8.1 域名

**dex 内未发现域名常量**（搜索方式：`strings <dex> | grep -E 'https?://'` → 0 命中）。

### 8.2 接口清单

**本任务不产出接口清单**（无静态证据，禁止编造）。

---

## 9. 加固 / 混淆 / 证书绑定 / 设备指纹

| 检查项 | 结果 | 依据 |
|---|---|---|
| 加固壳 | **未检出已知壳特征**：无 `assets/ijiami.*`、无 `libDexHelper.so`、无 `libjiagu*.so`；但 dex 只有 4 个壳桩类，说明**业务 dex 本身就不需要**（业务在 Flutter） | `unzip -l` + `SEC` 扫描仅 2 张 png 命中 |
| dex 性质 | **壳桩**，4 个类 | 纯 python 解析 `class_defs` |
| 证书绑定 | **dex 内未发现**；native 侧 → **待确认** | — |
| 反调试 | **未发现明显特征**（无 frida/ptrace 相关 so） | `unzip -l` |
| 设备指纹 | **待确认**（dex 侧无实现） | — |

### 9.1 【新增③·必填】加固 / 风控 SDK 清单

| SDK | 文件或包名 | 数量 | 影响 |
|---|---|---|---|
| Flutter 引擎 | `libflutter.so`（11107920 B） | 1 | 业务快照容器 |
| Dart AOT 业务 | `libapp.so`（18219952 B） | 1 | **协议全在这里，静态不可读** |
| 播放器 | `libmpv.so` **25136224 B**、`libffmpegJNI.so` 1726712 B、`libttmplayer_lite.so` 1502480 B | 3 | 与网盘协议无关 |
| AI / 推理 | `libxnn.so` 2680336 B、`libAPSE_7.0.1.so` 3676856 B、`libcvenginelite.so` 2051440 B、`assets/model_bankframe.zip` 1866775 B | 4 | 与协议无关（图像/证件） |
| 广告 SDK | `assets/gdt_plugin/gdtadv2.jar` 2083431 B、`assets/bdxadsdk.jar` 1769134 B | 2 | 与协议无关 |
| 模拟器检测 | `libijm-emulator.so` 166360 B | 1 | **注意**：有爱加密的「模拟器检测」单库，但**没有** `ijiami.dat/ajm` 载荷 → 推测为残留或独立组件 → 待确认 |

**判定：无整包加固，但存在 1 个模拟器检测库（来源待确认）。** 由于业务在 Dart AOT，加固与否对「静态还原协议」这件事本身已无影响。

---

## 10. 线索可信度汇总

| 结论 | 可信度 |
|---|---|
| 123 云盘是 `Flutter 主体` | 已验证 |
| dex 仅 4 个壳桩类，无业务实现 | 已验证 |
| `libapp.so` 18219952 B（仅 arm64-v8a） | 已验证 |
| `flutter_assets` 1027 文件 / 16898862 B | 已验证 |
| 无已知整包加固壳 | 已验证（搜索方式已写） |
| `libijm-emulator.so` 来源 | 待确认 |
| 登录/列表/上传/下载协议 | **待确认（静态不可得，需动态分析）** |
| 123 云盘原生登录可行性 | **不在本文范围**（属 T-002c） |

---

## 11. 后续需动态分析

1. **抓包**（需用户 CA；`networkSecurityConfig` 为资源 ID，实际信任集待确认）；
2. **Dart AOT 逆向**（成本高，本任务不做）；
3. **不需要脱壳**（无整包壳，dex 里也没有业务）；
4. **登录流程**：只能靠动态分析或官方 OpenAPI 文档（若有）确认。

---

## 附录 A 非 API 域名 / 链接

**dex 内未发现。**

## 附录 B 反编译失败类清单

**不适用**（未执行完整反编译）。

## 附录 C 三条不变项速查

| 项 | 本文件的做法 | 反例 |
|---|---|---|
| 【不变⑦】行号定位 | 给 sha256 / 字节数 / 脚本路径 | 写「Flutter 应用」就完事 |
| 【不变⑧】可信度三档 | 协议一律「待确认」 | 凭空写 API 路径 |
| 【不变⑨】未发现写搜索方式 | §3、§8.1、§9 都写了具体 grep/strings | 只写「未发现」 |
