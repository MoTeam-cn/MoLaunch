/**
 * 全局弹窗服务
 */

import { ref } from 'vue'

export type ModalType = 'error' | 'warning' | 'info' | 'success'

interface ModalOptions {
  type: ModalType
  title: string
  message: string
  /** 富文本消息（经 DOMPurify 消毒后渲染，优先于 message） */
  messageHtml?: string
  details?: string
  confirmText?: string
  showCancel?: boolean
  cancelText?: string
  onConfirm?: () => void
  onCancel?: () => void
  /** 输入框模式 */
  showInput?: boolean
  inputValue?: string
  inputPlaceholder?: string
  onConfirmInput?: (value: string) => void
}

/** MessageDrawer 组件实例对外暴露的接口（与 MessageDrawer.vue defineExpose 对应） */
export interface ModalInstance {
  show: (opts: ModalOptions) => void
  error: (title: string, message: string, details?: string) => void
  warning: (title: string, message: string, details?: string) => void
  info: (title: string, message: string, details?: string) => void
  success: (title: string, message: string, details?: string) => void
  confirm: (
    title: string,
    message: string,
    onConfirm: () => void,
    onCancel?: () => void,
    opts?: { messageHtml?: string },
  ) => void
  prompt: (
    title: string,
    message: string,
    onConfirm: (value: string) => void,
    opts?: { defaultValue?: string; placeholder?: string; onCancel?: () => void },
  ) => void
}

const modalRef = ref<ModalInstance | null>(null)

export function setModalRef(ref: ModalInstance | null) {
  modalRef.value = ref
}

export function showModal(opts: ModalOptions) {
  modalRef.value?.show(opts)
}

export function showError(title: string, message: string, details?: string) {
  modalRef.value?.error(title, message, details)
}

export function showWarning(title: string, message: string, details?: string) {
  modalRef.value?.warning(title, message, details)
}

export function showInfo(title: string, message: string, details?: string) {
  modalRef.value?.info(title, message, details)
}

export function showSuccess(title: string, message: string, details?: string) {
  modalRef.value?.success(title, message, details)
}

export function showConfirm(
  title: string,
  message: string,
  onConfirm: () => void,
  onCancel?: () => void,
  opts?: { messageHtml?: string },
) {
  modalRef.value?.confirm(title, message, onConfirm, onCancel, opts)
}

/**
 * Promise 化确认弹窗
 *
 * `showConfirm` 的回调式签名在 async 函数中容易误用（忘记传回调、误以为返回 Promise）。
 * 此函数包装为 `Promise<boolean>`，适配 `await` 场景。
 *
 * @param title 标题
 * @param message 提示消息
 * @returns true=确认，false=取消
 */
export function showConfirmAsync(
  title: string,
  message: string,
  opts?: { messageHtml?: string },
): Promise<boolean> {
  return new Promise((resolve) => {
    showConfirm(title, message, () => resolve(true), () => resolve(false), opts)
  })
}

/**
 * 输入框弹窗（替代 window.prompt）
 *
 * @param title 标题
 * @param message 提示消息
 * @param onConfirm 确认回调，接收输入值
 * @param opts 可选：defaultValue 默认值、placeholder 占位符、onCancel 取消回调
 */
export function showPrompt(
  title: string,
  message: string,
  onConfirm: (value: string) => void,
  opts?: { defaultValue?: string; placeholder?: string; onCancel?: () => void },
) {
  modalRef.value?.prompt(title, message, onConfirm, opts)
}
