//! Provider 抽象（深模块 seam）。
//!
//! 设计要点：
//! - [`ProviderBackend`] 是 **Rust 内部 trait**，不直接导出给 UniFFI。
//!   UniFFI 0.32.2 支持导出 trait，但这里刻意不导：接口面越小越稳，
//!   Kotlin 只需要「拿一个会话、调通用方法」，不需要看到 trait。
//! - trait 里不出现任何网盘专有概念（签名、指纹、分片、沙箱 id 都不在）。
//! - 所有方法都接收 [`SessionContext`]，由实现自己校验会话是否有效。

use std::path::Path;
use std::sync::Arc;

use crate::error::NetdiskError;
use crate::model::{AuthInput, ListOptions, Page, ProviderInfo, Quota, RemotePath, TransferOpts, TransferReport};

/// 一次会话的上下文。
///
/// `id` 是 Provider 自己签发的不透明字符串，**不是**网盘 token：
/// 真实 Adapter 里它只会被映射到内部保持的 token，不经 UniFFI 暴露。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionContext {
    pub id: String,
}

impl SessionContext {
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }
}

/// 统一 Provider 抽象。
///
/// 任何网盘（含 Mock）都实现这一套；两个以上实现才说明 seam 真的成立
/// （见 `docs/ARCHITECTURE.md` §2「真 seam 判定」）。
pub trait ProviderBackend: Send + Sync {
    /// Provider 自描述信息。
    fn info(&self) -> ProviderInfo;

    /// 认证并签发会话。实现自己决定接受哪些 [`AuthInput`]。
    fn connect(&self, auth: &AuthInput) -> Result<SessionContext, NetdiskError>;

    /// 列目录（支持分页）。
    fn list(
        &self,
        session: &SessionContext,
        path: &RemotePath,
        options: &ListOptions,
    ) -> Result<Page, NetdiskError>;

    /// 下载到本地文件。
    fn download(
        &self,
        session: &SessionContext,
        from: &RemotePath,
        to: &Path,
        options: &TransferOpts,
    ) -> Result<TransferReport, NetdiskError>;

    /// 从本地文件上传。
    fn upload(
        &self,
        session: &SessionContext,
        from: &Path,
        to: &RemotePath,
        options: &TransferOpts,
    ) -> Result<TransferReport, NetdiskError>;

    /// 删除（目录需为空，否则由实现返回明确错误）。
    fn remove(&self, session: &SessionContext, path: &RemotePath) -> Result<(), NetdiskError>;

    /// 重命名 / 移动。
    fn rename(
        &self,
        session: &SessionContext,
        from: &RemotePath,
        to: &RemotePath,
    ) -> Result<(), NetdiskError>;

    /// 新建文件夹。
    fn mkdir(&self, session: &SessionContext, path: &RemotePath) -> Result<(), NetdiskError>;

    /// 查询配额。
    fn quota(&self, session: &SessionContext) -> Result<Quota, NetdiskError>;
}

/// 契约测试的输入。
///
/// 两个 Mock 各自构造一份，跑**同一套**断言，从而证明统一接口不是为某一家特制的。
pub struct ConformanceFixture {
    /// 被测实现。
    pub backend: Arc<dyn ProviderBackend>,
    /// 必定成功的账号密码输入。
    pub password: AuthInput,
    /// 必定成功的短信输入。
    pub sms: AuthInput,
    /// 必定成功的扫码输入；该 Provider 不支持扫码时传 None。
    pub scan: Option<AuthInput>,
    /// 必定失败的密码输入。
    pub bad_password: AuthInput,
    /// 根目录下必定存在的目录名（两个 Mock 各自不同，用来证明数据独立）。
    pub seed_dir: &'static str,
}

/// 契约测试用的固定载荷：长度与内容都写死，断言才能精确到字节。
const LOCAL_PAYLOAD: &[u8] = b"hello netdiskhub";

/// 跑一遍统一契约。
///
/// 断言覆盖：认证成功/失败、列目录与分页、配额、上传/下载/删除/重命名/新建目录、
/// 以及四类错误（无效路径、无效会话、同名冲突、目标不存在）。
pub fn run_conformance(fixture: &ConformanceFixture) {
    let backend = fixture.backend.as_ref();
    let info = backend.info();
    assert!(!info.id.is_empty(), "ProviderInfo.id 不能为空");
    assert!(!info.display_name.is_empty(), "ProviderInfo.display_name 不能为空");

    // ---- 认证：失败 -
    let err = backend.connect(&fixture.bad_password).expect_err("错误密码必须失败");
    assert!(matches!(err, NetdiskError::AuthFailed { .. }), "错误密码应返回 AuthFailed，实际 {err:?}");

    // ---- 认证：成功（密码） -
    let session = backend.connect(&fixture.password).expect("正确密码必须成功");
    assert!(!session.id.is_empty(), "会话 id 不能为空");

    // ---- 认证：成功（短信） -
    let sms_session = backend.connect(&fixture.sms).expect("正确短信验证码必须成功");
    assert!(!sms_session.id.is_empty());

    // ---- 认证：扫码（按各 Provider 是否支持） -
    match &fixture.scan {
        Some(scan) => {
            let scan_session = backend.connect(scan).expect("扫码凭证必须被接受");
            assert!(!scan_session.id.is_empty());
        }
        None => {
            let err = backend
                .connect(&AuthInput::ScanCode { ticket: "any-ticket".to_string() })
                .expect_err("不支持扫码时必须失败");
            assert!(matches!(err, NetdiskError::Unsupported { .. }), "应返回 Unsupported，实际 {err:?}");
        }
    }

    // ---- 列目录：根目录含种子目录 -
    let root = RemotePath::parse("/").unwrap();
    let page = backend.list(&session, &root, &ListOptions::default()).expect("列根目录必须成功");
    assert!(!page.entries.is_empty(), "根目录不能为空");
    let dir = page
        .entries
        .iter()
        .find(|e| e.name == fixture.seed_dir)
        .unwrap_or_else(|| panic!("根目录应包含 {}，实际 {:?}", fixture.seed_dir, page.entries.iter().map(|e| &e.name).collect::<Vec<_>>()));
    assert!(dir.is_dir, "{} 应是目录", fixture.seed_dir);

    // ---- 分页：逐页取完，条目集合与一次性取完完全一致 -
    // 先用足够大的页长一次性取全，作为「总数」基准。
    let full = backend
        .list(&session, &root, &ListOptions { page_size: 64, cursor: None })
        .expect("一次性列目录必须成功");
    assert!(!full.has_more, "page_size=64 时应能一次取完根目录");
    assert!(full.next_cursor.is_none(), "一次性取完时 next_cursor 必须为 None");
    let expected: Vec<String> = full.entries.iter().map(|entry| entry.name.clone()).collect();

    let mut seen: Vec<String> = Vec::new();
    let mut cursor: Option<String> = None;
    let mut pages = 0usize;
    loop {
        let page = backend
            .list(&session, &root, &ListOptions { page_size: 2, cursor: cursor.clone() })
            .expect("分页列目录必须成功");
        assert!(page.entries.len() <= 2, "page_size=2 时单页不能超过 2 条");
        for entry in &page.entries {
            seen.push(entry.name.clone());
        }
        pages += 1;
        if !page.has_more {
            assert!(page.next_cursor.is_none(), "has_more=false 时 next_cursor 必须为 None");
            break;
        }
        assert!(page.next_cursor.is_some(), "has_more=true 时必须给出 next_cursor");
        cursor = page.next_cursor;
        assert!(pages < 50, "分页未收敛，可能 next_cursor 没有推进");
    }
    let expected_pages = expected.len().div_ceil(2);
    assert_eq!(pages, expected_pages, "根目录 {:?} 共 {} 条，page_size=2 下应为 {} 页", expected, expected.len(), expected_pages);
    assert_eq!(seen, expected, "分页遍历结果应与一次性取完完全一致");

    // ---- 配额 -
    let quota = backend.quota(&session).expect("配额查询必须成功");
    assert!(quota.total_bytes.is_some(), "Mock 必须给出总容量");
    assert!(quota.used_bytes <= quota.total_bytes.unwrap(), "已用不能超过总量");

    // ---- mkdir -
    let new_dir = RemotePath::parse("/conformance-dir").unwrap();
    backend.mkdir(&session, &new_dir).expect("新建目录必须成功");
    let err = backend.mkdir(&session, &new_dir).expect_err("重复新建必须失败");
    assert!(matches!(err, NetdiskError::AlreadyExists { .. }), "应返回 AlreadyExists，实际 {err:?}");

    // ---- upload -
    // 临时文件名带 Provider id：两个 Mock 的契约测试会并行跑，共用文件名会互相删文件。
    let tag = info.id.clone();
    let local = std::env::temp_dir().join(format!("netdiskhub-conformance-{tag}-upload.txt"));
    std::fs::write(&local, LOCAL_PAYLOAD).expect("写临时文件失败");
    let remote = RemotePath::parse("/conformance-dir/hello.txt").unwrap();
    let report = backend
        .upload(&session, &local, &remote, &TransferOpts::default())
        .expect("上传必须成功");
    assert_eq!(report.bytes, LOCAL_PAYLOAD.len() as u64, "上传字节数应等于文件大小");
    assert_eq!(report.resumed_from, 0);
    let err = backend
        .upload(&session, &local, &remote, &TransferOpts::default())
        .expect_err("同名上传未覆盖时必须失败");
    assert!(matches!(err, NetdiskError::AlreadyExists { .. }), "应返回 AlreadyExists，实际 {err:?}");
    let overwrite = TransferOpts { overwrite: true, resume_from: 0 };
    backend.upload(&session, &local, &remote, &overwrite).expect("覆盖上传必须成功");

    // ---- download（含断点续传） -
    let out = std::env::temp_dir().join(format!("netdiskhub-conformance-{tag}-download.txt"));
    let _ = std::fs::remove_file(&out);
    let report = backend
        .download(&session, &remote, &out, &TransferOpts::default())
        .expect("下载必须成功");
    assert_eq!(report.bytes, LOCAL_PAYLOAD.len() as u64);
    assert_eq!(std::fs::read(&out).unwrap(), LOCAL_PAYLOAD);

    let resumed = std::env::temp_dir().join(format!("netdiskhub-conformance-{tag}-resume.txt"));
    std::fs::write(&resumed, &LOCAL_PAYLOAD[..6]).unwrap();
    let resume_opts = TransferOpts { overwrite: false, resume_from: 6 };
    let report = backend
        .download(&session, &remote, &resumed, &resume_opts)
        .expect("续传下载必须成功");
    assert_eq!(report.resumed_from, 6, "续传起点应如实回报");
    assert_eq!(report.bytes, LOCAL_PAYLOAD.len() as u64 - 6, "续传只写剩余字节");
    assert_eq!(std::fs::read(&resumed).unwrap(), LOCAL_PAYLOAD);

    // ---- rename -
    let renamed = RemotePath::parse("/conformance-dir/hello-renamed.txt").unwrap();
    backend.rename(&session, &remote, &renamed).expect("重命名必须成功");
    let entries = backend
        .list(&session, &new_dir, &ListOptions::default())
        .expect("列新建目录必须成功");
    assert!(entries.entries.iter().any(|e| e.name == "hello-renamed.txt"));

    // ---- 目标不存在 -
    let missing = RemotePath::parse("/conformance-dir/nope.txt").unwrap();
    let err = backend.remove(&session, &missing).expect_err("删除不存在的目标必须失败");
    assert!(matches!(err, NetdiskError::NotFound { .. }), "应返回 NotFound，实际 {err:?}");
    let err = backend
        .download(&session, &missing, &out, &TransferOpts::default())
        .expect_err("下载不存在的目标必须失败");
    assert!(matches!(err, NetdiskError::NotFound { .. }), "应返回 NotFound，实际 {err:?}");

    // ---- 无效路径 -
    let err = RemotePath::parse("/conformance-dir/../etc").expect_err(".. 段必须被拒绝");
    assert!(matches!(err, NetdiskError::InvalidArgument { .. }), "应返回 InvalidArgument，实际 {err:?}");
    let bad_child = RemotePath::parse("/no-such-parent/child.txt").unwrap();
    let err = backend
        .upload(&session, &local, &bad_child, &TransferOpts::default())
        .expect_err("父目录不存在时必须失败");
    assert!(matches!(err, NetdiskError::InvalidArgument { .. }), "应返回 InvalidArgument，实际 {err:?}");

    // ---- 无效会话 -
    let ghost = SessionContext::new("definitely-not-a-real-session");
    let err = backend.list(&ghost, &root, &ListOptions::default()).expect_err("无效会话必须失败");
    assert!(matches!(err, NetdiskError::InvalidSession { .. }), "应返回 InvalidSession，实际 {err:?}");

    // ---- 清理 -
    backend.remove(&session, &renamed).expect("删除文件必须成功");
    backend.remove(&session, &new_dir).expect("删除空目录必须成功");
    let _ = std::fs::remove_file(&local);
    let _ = std::fs::remove_file(&out);
    let _ = std::fs::remove_file(&resumed);
}