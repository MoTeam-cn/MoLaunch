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
import { ArrowDownTrayIcon } from '@heroicons/vue/24/solid'
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
    enter-active-class="transition ease-out duration-250"
    enter-from-class="opacity-0 translate-y-2"
    enter-to-class="opacity-100 translate-y-0"
    leave-active-class="transition ease-in duration-200"
    leave-from-class="opacity-100 translate-y-0"
    leave-to-class="opacity-0 translate-y-2"
  >
    <!-- 有下载任务时显示（下载开始/结束由后端 download-panel-state 事件驱动） -->
    <!-- 视觉与 BackToTop 完全统一：44px 纯色圆钮 + solid 白图标 + 相同阴影/动效 -->
    <button
      v-if="versionStore.downloading"
      :class="['download-panel-btn', positionClass]"
      @click="goToDownloads"
    >
      <ArrowDownTrayIcon class="w-5 h-5 text-white" />

      <!-- 进度环（仅下载按钮独有，白描边圆环） -->
      <svg class="panel-ring" viewBox="0 0 44 44">
        <circle
          cx="22" cy="22" r="19"
          fill="none"
          stroke="rgba(255,255,255,0.3)"
          stroke-width="2"
        />
        <circle
          cx="22" cy="22" r="19"
          fill="none"
          stroke="white"
          stroke-width="2"
          stroke-linecap="round"
          stroke-dasharray="119.38"
          :stroke-dashoffset="119.38 - (119.38 * (versionStore.downloadProgress?.percentage || 0) / 100)"
          class="transition-all duration-300"
        />
      </svg>
    </button>
  </transition>
</template>

<style scoped>
.download-panel-btn {
  position: fixed;
  right: 24px;
  z-index: 10001;
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
  transition: background-color 0.2s ease, transform 0.2s ease, box-shadow 0.2s ease, bottom 0.2s ease;
}

.download-panel-btn:hover {
  background: var(--color-primary-700);
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgb(var(--color-primary-rgb-600) / 0.4);
}

.download-panel-btn:active {
  transform: translateY(0) scale(0.95);
}

.panel-ring {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  transform: rotate(-90deg);
  pointer-events: none;
}
</style>
