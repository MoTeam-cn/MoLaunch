/**
 * 全局 Toast 工具
 */

interface ToastRef {
  success: (text: string) => void
  error: (text: string) => void
  warning: (text: string) => void
  info: (text: string) => void
}

let toastRef: ToastRef | null = null

export function setToastRef(ref: ToastRef) {
  toastRef = ref
}

export function showInfo(text: string) {
  toastRef?.info(text)
}

export function showSuccess(text: string) {
  toastRef?.success(text)
}

export function showError(text: string) {
  toastRef?.error(text)
}

export function showWarning(text: string) {
  toastRef?.warning(text)
}
