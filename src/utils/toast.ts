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

function getRef(): ToastRef | null {
  return toastRef
}

export function setToastRef(ref: ToastRef) {
  toastRef = ref
}

export function showInfo(text: string) {
  getRef()?.info(text)
}

export function showSuccess(text: string) {
  getRef()?.success(text)
}

export function showError(text: string) {
  getRef()?.error(text)
}

export function showWarning(text: string) {
  getRef()?.warning(text)
}
