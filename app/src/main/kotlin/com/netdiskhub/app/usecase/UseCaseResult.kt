package com.netdiskhub.app.usecase

import com.netdiskhub.core.NetdiskException

/** UseCase 层的统一错误分类。只表达通用语义，不含任何网盘专有概念。 */
enum class ErrorKind {
    AUTH_FAILED,
    HUMAN_VERIFICATION_REQUIRED,
    RATE_LIMITED,
    NETWORK,
    INVALID_ARGUMENT,
    UNSUPPORTED,
    NOT_FOUND,
    ALREADY_EXISTS,
    INVALID_SESSION,
    UNEXPECTED,
}

/**
 * UseCase 层统一错误。
 *
 * [detail] 只用于展示与诊断，调用方不得把完整错误对象原样写入日志；
 * 本层不把凭据、token、验证码、ticket 放进任何字段。
 */
data class UseCaseError(
    val kind: ErrorKind,
    val detail: String,
    val retryAfterMs: Long? = null,
    val path: String? = null,
) {
    fun isRetryable(): Boolean = kind == ErrorKind.NETWORK || kind == ErrorKind.RATE_LIMITED
}

/** UseCase 层统一结果：要么成功带值，要么失败带分类错误。 */
sealed interface UseCaseResult<out T> {
    data class Success<T>(val value: T) : UseCaseResult<T>
    data class Failure(val error: UseCaseError) : UseCaseResult<Nothing>
}

fun <T> UseCaseResult<T>.valueOrNull(): T? = when (this) {
    is UseCaseResult.Success -> value
    is UseCaseResult.Failure -> null
}

fun <T> UseCaseResult<T>.errorOrNull(): UseCaseError? = when (this) {
    is UseCaseResult.Success -> null
    is UseCaseResult.Failure -> error
}

/** 会话句柄不存在或已被释放。属于 Kotlin 侧生命周期错误，不是 Rust 侧错误。 */
internal class UnknownSessionException : Exception("会话句柄不存在或已释放")

internal class InvalidParameterException(detail: String) : IllegalArgumentException(detail)

/** 阻塞调用跑到了 Android 主线程。属于开发者错误，直接抛出而不是转成业务错误。 */
internal class MainThreadViolationException(operation: String) : IllegalStateException(
    "UseCase 阻塞调用不允许在 Android 主线程执行：" + operation
)

internal fun mapThrowable(throwable: Throwable): UseCaseError = when (throwable) {
    is NetdiskException -> mapException(throwable)
    is UnknownSessionException ->
        UseCaseError(ErrorKind.INVALID_SESSION, throwable.message ?: "会话句柄不存在或已释放")
    is IllegalArgumentException ->
        UseCaseError(ErrorKind.INVALID_ARGUMENT, throwable.message ?: "参数无效")
    is IllegalStateException ->
        UseCaseError(ErrorKind.UNEXPECTED, throwable.message ?: "状态非法")
    else -> UseCaseError(
        ErrorKind.UNEXPECTED,
        throwable.javaClass.simpleName + ": " + (throwable.message ?: "无消息"),
    )
}

private fun mapException(exception: NetdiskException): UseCaseError = when (exception) {
    is NetdiskException.AuthFailed -> UseCaseError(ErrorKind.AUTH_FAILED, exception.detail)
    is NetdiskException.HumanVerificationRequired ->
        UseCaseError(ErrorKind.HUMAN_VERIFICATION_REQUIRED, exception.detail)
    is NetdiskException.RateLimited -> UseCaseError(
        ErrorKind.RATE_LIMITED,
        "请求被限流",
        retryAfterMs = exception.retryAfterMs.toLong(),
    )
    is NetdiskException.Network -> UseCaseError(ErrorKind.NETWORK, exception.detail)
    is NetdiskException.InvalidArgument -> UseCaseError(ErrorKind.INVALID_ARGUMENT, exception.detail)
    is NetdiskException.Unsupported -> UseCaseError(ErrorKind.UNSUPPORTED, exception.detail)
    is NetdiskException.NotFound -> UseCaseError(ErrorKind.NOT_FOUND, "路径不存在", path = exception.path)
    is NetdiskException.AlreadyExists ->
        UseCaseError(ErrorKind.ALREADY_EXISTS, "路径已存在", path = exception.path)
    is NetdiskException.InvalidSession -> UseCaseError(ErrorKind.INVALID_SESSION, exception.detail)
    is NetdiskException.Internal -> UseCaseError(ErrorKind.UNEXPECTED, exception.detail)
}
