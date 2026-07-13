<script setup lang="ts">
/**
 * 弹窗组件
 * 支持四种类型：error / warning / info / success
 * 支持确认对话框（showCancel）
 * 支持输入框模式（showInput，替代 window.prompt）
 */

import { ref, computed, nextTick } from 'vue'
import {
  ExclamationTriangleIcon,
  XCircleIcon,
  InformationCircleIcon,
  CheckCircleIcon,
} from '@heroicons/vue/24/outline'

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
  /** 输入框模式：显示输入框，确认时回调接收输入值 */
  showInput?: boolean
  inputValue?: string
  inputPlaceholder?: string
  onConfirmInput?: (value: string) => void
}

const visible = ref(false)
const options = ref<ModalOptions>({
  type: 'info',
  title: '',
  message: '',
})
const inputValue = ref('')
const inputRef = ref<HTMLInputElement | null>(null)

const icon = computed(() => {
  switch (options.value.type) {
    case 'error': return XCircleIcon
    case 'warning': return ExclamationTriangleIcon
    case 'success': return CheckCircleIcon
    default: return InformationCircleIcon
  }
})

const iconColor = computed(() => {
  switch (options.value.type) {
    case 'error': return 'text-red-500'
    case 'warning': return 'text-amber-500'
    case 'success': return 'text-green-500'
    default: return 'text-primary-500'
  }
})

const showDetails = ref(false)

function show(opts: ModalOptions) {
  options.value = opts
  showDetails.value = false
  inputValue.value = opts.inputValue ?? ''
  visible.value = true
  // 输入框模式下自动聚焦
  if (opts.showInput) {
    nextTick(() => {
      inputRef.value?.focus()
      inputRef.value?.select()
    })
  }
}

function handleConfirm() {
  visible.value = false
  if (options.value.showInput) {
    options.value.onConfirmInput?.(inputValue.value)
  } else {
    options.value.onConfirm?.()
  }
}

function handleCancel() {
  visible.value = false
  options.value.onCancel?.()
}

function copyDetails() {
  if (options.value.details) {
    navigator.clipboard.writeText(options.value.details)
  }
}

defineExpose({
  show,
  error: (title: string, message: string, details?: string) => {
    show({ type: 'error', title, message, details })
  },
  warning: (title: string, message: string, details?: string) => {
    show({ type: 'warning', title, message, details })
  },
  info: (title: string, message: string, details?: string) => {
    show({ type: 'info', title, message, details })
  },
  success: (title: string, message: string, details?: string) => {
    show({ type: 'success', title, message, details })
  },
  confirm: (title: string, message: string, onConfirm: () => void, onCancel?: () => void) => {
    show({
      type: 'warning',
      title,
      message,
      showCancel: true,
      onConfirm,
      onCancel,
    })
  },
  prompt: (
    title: string,
    message: string,
    onConfirm: (value: string) => void,
    opts?: { defaultValue?: string; placeholder?: string; onCancel?: () => void },
  ) => {
    show({
      type: 'info',
      title,
      message,
      showInput: true,
      showCancel: true,
      inputValue: opts?.defaultValue ?? '',
      inputPlaceholder: opts?.placeholder,
      onConfirmInput: onConfirm,
      onCancel: opts?.onCancel,
    })
  },
})
</script>

<template>
  <teleport to="body">
    <transition
      enter-active-class="transition ease-out duration-150"
      enter-from-class="opacity-0"
      enter-to-class="opacity-100"
      leave-active-class="transition ease-in duration-100"
      leave-from-class="opacity-100"
      leave-to-class="opacity-0"
    >
      <div
        v-if="visible"
        class="fixed inset-0 z-[9999] flex items-center justify-center p-4"
        @click.self="handleCancel"
      >
        <div class="absolute inset-0 bg-black/40" />

        <div class="relative w-full max-w-sm bg-white rounded-lg shadow-xl">
          <!-- 内容 -->
          <div class="p-5">
            <!-- 标题行 -->
            <div class="flex items-center gap-3">
              <component :is="icon" class="w-5 h-5 shrink-0" :class="iconColor" />
              <h3 class="text-sm font-semibold text-gray-900">{{ options.title }}</h3>
            </div>
            <!-- 消息 -->
            <p class="mt-2 ml-8 text-sm text-gray-600 leading-relaxed">{{ options.message }}</p>

            <!-- 输入框 -->
            <div v-if="options.showInput" class="mt-3 ml-8">
              <input
                ref="inputRef"
                v-model="inputValue"
                type="text"
                class="w-full rounded-lg border border-gray-300 px-3 py-2 text-sm text-gray-900 outline-none transition-colors focus:border-primary-500 focus:ring-1 focus:ring-primary-500"
                :placeholder="options.inputPlaceholder"
                @keyup.enter="handleConfirm"
              >
            </div>

            <!-- 详情 -->
            <div v-if="options.details" class="mt-3 ml-8">
              <button
                class="text-xs text-gray-400 hover:text-gray-600 transition-colors"
                @click="showDetails = !showDetails"
              >
                {{ showDetails ? '收起' : '查看详情' }}
              </button>
              <div v-if="showDetails" class="mt-2 p-2.5 bg-gray-50 rounded text-xs text-gray-600 font-mono break-all max-h-32 overflow-y-auto">
                {{ options.details }}
                <button
                  class="mt-1.5 text-xs text-primary-600 hover:text-primary-700 transition-colors"
                  @click="copyDetails"
                >
                  复制
                </button>
              </div>
            </div>
          </div>

          <!-- 按钮栏 -->
          <div class="flex justify-end gap-2 px-5 py-3 bg-gray-50 rounded-b-lg">
            <button
              v-if="options.showCancel"
              class="px-4 py-1.5 text-sm text-gray-600 hover:bg-gray-200 rounded transition-colors"
              @click="handleCancel"
            >
              {{ options.cancelText || '取消' }}
            </button>
            <button
              class="px-4 py-1.5 text-sm text-white bg-primary-600 hover:bg-primary-700 rounded transition-colors"
              @click="handleConfirm"
            >
              {{ options.confirmText || '确定' }}
            </button>
          </div>
        </div>
      </div>
    </transition>
  </teleport>
</template>
