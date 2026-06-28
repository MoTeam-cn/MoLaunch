<script setup lang="ts">
/**
 * 返回顶部按钮组件
 * 蓝色主题 + 涟漪动画 + 平滑出现
 */

import { ref, onMounted, onUnmounted } from 'vue'
import { ArrowUpIcon } from '@heroicons/vue/24/solid'

const visible = ref(false)
const isHover = ref(false)
let activeEl: Element | null = null

function onScroll(e: Event) {
  const el = e.target as Element
  if (!el || !('scrollTop' in el) || !('scrollHeight' in el)) return

  const scrollHeight = el.scrollHeight
  const clientHeight = el.clientHeight
  const scrollTop = el.scrollTop

  // 内容不够长，不显示
  if (scrollHeight <= clientHeight * 1.5 || scrollHeight < 600) {
    visible.value = false
    return
  }

  // 滚动超过内容的 1/3 才显示
  const scrollRatio = scrollTop / (scrollHeight - clientHeight)
  visible.value = scrollRatio > 0.33
  activeEl = el
}

function scrollToTop() {
  const el = activeEl || document.documentElement
  el.scrollTo({
    top: 0,
    behavior: 'smooth',
  })
}

onMounted(() => {
  document.addEventListener('scroll', onScroll, { capture: true, passive: true })
})

onUnmounted(() => {
  document.removeEventListener('scroll', onScroll, { capture: true })
})
</script>

<template>
  <Transition name="back-to-top">
    <button
      v-if="visible"
      class="back-to-top-btn"
      title="返回顶部"
      @click="scrollToTop"
      @mouseenter="isHover = true"
      @mouseleave="isHover = false"
    >
      <!-- 涟漪背景 -->
      <span class="ripple-bg" :class="{ 'ripple-active': isHover }"></span>
      
      <!-- 图标 -->
      <ArrowUpIcon class="w-5 h-5 text-white relative z-10 transition-transform duration-300" :class="{ '-translate-y-0.5': isHover }" />
      
      <!-- 外圈光晕 -->
      <span class="glow-ring" :class="{ 'glow-active': isHover }"></span>
    </button>
  </Transition>
</template>

<style scoped>
.back-to-top-btn {
  position: fixed;
  bottom: 24px;
  right: 24px;
  z-index: 50;
  width: 44px;
  height: 44px;
  border-radius: 50%;
  background: linear-gradient(135deg, #3b82f6 0%, #2563eb 100%);
  border: none;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow: 
    0 4px 14px rgba(37, 99, 235, 0.4),
    0 2px 6px rgba(37, 99, 235, 0.2);
  transition: all 0.3s cubic-bezier(0.34, 1.56, 0.64, 1);
  overflow: hidden;
}

.back-to-top-btn:hover {
  transform: translateY(-2px) scale(1.05);
  box-shadow: 
    0 6px 20px rgba(37, 99, 235, 0.5),
    0 4px 10px rgba(37, 99, 235, 0.3);
}

.back-to-top-btn:active {
  transform: translateY(0) scale(0.95);
}

/* 涟漪背景 */
.ripple-bg {
  position: absolute;
  inset: 0;
  border-radius: 50%;
  background: radial-gradient(circle at center, rgba(255,255,255,0.3) 0%, transparent 70%);
  transform: scale(0);
  opacity: 0;
  transition: all 0.6s ease-out;
}

.ripple-active {
  transform: scale(2.5);
  opacity: 1;
}

/* 外圈光晕 */
.glow-ring {
  position: absolute;
  inset: -4px;
  border-radius: 50%;
  border: 2px solid rgba(59, 130, 246, 0);
  transition: all 0.4s ease-out;
}

.glow-active {
  inset: -8px;
  border-color: rgba(59, 130, 246, 0.3);
}

/* 进入/离开动画 */
.back-to-top-enter-active {
  animation: slide-in 0.5s cubic-bezier(0.34, 1.56, 0.64, 1);
}

.back-to-top-leave-active {
  animation: slide-out 0.3s ease-in forwards;
}

@keyframes slide-in {
  0% {
    opacity: 0;
    transform: translateY(20px) scale(0.8);
  }
  60% {
    opacity: 1;
    transform: translateY(-4px) scale(1.02);
  }
  100% {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
}

@keyframes slide-out {
  to {
    opacity: 0;
    transform: translateY(10px) scale(0.9);
  }
}
</style>
