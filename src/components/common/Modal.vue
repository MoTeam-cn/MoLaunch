<script setup lang="ts">
/**
 * 自定义弹窗组件 - PCL2 风格
 */

import { ref, computed } from 'vue'
import {
  ExclamationTriangleIcon,
  XCircleIcon,
  InformationCircleIcon,
  CheckCircleIcon,
  XMarkIcon,
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
    case 'error':
      return XCircleIcon
    case 'warning':
      return ExclamationTriangleIcon
    case 'success':
      return CheckCircleIcon
    default:
      return InformationCircleIcon
  }
})

const colors = computed(() => {
  switch (options.value.type) {
    case 'error':
      return {
        bg: 'bg-red-50 dark:bg-red-900/20',
        border: 'border-red-200 dark:border-red-800',
        icon: 'text-red-500',
        title: 'text-red-800 dark:text-red-200',
        button: 'bg-red-600 hover:bg-red-700',
        buttonSecondary: 'bg-red-100 text-red-700 hover:bg-red-200 dark:bg-red-900/50 dark:text-red-400 dark:hover:bg-red-900',
      }
    case 'warning':
      return {
        bg: 'bg-yellow-50 dark:bg-yellow-900/20',
        border: 'border-yellow-200 dark:border-yellow-800',
        icon: 'text-yellow-500',
        title: 'text-yellow-800 dark:text-yellow-200',
        button: 'bg-yellow-600 hover:bg-yellow-700',
        buttonSecondary: 'bg-yellow-100 text-yellow-700 hover:bg-yellow-200 dark:bg-yellow-900/50 dark:text-yellow-400 dark:hover:bg-yellow-900',
      }
    case 'success':
      return {
        bg: 'bg-green-50 dark:bg-green-900/20',
        border: 'border-green-200 dark:border-green-800',
        icon: 'text-green-500',
        title: 'text-green-800 dark:text-green-200',
        button: 'bg-green-600 hover:bg-green-700',
        buttonSecondary: 'bg-green-100 text-green-700 hover:bg-green-200 dark:bg-green-900/50 dark:text-green-400 dark:hover:bg-green-900',
      }
    default:
      return {
        bg: 'bg-blue-50 dark:bg-blue-900/20',
        border: 'border-blue-200 dark:border-blue-800',
        icon: 'text-blue-500',
        title: 'text-blue-800 dark:text-blue-200',
        button: 'bg-blue-600 hover:bg-blue-700',
        buttonSecondary: 'bg-blue-100 text-blue-700 hover:bg-blue-200 dark:bg-blue-900/50 dark:text-blue-400 dark:hover:bg-blue-900',
      }
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

// 暴露方法
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
      enter-active-class="transition ease-out duration-200"
      enter-from-class="opacity-0"
      enter-to-class="opacity-100"
      leave-active-class="transition ease-in duration-150"
      leave-from-class="opacity-100"
      leave-to-class="opacity-0"
    >
      <div
        v-if="visible"
        class="fixed inset-0 z-[9999] flex items-center justify-center p-4"
        @click.self="handleCancel"
      >
        <!-- 背景遮罩 -->
        <div class="absolute inset-0 bg-black/50 backdrop-blur-sm" />

        <!-- 弹窗主体 -->
        <div
          class="relative w-full max-w-md bg-white dark:bg-gray-800 rounded-xl shadow-2xl overflow-hidden"
          :class="colors.border"
          style="border-width: 2px"
        >
          <!-- 背景装饰 -->
          <div class="absolute inset-0 opacity-5 pointer-events-none">
            <svg class="w-full h-full" viewBox="0 0 200 200" fill="none">
              <path d="M100 0L200 100L100 200L0 100Z" stroke="currentColor" stroke-width="2" />
              <path d="M50 50L150 50L150 150L50 150Z" stroke="currentColor" stroke-width="2" />
            </svg>
          </div>

          <!-- 内容 -->
          <div class="relative p-6">
            <!-- 头部 -->
            <div class="flex items-start">
              <div class="flex-shrink-0">
                <component :is="icon" class="w-8 h-8" :class="colors.icon" />
              </div>
              <div class="ml-4 flex-1">
                <h3 class="text-lg font-semibold" :class="colors.title">
                  {{ options.title }}
                </h3>
                <p class="mt-2 text-sm text-gray-600 dark:text-gray-400">
                  {{ options.message }}
                </p>
              </div>
              <button
                class="flex-shrink-0 ml-4 p-1 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
                @click="handleCancel"
              >
                <XMarkIcon class="w-5 h-5 text-gray-400" />
              </button>
            </div>

            <!-- 详情 -->
            <div v-if="options.details" class="mt-4">
              <button
                class="text-xs text-gray-500 hover:text-gray-700 dark:hover:text-gray-300"
                @click="showDetails = !showDetails"
              >
                {{ showDetails ? '隐藏详情' : '显示详情' }}
              </button>
              <transition
                enter-active-class="transition ease-out duration-100"
                enter-from-class="opacity-0 -translate-y-1"
                enter-to-class="opacity-100 translate-y-0"
                leave-active-class="transition ease-in duration-75"
                leave-from-class="opacity-100 translate-y-0"
                leave-to-class="opacity-0 -translate-y-1"
              >
                <div
                  v-if="showDetails"
                  class="mt-2 p-3 bg-gray-100 dark:bg-gray-700 rounded-lg"
                >
                  <pre class="text-xs text-gray-700 dark:text-gray-300 whitespace-pre-wrap break-all">{{ options.details }}</pre>
                  <button
                    class="mt-2 text-xs px-2 py-1 bg-gray-200 dark:bg-gray-600 rounded hover:bg-gray-300 dark:hover:bg-gray-500 transition-colors"
                    @click="copyDetails"
                  >
                    复制
                  </button>
                </div>
              </transition>
            </div>

            <!-- 按钮 -->
            <div class="mt-6 flex justify-end space-x-3">
              <button
                v-if="options.showCancel"
                class="px-4 py-2 text-sm font-medium rounded-lg transition-colors"
                :class="colors.buttonSecondary"
                @click="handleCancel"
              >
                {{ options.cancelText || '取消' }}
              </button>
              <button
                class="px-4 py-2 text-sm font-medium text-white rounded-lg transition-colors"
                :class="colors.button"
                @click="handleConfirm"
              >
                {{ options.confirmText || '确定' }}
              </button>
            </div>
          </div>
        </div>
      </div>
    </transition>
  </teleport>
</template>
