# 安全（SECURITY）

- 状态：草案 v0.1
- 产出任务：T-001
- 最后更新：2026-11-26

## 1. token 本地存储

**方案：Android Keystore。**

- 由 Kotlin 侧封装读写，**Rust 侧不持有持久化 token**
- 使用 Keystore 支持的加密能力（如 AES 密钥存 Keystore，密文存私有存储）
- token 不落明文到 SharedPreferences / 普通文件 / 日志
- 不使用第三方向导库（属新增依赖，需 ADR）
- 卸载 App 即失效，不做跨设备同步

## 2. 日志脱敏规则

必须脱敏的字段（**用固定占位替换，不截断保留片段**）：

| 类别 | 字段 | 处理 |
|---|---|---|
| 凭据 | token、access_token、refresh_token、session、cookie、authorization | `<REDACTED>` |
| 密码 | password、passwd、pwd、pin | `<REDACTED>` |
| 验证码 | sms_code、captcha、otp | `<REDACTED>` |
| 账号 | 手机号、邮箱、UID | 中间打码：`137****6789` |
| 设备 | IMEI、Android ID、MAC、序列号、GAID | `<REDACTED>` |
| 路径 | 本地绝对路径 | 只保留相对文件名 |
| 链接 | 含签名的下载/上传 URL | 去掉 query 中的签名字段 |

规则：

1. 脱敏在**写入日志之前**执行，不是事后过滤
2. 日志脱敏默认开启，MVP 不允许关闭
3. 网络请求日志只记 method + 去参路径 + 状态码 + 耗时
4. 禁止在异常堆栈里带出请求体

## 3. 凭据不进仓库、不进 agent 上下文

- `reverse/` 在 .gitignore，不进 Git
- `*.cookie` / `*.token` / `.env` 在 .gitignore
- 真实账号、token、cookie 一律不提交
- **不把真实凭据贴进 agent 对话上下文**
- 不把真实抓包原样放进 docs/

## 4. 测试用 mock 或脱敏样本

- 单元测试用 mock，不打真实接口
- 契约测试用固定响应样本，样本必须先脱敏
- 脱敏样本放 `docs/evidence/`（不进 `raw/`）
- 需要真实响应的探索性测试，在 `docs/evidence/raw/`（已 gitignore）进行，
  结果必须脱敏后才能引用

## 5. 登录方式安全约束

登录必须是 APP 内原生登录（详见 PROJECT_CHARTER.md「登录方式」）。
禁止 OAuth、WebView 抓 cookie、手动粘贴 token、网页端会话复用。
这类方案不仅功能不合格，也会把凭据暴露在不可控的中间层。

## 6. 公开仓库 CI 安全

仓库为公开仓库，Actions 日志和 artifact 默认对所有人可见。

硬约束：
1. CI 日志中不得出现真实 token / cookie / 账号 / 手机号
2. 测试数据必须是脱敏样本，不得含真实用户信息
3. 必须使用真实凭据的 step，凭据走 GitHub Secrets，且该 step 日志显式屏蔽输出
4. artifact 上传前检查：APK 不得打包真实凭据，测试报告不得含敏感字符串
5. docs/evidence/ 下的索引文件：涉及密钥时只写来源定位，不写密钥值本身

CI workflow 审查清单（T-002e 搭建时逐条核对）：
- [ ] 所有 secrets 通过 `${{ secrets.XXX }}` 引用，无硬编码
- [ ] 日志中无 echo 敏感变量
- [ ] artifact 上传前有脱敏检查步骤
- [ ] 测试 mock 数据已脱敏
- [ ] commit message 无"破解 / 绕过 / 封堵"类描述
