<script setup lang="ts">
/**
 * 返回顶部按钮组件
 */

import { ref, onMounted, onUnmounted } from 'vue'
import { ArrowUpIcon } from '@heroicons/vue/24/outline'

interface Props {
  target?: HTMLElement | null
  threshold?: number
}

const props = withDefaults(defineProps<Props>(), {
  target: null,
  threshold: 300,
})

const visible = ref(false)

function checkScroll() {
  const el = props.target || document.documentElement
  visible.value = el.scrollTop > props.threshold
}

function scrollToTop() {
  const el = props.target || document.documentElement
  el.scrollTo({
    top: 0,
    behavior: 'smooth',
  })
}

onMounted(() => {
  const el = props.target || window
  el.addEventListener('scroll', checkScroll, { passive: true })
})

onUnmounted(() => {
  const el = props.target || window
  el.removeEventListener('scroll', checkScroll)
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
    <button
      v-if="visible"
      class="fixed bottom-6 right-6 z-50 w-10 h-10 bg-white dark:bg-gray-800 rounded-full shadow-lg border border-gray-200 dark:border-gray-700 flex items-center justify-center hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors"
      title="返回顶部"
      @click="scrollToTop"
    >
      <ArrowUpIcon class="w-5 h-5 text-gray-600 dark:text-gray-400" />
    </button>
  </transition>
</template>
