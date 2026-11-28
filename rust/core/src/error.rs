//! 统一错误类型。
//!
//! 设计约束：
//! - 只表达「通用语义」，不含任何网盘专有名（不出现 baidu / nutstore / sign / zid 之类）；
//! - Kotlin 侧只需要按类别决定 UI（提示重试、要求人机验证、判登录失效……）；
//! - 不携带原始响应体、token、cookie 等敏感内容，只保留可读说明与定位用字段。

use std::fmt;

/// 统一错误分类。
///
/// 变体覆盖本项目四类真实场景：认证、风控挑战、限流、网络与参数/状态错误。
/// 后续真实 Adapter 只能映射到这些类别，不得自行扩展「网盘专有错误」。
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Error)]
pub enum NetdiskError {
    /// 认证失败：账号不存在、密码错误、验证码错误、二维码失效等。
    AuthFailed { message: String },
    /// 需要人机验证：调用方必须先完成挑战，再带着新的凭据重试。
    HumanVerificationRequired { message: String },
    /// 被限流或触发频控，调用方应按 retry_after_ms 退避。
    RateLimited { retry_after_ms: u64 },
    /// 网络层错误：超时、连接中断、DNS、TLS 等。
    Network { message: String },
    /// 参数不合法：路径非法、选项越界、本地文件不可读等。
    InvalidArgument { message: String },
    /// 该 Provider 不支持此操作。
    Unsupported { message: String },
    /// 目标不存在。
    NotFound { path: String },
    /// 目标已存在（例如同名文件、目标目录已被占用）。
    AlreadyExists { path: String },
    /// 会话句柄无效或已失效。
    InvalidSession { message: String },
    /// 其余未归类的内部错误。
    Internal { message: String },
}

impl NetdiskError {
    /// 便于内部构造短错误。
    pub(crate) fn invalid_argument(message: impl Into<String>) -> Self {
        Self::InvalidArgument { message: message.into() }
    }

    pub(crate) fn internal(message: impl Into<String>) -> Self {
        Self::Internal { message: message.into() }
    }

    pub(crate) fn unsupported(message: impl Into<String>) -> Self {
        Self::Unsupported { message: message.into() }
    }

    pub(crate) fn auth_failed(message: impl Into<String>) -> Self {
        Self::AuthFailed { message: message.into() }
    }
}

impl fmt::Display for NetdiskError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AuthFailed { message } => write!(f, "认证失败：{message}"),
            Self::HumanVerificationRequired { message } => {
                write!(f, "需要人机验证：{message}")
            }
            Self::RateLimited { retry_after_ms } => {
                write!(f, "请求被限流，{retry_after_ms} 毫秒后重试")
            }
            Self::Network { message } => write!(f, "网络错误：{message}"),
            Self::InvalidArgument { message } => write!(f, "参数错误：{message}"),
            Self::Unsupported { message } => write!(f, "不支持的操作：{message}"),
            Self::NotFound { path } => write!(f, "目标不存在：{path}"),
            Self::AlreadyExists { path } => write!(f, "目标已存在：{path}"),
            Self::InvalidSession { message } => write!(f, "会话无效：{message}"),
            Self::Internal { message } => write!(f, "内部错误：{message}"),
        }
    }
}

impl std::error::Error for NetdiskError {}
