<script setup lang="ts">
/**
 * 返回顶部按钮组件
 * 与右下角下载面板同款简约风格：纯色圆钮 + 白色图标，无装饰动画
 *
 * 显示策略：
 * - 仅响应位于主内容区 <main> 内的外部滚动容器，过滤弹窗 / 下拉框等浮层
 * - 标记 `data-inner-scroll` 的内部滚动容器（如 AI 聊天消息列表）不触发，
 *   避免全局右下角按钮遮挡页面内操作（如发送按钮）
 * - 迟滞阈值：未显示时需 scrollTop > 400 才出现；已显示时仅在 scrollTop ≤ 一屏高度时隐藏
 *   （避免在临界点反复闪烁）
 * - 路由切换时重置状态，防止旧按钮残留到新页面
 */

import { ref, watch, onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import { ArrowUpIcon } from '@heroicons/vue/24/solid'
import { backToTopVisible, backToTopEnabled } from '@/composables/useFloatingButtonState'

const visible = ref(false)
let activeEl: Element | null = null
const router = useRouter()

/** 未显示时的出现阈值（像素） */
const SHOW_THRESHOLD = 400

// 同步可见状态到共享 ref，供 DownloadPanel 调整位置
watch(visible, (v) => { backToTopVisible.value = v })

// 页内视图白名单：展开操作页（如 LoaderSelect）时禁用，避免残留按钮遮挡右下角操作
watch(backToTopEnabled, (enabled) => {
  if (!enabled) {
    visible.value = false
    activeEl = null
    return
  }
  // 恢复启用后延迟至过渡结束，重新检测当前滚动容器是否已越过阈值
  setTimeout(() => {
    const main = document.querySelector('main')
    if (!main) return
    const scroller = findScrolledContainer(main)
    if (scroller) onScroll({ target: scroller } as unknown as Event)
  }, 450)
})

function onScroll(e: Event) {
  const el = e.target as Element | null
  if (!el || !(el instanceof Element)) return
  if (!('scrollTop' in el) || !('scrollHeight' in el)) return

  // 页内白名单禁用时不响应滚动
  if (!backToTopEnabled.value) return

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
    >
      <ArrowUpIcon class="w-5 h-5 text-white" />
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
  background: var(--color-primary-600);
  border: none;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow: 0 2px 8px rgb(var(--color-primary-rgb-600) / 0.35);
  transition: background-color 0.2s ease, transform 0.2s ease, box-shadow 0.2s ease;
}

.back-to-top-btn:hover {
  background: var(--color-primary-700);
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgb(var(--color-primary-rgb-600) / 0.4);
}

.back-to-top-btn:active {
  transform: translateY(0) scale(0.95);
}

/* 进入/离开动画（简洁淡入滑入） */
.back-to-top-enter-active {
  animation: slide-in 0.25s ease-out;
}

.back-to-top-leave-active {
  animation: slide-out 0.2s ease-in forwards;
}

@keyframes slide-in {
  from {
    opacity: 0;
    transform: translateY(8px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

@keyframes slide-out {
  to {
    opacity: 0;
    transform: translateY(8px);
  }
}
</style>
