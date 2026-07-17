<script setup lang="ts">
/**
 * 下载进度浮层（粘性底部）
 *
 * - 文件名 + 速度
 * - 进度条（渐变色）
 * - 已下载/总大小 + 百分比
 *
 * 内部 helper：formatSpeed、downloadPercent（仅本组件使用）
 */
import { formatBytes } from '@/utils/format'
import type { CommunityDownloadProgress } from '@/composables/useCommunityDownload'

const props = defineProps<{
  progress: CommunityDownloadProgress
}>()

function formatSpeed(bytesPerSec: number): string {
  if (bytesPerSec >= 1_048_576) return (bytesPerSec / 1_048_576).toFixed(1) + ' MB/s'
  if (bytesPerSec >= 1024) return (bytesPerSec / 1024).toFixed(0) + ' KB/s'
  return bytesPerSec + ' B/s'
}

function downloadPercent(): number {
  if (!props.progress || props.progress.total === 0) return 0
  return Math.min(100, (props.progress.downloaded / props.progress.total) * 100)
}
</script>

<template>
  <div class="sticky bottom-0 left-0 right-0 bg-white border-t border-gray-200 px-4 py-2 shadow-lg">
    <div class="flex items-center justify-between mb-1">
      <span class="text-xs text-gray-600 truncate flex-1">
        {{ progress.fileName }}
      </span>
      <span class="text-xs text-gray-500 ml-2">
        {{ formatSpeed(progress.speed) }}
      </span>
    </div>
    <div class="h-1.5 overflow-hidden rounded-full bg-gray-100">
      <div
        class="h-full rounded-full bg-gradient-to-r from-primary-400 to-primary-600 transition-all duration-300 ease-out"
        :style="{ width: downloadPercent() + '%' }"
      />
    </div>
    <div class="flex items-center justify-between mt-1 text-[11px] text-gray-400">
      <span>{{ formatBytes(progress.downloaded) }} / {{ progress.total ? formatBytes(progress.total) : '未知' }}</span>
      <span>{{ downloadPercent().toFixed(1) }}%</span>
    </div>
  </div>
</template>
