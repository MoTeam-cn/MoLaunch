<script setup lang="ts">
/**
 * 返回顶部按钮组件
 * 只在内容足够长且滚动超过 1/3 时显示
 */

import { ref, onMounted, onUnmounted } from 'vue'
import { ArrowUpIcon } from '@heroicons/vue/24/outline'
import Tooltip from '@/components/common/Tooltip.vue'

const visible = ref(false)
let activeEl: Element | null = null

function onScroll(e: Event) {
  const el = e.target as Element
  if (!el || !('scrollTop' in el) || !('scrollHeight' in el)) return

  const scrollHeight = el.scrollHeight
  const clientHeight = el.clientHeight
  const scrollTop = el.scrollTop

  // 内容不够长，不显示（至少需要超出容器 50%）
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
  <transition
    enter-active-class="transition ease-out duration-200"
    enter-from-class="opacity-0 scale-95"
    enter-to-class="opacity-100 scale-100"
    leave-active-class="transition ease-in duration-150"
    leave-from-class="opacity-100 scale-100"
    leave-to-class="opacity-0 scale-95"
  >
    <Tooltip v-if="visible" text="返回顶部" position="left">
      <button
        class="fixed bottom-6 right-6 z-50 w-10 h-10 bg-white rounded-full shadow-lg border border-gray-200 flex items-center justify-center hover:bg-gray-50 transition-colors"
        @click="scrollToTop"
      >
        <ArrowUpIcon class="w-5 h-5 text-gray-600" />
      </button>
    </Tooltip>
  </transition>
</template>
