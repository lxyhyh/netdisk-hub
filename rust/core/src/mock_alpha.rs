//! MockProvider Alpha：行为基准实现。
//!
//! 用一个纯内存虚拟文件系统模拟一家「规整」的网盘：三种认证全部支持、分页默认 2 条、
//! 目录内不允许重名、错误一律按统一分类返回。

use std::collections::BTreeSet;
use std::path::Path;
use std::sync::Mutex;

use crate::error::NetdiskError;
use crate::mock_common::MockFs;
use crate::model::{AuthInput, ListOptions, Page, ProviderInfo, Quota, RemotePath, TransferOpts, TransferReport};
use crate::provider::{ProviderBackend, SessionContext};

/// Alpha：三种认证全支持，默认页长 2。
pub struct MockAlpha {
    fs: Mutex<MockFs>,
    sessions: Mutex<BTreeSet<String>>,
    next_session: Mutex<u64>,
}

impl MockAlpha {
    pub fn new() -> Self {
        let mut fs = MockFs::new(1073741824, "mock-alpha", 2);
        let root = RemotePath::parse("/").unwrap();
        fs.mkdir(&root.join("documents")).unwrap();
        fs.mkdir(&root.join("documents").join("archive")).unwrap();
        fs.put_file(&RemotePath::parse("/readme.md").unwrap(), b"alpha readme".to_vec(), &TransferOpts::default()).unwrap();
        fs.put_file(&RemotePath::parse("/notes.txt").unwrap(), b"alpha notes".to_vec(), &TransferOpts::default()).unwrap();
        fs.put_file(&RemotePath::parse("/photo.png").unwrap(), b"alpha png".to_vec(), &TransferOpts::default()).unwrap();
        fs.put_file(&RemotePath::parse("/documents/report.txt").unwrap(), b"alpha report".to_vec(), &TransferOpts::default()).unwrap();
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

impl Default for MockAlpha {
    fn default() -> Self {
        Self::new()
    }
}

const SESSION_PREFIX: &str = "mock-alpha";
const ACCOUNT: &str = "alpha@example.test";
const PASSWORD: &str = "alpha-pass-123";
const PHONE: &str = "13000000001";
const SMS_CODE: &str = "111111";
const SCAN_TICKET: &str = "alpha-scan-ticket";

impl ProviderBackend for MockAlpha {
    fn info(&self) -> ProviderInfo {
        ProviderInfo {
            id: "mock-alpha".to_string(),
            display_name: "Mock Alpha".to_string(),
            auth_methods: vec!["password".to_string(), "sms_code".to_string(), "scan_code".to_string()],
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
                if ticket == SCAN_TICKET {
                    Ok(self.issue_session())
                } else {
                    Err(NetdiskError::auth_failed("二维码凭证无效或已过期"))
                }
            }
        }
    }

    fn list(&self, session: &SessionContext, path: &RemotePath, options: &ListOptions) -> Result<Page, NetdiskError> {
        self.ensure_session(session)?;
        let _ = options;
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
            backend: Arc::new(MockAlpha::new()),
            password: AuthInput::Password { account: ACCOUNT.to_string(), password: PASSWORD.to_string() },
            sms: AuthInput::SmsCode { phone: PHONE.to_string(), code: SMS_CODE.to_string(), request_id: Some("req-1".to_string()) },
            scan: Some(AuthInput::ScanCode { ticket: SCAN_TICKET.to_string() }),
            bad_password: AuthInput::Password { account: ACCOUNT.to_string(), password: "wrong-password".to_string() },
            seed_dir: "documents",
        };
        run_conformance(&fixture);
    }

    #[test]
    fn virtual_root_is_independent() {
        let backend = MockAlpha::new();
        let session = backend
            .connect(&AuthInput::Password { account: ACCOUNT.to_string(), password: PASSWORD.to_string() })
            .unwrap();
        let root = RemotePath::parse("/").unwrap();
        let page = backend.list(&session, &root, &ListOptions { page_size: 64, cursor: None }).unwrap();
        let names: Vec<&str> = page.entries.iter().map(|entry| entry.name.as_str()).collect();
        assert_eq!(names, vec!["documents", "notes.txt", "photo.png", "readme.md"]);
    }

    #[test]
    fn rejects_foreign_session() {
        let backend = MockAlpha::new();
        let err = backend
            .list(&SessionContext::new("forged"), &RemotePath::parse("/").unwrap(), &ListOptions::default())
            .unwrap_err();
        assert!(matches!(err, NetdiskError::InvalidSession { .. }));
    }
}