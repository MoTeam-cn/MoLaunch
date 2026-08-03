/**
 * 异步工具函数
 * 提供 `safeCall` 高阶函数，统一 try/catch + console.error 错误处理。
 */

/**
 * 安全执行异步函数，捕获异常并打印到控制台。
 *
 * - 成功：返回结果
 * - 失败：打印 errorLog + 可选 onError 回调，返回 undefined
 *
 * @example
 * // 基本用法
 * const data = await safeCall(() => fetchData(), 'fetchData')
 *
 * // 带错误回调
 * const result = await safeCall(
 *   () => api.deleteMod(id),
 *   'deleteMod',
 *   (e) => showError('删除失败：' + String(e)),
 * )
 */
export async function safeCall<T>(
  fn: () => Promise<T>,
  label: string,
  onError?: (error: unknown) => void,
): Promise<T | undefined> {
  try {
    return await fn()
  } catch (e) {
    console.error(`Failed to ${label}:`, e)
    onError?.(e)
    return undefined
  }
}

/**
 * 安全执行同步函数，捕获异常并打印到控制台。
 *
 * @example
 * const value = safeCallSync(() => JSON.parse(str), 'parseJSON')
 */
export function safeCallSync<T>(
  fn: () => T,
  label: string,
  onError?: (error: unknown) => void,
): T | undefined {
  try {
    return fn()
  } catch (e) {
    console.error(`Failed to ${label}:`, e)
    onError?.(e)
    return undefined
  }
}

/**
 * 判断错误是否为用户主动取消下载导致的错误
 *
 * 后端在 `download_cancel_flag` 触发时返回 `下载已取消` 或包含该字样的错误
 * （如 `下载整合包失败: 下载已取消`），此类错误不应弹错误窗，仅 toast 提示即可。
 *
 * @example
 * try {
 *   await installModpack(...)
 * } catch (e) {
 *   const msg = e instanceof Error ? e.message : String(e)
 *   if (isCancelledError(msg)) {
 *     toastInfo('下载已取消')
 *     versionStore.finishDownload()
 *   } else {
 *     showModal({ type: 'error', ... })
 *   }
 * }
 */
export function isCancelledError(error: unknown): boolean {
  const msg = error instanceof Error ? error.message : String(error)
  return msg.includes('下载已取消') || msg.includes('下载被取消')
}
