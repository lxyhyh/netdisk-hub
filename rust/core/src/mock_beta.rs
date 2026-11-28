//! MockProvider Beta：故意与 Alpha 行为不同的第二实现。
//!
//! 用来证明统一接口不是为某一家特制的：
//! - 不支持扫码（返回 `Unsupported`，而不是假装成功）；
//! - 单页上限 3 条、超过 100 条直接 `InvalidArgument`；
//! - 根目录内容与 Alpha 完全不同；
//! - 目录非空时删除返回 `AlreadyExists` 之外的明确错误（见实现）。

use std::collections::BTreeSet;
use std::path::Path;
use std::sync::Mutex;

use crate::error::NetdiskError;
use crate::mock_common::MockFs;
use crate::model::{AuthInput, ListOptions, Page, ProviderInfo, Quota, RemotePath, TransferOpts, TransferReport};
use crate::provider::{ProviderBackend, SessionContext};

/// Beta：不支持扫码，单页上限 3。
pub struct MockBeta {
    fs: Mutex<MockFs>,
    sessions: Mutex<BTreeSet<String>>,
    next_session: Mutex<u64>,
}

impl MockBeta {
    pub fn new() -> Self {
        let mut fs = MockFs::new(1099511627776, "mock-beta", 3);
        let root = RemotePath::parse("/").unwrap();
        fs.mkdir(&root.join("shared")).unwrap();
        fs.mkdir(&root.join("shared").join("team")).unwrap();
        fs.put_file(&RemotePath::parse("/a.txt").unwrap(), b"beta a".to_vec(), &TransferOpts::default()).unwrap();
        fs.put_file(&RemotePath::parse("/b.txt").unwrap(), b"beta b".to_vec(), &TransferOpts::default()).unwrap();
        fs.put_file(&RemotePath::parse("/shared/memo.txt").unwrap(), b"beta memo".to_vec(), &TransferOpts::default()).unwrap();
        fs.put_file(&RemotePath::parse("/shared/team/plan.md").unwrap(), b"beta plan".to_vec(), &TransferOpts::default()).unwrap();
        Self { fs: Mutex::new(fs), sessions: Mutex::new(BTreeSet::new()), next_session: Mutex::new(1) }
    }

    fn issue_session(&self) -> SessionContext {
        let mut counter = self.next_session.lock().unwrap();
        let id = format!("{}-{}", SESSION_PREFIX, *counter);
        *counter += 1;
        self.sessions.lock().unwrap().insert(id.clone());
        SessionContext::new(id)
    }

    fn ensure_session(&self, session: &SessionContext) -> Result<(), NetdiskError> {
        if self.sessions.lock().unwrap().contains(&session.id) {
            Ok(())
        } else {
            Err(NetdiskError::InvalidSession {
                message: format!("会话句柄未被本 Provider 签发：{}", session.id),
            })
        }
    }
}

impl Default for MockBeta {
    fn default() -> Self {
        Self::new()
    }
}

const SESSION_PREFIX: &str = "mock-beta";
const ACCOUNT: &str = "beta@example.test";
const PASSWORD: &str = "beta-pass-456";
const PHONE: &str = "13000000002";
const SMS_CODE: &str = "222222";
/// Beta 不支持扫码，这里保留一个凭证常量供「传了也必须被拒绝」的反向测试使用。
#[cfg(test)]
const SCAN_TICKET: &str = "beta-scan-ticket";

impl ProviderBackend for MockBeta {
    fn info(&self) -> ProviderInfo {
        ProviderInfo {
            id: "mock-beta".to_string(),
            display_name: "Mock Beta".to_string(),
            auth_methods: vec!["password".to_string(), "sms_code".to_string()],
            supports_cursor: true,
        }
    }

    fn connect(&self, auth: &AuthInput) -> Result<SessionContext, NetdiskError> {
        match auth {
            AuthInput::Password { account, password } => {
                if account == ACCOUNT && password == PASSWORD {
                    Ok(self.issue_session())
                } else {
                    Err(NetdiskError::auth_failed("账号或密码不正确"))
                }
            }
            AuthInput::SmsCode { phone, code, .. } => {
                if phone == PHONE && code == SMS_CODE {
                    Ok(self.issue_session())
                } else {
                    Err(NetdiskError::auth_failed("验证码不正确"))
                }
            }
            AuthInput::ScanCode { ticket } => {
                let _ = ticket;
                Err(NetdiskError::unsupported("本 Provider 不支持扫码登录"))
            }
        }
    }

    fn list(&self, session: &SessionContext, path: &RemotePath, options: &ListOptions) -> Result<Page, NetdiskError> {
        self.ensure_session(session)?;
        const MAX_PAGE_SIZE: u32 = 100;
        if options.page_size > MAX_PAGE_SIZE {
            return Err(NetdiskError::invalid_argument(format!("page_size 不能超过 {MAX_PAGE_SIZE}")));
        }
        self.fs.lock().unwrap().list(path, options)
    }

    fn download(&self, session: &SessionContext, from: &RemotePath, to: &Path, options: &TransferOpts) -> Result<TransferReport, NetdiskError> {
        self.ensure_session(session)?;
        let data = self.fs.lock().unwrap().read_all(from)?;
        let total = data.len() as u64;
        let resumed_from = options.resume_from.min(total);
        let tail = &data[resumed_from as usize..];
        let mut file = if resumed_from > 0 {
            std::fs::OpenOptions::new().append(true).open(to)
        } else {
            std::fs::File::create(to)
        }
        .map_err(|err| NetdiskError::invalid_argument(format!("无法写入本地文件：{err}")))?;
        std::io::Write::write_all(&mut file, tail)
            .map_err(|err| NetdiskError::internal(format!("写本地文件失败：{err}")))?;
        Ok(TransferReport { bytes: tail.len() as u64, resumed_from })
    }

    fn upload(&self, session: &SessionContext, from: &Path, to: &RemotePath, options: &TransferOpts) -> Result<TransferReport, NetdiskError> {
        self.ensure_session(session)?;
        let data = std::fs::read(from)
            .map_err(|err| NetdiskError::invalid_argument(format!("无法读取本地文件：{err}")))?;
        self.fs.lock().unwrap().put_file(to, data, options)
    }

    fn remove(&self, session: &SessionContext, path: &RemotePath) -> Result<(), NetdiskError> {
        self.ensure_session(session)?;
        self.fs.lock().unwrap().remove(path)
    }

    fn rename(&self, session: &SessionContext, from: &RemotePath, to: &RemotePath) -> Result<(), NetdiskError> {
        self.ensure_session(session)?;
        self.fs.lock().unwrap().rename(from, to)
    }

    fn mkdir(&self, session: &SessionContext, path: &RemotePath) -> Result<(), NetdiskError> {
        self.ensure_session(session)?;
        self.fs.lock().unwrap().mkdir(path)
    }

    fn quota(&self, session: &SessionContext) -> Result<Quota, NetdiskError> {
        self.ensure_session(session)?;
        Ok(self.fs.lock().unwrap().quota())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::{run_conformance, ConformanceFixture};
    use std::sync::Arc;

    #[test]
    fn satisfies_unified_contract() {
        let fixture = ConformanceFixture {
            backend: Arc::new(MockBeta::new()),
            password: AuthInput::Password { account: ACCOUNT.to_string(), password: PASSWORD.to_string() },
            sms: AuthInput::SmsCode { phone: PHONE.to_string(), code: SMS_CODE.to_string(), request_id: Some("req-1".to_string()) },
            scan: None,
            bad_password: AuthInput::Password { account: ACCOUNT.to_string(), password: "wrong-password".to_string() },
            seed_dir: "shared",
        };
        run_conformance(&fixture);
    }

    #[test]
    fn virtual_root_is_independent() {
        let backend = MockBeta::new();
        let session = backend
            .connect(&AuthInput::Password { account: ACCOUNT.to_string(), password: PASSWORD.to_string() })
            .unwrap();
        let root = RemotePath::parse("/").unwrap();
        let page = backend.list(&session, &root, &ListOptions { page_size: 64, cursor: None }).unwrap();
        let names: Vec<&str> = page.entries.iter().map(|entry| entry.name.as_str()).collect();
        assert_eq!(names, vec!["a.txt", "b.txt", "shared"]);
    }

    #[test]
    fn rejects_foreign_session() {
        let backend = MockBeta::new();
        let err = backend
            .list(&SessionContext::new("forged"), &RemotePath::parse("/").unwrap(), &ListOptions::default())
            .unwrap_err();
        assert!(matches!(err, NetdiskError::InvalidSession { .. }));
    }

    #[test]
    fn rejects_scan_login() {
        let backend = MockBeta::new();
        let err = backend
            .connect(&AuthInput::ScanCode { ticket: SCAN_TICKET.to_string() })
            .unwrap_err();
        assert!(matches!(err, NetdiskError::Unsupported { .. }), "Beta 不支持扫码，应返回 Unsupported，实际 {err:?}");
    }
}