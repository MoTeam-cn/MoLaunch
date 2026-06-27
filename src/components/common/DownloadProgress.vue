<script setup lang="ts">
/**
 * 下载进度组件
 */

import { useVersionStore } from '@/stores/version'

const versionStore = useVersionStore()

function formatSpeed(bytesPerSec: number): string {
  if (bytesPerSec <= 0) return ''
  if (bytesPerSec >= 1024 * 1024) return (bytesPerSec / 1024 / 1024).toFixed(1) + ' MB/s'
  if (bytesPerSec >= 1024) return (bytesPerSec / 1024).toFixed(0) + ' KB/s'
  return bytesPerSec + ' B/s'
}

function formatBytes(bytes: number): string {
  if (bytes <= 0) return '0'
  if (bytes >= 1024 * 1024 * 1024) return (bytes / 1024 / 1024 / 1024).toFixed(1) + ' GB'
  if (bytes >= 1024 * 1024) return (bytes / 1024 / 1024).toFixed(0) + ' MB'
  if (bytes >= 1024) return (bytes / 1024).toFixed(0) + ' KB'
  return bytes + ' B'
}
</script>

<template>
  <transition
    enter-active-class="transition ease-out duration-200"
    enter-from-class="opacity-0 -translate-y-2"
    enter-to-class="opacity-100 translate-y-0"
    leave-active-class="transition ease-in duration-150"
    leave-from-class="opacity-100 translate-y-0"
    leave-to-class="opacity-0 -translate-y-2"
  >
    <div v-if="versionStore.downloading" class="mx-6 mt-4 p-3 bg-white rounded-lg border border-gray-200">
      <!-- 状态行 -->
      <div class="flex items-center gap-3">
        <div class="animate-spin rounded-full h-4 w-4 border-2 border-gray-300 border-t-primary-600"></div>
        <div class="flex-1 min-w-0">
          <div class="flex items-center justify-between">
            <span class="text-sm font-medium text-gray-900">
              正在下载 {{ versionStore.downloadingVersion }}
            </span>
            <span class="text-xs text-gray-500">
              {{ versionStore.downloadProgress?.stage || '准备中...' }}
            </span>
          </div>
        </div>
      </div>
      <!-- 进度条 -->
      <div class="mt-3">
        <div v-if="(versionStore.downloadProgress?.percentage || 0) > 0" class="w-full bg-gray-100 rounded-full overflow-hidden" style="height: 6px">
          <div
            class="h-full bg-primary-500 rounded-full transition-all duration-300"
            :style="{ width: `${versionStore.downloadProgress.percentage}%` }"
          ></div>
        </div>
        <div v-else class="w-full bg-gray-100 rounded-full overflow-hidden" style="height: 6px">
          <div class="h-full bg-primary-400 rounded-full animate-sweep" style="width: 30%"></div>
        </div>
        <div class="flex items-center justify-between mt-2 text-xs text-gray-500">
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
