//! 统一数据模型。
//!
//! 这一层只描述「任何网盘都成立」的概念：路径、文件条目、分页、配额、认证输入。
//! 网盘专有字段（签名、设备指纹、沙箱 id、版本号……）一律不进入本文件，
//! 它们属于各 Adapter 内部实现。

/// 规范化后的远程路径。
///
/// 约定：
/// - 一律用 `/` 分隔；
/// - 根目录是 `"/"`；
/// - 不以 `/` 结尾（根目录除外）；
/// - 不含 `.` / `..` 段（构造时会拒绝，见 [`RemotePath::parse`]）。
///
/// 之所以做成独立类型而不是裸 `String`：路径规则要在 Rust 侧统一收口，
/// 否则每个 Adapter 各写一套拼接逻辑，Kotlin 侧就无法保证一致。
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct RemotePath {
    /// 规范化后的路径文本。Kotlin 侧只读，不自行拼接。
    pub value: String,
}

impl RemotePath {
    /// 根目录常量。
    pub const ROOT: &'static str = "/";

    /// 解析并规范化路径。
    ///
    /// 规则（与 [`RemotePath`] 的文档一致）：
    /// - 空串按根目录处理；
    /// - 折叠重复的 `/`；
    /// - 拒绝 `.` 与 `..` 段（避免越权访问与歧义）；
    /// - 拒绝 `\0` 与控制字符。
    pub fn parse(raw: &str) -> Result<Self, crate::error::NetdiskError> {
        if raw.is_empty() {
            return Ok(Self { value: Self::ROOT.to_string() });
        }
        if raw.chars().any(|c| c == '\0' || (c.is_control() && c != '\t')) {
            return Err(crate::error::NetdiskError::invalid_argument(
                "路径中包含控制字符",
            ));
        }
        let mut segments: Vec<&str> = Vec::new();
        for segment in raw.split('/') {
            if segment.is_empty() {
                continue;
            }
            if segment == "." || segment == ".." {
                return Err(crate::error::NetdiskError::invalid_argument(
                    "路径中不允许出现 . 或 .. 段",
                ));
            }
            segments.push(segment);
        }
        if segments.is_empty() {
            return Ok(Self { value: Self::ROOT.to_string() });
        }
        Ok(Self { value: format!("/{}", segments.join("/")) })
    }

    /// 父目录。根目录的父目录仍是根目录。
    pub fn parent(&self) -> Self {
        match self.value.rfind('/') {
            Some(0) | None => Self { value: Self::ROOT.to_string() },
            Some(idx) => Self { value: self.value[..idx].to_string() },
        }
    }

    /// 最后一段名称。根目录返回空串。
    pub fn name(&self) -> &str {
        match self.value.rfind('/') {
            Some(idx) => &self.value[idx + 1..],
            None => &self.value,
        }
    }

    /// 是否为根目录。
    pub fn is_root(&self) -> bool {
        self.value == Self::ROOT
    }

    /// 拼接子路径。
    pub fn join(&self, child: &str) -> Self {
        if self.is_root() {
            Self { value: format!("/{child}") }
        } else {
            Self { value: format!("{}/{child}", self.value) }
        }
    }
}

/// 一个远程文件或目录条目。
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct RemoteFile {
    /// 条目名（不含父路径）。
    pub name: String,
    /// 条目完整路径。
    pub path: RemotePath,
    /// 是否为目录。
    pub is_dir: bool,
    /// 字节大小；目录为 0。
    pub size: u64,
    /// 修改时间（Unix 毫秒）。取不到时为 None，不编造。
    pub modified_ms: Option<u64>,
    /// 服务端条目 id。MockProvider 也会给出，便于测试幂等性；取不到时为 None。
    pub id: Option<String>,
}

/// 列目录参数。
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct ListOptions {
    /// 单页最多返回多少条。0 表示使用 Provider 默认值。
    pub page_size: u32,
    /// 上一页返回的游标；首页传 None。
    pub cursor: Option<String>,
}

impl Default for ListOptions {
    fn default() -> Self {
        Self { page_size: 0, cursor: None }
    }
}

/// 一页目录内容。
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct Page {
    /// 本页条目。
    pub entries: Vec<RemoteFile>,
    /// 下一页游标；已到末页时为 None。
    pub next_cursor: Option<String>,
    /// 是否还有下一页。显式给出，避免 Kotlin 靠 `next_cursor` 是否为空反推。
    pub has_more: bool,
}

/// 配额信息。
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct Quota {
    /// 已用字节。
    pub used_bytes: u64,
    /// 总容量字节；无限容量或取不到时为 None。
    pub total_bytes: Option<u64>,
    /// 文件数；取不到时为 None。
    pub file_count: Option<u64>,
}

/// 传输参数。
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct TransferOpts {
    /// 是否启用覆盖。false 时目标已存在返回 [`crate::error::NetdiskError::AlreadyExists`]。
    pub overwrite: bool,
    /// 断点续传起点（字节）。0 表示从头开始；MockProvider 会如实按此跳过前缀。
    pub resume_from: u64,
}

impl Default for TransferOpts {
    fn default() -> Self {
        Self { overwrite: false, resume_from: 0 }
    }
}

/// 传输结果。
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct TransferReport {
    /// 本次实际写入/读取的字节数。
    pub bytes: u64,
    /// 实际生效的续传起点。
    pub resumed_from: u64,
}

/// 认证输入。
///
/// 只保留本项目允许的三种原生形态（见 AGENTS.md 第 9 条）：
/// 账号密码、短信验证码、扫码。**不含** OAuth / WebView / cookie / 手动粘贴 token。
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Enum)]
pub enum AuthInput {
    /// 账号密码。
    Password {
        /// 账号（邮箱/手机号/用户名，具体含义由 Adapter 决定）。
        account: String,
        /// 明文密码。只在内存中短暂存在，不落盘、不进日志。
        password: String,
    },
    /// 短信验证码。
    SmsCode {
        /// 手机号。
        phone: String,
        /// 用户收到的验证码。
        code: String,
        /// 发码时返回的请求 id；部分网盘需要，没有时传 None。
        request_id: Option<String>,
    },
    /// 扫码登录：由调用方轮询二维码状态，确认后用一次性凭证换会话。
    ScanCode {
        /// 二维码内容中的一次性凭证。
        ticket: String,
    },
}

impl AuthInput {
    /// 用于日志与错误的脱敏描述：**绝不包含密码/验证码/ticket 原文**。
    pub fn redacted_kind(&self) -> &'static str {
        match self {
            Self::Password { .. } => "password",
            Self::SmsCode { .. } => "sms_code",
            Self::ScanCode { .. } => "scan_code",
        }
    }
}

/// Provider 的自描述信息。Kotlin 用它渲染账号列表，不需要知道任何协议细节。
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct ProviderInfo {
    /// 内部标识，例如 `mock-alpha`。
    pub id: String,
    /// 展示名，例如 `Mock Alpha`。
    pub display_name: String,
    /// 支持哪些认证形态（对应 [`AuthInput`] 的三种）。
    pub auth_methods: Vec<String>,
    /// 是否支持分页游标。
    pub supports_cursor: bool,
}
