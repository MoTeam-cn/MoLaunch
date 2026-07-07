/**
 * 全局弹窗服务
 */

import { ref } from 'vue'

export type ModalType = 'error' | 'warning' | 'info' | 'success'

interface ModalOptions {
  type: ModalType
  title: string
  message: string
  details?: string
  confirmText?: string
  showCancel?: boolean
  cancelText?: string
  onConfirm?: () => void
  onCancel?: () => void
}

/** Modal 组件实例对外暴露的接口（与 Modal.vue defineExpose 对应） */
export interface ModalInstance {
  show: (opts: ModalOptions) => void
  error: (title: string, message: string, details?: string) => void
  warning: (title: string, message: string, details?: string) => void
  info: (title: string, message: string, details?: string) => void
  success: (title: string, message: string, details?: string) => void
  confirm: (title: string, message: string, onConfirm: () => void, onCancel?: () => void) => void
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

export function showConfirm(title: string, message: string, onConfirm: () => void, onCancel?: () => void) {
  modalRef.value?.confirm(title, message, onConfirm, onCancel)
}
