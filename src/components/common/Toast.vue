<script setup lang="ts">
/**
 * Toast 全局提示组件
 * 左下角显示，左侧彩色竖条 + 白色背景 + 图标
 */

import { ref } from 'vue'
import {
  CheckCircleIcon,
  XCircleIcon,
  InformationCircleIcon,
  ExclamationTriangleIcon,
} from '@heroicons/vue/24/solid'

export type ToastType = 'success' | 'error' | 'warning' | 'info'

interface ToastItem {
  id: number
  text: string
  type: ToastType
  hiding: boolean
}

const toasts = ref<ToastItem[]>([])
let nextId = 0
const MAX_COUNT = 20

const typeConfig = {
  success: { accent: 'bg-green-500', icon: CheckCircleIcon, iconColor: 'text-green-500' },
  error: { accent: 'bg-red-500', icon: XCircleIcon, iconColor: 'text-red-500' },
  warning: { accent: 'bg-amber-500', icon: ExclamationTriangleIcon, iconColor: 'text-amber-500' },
  info: { accent: 'bg-blue-500', icon: InformationCircleIcon, iconColor: 'text-blue-500' },
}

function calcDuration(text: string): number {
  const len = Math.min(Math.max(text.length, 5), 23)
  return 800 + len * 180
}

function show(type: ToastType, text: string) {
  const existing = toasts.value.find(t => t.text === text && !t.hiding)
  if (existing) {
    shake(existing)
    return
  }
  if (toasts.value.length >= MAX_COUNT) return

  const id = nextId++
  toasts.value.push({ id, text, type, hiding: false })

  const duration = calcDuration(text)
  setTimeout(() => dismiss(id), duration)
}

function dismiss(id: number) {
  const toast = toasts.value.find(t => t.id === id)
  if (!toast || toast.hiding) return
  toast.hiding = true
  setTimeout(() => {
    toasts.value = toasts.value.filter(t => t.id !== id)
  }, 400)
}

function shake(toast: ToastItem) {
  const idx = toasts.value.findIndex(t => t.id === toast.id)
  if (idx === -1) return
  toast.hiding = true
  setTimeout(() => {
    toast.hiding = false
    const duration = calcDuration(toast.text)
    setTimeout(() => dismiss(toast.id), duration)
  }, 300)
}

defineExpose({
  success: (text: string) => show('success', text),
  error: (text: string) => show('error', text),
  warning: (text: string) => show('warning', text),
  info: (text: string) => show('info', text),
})
</script>

<template>
  <teleport to="body">
    <div class="toast-container">
      <div
        v-for="toast in toasts"
        :key="toast.id"
        class="toast-item"
        :class="toast.hiding ? 'toast-hiding' : 'toast-enter'"
      >
        <!-- 左侧彩色竖条 -->
        <div class="toast-accent" :class="typeConfig[toast.type].accent" />
        <!-- 图标 -->
        <component
          :is="typeConfig[toast.type].icon"
          class="toast-icon"
          :class="typeConfig[toast.type].iconColor"
        />
        <!-- 文字 -->
        <span class="toast-text">{{ toast.text }}</span>
      </div>
    </div>
  </teleport>
</template>

<style scoped>
.toast-container {
  position: fixed;
  left: 0;
  bottom: 20px;
  display: flex;
  flex-direction: column;
  gap: 6px;
  pointer-events: none;
  z-index: 10001;
}

.toast-item {
  display: flex;
  align-items: center;
  height: 32px;
  border-radius: 0 8px 8px 0;
  background: rgba(255, 255, 255, 0.95);
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.1), 0 1px 3px rgba(0, 0, 0, 0.06);
  overflow: hidden;
  opacity: 0;
  transform: translateX(-80px);
}

.toast-accent {
  width: 3px;
  height: 100%;
  flex-shrink: 0;
}

.toast-icon {
  width: 16px;
  height: 16px;
  margin-left: 10px;
  flex-shrink: 0;
}

.toast-text {
  color: #1f2937;
  font-size: 13px;
  font-weight: 500;
  padding: 0 12px 0 8px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  display: block;
  max-width: 320px;
}

/* 进入动画 */
.toast-enter {
  animation: toast-in 0.45s cubic-bezier(0.34, 1.56, 0.64, 1) forwards;
}

@keyframes toast-in {
  0% {
    transform: translateX(-80px);
    opacity: 0;
  }
  50% {
    opacity: 1;
  }
  75% {
    transform: translateX(3px);
  }
  100% {
    transform: translateX(0);
    opacity: 1;
  }
}

/* 退出动画 */
.toast-hiding {
  animation:
    toast-out 0.25s ease-in forwards,
    toast-collapse 0.15s ease-out 0.2s forwards;
}

@keyframes toast-out {
  to {
    transform: translateX(-60px);
    opacity: 0;
  }
}
@keyframes toast-collapse {
  to {
    height: 0;
    margin-bottom: 0;
    opacity: 0;
  }
}
</style>
