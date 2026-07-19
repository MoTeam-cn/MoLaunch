/**
 * 异步工具函数
 *
 * 提供 `safeCall` 高阶函数，统一处理 try/catch + console.error 样板，
 * 消除项目中 55+ 处重复的错误吞没模式。
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
