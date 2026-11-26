# 反编译工具链记录（tooling）

- 产出任务：T-002b
- 记录时间：2026-11-26
- 用途：记录本项目反编译所使用的全部工具，确保可复现

## 1. jadx

| 项 | 值 |
|---|---|
| 工具名 | jadx（CLI，非 GUI） |
| 版本 | **1.5.7** |
| 来源 URL | `https://github.com/skylot/jadx/releases/download/v1.5.7/jadx-1.5.7.zip` |
| 下载时间 | 2026-11-26 21:50（UTC+8） |
| 文件大小 | 74766896 字节（与 GitHub Content-Length 一致） |
| SHA256 | `1755e4242ade71a077438723689f404079592ec7764a3a0572f98fef94ebcbf3` |
| Release 发布日 | 2026-11-18 |
| 本地路径 | `reverse/_tools/jadx-1.5.7/`（不进 Git） |
| 解压后入口 | `reverse/_tools/jadx-1.5.7/bin/jadx` |

### 校验说明

**SHA256 无法与官方发布页提供的值比对**，原因：

- jadx release 的资产清单中**没有** `.sha256` / `.md5` / `.sig` 类校验文件
- 查询 GitHub API 复核时遇到 **HTTP 403 rate limit exceeded**，未能重新确认资产清单

因此上述 SHA256 是**本地下载后自算值**，仅用于：
1. 证明本次下载文件完整（与声明大小 74766896 字节一致）
2. 未来重下时比对，确认是同一份文件

**不构成**对上游发布物的密码学校验。若需严格校验，应改用可验证签名的分发渠道。
此项记为**待确认**。

### 运行环境

| 项 | 值 |
|---|---|
| JDK | OpenJDK 21.0.13.1（build 21.0.13.1+2-1-24.04.1-Ubuntu） |
| java -version | `openjdk version "21.0.13.1" 2026-11-17` |
| 系统 | Ubuntu 24.04 aarch64（proot，Android 手机） |

### 验证命令与输出

```bash
$ unzip -t jadx-1.5.7.zip > /dev/null && echo OK
OK 完整

$ ./jadx-1.5.7/bin/jadx --version
1.5.7

$ java -jar jadx-1.5.7/lib/jadx-1.5.7-all.jar --version
WARN  - Can't load locale from config, using default
1.5.7
```

两种调用方式均可用。CLI 脚本方式无 WARN。

## 2. 其他已装工具（T-001 时盘点，未在本任务使用）

| 工具 | 版本 |
|---|---|
| apktool | 2.7.0 |
| baksmali / smali | 可用 |
| aapt | 可用 |
| JDK | 21.0.13.1（另有 17） |

## 3. 变更记录

| 日期 | 变更 |
|---|---|
| 2026-11-26 | 首次记录：安装 jadx 1.5.7 |
