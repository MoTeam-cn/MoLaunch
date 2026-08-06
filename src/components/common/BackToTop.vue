<script setup lang="ts">
/**
 * 返回顶部按钮组件
 * 蓝色主题 + 涟漪动画 + 平滑出现
 *
 * 显示策略：
 * - 仅响应位于主内容区 <main> 内的外部滚动容器，过滤弹窗 / 下拉框等浮层
 * - 标记 `data-inner-scroll` 的内部滚动容器（如 AI 聊天消息列表）不触发，
 *   避免全局右下角按钮遮挡页面内操作（如发送按钮）
 * - 迟滞阈值：未显示时需 scrollTop > 700 才出现；已显示时仅在 scrollTop ≤ 一屏高度时隐藏
 *   （避免在临界点反复闪烁）
 * - 路由切换时重置状态，防止旧按钮残留到新页面
 */

import { ref, watch, onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import { ArrowUpIcon } from '@heroicons/vue/24/solid'
import { backToTopVisible } from '@/composables/useFloatingButtonState'

const visible = ref(false)
const isHover = ref(false)
let activeEl: Element | null = null
const router = useRouter()

/** 未显示时的出现阈值（像素） */
const SHOW_THRESHOLD = 400

// 同步可见状态到共享 ref，供 DownloadPanel 调整位置
watch(visible, (v) => { backToTopVisible.value = v })

function onScroll(e: Event) {
  const el = e.target as Element | null
  if (!el || !(el instanceof Element)) return
  if (!('scrollTop' in el) || !('scrollHeight' in el)) return

  // 仅响应主内容区内的外部滚动容器，过滤 Teleport 弹层 / 下拉框等
  if (!el.closest('main')) return

  // 跳过标记为内部滚动的容器（如 AI 聊天消息列表），避免遮挡页面内操作
  if (el.closest('[data-inner-scroll]')) return

  const clientHeight = el.clientHeight
  const scrollTop = el.scrollTop

  if (visible.value) {
    // 已显示：滚动回落到一屏高度以内时才隐藏
    if (scrollTop <= clientHeight) {
      visible.value = false
    }
  } else {
    // 未显示：超过阈值时才出现
    if (scrollTop > SHOW_THRESHOLD) {
      visible.value = true
      activeEl = el
    }
  }
}

function scrollToTop() {
  const el = activeEl && activeEl.isConnected ? activeEl : document.querySelector('main')
  if (!el) return
  el.scrollTo({
    top: 0,
    behavior: 'smooth',
  })
}

/** 路由切换后重置状态，并检查新页面是否已处于滚动位置（如浏览器恢复滚动位置） */
function refreshAfterRouteChange() {
  visible.value = false
  activeEl = null
  // App.vue 使用 fade 过渡（200ms out + 200ms in），延迟至过渡结束后再检测
  setTimeout(() => {
    const main = document.querySelector('main')
    if (!main) return
    const scroller = findScrolledContainer(main)
    if (scroller) onScroll({ target: scroller } as unknown as Event)
  }, 450)
}

/** 在主内容区内查找已发生滚动的外部容器（用于路由切换后恢复按钮显示状态） */
function findScrolledContainer(root: Element): Element | null {
  const candidates = root.querySelectorAll('*')
  for (let i = 0; i < candidates.length; i++) {
    const node = candidates[i]
    if (node.closest('[data-inner-scroll]')) continue
    if (node.scrollTop > 0 && node.clientHeight < node.scrollHeight) {
      return node
    }
  }
  return null
}

onMounted(() => {
  document.addEventListener('scroll', onScroll, { capture: true, passive: true })
  router.afterEach(refreshAfterRouteChange)
})

onUnmounted(() => {
  document.removeEventListener('scroll', onScroll, { capture: true })
})
</script>

<template>
  <Transition name="back-to-top">
    <!-- 保留原生 button：回到顶部按钮（back-to-top-btn 自定义样式 + fixed 定位），
         Button.vue 的 scoped size 类与布局不适合浮动定位按钮 -->
    <button
      v-if="visible"
      class="back-to-top-btn"
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
  background: linear-gradient(135deg, var(--color-primary-500) 0%, var(--color-primary-600) 100%);
  border: none;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow:
    0 4px 14px rgb(var(--color-primary-rgb-600) / 0.4),
    0 2px 6px rgb(var(--color-primary-rgb-600) / 0.2);
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
