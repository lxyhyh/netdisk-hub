# Agent 工作协议

1. 开始任务前先读：docs/PROJECT_CHARTER.md、docs/SCOPE.md、docs/ARCHITECTURE.md、当前任务卡。
2. 先复述：项目唯一目标、本阶段目标、本任务非目标、验收标准。不要直接写代码。
3. 只做任务卡范围内的事。新增依赖、改架构、扩大范围必须先写 ADR 提案，等我批准。
4. 接口必须写证据链：来源 APK 类/方法或网页 JS 文件/函数、请求、响应、错误码、可信度。
   可信度只能标：已验证 / 推测 / 待确认。推测必须显式标注，禁止编造字段。
5. 不提交真实 cookie/token/账号。日志脱敏。测试用 mock 或脱敏样本。
6. 每个任务结束更新：docs/TASKS.md、docs/PROVIDERS.md、docs/API_FACTS.md、CHANGELOG，并运行 lint/typecheck/test。
7. 输出 PR 式摘要：改了什么、为什么、怎么验证、风险、未解决问题。
8. 项目为个人自用；代码按可维护工程做，不把真实凭据散播出去。
9. 登录必须是 APP 内原生登录：账号密码 / 短信验证码 / 扫码，
   由 Rust Adapter 走官方客户端协议换取 token，存 Android Keystore。
   扫码登录的 token 必须由客户端协议换取，不得从 WebView 抓取。
   禁止 OAuth 第三方授权、WebView 登录、cookie 提取、手动粘贴 token、网页端会话复用。
   若某网盘原生登录不可行，记录为“不可行”并附证据，
   不得降级为 OAuth/WebView/cookie 方案，也不得静默跳过。

10. 本地不具备编译条件，所有编译、构建、测试、打包必须通过 GitHub Actions。
    不得声称"本地已编译通过"或"本地测试通过"。
    每完成一批改动即 commit + push，等 CI 结果，不假设改动正确。
    CI 失败时先看日志定位，不得盲目重试。

11. 任务卡中所有涉及"验证编译""验证测试""验证构建"的验收项，
    一律以 CI 运行结果为准。CI 未跑通的任务，状态不得标"已完成"。

12. 仓库为公开仓库。CI 日志、artifact、docs/evidence/ 索引均对所有人可见。
    不得在代码、日志、测试数据、索引文件中写入真实 token / cookie / 账号 / 手机号 / 密钥值。
    涉及密钥时只写来源定位（类#方法），不写值本身。
    commit message 不得写"破解""绕过""封堵"类描述。
