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

const modalRef = ref<any>(null)

export function setModalRef(ref: any) {
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
