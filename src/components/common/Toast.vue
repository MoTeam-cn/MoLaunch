<script setup lang="ts">
/**
 * Toast 全局提示组件
 * 左下角显示，左侧彩色竖条 + 白色背景 + 图标
 * 长消息默认单行省略，点击 toast 展开换行显示完整内容（高度受限，超出省略）
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
  expanded: boolean
  /** 自动关闭定时器句柄（悬停/展开时清空暂停，移出/收起时重建） */
  timer?: number
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
  const toast: ToastItem = { id, text, type, hiding: false, expanded: false }
  toasts.value.push(toast)

  const duration = calcDuration(text)
  toast.timer = window.setTimeout(() => dismiss(id), duration)
}

/** 点击切换展开/收起（长消息可查看完整内容）；展开后暂停自动关闭，收起后恢复 */
function toggleExpand(id: number) {
  const toast = toasts.value.find(t => t.id === id)
  if (!toast) return
  toast.expanded = !toast.expanded
  if (toast.expanded) {
    pauseAutoDismiss(id)
  } else {
    resumeAutoDismiss(id)
  }
}

/** 鼠标悬停（或展开）时暂停自动关闭 */
function pauseAutoDismiss(id: number) {
  const toast = toasts.value.find(t => t.id === id)
  if (!toast || toast.hiding) return
  if (toast.timer !== undefined) {
    window.clearTimeout(toast.timer)
    toast.timer = undefined
  }
}

/** 鼠标移出（或收起）后恢复自动关闭：固定 2s 后关闭 */
function resumeAutoDismiss(id: number) {
  const toast = toasts.value.find(t => t.id === id)
  if (!toast || toast.hiding) return
  if (toast.timer !== undefined) return
  toast.timer = window.setTimeout(() => dismiss(id), 2000)
}

function dismiss(id: number) {
  const toast = toasts.value.find(t => t.id === id)
  if (!toast || toast.hiding) return
  if (toast.timer !== undefined) {
    window.clearTimeout(toast.timer)
    toast.timer = undefined
  }
  toast.hiding = true
  setTimeout(() => {
    toasts.value = toasts.value.filter(t => t.id !== id)
  }, 400)
}

function shake(toast: ToastItem) {
  const idx = toasts.value.findIndex(t => t.id === toast.id)
  if (idx === -1) return
  if (toast.timer !== undefined) {
    window.clearTimeout(toast.timer)
    toast.timer = undefined
  }
  toast.hiding = true
  setTimeout(() => {
    toast.hiding = false
    const duration = calcDuration(toast.text)
    toast.timer = window.setTimeout(() => dismiss(toast.id), duration)
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
        :class="[
          toast.hiding ? 'toast-hiding' : 'toast-enter',
          toast.expanded ? 'toast-expanded' : '',
        ]"
        @click="toggleExpand(toast.id)"
        @mouseenter="pauseAutoDismiss(toast.id)"
        @mouseleave="resumeAutoDismiss(toast.id)"
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
  position: relative;
  display: flex;
  align-items: center;
  height: 32px;
  border-radius: 0 8px 8px 0;
  background: rgba(255, 255, 255, 0.95);
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.1), 0 1px 3px rgba(0, 0, 0, 0.06);
  overflow: hidden;
  /* 基类必须是可见的结束态：进入动画的 0% 关键帧已提供初始隐藏态
     （translateX(-80px)+opacity:0），若基类设为隐藏态，退出时 toast-enter 的
     forwards 填充被移除会让元素瞬间回落成不可见，导致滑出动画完全看不到 */
  opacity: 1;
  cursor: pointer;
  /* 容器设了 pointer-events:none 防止空白区挡点击，toast 项自身必须恢复，
     否则 click / mouseenter 事件全部穿透无法触发 */
  pointer-events: auto;
}

/* 展开态：允许换行显示完整内容，高度受限（约 4 行） */
.toast-item.toast-expanded {
  align-items: flex-start;
  height: auto;
  max-height: 104px;
  padding-top: 6px;
  padding-bottom: 6px;
  overflow-y: auto;
}

.toast-expanded .toast-text {
  white-space: normal;
  word-break: break-all;
  max-width: 340px;
  max-height: 92px;
  overflow-y: auto;
}

.toast-accent {
  position: absolute;
  left: 0;
  top: 0;
  bottom: 0;
  width: 3px;
  flex-shrink: 0;
}

.toast-icon {
  width: 16px;
  height: 16px;
  margin-left: 13px;
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
    toast-out 0.3s ease-in forwards,
    toast-collapse 0.15s ease-out 0.25s forwards;
}

/* 从右往左滑回去（与进入方向相反），并淡出 */
@keyframes toast-out {
  to {
    transform: translateX(-100%);
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
