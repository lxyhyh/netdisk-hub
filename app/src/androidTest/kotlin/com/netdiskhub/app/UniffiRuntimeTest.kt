package com.netdiskhub.app

import androidx.test.platform.app.InstrumentationRegistry
import com.netdiskhub.core.AuthInput
import com.netdiskhub.core.ListOptions
import com.netdiskhub.core.NetdiskClient
import com.netdiskhub.core.NetdiskException
import com.netdiskhub.core.RemotePath
import com.netdiskhub.core.Session
import com.netdiskhub.core.TransferOpts
import com.netdiskhub.core.probeMagic
import com.netdiskhub.core.providerIds
import com.sun.jna.Native
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Assert.fail
import org.junit.Test
import java.io.File

/**
 * T-003：在真实 Android 运行时验证「APK 里的 Rust 核心 + JNA + UniFFI 绑定」确实能跑通。
 *
 * 全部断言都在设备/模拟器上执行，不使用宿主机 Rust 测试代替。
 * 十个用例与任务卡里的十项一一对应，方法名以 t01..t10 开头，便于从 CI 日志定位。
 */
class UniffiRuntimeTest {

    private val alphaAccount = "alpha@example.test"
    private val alphaPassword = "alpha-pass-123"
    private val alphaPhone = "13000000001"
    private val alphaSmsCode = "111111"
    private val alphaScanTicket = "alpha-scan-ticket"
    private val betaAccount = "beta@example.test"
    private val betaPassword = "beta-pass-456"
    private val betaPhone = "13000000002"
    private val betaSmsCode = "222222"

    private fun alphaClient(): NetdiskClient = Bg.call { NetdiskClient("mock-alpha") }

    private fun betaClient(): NetdiskClient = Bg.call { NetdiskClient("mock-beta") }

    private fun alphaSession(client: NetdiskClient): Session = Bg.call {
        client.connect(AuthInput.Password(account = alphaAccount, password = alphaPassword))
    }

    /** 1. APK 成功加载 libnetdiskhub_core.so 和 JNA 原生库。 */
    @Test
    fun t01_nativeLibrariesAreLoaded() {
        // 先真的调一次 Rust：这一步会触发 JNA 的 Native.register 与 cdylib 的 dlopen，
        // 只有真加载成功才能拿到返回值。
        val magic = Bg.call { probeMagic() }
        assertEquals("probeMagic 未返回预期值，说明 Rust 库没有真正跑起来", 65538u, magic)

        // 地址空间证据：Android 在 extractNativeLibs=false（minSdk>=23 的默认值）时
        // 直接在 base.apk 内部 mmap 压缩包里的 .so，此时 /proc/self/maps 的路径列是
        // APK 路径而不是 .so 文件名（第 13 次跑 t01 就是按文件名断言而误报失败的）。
        // 所以这里两种形态都认：解压出来的 .so 文件，或已映射的 base.apk。
        val maps = File("/proc/self/maps").readText()
        val apkPath = InstrumentationRegistry.getInstrumentation().targetContext.applicationInfo.sourceDir
        val mappedByName = maps.contains("libnetdiskhub_core.so")
        val mappedFromApk = apkPath.isNotEmpty() && maps.contains(apkPath)
        assertTrue(
            "进程地址空间里既没有 libnetdiskhub_core.so，也没有映射 base.apk（apk=$apkPath）",
            mappedByName || mappedFromApk,
        )
        assertEquals("JNA 版本号未取到，说明 jnidispatch 没有真正加载", true, Native.VERSION.startsWith("5."))
    }

    /** 2. Kotlin 调用 probeMagic()，返回值必须是 0x00010002。 */
    @Test
    fun t02_probeMagicReturnsAgreedValue() {
        val magic = Bg.call { probeMagic() }
        assertEquals(0x0001_0002u, magic)
        assertEquals(65538u, magic)
    }

    /** 3. Kotlin 调用 providerIds()，获得 mock-alpha 与 mock-beta。 */
    @Test
    fun t03_providerIdsAreExposed() {
        val ids = Bg.call { providerIds() }
        assertEquals(listOf("mock-alpha", "mock-beta"), ids)
    }

    /** 4. 分别创建两个 NetdiskClient。 */
    @Test
    fun t04_twoClientsCanBeCreated() {
        val alpha = alphaClient()
        val beta = betaClient()
        try {
            val alphaInfo = Bg.call { alpha.info() }
            val betaInfo = Bg.call { beta.info() }
            assertEquals("mock-alpha", alphaInfo.id)
            assertEquals("mock-beta", betaInfo.id)
            assertEquals(listOf("password", "sms_code", "scan_code"), alphaInfo.authMethods)
            assertEquals(listOf("password", "sms_code"), betaInfo.authMethods)
            assertTrue(alphaInfo.supportsCursor)
            assertTrue(betaInfo.supportsCursor)
        } finally {
            alpha.close()
            beta.close()
        }
    }

    /** 5. 通过 UniFFI 实际执行登录，获得不透明 Session。 */
    @Test
    fun t05_loginYieldsOpaqueSession() {
        val alpha = alphaClient()
        try {
            val session = alphaSession(alpha)
            // Session 是不透明对象：Kotlin 侧除 close() 外没有任何可读属性，
            // 这里能做的只有「拿它继续调接口」——本身就是不透明性的证明。
            val page = Bg.call {
                alpha.list(session, RemotePath("/"), ListOptions(pageSize = 64u, cursor = null))
            }
            assertFalse("登录后列目录不应为空", page.entries.isEmpty())
            session.close()
        } finally {
            alpha.close()
        }
    }

    /** 6. 实际执行列目录、分页、配额查询。 */
    @Test
    fun t06_listPagingAndQuotaWork() {
        val alpha = alphaClient()
        try {
            val session = alphaSession(alpha)
            val root = RemotePath("/")

            val first = Bg.call { alpha.list(session, root, ListOptions(pageSize = 2u, cursor = null)) }
            assertEquals(listOf("documents", "notes.txt"), first.entries.map { entry -> entry.name })
            assertTrue("第一页之后应当还有数据", first.hasMore)
            assertEquals("notes.txt", first.nextCursor)

            val second = Bg.call {
                alpha.list(session, root, ListOptions(pageSize = 2u, cursor = first.nextCursor))
            }
            assertEquals(listOf("photo.png", "readme.md"), second.entries.map { entry -> entry.name })
            assertFalse("第二页之后不应再有数据", second.hasMore)
            assertNull("最后一页不应再有游标", second.nextCursor)

            // pageSize = 0 表示用 Provider 默认页长（Alpha 是 2）
            val defaulted = Bg.call { alpha.list(session, root, ListOptions(pageSize = 0u, cursor = null)) }
            assertEquals(2, defaulted.entries.size)

            val quota = Bg.call { alpha.quota(session) }
            assertEquals("配额总量", 1073741824uL, quota.totalBytes)
            assertEquals("已用配额", 44uL, quota.usedBytes)
            session.close()
        } finally {
            alpha.close()
        }
    }

    /** 7. 实际执行创建目录、上传、下载、重命名、删除。 */
    @Test
    fun t07_fullFileOperationChain() {
        val dir = scratchDir("t07")
        val alpha = alphaClient()
        try {
            val session = alphaSession(alpha)
            val payload = "hello-t003"
            val localIn = writeLocalFile(dir, "in.txt", payload)
            val localOut = File(dir, "out.txt")

            Bg.call { alpha.mkdir(session, RemotePath("/t003")) }
            val uploaded = Bg.call {
                alpha.upload(
                    session,
                    localIn.absolutePath,
                    RemotePath("/t003/in.txt"),
                    TransferOpts(overwrite = false, resumeFrom = 0uL),
                )
            }
            assertEquals(payload.toByteArray().size.toULong(), uploaded.bytes)

            val stat = Bg.call { alpha.stat(session, RemotePath("/t003/in.txt")) }
            assertEquals("in.txt", stat.name)
            assertFalse(stat.isDir)
            assertEquals(payload.toByteArray().size.toULong(), stat.size)

            val downloaded = Bg.call {
                alpha.download(
                    session,
                    RemotePath("/t003/in.txt"),
                    localOut.absolutePath,
                    TransferOpts(overwrite = true, resumeFrom = 0uL),
                )
            }
            assertEquals(payload.toByteArray().size.toULong(), downloaded.bytes)
            assertEquals(payload, localOut.readText())

            Bg.call { alpha.rename(session, RemotePath("/t003/in.txt"), RemotePath("/t003/renamed.txt")) }
            try {
                Bg.call { alpha.stat(session, RemotePath("/t003/in.txt")) }
                fail("重命名后旧路径仍可 stat")
            } catch (expected: NetdiskException.NotFound) {
                assertTrue(expected.path.contains("in.txt"))
            }

            Bg.call { alpha.remove(session, RemotePath("/t003/renamed.txt")) }
            Bg.call { alpha.remove(session, RemotePath("/t003")) }

            val root = Bg.call {
                alpha.list(session, RemotePath("/"), ListOptions(pageSize = 64u, cursor = null))
            }
            assertFalse("清理后根目录不应残留 t003", root.entries.any { entry -> entry.name == "t003" })
            session.close()
        } finally {
            alpha.close()
        }
    }

    /** 8. MockBeta 不支持扫码，Kotlin 必须收到 Unsupported。 */
    @Test
    fun t08_betaScanLoginIsUnsupported() {
        val beta = betaClient()
        try {
            try {
                Bg.call { beta.connect(AuthInput.ScanCode(ticket = "any-ticket")) }
                fail("Beta 不支持扫码，却登录成功了")
            } catch (expected: NetdiskException.Unsupported) {
                assertTrue(expected.detail.contains("不支持扫码"))
            }
            // 反过来：Alpha 的扫码是支持的，说明不是「所有扫码都失败」
            val alpha = alphaClient()
            try {
                val session = Bg.call { alpha.connect(AuthInput.ScanCode(ticket = alphaScanTicket)) }
                session.close()
            } finally {
                alpha.close()
            }
        } finally {
            beta.close()
        }
    }

    /** 9. 错误密码、无效路径、无效 Session 等错误能正确跨过 UniFFI 边界。 */
    @Test
    fun t09_errorsCrossTheUniffiBoundary() {
        val alpha = alphaClient()
        val beta = betaClient()
        try {
            // 9.1 错误密码 → AuthFailed，且错误文案不回显密码
            try {
                Bg.call { alpha.connect(AuthInput.Password(account = alphaAccount, password = "totally-wrong")) }
                fail("错误密码竟然登录成功")
            } catch (expected: NetdiskException.AuthFailed) {
                assertFalse("错误文案回显了用户输入", expected.detail.contains("totally-wrong"))
            }

            // 9.2 错误验证码 → AuthFailed
            try {
                Bg.call { alpha.connect(AuthInput.SmsCode(phone = alphaPhone, code = "000000", requestId = null)) }
                fail("错误验证码竟然登录成功")
            } catch (expected: NetdiskException.AuthFailed) {
                assertTrue(expected.detail.contains("验证码"))
            }

            // 9.3 未知 Provider → NotFound
            try {
                Bg.call { NetdiskClient("mock-gamma") }
                fail("未知 Provider 竟然创建成功")
            } catch (expected: NetdiskException.NotFound) {
                assertTrue(expected.path.contains("mock-gamma"))
            }

            val session = alphaSession(alpha)
            try {
                // 9.4 含 . / .. 段的路径由 Rust 侧统一拒绝（Kotlin 直接构造 RemotePath 也绕不过）
                try {
                    Bg.call {
                        alpha.list(session, RemotePath("/a/../b"), ListOptions(pageSize = 0u, cursor = null))
                    }
                    fail("包含 .. 的路径竟然被接受")
                } catch (expected: NetdiskException.InvalidArgument) {
                    assertTrue(expected.detail.contains(".."))
                }

                // 9.5 不存在的路径 → NotFound
                try {
                    Bg.call {
                        alpha.list(session, RemotePath("/no-such-dir"), ListOptions(pageSize = 0u, cursor = null))
                    }
                    fail("不存在的目录竟然列出来了")
                } catch (expected: NetdiskException.NotFound) {
                    assertEquals("/no-such-dir", expected.path)
                }

                // 9.6 用 Alpha 签发的 Session 去调 Beta → InvalidSession
                try {
                    Bg.call { beta.list(session, RemotePath("/"), ListOptions(pageSize = 0u, cursor = null)) }
                    fail("跨 Provider 使用会话竟然成功")
                } catch (expected: NetdiskException.InvalidSession) {
                    assertTrue(expected.detail.contains("未被本 Provider 签发"))
                }
            } finally {
                session.close()
            }

            // 9.7 已销毁的句柄不能再用。
            //
            // 这里刻意用「关闭后再调 client.info()」而不是「关闭后再拿着 Session 调 list」：
            // UniFFI 0.32.2 生成的下层转换器 FfiConverterTypeSession.lower 直接调
            // uniffiCloneHandle()，它不检查 callCounter，把已释放的句柄传给 Rust 侧
            // clone_handle（对已释放的 Arc 做 increment_strong_count）属于 use-after-free。
            // 该行为在绑定里就能看到，不能拿它当断言，否则测试可能直接把进程打崩。
            // 而 client 自身的方法都走 callWithHandle，销毁后必然抛 IllegalStateException。
            val throwaway = NetdiskClient("mock-alpha")
            throwaway.close()
            assertTrue("close() 之后 uniffiIsDestroyed 应为 true", throwaway.uniffiIsDestroyed)
            try {
                Bg.call { throwaway.info() }
                fail("已关闭的 client 竟然还能用")
            } catch (expected: IllegalStateException) {
                assertTrue(
                    "异常信息应说明对象已销毁：" + expected.message,
                    expected.message?.contains("destroyed") == true,
                )
            }
        } finally {
            alpha.close()
            beta.close()
        }
    }

    /** 10. 运行过程没有把密码、验证码、ticket 或会话信息写入日志。 */
    @Test
    fun t10_credentialsNeverReachLogcat() {
        // 先真的把这几种凭据都送进 UniFFI 一次，确保日志断言覆盖到实际调用路径
        val alpha = alphaClient()
        val beta = betaClient()
        try {
            val a = Bg.call {
                alpha.connect(AuthInput.Password(account = alphaAccount, password = alphaPassword))
            }
            a.close()
            val b = Bg.call {
                alpha.connect(AuthInput.SmsCode(phone = alphaPhone, code = alphaSmsCode, requestId = null))
            }
            b.close()
            val c = Bg.call { alpha.connect(AuthInput.ScanCode(ticket = alphaScanTicket)) }
            c.close()
            val d = Bg.call {
                beta.connect(AuthInput.Password(account = betaAccount, password = betaPassword))
            }
            d.close()
            val e = Bg.call {
                beta.connect(AuthInput.SmsCode(phone = betaPhone, code = betaSmsCode, requestId = null))
            }
            e.close()
        } finally {
            alpha.close()
            beta.close()
        }

        val dump = logcatDump()
        // 口令与 ticket 都是长随机串，子串命中即为泄漏
        for (secret in listOf(alphaPassword, betaPassword, alphaScanTicket)) {
            assertFalse("logcat 中出现了凭据明文：" + secret, dump.contains(secret))
        }
        // 纯数字验证码可能偶然出现在时间戳里，用「前后不是字母数字」的边界匹配
        for (code in listOf(alphaSmsCode, betaSmsCode)) {
            val pattern = Regex("(?<![0-9A-Za-z])" + code + "(?![0-9A-Za-z])")
            assertFalse("logcat 中出现了验证码明文：" + code, pattern.containsMatchIn(dump))
        }
    }
}
