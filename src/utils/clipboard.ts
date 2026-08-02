/**
 * 剪贴板复制工具
 *
 * 统一 navigator.clipboard 调用与错误处理：复制成功可选 toast 提示。
 */
import { toastSuccess, toastError } from './toast'

export async function copyToClipboard(text: string, options?: { toast?: boolean }): Promise<boolean> {
  try {
    await navigator.clipboard.writeText(text)
    if (options?.toast) toastSuccess('已复制')
    return true
  } catch {
    if (options?.toast) toastError('复制失败')
    return false
  }
}
