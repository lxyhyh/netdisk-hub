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
/// 命名约定：文本字段一律叫 `detail`，**不要叫 `message`**。
///
/// UniFFI 0.32.2 的 Kotlin 生成器会把变体字段直接生成为同名属性，
/// 而 `kotlin.Throwable` 已经有 `val message: String?`；两者同名会让生成的
/// `sealed class ... : Exception()` 报 `Conflicting declarations: val message: String`，
/// Android 编译直接失败（实测于 CI，`netdiskhub_core.kt:2569`）。
/// 已在本地生成绑定并核对：改名为 `detail` 后无冲突。
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Error)]
pub enum NetdiskError {
    /// 认证失败：账号不存在、密码错误、验证码错误、二维码失效等。
    AuthFailed { detail: String },
    /// 需要人机验证：调用方必须先完成挑战，再带着新的凭据重试。
    HumanVerificationRequired { detail: String },
    /// 被限流或触发频控，调用方应按 retry_after_ms 退避。
    RateLimited { retry_after_ms: u64 },
    /// 网络层错误：超时、连接中断、DNS、TLS 等。
    Network { detail: String },
    /// 参数不合法：路径非法、选项越界、本地文件不可读等。
    InvalidArgument { detail: String },
    /// 该 Provider 不支持此操作。
    Unsupported { detail: String },
    /// 目标不存在。
    NotFound { path: String },
    /// 目标已存在（例如同名文件、目标目录已被占用）。
    AlreadyExists { path: String },
    /// 会话句柄无效或已失效。
    InvalidSession { detail: String },
    /// 其余未归类的内部错误。
    Internal { detail: String },
}

impl NetdiskError {
    /// 便于内部构造短错误。
    pub(crate) fn invalid_argument(detail: impl Into<String>) -> Self {
        Self::InvalidArgument { detail: detail.into() }
    }

    pub(crate) fn internal(detail: impl Into<String>) -> Self {
        Self::Internal { detail: detail.into() }
    }

    pub(crate) fn unsupported(detail: impl Into<String>) -> Self {
        Self::Unsupported { detail: detail.into() }
    }

    pub(crate) fn auth_failed(detail: impl Into<String>) -> Self {
        Self::AuthFailed { detail: detail.into() }
    }
}

impl fmt::Display for NetdiskError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AuthFailed { detail } => write!(f, "认证失败：{detail}"),
            Self::HumanVerificationRequired { detail } => {
                write!(f, "需要人机验证：{detail}")
            }
            Self::RateLimited { retry_after_ms } => {
                write!(f, "请求被限流，{retry_after_ms} 毫秒后重试")
            }
            Self::Network { detail } => write!(f, "网络错误：{detail}"),
            Self::InvalidArgument { detail } => write!(f, "参数错误：{detail}"),
            Self::Unsupported { detail } => write!(f, "不支持的操作：{detail}"),
            Self::NotFound { path } => write!(f, "目标不存在：{path}"),
            Self::AlreadyExists { path } => write!(f, "目标已存在：{path}"),
            Self::InvalidSession { detail } => write!(f, "会话无效：{detail}"),
            Self::Internal { detail } => write!(f, "内部错误：{detail}"),
        }
    }
}

impl std::error::Error for NetdiskError {}