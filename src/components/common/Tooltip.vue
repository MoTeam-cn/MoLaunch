<script setup lang="ts">
/**
 * 自定义 Tooltip 组件
 * Teleport 到 body，fixed 定位，自动边界检测
 */

import { ref, nextTick, onUnmounted } from 'vue'

interface Props {
  text: string
  position?: 'top' | 'bottom' | 'left' | 'right'
  delay?: number
}

const props = withDefaults(defineProps<Props>(), {
  position: 'top',
  delay: 300,
})

const visible = ref(false)
const triggerRef = ref<HTMLElement | null>(null)
const tipRef = ref<HTMLElement | null>(null)
const tipStyle = ref<Record<string, string>>({})
let timer: ReturnType<typeof setTimeout> | null = null

function show() {
  timer = setTimeout(() => {
    visible.value = true
    nextTick(calcPosition)
  }, props.delay)
}

function hide() {
  if (timer) { clearTimeout(timer); timer = null }
  visible.value = false
}

function calcPosition() {
  if (!triggerRef.value || !tipRef.value) return
  const r = triggerRef.value.getBoundingClientRect()
  const tipW = tipRef.value.offsetWidth
  const tipH = tipRef.value.offsetHeight
  const gap = 6
  const vw = window.innerWidth
  let top = 0
  let left = 0

  if (props.position === 'top') {
    top = r.top - gap - tipH
    left = r.left + r.width / 2 - tipW / 2
  } else if (props.position === 'bottom') {
    top = r.bottom + gap
    left = r.left + r.width / 2 - tipW / 2
  } else if (props.position === 'left') {
    top = r.top + r.height / 2 - tipH / 2
    left = r.left - gap - tipW
  } else {
    top = r.top + r.height / 2 - tipH / 2
    left = r.right + gap
  }

  // 边界修正
  if (left < 8) left = 8
  if (left + tipW > vw - 8) left = vw - tipW - 8
  if (top < 8) top = 8

  tipStyle.value = {
    position: 'fixed',
    top: `${top}px`,
    left: `${left}px`,
    zIndex: '9999',
  }
}

onUnmounted(() => { if (timer) clearTimeout(timer) })
</script>

<template>
  <div ref="triggerRef" class="tooltip-trigger" @mouseenter="show" @mouseleave="hide" @focus="show" @blur="hide">
    <slot />
    <teleport to="body">
      <div v-if="visible" ref="tipRef" :style="tipStyle" class="tooltip-body">
        {{ text }}
      </div>
    </teleport>
  </div>
</template>

<style scoped>
.tooltip-trigger {
  display: inline-flex;
}
</style>

<style>
.tooltip-body {
  padding: 6px 12px;
  background: #1f2937;
  color: #f9fafb;
  font-size: 12px;
  line-height: 1.5;
  border-radius: 6px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.2);
  width: max-content;
  max-width: 360px;
  word-break: break-word;
}
</style>
