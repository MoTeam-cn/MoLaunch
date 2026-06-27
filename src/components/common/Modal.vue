<script setup lang="ts">
/**
 * 弹窗组件
 */

import { ref, computed } from 'vue'
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
}

const visible = ref(false)
const options = ref<ModalOptions>({
  type: 'info',
  title: '',
  message: '',
})

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
  visible.value = true
}

function handleConfirm() {
  visible.value = false
  options.value.onConfirm?.()
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
