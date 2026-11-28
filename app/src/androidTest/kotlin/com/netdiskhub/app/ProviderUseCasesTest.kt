package com.netdiskhub.app

import android.app.InstrumentationRegistry
import com.netdiskhub.app.usecase.ErrorKind
import com.netdiskhub.app.usecase.FilePage
import com.netdiskhub.app.usecase.ProviderSummary
import com.netdiskhub.app.usecase.ProviderUseCases
import com.netdiskhub.app.usecase.SessionHandle
import com.netdiskhub.app.usecase.UseCaseResult
import com.netdiskhub.app.usecase.errorOrNull
import com.netdiskhub.app.usecase.valueOrNull
import com.netdiskhub.core.AuthInput
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Assert.fail
import org.junit.Test
import java.util.concurrent.atomic.AtomicReference

/**
 * T-103：Kotlin UseCase 层的运行时测试。
 *
 * 全部用例都在真实 Android 运行时执行（UniFFI 绑定 + JNA + Rust 核心都是真的），
 * 而且两个 MockProvider 走的是**同一套** UseCase 调用方式 —— 用例本身不针对任何一家写分支。
 */
class ProviderUseCasesTest {

    private val alphaPassword = AuthInput.Password(account = "alpha@example.test", password = "alpha-pass-123")
    private val betaPassword = AuthInput.Password(account = "beta@example.test", password = "beta-pass-456")

    private fun <T> ok(result: UseCaseResult<T>): T {
        when (result) {
            is UseCaseResult.Success -> return result.value
            is UseCaseResult.Failure -> fail("期望成功，实际失败：" + result.error.kind + " / " + result.error.detail)
        }
        throw IllegalStateException("unreachable")
    }

    private fun <T> err(result: UseCaseResult<T>): com.netdiskhub.app.usecase.UseCaseError {
        when (result) {
            is UseCaseResult.Failure -> return result.error
            is UseCaseResult.Success -> fail("期望失败，实际成功：" + result.value)
        }
        throw IllegalStateException("unreachable")
    }

    /** Provider 列表：两个 Mock 都能列出来，字段是通用概念。 */
    @Test
    fun providersAreListedGenerically() {
        val useCases = ProviderUseCases()
        try {
            val providers: List<ProviderSummary> = ok(Bg.call { useCases.providers() })
            assertEquals(listOf("mock-alpha", "mock-beta"), providers.map { it.id })
            assertEquals(listOf("Mock Alpha", "Mock Beta"), providers.map { it.displayName })
            assertEquals(listOf("password", "sms_code", "scan_code"), providers[0].authMethods)
            assertEquals(listOf("password", "sms_code"), providers[1].authMethods)
            assertTrue(providers.all { it.supportsCursor })
        } finally {
            useCases.close()
        }
    }

    /** 同一套 UseCase 对两个 Provider 做登录，调用方式完全一致。 */
    @Test
    fun sameCallShapeWorksForBothProviders() {
        val useCases = ProviderUseCases()
        try {
            for ((providerId, auth) in listOf("mock-alpha" to alphaPassword, "mock-beta" to betaPassword)) {
                val handle: SessionHandle = ok(Bg.call { useCases.openSession(providerId, auth) })
                assertEquals(providerId, handle.providerId)

                val page: FilePage = ok(Bg.call { useCases.listFiles(handle, "/") })
                assertFalse(providerId + " 根目录不应为空", page.entries.isEmpty())

                val quota = ok(Bg.call { useCases.quota(handle) })
                assertNotNull(quota.totalBytes)

                ok(Bg.call { useCases.closeSession(handle) })
            }
        } finally {
            useCases.close()
        }
    }

    /** 登录失败：错误密码 → AUTH_FAILED，且错误对象里没有明文口令。 */
    @Test
    fun wrongPasswordBecomesAuthFailedError() {
        val useCases = ProviderUseCases()
        try {
            for (providerId in listOf("mock-alpha", "mock-beta")) {
                val failure = err(
                    Bg.call {
                        useCases.openSession(
                            providerId,
                            AuthInput.Password(account = "someone@example.test", password = "wrong-secret-value"),
                        )
                    },
                )
                assertEquals(ErrorKind.AUTH_FAILED, failure.kind)
                assertFalse("错误对象泄漏了口令", failure.detail.contains("wrong-secret-value"))
                assertFalse(failure.isRetryable())
            }
        } finally {
            useCases.close()
        }
    }

    /** Beta 不支持扫码：UseCase 把 UniFFI 的 Unsupported 统一成 UNSUPPORTED。 */
    @Test
    fun betaScanLoginBecomesUnsupportedError() {
        val useCases = ProviderUseCases()
        try {
            val failure = err(
                Bg.call { useCases.openSession("mock-beta", AuthInput.ScanCode(ticket = "whatever")) },
            )
            assertEquals(ErrorKind.UNSUPPORTED, failure.kind)
            assertTrue(failure.detail.contains("不支持扫码"))

            // 同一个方法在 Alpha 上成功 —— 证明差异来自 Provider 能力，不是 UseCase 特判
            val handle = ok(
                Bg.call { useCases.openSession("mock-alpha", AuthInput.ScanCode(ticket = "alpha-scan-ticket")) },
            )
            ok(Bg.call { useCases.closeSession(handle) })
        } finally {
            useCases.close()
        }
    }

    /** 列目录与分页：游标翻页能拿到第二页且终止。 */
    @Test
    fun pagingWalksAllEntries() {
        val useCases = ProviderUseCases()
        try {
            val handle = ok(Bg.call { useCases.openSession("mock-alpha", alphaPassword) })
            val first = ok(Bg.call { useCases.listFiles(handle, "/", pageSize = 2u) })
            assertEquals(2, first.entries.size)
            assertTrue(first.hasMore)

            val second = ok(
                Bg.call { useCases.listFiles(handle, "/", pageSize = 2u, cursor = first.nextCursor) },
            )
            assertFalse(second.hasMore)
            assertNull(second.nextCursor)

            val names = (first.entries + second.entries).map { it.name }
            assertEquals(listOf("documents", "notes.txt", "photo.png", "readme.md"), names)
            ok(Bg.call { useCases.closeSession(handle) })
        } finally {
            useCases.close()
        }
    }

    /** 文件操作完整链路：新建目录 → 上传 → 详情 → 下载 → 重命名 → 删除。 */
    @Test
    fun fullFileOperationChain() {
        val dir = scratchDir("usecase")
        val useCases = ProviderUseCases()
        try {
            val handle = ok(Bg.call { useCases.openSession("mock-alpha", alphaPassword) })
            val payload = "usecase-payload"
            val localIn = writeLocalFile(dir, "up.txt", payload)
            val localOut = java.io.File(dir, "down.txt")

            ok(Bg.call { useCases.mkdir(handle, "/t103") })

            val uploaded = ok(
                Bg.call { useCases.upload(handle, localIn.absolutePath, "/t103/up.txt") },
            )
            assertEquals(payload.toByteArray().size.toULong(), uploaded.bytes)

            val stat = ok(Bg.call { useCases.statFile(handle, "/t103/up.txt") })
            assertEquals("up.txt", stat.name)
            assertFalse(stat.isDir)

            val downloaded = ok(
                Bg.call { useCases.download(handle, "/t103/up.txt", localOut.absolutePath) },
            )
            assertEquals(payload.toByteArray().size.toULong(), downloaded.bytes)
            assertEquals(payload, localOut.readText())

            ok(Bg.call { useCases.rename(handle, "/t103/up.txt", "/t103/moved.txt") })
            assertEquals("moved.txt", ok(Bg.call { useCases.statFile(handle, "/t103/moved.txt") }).name)

            ok(Bg.call { useCases.remove(handle, "/t103/moved.txt") })
            ok(Bg.call { useCases.remove(handle, "/t103") })

            val root = ok(Bg.call { useCases.listFiles(handle, "/", pageSize = 64u) })
            assertFalse(root.entries.any { it.name == "t103" })
            ok(Bg.call { useCases.closeSession(handle) })
        } finally {
            useCases.close()
        }
    }

    /** 错误传播：不存在的路径 → NOT_FOUND 且带回路径。 */
    @Test
    fun missingPathBecomesNotFound() {
        val useCases = ProviderUseCases()
        try {
            val handle = ok(Bg.call { useCases.openSession("mock-alpha", alphaPassword) })
            val failure = err(Bg.call { useCases.listFiles(handle, "/never-existed") })
            assertEquals(ErrorKind.NOT_FOUND, failure.kind)
            assertEquals("/never-existed", failure.path)
            ok(Bg.call { useCases.closeSession(handle) })
        } finally {
            useCases.close()
        }
    }

    /** 非法参数：路径含 .. 由 Rust 拒绝，UseCase 统一成 INVALID_ARGUMENT。 */
    @Test
    fun illegalPathBecomesInvalidArgument() {
        val useCases = ProviderUseCases()
        try {
            val handle = ok(Bg.call { useCases.openSession("mock-alpha", alphaPassword) })
            val failure = err(Bg.call { useCases.listFiles(handle, "/a/../b") })
            assertEquals(ErrorKind.INVALID_ARGUMENT, failure.kind)
            ok(Bg.call { useCases.closeSession(handle) })
        } finally {
            useCases.close()
        }
    }

    /** Session 生命周期：关闭后句柄失效，重复关闭与继续使用都返回 INVALID_SESSION。 */
    @Test
    fun closedSessionBecomesInvalidSession() {
        val useCases = ProviderUseCases()
        try {
            val handle = ok(Bg.call { useCases.openSession("mock-alpha", alphaPassword) })
            ok(Bg.call { useCases.closeSession(handle) })

            assertEquals(
                ErrorKind.INVALID_SESSION,
                err(Bg.call { useCases.closeSession(handle) }).kind,
            )
            assertEquals(
                ErrorKind.INVALID_SESSION,
                err(Bg.call { useCases.listFiles(handle, "/") }).kind,
            )
            assertEquals(
                ErrorKind.INVALID_SESSION,
                err(Bg.call { useCases.quota(handle) }).kind,
            )
        } finally {
            useCases.close()
        }
    }

    /**
     * 主线程保护：阻塞调用在主线程直接抛异常，而不是悄悄阻塞 UI。
     *
     * 注意：JUnit 用例本身跑在 instrumentation 线程上，Looper.myLooper() 为 null，
     * 所以必须用 runOnMainSync 真正切到应用主线程，否则这条用例什么也证明不了。
     */
    @Test
    fun mainThreadCallsAreRejected() {
        val useCases = ProviderUseCases()
        try {
            val thrown = AtomicReference<Throwable?>(null)
            InstrumentationRegistry.getInstrumentation().runOnMainSync {
                thrown.set(
                    try {
                        useCases.providers()
                        null
                    } catch (t: Throwable) {
                        t
                    },
                )
            }
            assertNotNull("主线程调用阻塞 UseCase 竟然没有报错", thrown.get())
            val error: Throwable = thrown.get()!!
            assertTrue("期望 IllegalStateException，实际 " + error.javaClass.name, error is IllegalStateException)
            assertTrue("异常信息里应写明主线程：" + error.message, error.message?.contains("主线程") == true)
        } finally {
            useCases.close()
        }
    }

    /** 未知 Provider：NotFound 被映射成 NOT_FOUND，且不是 UNEXPECTED。 */
    @Test
    fun unknownProviderBecomesNotFound() {
        val useCases = ProviderUseCases()
        try {
            val failure = err(Bg.call { useCases.openSession("mock-gamma", alphaPassword) })
            assertEquals(ErrorKind.NOT_FOUND, failure.kind)
            assertTrue(failure.path?.contains("mock-gamma") == true)
        } finally {
            useCases.close()
        }
    }

    /** 结果包装的基本语义（valueOrNull / errorOrNull）。 */
    @Test
    fun resultAccessorsBehaveAsDocumented() {
        val success: UseCaseResult<Int> = UseCaseResult.Success(7)
        assertEquals(7, success.valueOrNull())
        assertNull(success.errorOrNull())

        val failure: UseCaseResult<Int> = UseCaseResult.Failure(
            com.netdiskhub.app.usecase.UseCaseError(ErrorKind.NETWORK, "network"),
        )
        assertNull(failure.valueOrNull())
        assertEquals(ErrorKind.NETWORK, failure.errorOrNull()?.kind)
        assertTrue(failure.errorOrNull()?.isRetryable() == true)
    }
}
