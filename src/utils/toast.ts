/**
 * 全局 Toast 工具
 *
 * 命名约定：使用 `toastSuccess` / `toastError` / `toastWarning` / `toastInfo`，
 * toast 前缀避免与 modal.ts 的 showError/showSuccess 同名冲突。
 *
 * 控制台日志策略：除 success 外的方法均同步打印到 console，方便追踪问题。
 */

interface ToastRef {
  success: (text: string) => void
  error: (text: string) => void
  warning: (text: string) => void
  info: (text: string) => void
}

let toastRef: ToastRef | null = null

function getRef(): ToastRef | null {
  return toastRef
}

export function setToastRef(ref: ToastRef | null) {
  toastRef = ref
}

/** 推荐函数名：toast 前缀，避免与 modal.ts 的 showError/showSuccess 冲突 */
export function toastInfo(text: string) {
  console.info('[Toast Info]', text)
  getRef()?.info(text)
}

export function toastSuccess(text: string) {
  getRef()?.success(text)
}

export function toastError(text: string) {
  console.error('[Toast Error]', text)
  getRef()?.error(text)
}

export function toastWarning(text: string) {
  console.warn('[Toast Warning]', text)
  getRef()?.warning(text)
}
