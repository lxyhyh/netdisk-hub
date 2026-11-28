package com.netdiskhub.app

import androidx.test.platform.app.InstrumentationRegistry
import java.io.File
import java.util.concurrent.Callable
import java.util.concurrent.ExecutionException
import java.util.concurrent.Executors
import java.util.concurrent.TimeUnit

/**
 * 仪器测试的公共工具。
 *
 * 两件事：
 * 1. Bg.call 把阻塞的 UniFFI 调用放到独立后台线程执行 —— UseCase 层会主动拒绝主线程调用，
 *    而且真实的网盘调用本来就阻塞，测试必须显式地在后台跑。
 * 2. logcatDump 读取本进程的 logcat，用于「日志里不得出现凭据」的运行时断言。
 */
internal object Bg {
    private val pool = Executors.newSingleThreadExecutor { runnable ->
        Thread(runnable, "netdiskhub-test-worker")
    }

    fun <T> call(block: () -> T): T =
        try {
            pool.submit(Callable { block() }).get(120, TimeUnit.SECONDS)
        } catch (failure: ExecutionException) {
            // Future.get 会把异步体内的异常包成 ExecutionException；
            // 测试要按原始类型（NetdiskException.NotFound 等）断言，必须解开这一层。
            throw failure.cause ?: failure
        } catch (interrupted: InterruptedException) {
            Thread.currentThread().interrupt()
            throw interrupted
        }
}

/** 测试用的本地临时目录：应用自己的 cache 目录，不依赖 runner 上的固定路径。 */
internal fun scratchDir(name: String): File {
    val base = InstrumentationRegistry.getInstrumentation().targetContext.cacheDir
    val dir = File(base, "netdiskhub-test/" + name)
    if (dir.exists()) {
        dir.deleteRecursively()
    }
    check(dir.mkdirs() || dir.isDirectory) { "无法创建测试目录: " + dir.absolutePath }
    return dir
}

internal fun writeLocalFile(dir: File, name: String, content: String): File {
    val file = File(dir, name)
    file.writeText(content)
    return file
}

/** 读取本进程的 logcat。退出码非 0 直接失败，避免「读不到日志」被当成通过。 */
internal fun logcatDump(): String {
    val process = ProcessBuilder("logcat", "-d", "-v", "threadtime")
        .redirectErrorStream(true)
        .start()
    val text = process.inputStream.bufferedReader().use { reader -> reader.readText() }
    val code = process.waitFor()
    if (code != 0) {
        throw IllegalStateException("logcat -d 退出码 " + code)
    }
    return text
}
