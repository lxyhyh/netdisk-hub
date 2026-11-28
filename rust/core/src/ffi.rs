//! UniFFI 对外边界。
//!
//! 这一层只做「翻译」，不含任何逻辑：
//! - **不导出** [`ProviderBackend`] trait：UniFFI 0.32.2 虽能导出 trait，但那样会把
//!   实现细节漏进 Kotlin 接口。这里改用 `#[derive(uniffi::Object)]` 的句柄；
//! - [`Session`] 是不透明对象（opaque object），**Kotlin 拿不到里面的任何字符串**，
//!   明文 token / cookie 永远不会跨过这条边界；
//! - 所有方法都是纯转发，错误类型原样透出 [`NetdiskError`]。

use std::path::PathBuf;
use std::sync::Arc;

use crate::error::NetdiskError;
use crate::mock_alpha::MockAlpha;
use crate::mock_beta::MockBeta;
use crate::model::{AuthInput, ListOptions, Page, ProviderInfo, Quota, RemoteFile, RemotePath, TransferOpts, TransferReport};
use crate::provider::{ProviderBackend, SessionContext};

/// 已注册的 Provider 标识列表。Kotlin 用它渲染账号类型选择器。
/// 把边界上传进来的 RemotePath 重新规范化一遍。
///
/// RemotePath 是 UniFFI record，Kotlin 侧可以直接 new 出任意 value，
/// 绕过 RemotePath::parse 的校验。所有对外方法入口都过这一层，
/// 保证「无论怎么构造，进入 ProviderBackend 的路径一定是规范化过的」。
fn normalize(path: &RemotePath) -> Result<RemotePath, NetdiskError> {
    RemotePath::parse(&path.value)
}

#[uniffi::export]
pub fn provider_ids() -> Vec<String> {
    vec![MockAlpha::new().info().id, MockBeta::new().info().id]
}

/// 一次登录会话。
///
/// 只持有 Provider 内部签发的不透明 id，**不暴露 token**；Kotlin 侧只能把它原样传回来。
#[derive(uniffi::Object)]
pub struct Session {
    context: SessionContext,
}

impl Session {
    fn wrap(context: SessionContext) -> Arc<Self> {
        Arc::new(Self { context })
    }
}

/// Provider 句柄。
///
/// Kotlin 侧典型用法：
/// ```kotlin
/// val client = NetdiskClient("mock-alpha")
/// val session = client.connect(AuthInput.Password("alpha@example.test", "…"))
/// val page = client.list(session, RemotePath("/"), ListOptions(0u, null))
/// ```
#[derive(uniffi::Object)]
pub struct NetdiskClient {
    backend: Arc<dyn ProviderBackend>,
}

#[uniffi::export]
impl NetdiskClient {
    /// 按 id 创建句柄。未知 id 返回 [`NetdiskError::NotFound`]。
    #[uniffi::constructor]
    pub fn new(provider_id: String) -> Result<Arc<Self>, NetdiskError> {
        let backend: Arc<dyn ProviderBackend> = match provider_id.as_str() {
            "mock-alpha" => Arc::new(MockAlpha::new()),
            "mock-beta" => Arc::new(MockBeta::new()),
            other => {
                return Err(NetdiskError::NotFound { path: format!("未知的 Provider：{other}") });
            }
        };
        Ok(Arc::new(Self { backend }))
    }

    /// 自描述信息。
    pub fn info(&self) -> ProviderInfo {
        self.backend.info()
    }

    /// 登录。
    pub fn connect(&self, auth: &AuthInput) -> Result<Arc<Session>, NetdiskError> {
        self.backend.connect(auth).map(Session::wrap)
    }

    /// 列目录。
    pub fn list(&self, session: &Arc<Session>, path: &RemotePath, options: ListOptions) -> Result<Page, NetdiskError> {
        let path = normalize(path)?;
        self.backend.list(&session.context, &path, &options)
    }

    /// 下载到本地路径（绝对或相对路径，由 Kotlin 侧决定）。
    pub fn download(&self, session: &Arc<Session>, from: &RemotePath, to: String, options: TransferOpts) -> Result<TransferReport, NetdiskError> {
        let from = normalize(from)?;
        self.backend.download(&session.context, &from, &PathBuf::from(to), &options)
    }

    /// 从本地路径上传。
    pub fn upload(&self, session: &Arc<Session>, from: String, to: &RemotePath, options: TransferOpts) -> Result<TransferReport, NetdiskError> {
        let to = normalize(to)?;
        self.backend.upload(&session.context, &PathBuf::from(from), &to, &options)
    }

    /// 删除。
    pub fn remove(&self, session: &Arc<Session>, path: &RemotePath) -> Result<(), NetdiskError> {
        let path = normalize(path)?;
        self.backend.remove(&session.context, &path)
    }

    /// 重命名 / 移动。
    pub fn rename(&self, session: &Arc<Session>, from: &RemotePath, to: &RemotePath) -> Result<(), NetdiskError> {
        let from = normalize(from)?;
        let to = normalize(to)?;
        self.backend.rename(&session.context, &from, &to)
    }

    /// 新建文件夹。
    pub fn mkdir(&self, session: &Arc<Session>, path: &RemotePath) -> Result<(), NetdiskError> {
        let path = normalize(path)?;
        self.backend.mkdir(&session.context, &path)
    }

    /// 查询配额。
    pub fn quota(&self, session: &Arc<Session>) -> Result<Quota, NetdiskError> {
        self.backend.quota(&session.context)
    }

    /// 取单个条目详情（下载前查大小、重命名后确认都靠它）。
    pub fn stat(&self, session: &Arc<Session>, path: &RemotePath) -> Result<RemoteFile, NetdiskError> {
        let path = normalize(path)?;
        let parent = path.parent();
        let options = ListOptions { page_size: 0, cursor: None };
        let mut cursor: Option<String> = None;
        let name = path.name().to_string();
        loop {
            let page = self.backend.list(&session.context, &parent, &ListOptions { page_size: options.page_size, cursor: cursor.clone() })?;
            if let Some(entry) = page.entries.into_iter().find(|entry| entry.name == name) {
                return Ok(entry);
            }
            if !page.has_more {
                return Err(NetdiskError::NotFound { path: path.value.clone() });
            }
            cursor = page.next_cursor;
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    fn alpha_session() -> (Arc<NetdiskClient>, Arc<Session>) {
        let client = NetdiskClient::new("mock-alpha".to_string()).unwrap();
        let session = client
            .connect(&AuthInput::Password {
                account: "alpha@example.test".to_string(),
                password: "alpha-pass-123".to_string(),
            })
            .unwrap();
        (client, session)
    }

    #[test]
    fn unknown_provider_is_rejected() {
        match NetdiskClient::new("no-such-provider".to_string()) {
            Ok(_) => panic!("未知 Provider 必须失败"),
            Err(err) => assert!(matches!(err, NetdiskError::NotFound { .. }), "实际 {err:?}"),
        }
    }

    #[test]
    fn hand_built_remote_path_is_normalized_at_the_boundary() {
        let (client, session) = alpha_session();
        // 直接构造 record，绕过 RemotePath::parse 的校验
        let evil = RemotePath { value: "/documents/../readme.md".to_string() };
        match client.stat(&session, &evil) {
            Ok(_) => panic!("带 .. 的路径必须在边界被拒绝"),
            Err(err) => assert!(matches!(err, NetdiskError::InvalidArgument { .. }), "实际 {err:?}"),
        }
        // 重复斜杠会被规范化，而不是被当成一个不存在的名字
        let messy = RemotePath { value: "//documents//report.txt".to_string() };
        let entry = client.stat(&session, &messy).unwrap();
        assert_eq!(entry.name, "report.txt");
        assert_eq!(entry.path.value, "/documents/report.txt");
    }

    #[test]
    fn provider_ids_are_stable() {
        assert_eq!(provider_ids(), vec!["mock-alpha".to_string(), "mock-beta".to_string()]);
    }

    #[test]
    fn stat_walks_pages_until_found() {
        // Alpha 默认页长 2，根目录有 4 项，stat 必须翻页才能找到 readme.md。
        let (client, session) = alpha_session();
        let entry = client.stat(&session, &RemotePath::parse("/readme.md").unwrap()).unwrap();
        assert_eq!(entry.name, "readme.md");
        assert!(!entry.is_dir);
        assert_eq!(entry.size, 12);

        let dir = client.stat(&session, &RemotePath::parse("/documents").unwrap()).unwrap();
        assert!(dir.is_dir);

        match client.stat(&session, &RemotePath::parse("/nope.txt").unwrap()) {
            Ok(_) => panic!("不存在的路径必须失败"),
            Err(err) => assert!(matches!(err, NetdiskError::NotFound { .. }), "实际 {err:?}"),
        }
    }

    #[test]
    fn errors_cross_the_boundary_without_leaking_credentials() {
        let client = NetdiskClient::new("mock-alpha".to_string()).unwrap();
        let err = match client.connect(&AuthInput::Password {
            account: "alpha@example.test".to_string(),
            password: "totally-wrong".to_string(),
        }) {
            Ok(_) => panic!("错误密码必须失败"),
            Err(err) => err,
        };
        let text = err.to_string();
        assert!(matches!(err, NetdiskError::AuthFailed { .. }), "实际 {err:?}");
        assert!(!text.contains("totally-wrong"), "错误文案不得回显用户输入的密码：{text}");
    }
}