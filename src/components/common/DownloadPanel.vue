<script setup lang="ts">
/**
 * 浮动下载按钮
 * 右下角圆形按钮 + 进度环，点击进入下载管理页面
 *
 * 位置协调：BackToTop 可见时上移（bottom-24=96px）腾出空间，
 * 不可见时贴底（bottom-6=24px），避免预留空位。
 */

import { computed } from 'vue'
import { useRouter } from 'vue-router'
import { useVersionStore } from '@/stores/version'
import { ArrowDownTrayIcon } from '@heroicons/vue/24/outline'
import { backToTopVisible } from '@/composables/useFloatingButtonState'

const router = useRouter()
const versionStore = useVersionStore()

/** BackToTop 可见时上移避让，不可见时贴底 */
const positionClass = computed(() =>
  backToTopVisible.value ? 'bottom-24' : 'bottom-6',
)

function goToDownloads() {
  router.push('/apps/downloads')
}
</script>

<template>
  <transition
    enter-active-class="transition ease-out duration-300"
    enter-from-class="opacity-0 scale-50"
    enter-to-class="opacity-100 scale-100"
    leave-active-class="transition ease-in duration-200"
    leave-from-class="opacity-100 scale-100"
    leave-to-class="opacity-0 scale-50"
  >
    <!-- 有下载任务时显示 -->
    <!-- 保留原生 button：浮动下载按钮（fixed w-14 h-14 rounded-full），
         Button.vue 的 scoped size 类固定 height/padding 无法承载圆形浮动按钮 -->
    <button
      v-if="versionStore.downloading"
      :class="['fixed right-6 z-[10001] w-14 h-14 bg-primary-600 rounded-full shadow-lg flex items-center justify-center hover:bg-primary-700 active:scale-95 transition-all group', positionClass]"
      @click="goToDownloads"
    >
      <ArrowDownTrayIcon class="w-6 h-6 text-white group-hover:scale-110 transition-transform" />
      
      <!-- 旋转光环 -->
      <svg class="absolute inset-0 w-full h-full -rotate-90" viewBox="0 0 56 56">
        <circle
          cx="28" cy="28" r="26"
          fill="none"
          stroke="rgba(255,255,255,0.3)"
          stroke-width="2"
        />
        <circle
          cx="28" cy="28" r="26"
          fill="none"
          stroke="white"
          stroke-width="2"
          stroke-linecap="round"
          :stroke-dasharray="163.36"
          :stroke-dashoffset="163.36 - (163.36 * (versionStore.downloadProgress?.percentage || 0) / 100)"
          class="transition-all duration-300"
        />
      </svg>
    </button>
  </transition>
</template>
