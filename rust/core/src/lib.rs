//! T-002e 第 3 轮：UniFFI 最小同步探针。
//!
//! 本文件不含业务逻辑、不实现任何网盘协议。
//! 存在的唯一目的是：让 UniFFI 0.32.2 在 cdylib 里写出接口元数据，
//! 使 uniffi-bindgen 能从中生成 Kotlin 绑定，并让 Kotlin 侧有一个可调用的同步入口。

/// 第 2 轮引入的交叉编译探针符号。
///
/// `rust.yml` 用 `llvm-nm -D --defined-only` 断言这个符号存在，
/// 删除或改名会让第 2 轮的验收变红，因此必须保留。
///
/// 返回值仅用于确认 `.so` 真的被链接并导出，不承载任何业务含义。
#[unsafe(no_mangle)]
pub extern "C" fn netdiskhub_probe() -> u32 {
    0x0001_0002
}

/// UniFFI 导出的最小同步探针。
///
/// 生成到 Kotlin 后等价于 `probeMagic(): UInt`。
/// 第 3 轮只在静态产物层面验证它，运行时调用属于 T-003。
#[uniffi::export]
pub fn probe_magic() -> u32 {
    0x0001_0002
}

uniffi::setup_scaffolding!();
