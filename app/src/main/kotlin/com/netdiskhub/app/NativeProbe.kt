package com.netdiskhub.app

import com.netdiskhub.core.probeMagic

/**
 * T-002e 第 3 轮：Kotlin 侧对 Rust 导出接口的**编译期**引用。
 *
 * 本文件不被任何 UI 调用，作用只有一个：让「Kotlin 能调用 Rust 导出函数」
 * 由编译器证明 —— 包名 `com.netdiskhub.core`、函数名 `probeMagic`、返回类型
 * `UInt`、以及对 com.sun.jna 的依赖，全部要真的对得上才能编译通过。
 * 光看 CI 日志里「绑定文件已生成」证明不了这件事。
 *
 * 运行时真正执行这段代码（以及在设备上验证返回值）属于 T-003，本轮不做。
 */
internal object NativeProbe {
    /** 等价于 Rust 侧 `probe_magic()`，返回 0x0001_0002。 */
    fun magic(): UInt = probeMagic()
}
