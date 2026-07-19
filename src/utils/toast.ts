/**
 * 全局 Toast 工具
 *
 * 命名约定（v2）：
 *   - `toastSuccess` / `toastError` / `toastWarning` / `toastInfo` 为推荐函数名
 *   - `showSuccess` / `showError` / `showWarning` / `showInfo` 为兼容别名（与 modal.ts 同名易混淆，不推荐）
 *
 * 当文件同时引入 toast 和 modal 时，必须使用 `toastXxx` 前缀避免命名冲突。
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

/** 兼容别名：与 modal.ts 同名，仅在文件未引入 modal 时可用 */
export const showInfo = toastInfo
export const showSuccess = toastSuccess
export const showError = toastError
export const showWarning = toastWarning
