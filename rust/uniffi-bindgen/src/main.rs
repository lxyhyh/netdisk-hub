//! 生成 UniFFI 绑定的命令行入口。
//!
//! 只是转调 `uniffi::uniffi_bindgen_main()`，与官方 0.32.2 的
//! `uniffi-bindgen.rs` 完全一致；真正的参数解析在 uniffi crate 内部。

fn main() {
    uniffi::uniffi_bindgen_main();
}
