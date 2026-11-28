//! 两个 MockProvider 共用的内存虚拟文件系统。
//!
//! 只做「文件系统模拟」：不涉及网络、不涉及认证。认证放在各自的 Provider 里，
//! 这样两个 Mock 的登录行为可以不同，而文件操作语义保持一致。

use std::collections::{BTreeMap, BTreeSet};

use crate::error::NetdiskError;
use crate::model::{ListOptions, Page, Quota, RemoteFile, RemotePath, TransferOpts, TransferReport};

/// 一个文件条目的元数据。
#[derive(Debug, Clone)]
struct Entry {
    data: Vec<u8>,
    modified_ms: u64,
    id: String,
}

/// 内存虚拟文件系统。
pub(crate) struct MockFs {
    dirs: BTreeSet<String>,
    files: BTreeMap<String, Entry>,
    total_bytes: u64,
    next_id: u64,
    id_prefix: &'static str,
    default_page_size: u32,
}

impl MockFs {
    pub(crate) fn new(total_bytes: u64, id_prefix: &'static str, default_page_size: u32) -> Self {
        let mut dirs = BTreeSet::new();
        dirs.insert(RemotePath::ROOT.to_string());
        Self { dirs, files: BTreeMap::new(), total_bytes, next_id: 1, id_prefix, default_page_size }
    }

    fn next_entry_id(&mut self) -> String {
        let n = self.next_id;
        self.next_id += 1;
        format!("{}-{}", self.id_prefix, n)
    }

    fn next_modified_ms(&self) -> u64 {
        // 用递增序号伪造时间戳：可预测、可断言，不依赖系统时钟。
        1_700_000_000_000 + self.next_id * 1_000
    }

    /// 目录是否存在。
    pub(crate) fn is_dir(&self, path: &RemotePath) -> bool {
        self.dirs.contains(&path.value)
    }

    /// 普通文件是否存在。
    pub(crate) fn is_file(&self, path: &RemotePath) -> bool {
        self.files.contains_key(&path.value)
    }

    fn exists(&self, path: &RemotePath) -> bool {
        self.is_dir(path) || self.is_file(path)
    }

    /// 新建目录。父目录必须存在且为目录。
    pub(crate) fn mkdir(&mut self, path: &RemotePath) -> Result<(), NetdiskError> {
        if self.exists(path) {
            return Err(NetdiskError::AlreadyExists { path: path.value.clone() });
        }
        let parent = path.parent();
        if !self.is_dir(&parent) {
            return Err(NetdiskError::invalid_argument(format!("父目录不存在：{}", parent.value)));
        }
        self.dirs.insert(path.value.clone());
        Ok(())
    }

    /// 列目录（支持游标分页）。
    pub(crate) fn list(&self, path: &RemotePath, options: &ListOptions) -> Result<Page, NetdiskError> {
        if !self.is_dir(path) {
            return Err(NetdiskError::NotFound { path: path.value.clone() });
        }
        let page_size = if options.page_size == 0 { self.default_page_size } else { options.page_size };
        let mut entries = self.children(path);
        if let Some(cursor) = &options.cursor {
            entries.retain(|entry| &entry.name > cursor);
        }
        let has_more = entries.len() > page_size as usize;
        let mut page_entries: Vec<RemoteFile> = entries.into_iter().take(page_size as usize).collect();
        let next_cursor = if has_more {
            page_entries.last().map(|entry| entry.name.clone())
        } else {
            None
        };
        page_entries.shrink_to_fit();
        Ok(Page { entries: page_entries, next_cursor, has_more })
    }

    fn children(&self, dir: &RemotePath) -> Vec<RemoteFile> {
        let prefix = if dir.is_root() { "/".to_string() } else { format!("{}/", dir.value) };
        let mut out: Vec<RemoteFile> = Vec::new();
        for path in &self.dirs {
            if path == &dir.value {
                continue;
            }
            if let Some(name) = path.strip_prefix(&prefix) {
                if !name.is_empty() && !name.contains('/') {
                    out.push(RemoteFile {
                        name: name.to_string(),
                        path: RemotePath { value: path.clone() },
                        is_dir: true,
                        size: 0,
                        modified_ms: None,
                        id: None,
                    });
                }
            }
        }
        for (path, entry) in &self.files {
            if let Some(name) = path.strip_prefix(&prefix) {
                if !name.is_empty() && !name.contains('/') {
                    out.push(RemoteFile {
                        name: name.to_string(),
                        path: RemotePath { value: path.clone() },
                        is_dir: false,
                        size: entry.data.len() as u64,
                        modified_ms: Some(entry.modified_ms),
                        id: Some(entry.id.clone()),
                    });
                }
            }
        }
        out.sort_by(|a, b| a.name.cmp(&b.name));
        out
    }

    /// 写入文件（供种子数据与上传共用）。
    pub(crate) fn put_file(&mut self, path: &RemotePath, data: Vec<u8>, options: &TransferOpts) -> Result<TransferReport, NetdiskError> {
        if path.is_root() {
            return Err(NetdiskError::invalid_argument("不能写入根目录"));
        }
        let parent = path.parent();
        if !self.is_dir(&parent) {
            return Err(NetdiskError::invalid_argument(format!("父目录不存在：{}", parent.value)));
        }
        if self.is_dir(path) {
            return Err(NetdiskError::AlreadyExists { path: path.value.clone() });
        }
        let resumed_from = match self.files.get(&path.value) {
            Some(existing) => {
                if !options.overwrite {
                    return Err(NetdiskError::AlreadyExists { path: path.value.clone() });
                }
                if options.resume_from > existing.data.len() as u64 {
                    return Err(NetdiskError::invalid_argument("续传起点超过已有文件长度"));
                }
                options.resume_from
            }
            None => {
                if options.resume_from != 0 {
                    return Err(NetdiskError::invalid_argument("目标不存在，无法续传"));
                }
                0
            }
        };
        let written = data.len() as u64 - resumed_from;
        let mut merged = match self.files.get(&path.value) {
            Some(existing) if resumed_from > 0 => existing.data[..resumed_from as usize].to_vec(),
            _ => Vec::new(),
        };
        merged.extend_from_slice(&data[resumed_from as usize..]);
        let id = match self.files.get(&path.value) {
            Some(existing) => existing.id.clone(),
            None => self.next_entry_id(),
        };
        let modified_ms = self.next_modified_ms();
        self.files.insert(path.value.clone(), Entry { data: merged, modified_ms, id });
        Ok(TransferReport { bytes: written, resumed_from })
    }

    /// 读取完整文件内容。续传切片由调用方负责，这样元数据（总长度）只有一处真相。
    pub(crate) fn read_all(&self, path: &RemotePath) -> Result<Vec<u8>, NetdiskError> {
        match self.files.get(&path.value) {
            Some(entry) => Ok(entry.data.clone()),
            None => {
                if self.is_dir(path) {
                    return Err(NetdiskError::invalid_argument(format!("目标是目录，不能下载：{}", path.value)));
                }
                Err(NetdiskError::NotFound { path: path.value.clone() })
            }
        }
    }

    /// 删除。目录必须为空。
    pub(crate) fn remove(&mut self, path: &RemotePath) -> Result<(), NetdiskError> {
        if self.files.remove(&path.value).is_some() {
            return Ok(());
        }
        if !self.is_dir(path) {
            return Err(NetdiskError::NotFound { path: path.value.clone() });
        }
        if path.is_root() {
            return Err(NetdiskError::invalid_argument("不能删除根目录"));
        }
        let prefix = format!("{}/", path.value);
        let has_child_dir = self.dirs.iter().any(|dir| dir.starts_with(&prefix));
        let has_child_file = self.files.keys().any(|file| file.starts_with(&prefix));
        if has_child_dir || has_child_file {
            return Err(NetdiskError::invalid_argument(format!("目录非空，不能删除：{}", path.value)));
        }
        self.dirs.remove(&path.value);
        Ok(())
    }

    /// 重命名 / 移动。
    pub(crate) fn rename(&mut self, from: &RemotePath, to: &RemotePath) -> Result<(), NetdiskError> {
        if !self.exists(from) {
            return Err(NetdiskError::NotFound { path: from.value.clone() });
        }
        if to.is_root() || self.exists(to) {
            return Err(NetdiskError::AlreadyExists { path: to.value.clone() });
        }
        let parent = to.parent();
        if !self.is_dir(&parent) {
            return Err(NetdiskError::invalid_argument(format!("目标父目录不存在：{}", parent.value)));
        }
        if let Some(mut entry) = self.files.remove(&from.value) {
            entry.modified_ms = self.next_modified_ms();
            self.files.insert(to.value.clone(), entry);
            return Ok(());
        }
        let old_prefix = format!("{}/", from.value);
        let new_prefix = format!("{}/", to.value);
        let moved_dirs: Vec<String> = self
            .dirs
            .iter()
            .filter(|dir| dir.starts_with(&old_prefix) || **dir == from.value)
            .cloned()
            .collect();
        for dir in moved_dirs {
            self.dirs.remove(&dir);
            let new_dir = if dir == from.value {
                to.value.clone()
            } else {
                format!("{}{}", new_prefix, &dir[old_prefix.len()..])
            };
            self.dirs.insert(new_dir);
        }
        let moved_files: Vec<String> = self
            .files
            .keys()
            .filter(|file| file.starts_with(&old_prefix))
            .cloned()
            .collect();
        for file in moved_files {
            if let Some(entry) = self.files.remove(&file) {
                let new_file = format!("{}{}", new_prefix, &file[old_prefix.len()..]);
                self.files.insert(new_file, entry);
            }
        }
        Ok(())
    }

    /// 配额。
    pub(crate) fn quota(&self) -> Quota {
        let used: u64 = self.files.values().map(|entry| entry.data.len() as u64).sum();
        Quota { used_bytes: used, total_bytes: Some(self.total_bytes), file_count: Some(self.files.len() as u64) }
    }
}