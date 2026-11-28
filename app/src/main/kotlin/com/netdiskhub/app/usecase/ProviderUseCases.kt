package com.netdiskhub.app.usecase

import android.os.Looper
import com.netdiskhub.core.AuthInput
import com.netdiskhub.core.ListOptions
import com.netdiskhub.core.NetdiskClient
import com.netdiskhub.core.Quota
import com.netdiskhub.core.RemoteFile
import com.netdiskhub.core.RemotePath
import com.netdiskhub.core.Session
import com.netdiskhub.core.TransferOpts
import com.netdiskhub.core.TransferReport
import com.netdiskhub.core.providerIds

/** Provider 的通用视图。字段与任何具体网盘无关。 */
data class ProviderSummary(
    val id: String,
    val displayName: String,
    val authMethods: List<String>,
    val supportsCursor: Boolean,
)

/** 一页文件。分页游标为 null 表示没有下一页。 */
data class FilePage(
    val entries: List<RemoteFile>,
    val nextCursor: String?,
    val hasMore: Boolean,
)

/**
 * Kotlin 侧的会话句柄。
 *
 * 只暴露 handleId 与 providerId；Rust 的 Session 对象、认证凭据、token 一律不出现。
 */
class SessionHandle internal constructor(
    val handleId: String,
    val providerId: String,
) {
    override fun toString(): String =
        "SessionHandle(handleId=" + handleId + ", providerId=" + providerId + ")"
}

/**
 * Kotlin UseCase 层：把 UniFFI 的阻塞调用包成「统一结果 + 统一错误」，并管理会话生命周期。
 *
 * 三条硬约束：
 * 1. 所有方法都是阻塞调用，必须在后台线程执行；在主线程调用直接抛 [MainThreadViolationException]。
 * 2. 只使用通用 Provider 概念，不出现任何具体网盘的字段、协议或特例。
 * 3. 不打印、不返回任何凭据、验证码、ticket 或会话 token。
 */
class ProviderUseCases : AutoCloseable {

    private val lock = Any()
    private val clients = LinkedHashMap<String, NetdiskClient>()
    private val sessions = LinkedHashMap<String, SessionEntry>()
    private var nextHandle: Long = 1L

    private class SessionEntry(
        val providerId: String,
        val client: NetdiskClient,
        val session: Session,
    )

    /** 列出所有可用 Provider 及其基本信息。 */
    fun providers(): UseCaseResult<List<ProviderSummary>> = guarded("providers") {
        providerIds().map { id ->
            val info = clientFor(id).info()
            ProviderSummary(
                id = info.id,
                displayName = info.displayName,
                authMethods = info.authMethods,
                supportsCursor = info.supportsCursor,
            )
        }
    }

    /** 用给定认证方式登录，成功后返回 Kotlin 侧会话句柄。 */
    fun openSession(providerId: String, auth: AuthInput): UseCaseResult<SessionHandle> =
        guarded("openSession") {
            val client = clientFor(providerId)
            val session = client.connect(auth)
            val handle = synchronized(lock) {
                val id = "kh-" + nextHandle
                nextHandle += 1
                sessions[id] = SessionEntry(providerId, client, session)
                SessionHandle(id, providerId)
            }
            handle
        }

    /** 关闭会话并释放 Rust 侧句柄。重复关闭返回 INVALID_SESSION。 */
    fun closeSession(handle: SessionHandle): UseCaseResult<Unit> = guarded("closeSession") {
        val entry = synchronized(lock) { sessions.remove(handle.handleId) }
            ?: throw UnknownSessionException()
        entry.session.close()
    }

    /** 列目录，支持游标分页。pageSize = 0 表示使用 Provider 默认页长。 */
    fun listFiles(
        handle: SessionHandle,
        path: String,
        pageSize: UInt = 0u,
        cursor: String? = null,
    ): UseCaseResult<FilePage> = guarded("listFiles") {
        val entry = entryFor(handle)
        val page = entry.client.list(
            entry.session,
            RemotePath(path),
            ListOptions(pageSize = pageSize, cursor = cursor),
        )
        FilePage(entries = page.entries, nextCursor = page.nextCursor, hasMore = page.hasMore)
    }

    /** 查询单个文件的详情。 */
    fun statFile(handle: SessionHandle, path: String): UseCaseResult<RemoteFile> =
        guarded("statFile") {
            val entry = entryFor(handle)
            entry.client.stat(entry.session, RemotePath(path))
        }

    /** 查询配额。 */
    fun quota(handle: SessionHandle): UseCaseResult<Quota> = guarded("quota") {
        val entry = entryFor(handle)
        entry.client.quota(entry.session)
    }

    /** 上传本地文件到远端路径。localPath 必须是宿主上的绝对路径。 */
    fun upload(
        handle: SessionHandle,
        localPath: String,
        remotePath: String,
        overwrite: Boolean = false,
        resumeFrom: ULong = 0uL,
    ): UseCaseResult<TransferReport> = guarded("upload") {
        val entry = entryFor(handle)
        entry.client.upload(
            entry.session,
            localPath,
            RemotePath(remotePath),
            TransferOpts(overwrite = overwrite, resumeFrom = resumeFrom),
        )
    }

    /** 下载远端文件到本地路径。 */
    fun download(
        handle: SessionHandle,
        remotePath: String,
        localPath: String,
        overwrite: Boolean = false,
        resumeFrom: ULong = 0uL,
    ): UseCaseResult<TransferReport> = guarded("download") {
        val entry = entryFor(handle)
        entry.client.download(
            entry.session,
            RemotePath(remotePath),
            localPath,
            TransferOpts(overwrite = overwrite, resumeFrom = resumeFrom),
        )
    }

    /** 删除文件或空目录。 */
    fun remove(handle: SessionHandle, path: String): UseCaseResult<Unit> = guarded("remove") {
        val entry = entryFor(handle)
        entry.client.remove(entry.session, RemotePath(path))
    }

    /** 重命名 / 移动。 */
    fun rename(handle: SessionHandle, from: String, to: String): UseCaseResult<Unit> =
        guarded("rename") {
            val entry = entryFor(handle)
            entry.client.rename(entry.session, RemotePath(from), RemotePath(to))
        }

    /** 新建目录。 */
    fun mkdir(handle: SessionHandle, path: String): UseCaseResult<Unit> = guarded("mkdir") {
        val entry = entryFor(handle)
        entry.client.mkdir(entry.session, RemotePath(path))
    }

    /** 释放全部会话与客户端。 */
    override fun close() {
        val entries = synchronized(lock) {
            val snapshot = sessions.values.toList()
            sessions.clear()
            snapshot
        }
        for (entry in entries) {
            runCatching { entry.session.close() }
        }
        val clientsToClose = synchronized(lock) {
            val snapshot = clients.values.toList()
            clients.clear()
            snapshot
        }
        for (client in clientsToClose) {
            runCatching { client.close() }
        }
    }

    private fun clientFor(providerId: String): NetdiskClient = synchronized(lock) {
        clients.getOrPut(providerId) { NetdiskClient(providerId) }
    }

    private fun entryFor(handle: SessionHandle): SessionEntry =
        synchronized(lock) { sessions[handle.handleId] } ?: throw UnknownSessionException()

    private fun <T> guarded(operation: String, block: () -> T): UseCaseResult<T> {
        requireBackgroundThread(operation)
        return try {
            UseCaseResult.Success(block())
        } catch (error: OutOfMemoryError) {
            throw error
        } catch (throwable: Throwable) {
            UseCaseResult.Failure(mapThrowable(throwable))
        }
    }

    private fun requireBackgroundThread(operation: String) {
        if (Looper.myLooper() != null && Looper.myLooper() === Looper.getMainLooper()) {
            throw MainThreadViolationException(operation)
        }
    }
}
