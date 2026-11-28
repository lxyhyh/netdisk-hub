package com.netdiskhub.app

import com.netdiskhub.core.AuthInput
import com.netdiskhub.core.ListOptions
import com.netdiskhub.core.NetdiskClient
import com.netdiskhub.core.NetdiskException
import com.netdiskhub.core.Page
import com.netdiskhub.core.ProviderInfo
import com.netdiskhub.core.Quota
import com.netdiskhub.core.RemotePath
import com.netdiskhub.core.Session
import com.netdiskhub.core.TransferOpts
import com.netdiskhub.core.TransferReport
import com.netdiskhub.core.providerIds

/**
 * T-101 / T-102：Kotlin 侧对统一 Provider 接口的**编译期**引用。
 *
 * 与 [NativeProbe] 同一思路：本对象不被 UI 调用，唯一作用是让编译器证明
 * 「Kotlin 能真正使用这套通用类型与方法」——只要 UniFFI 导出的名字、
 * 参数类型、返回值类型、错误类型有一处对不上，APK 就编译失败。
 *
 * 这里刻意只出现通用概念（路径、分页、配额、传输），
 * 任何网盘专有字段都不应该能在这里出现。
 *
 * 运行时行为验证（在设备上真的跑一遍 MockProvider）属于 T-003，本轮不做。
 */
internal object ProviderProbe {
    /** 已注册的 Provider 标识。 */
    fun ids(): List<String> = providerIds()

    /** 走一遍统一接口的完整形状（仅编译期用途，不做任何网络或磁盘操作）。 */
    @Suppress("UNUSED_PARAMETER")
    fun exercise(providerId: String, localPath: String) {
        val client: NetdiskClient = try {
            NetdiskClient(providerId)
        } catch (error: NetdiskException) {
            // 未知 Provider 走统一错误类型，Kotlin 侧不需要知道任何协议细节。
            // 这里只证明「错误类型能被 Kotlin 捕获」；具体分类由上层决定。
            requireNotNull(error.message ?: "unknown") { "统一错误类型必须可读" }
            return
        }
        client.use { netdisk ->
            val info: ProviderInfo = netdisk.info()
            val authMethods: List<String> = info.authMethods

            val session: Session = netdisk.connect(
                AuthInput.Password(account = "probe@example.test", password = "probe"),
            )
            val root = RemotePath("/")
            val page: Page = netdisk.list(session, root, ListOptions(pageSize = 0u, cursor = null))
            val names: List<String> = page.entries.map { entry -> entry.name }
            val quota: Quota = netdisk.quota(session)
            val total: ULong? = quota.totalBytes

            val dir = RemotePath("/probe-dir")
            netdisk.mkdir(session, dir)
            val uploaded: TransferReport = netdisk.upload(
                session, localPath, RemotePath("/probe-dir/a.txt"), TransferOpts(overwrite = false, resumeFrom = 0uL),
            )
            val downloaded: TransferReport = netdisk.download(
                session, RemotePath("/probe-dir/a.txt"), localPath, TransferOpts(overwrite = true, resumeFrom = 0uL),
            )
            netdisk.rename(session, RemotePath("/probe-dir/a.txt"), RemotePath("/probe-dir/b.txt"))
            netdisk.remove(session, RemotePath("/probe-dir/b.txt"))

            // 显式使用全部局部变量，避免「未使用变量」告警淹没真正的编译错误，
            // 同时把返回类型钉死：任何一处类型漂移都会让这里编译失败。
            val report = listOf(
                info.id, info.displayName, authMethods.joinToString("/"),
                page.hasMore.toString(), page.nextCursor ?: "-", names.size.toString(),
                quota.usedBytes.toString(), total?.toString() ?: "unlimited",
                uploaded.bytes.toString(), uploaded.resumedFrom.toString(),
                downloaded.bytes.toString(), downloaded.resumedFrom.toString(),
            )
            check(report.isNotEmpty())
        }
    }
}