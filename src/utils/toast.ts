/**
 * 全局 Toast 工具
 *
 * 命名约定：使用 `toastSuccess` / `toastError` / `toastWarning` / `toastInfo`，
 * toast 前缀避免与 modal.ts 的 showError/showSuccess 同名冲突。
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

export function setToastRef(ref: ToastRef) {
  toastRef = ref
}

/** 推荐函数名：toast 前缀，避免与 modal.ts 的 showError/showSuccess 冲突 */
export function toastInfo(text: string) {
  getRef()?.info(text)
}

export function toastSuccess(text: string) {
  getRef()?.success(text)
}

export function toastError(text: string) {
  getRef()?.error(text)
}

export function toastWarning(text: string) {
  getRef()?.warning(text)
}
