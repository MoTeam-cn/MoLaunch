<script setup lang="ts">
/**
 * 浮动下载面板
 * 右下角圆形按钮 + 展开的下载详情面板
 */

import { ref } from 'vue'
import { useVersionStore } from '@/stores/version'
import { ArrowDownTrayIcon, XMarkIcon } from '@heroicons/vue/24/outline'
import { formatBytes, formatSpeed } from '@/utils/format'

const versionStore = useVersionStore()
const expanded = ref(false)

function toggle() {
  expanded.value = !expanded.value
}
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
    <!-- 有下载任务时显示 -->
    <div v-if="versionStore.downloading" class="fixed bottom-6 right-6 z-50">
      <!-- 展开的面板 -->
      <transition
        enter-active-class="transition ease-out duration-200"
        enter-from-class="opacity-0 translate-y-2"
        enter-to-class="opacity-100 translate-y-0"
        leave-active-class="transition ease-in duration-150"
        leave-from-class="opacity-100 translate-y-0"
        leave-to-class="opacity-0 translate-y-2"
      >
        <div
          v-if="expanded"
          class="mb-3 w-72 bg-white rounded-xl shadow-xl border border-gray-200 overflow-hidden"
        >
          <!-- 标题 -->
          <div class="flex items-center justify-between px-4 py-3 border-b border-gray-100">
            <span class="text-sm font-semibold text-gray-900">下载任务</span>
            <button class="p-0.5 hover:bg-gray-100 rounded transition-colors" @click="expanded = false">
              <XMarkIcon class="w-4 h-4 text-gray-400" />
            </button>
          </div>

          <!-- 任务内容 -->
          <div class="p-4">
            <div class="flex items-center gap-3 mb-3">
              <div class="animate-spin rounded-full h-5 w-5 border-2 border-gray-200 border-t-primary-600"></div>
              <div class="flex-1 min-w-0">
                <p class="text-sm font-medium text-gray-900 truncate">
                  {{ versionStore.downloadingVersion }}
                </p>
                <p class="text-xs text-gray-500 mt-0.5">
                  {{ versionStore.downloadProgress?.stage || '准备中...' }}
                </p>
              </div>
            </div>

            <!-- 进度条 -->
            <div class="mb-3">
              <div v-if="(versionStore.downloadProgress?.percentage || 0) > 0" class="w-full bg-gray-100 rounded-full overflow-hidden" style="height: 6px">
                <div
                  class="h-full bg-primary-500 rounded-full transition-all duration-300"
                  :style="{ width: `${versionStore.downloadProgress.percentage}%` }"
                ></div>
              </div>
              <div v-else class="w-full bg-gray-100 rounded-full overflow-hidden" style="height: 6px">
                <div class="h-full bg-primary-400 rounded-full animate-sweep" style="width: 30%"></div>
              </div>
            </div>

            <!-- 详情 -->
            <div class="flex items-center justify-between text-xs text-gray-500">
              <span>
                <template v-if="versionStore.downloadProgress?.bytesTotal && versionStore.downloadProgress.bytesTotal > 0">
                  {{ formatBytes(versionStore.downloadProgress.bytesDownloaded) }} / {{ formatBytes(versionStore.downloadProgress.bytesTotal) }}
                </template>
                <template v-else-if="versionStore.downloadProgress?.total && versionStore.downloadProgress.total > 0">
                  {{ versionStore.downloadProgress.current }}/{{ versionStore.downloadProgress.total }} 文件
                </template>
                <template v-else>
                  正在处理...
                </template>
              </span>
              <span v-if="versionStore.downloadProgress?.speed && versionStore.downloadProgress.speed > 0">
                {{ formatSpeed(versionStore.downloadProgress.speed) }}
              </span>
            </div>
          </div>
        </div>
      </transition>

      <!-- 圆形按钮 -->
      <button
        class="w-12 h-12 bg-primary-600 rounded-full shadow-lg flex items-center justify-center hover:bg-primary-700 transition-colors relative"
        @click="toggle"
      >
        <ArrowDownTrayIcon class="w-5 h-5 text-white" />
        <!-- 旋转光环 -->
        <svg class="absolute inset-0 w-full h-full -rotate-90" viewBox="0 0 48 48">
          <circle
            cx="24" cy="24" r="22"
            fill="none"
            stroke="rgba(255,255,255,0.3)"
            stroke-width="2"
          />
          <circle
            cx="24" cy="24" r="22"
            fill="none"
            stroke="white"
            stroke-width="2"
            stroke-linecap="round"
            :stroke-dasharray="138.2"
            :stroke-dashoffset="138.2 - (138.2 * (versionStore.downloadProgress?.percentage || 0) / 100)"
            class="transition-all duration-300"
          />
        </svg>
      </button>
    </div>
  </transition>
</template>

<style scoped>
@keyframes sweep {
  0% { transform: translateX(-100%); }
  50% { transform: translateX(233%); }
  100% { transform: translateX(-100%); }
}
.animate-sweep {
  animation: sweep 1.5s ease-in-out infinite;
}
</style>
