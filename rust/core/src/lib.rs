//! NetdiskHub 统一 Provider 核心。
//!
//! 分层：
//! - [`model`]  通用数据模型（路径、条目、分页、配额、认证输入）；
//! - [`error`]  通用错误分类；
//! - [`provider`] 内部 trait + 契约测试；
//! - [`mock_alpha`] / [`mock_beta`] 两个行为不同的 Mock 实现；
//! - [`ffi`]     UniFFI 边界（句柄对象，不导出 trait、不暴露 token）。
//!
//! `netdiskhub_probe` 与 `probe_magic` 是 CI 构建检查依赖的探针，**不要改名或删除**。

pub mod error;
pub mod ffi;
pub mod mock_alpha;
pub mod mock_beta;
pub mod mock_common;
pub mod model;
pub mod provider;

/// CI 构建探针：被 `scripts/verify-apk-native-libs.py` 与 `rust.yml` 用于确认
/// `libnetdiskhub_core.so` 真的被编出来并打进 APK。返回值无业务含义。
#[unsafe(no_mangle)]
pub extern "C" fn netdiskhub_probe() -> u32 {
    0x0001_0002
}

/// UniFFI 探针：确认 Kotlin 绑定链路（生成 → 编译 → 打进 APK）是通的。
/// 返回值无业务含义，与 [`netdiskhub_probe`] 保持同一个常量。
#[uniffi::export]
pub fn probe_magic() -> u32 {
    0x0001_0002
}

uniffi::setup_scaffolding!();
