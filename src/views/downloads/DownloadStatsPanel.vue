<script setup lang="ts">
/**
 * 下载管理页左侧统计面板
 * 所有统计数据通过 props 接收（父组件已 computed）
 */
import { formatBytes, formatSpeed } from '@/utils/format'

defineProps<{
  currentStageName: string
  percentage: number
  speed: number
  bytesDownloaded: number
  bytesTotal: number
  filesRemaining: number
}>()
</script>

<template>
  <div class="w-56 border-r border-gray-200 bg-white flex flex-col shrink-0">
    <div class="flex-1 px-5 py-6 space-y-5">
      <!-- 当前阶段 -->
      <div>
        <div class="text-xs text-gray-500 mb-1">当前阶段</div>
        <div class="text-base font-semibold text-gray-900">{{ currentStageName }}</div>
      </div>

      <!-- 分割线 -->
      <div class="h-px bg-gradient-to-r from-gray-100 to-gray-200"></div>

      <!-- 总进度（加权平均） -->
      <div>
        <div class="text-xs text-gray-500 mb-1">总进度</div>
        <div class="text-2xl font-bold text-primary-600">
          {{ percentage.toFixed(1) }}%
        </div>
      </div>

      <!-- 分割线 -->
      <div class="h-px bg-gradient-to-r from-gray-100 to-gray-200"></div>

      <!-- 下载速度 -->
      <div>
        <div class="text-xs text-gray-500 mb-1">下载速度</div>
        <div class="text-lg font-semibold text-gray-900">
          {{ speed > 0 ? formatSpeed(speed) : '计算中...' }}
        </div>
      </div>

      <!-- 分割线 -->
      <div class="h-px bg-gradient-to-r from-gray-100 to-gray-200"></div>

      <!-- 已下载 / 总大小（累计） -->
      <div>
        <div class="text-xs text-gray-500 mb-1">已下载 / 总大小（累计）</div>
        <div class="text-sm font-medium text-gray-900">
          {{ formatBytes(bytesDownloaded) }}
        </div>
        <div class="text-xs text-gray-400">
          {{ bytesTotal > 0 ? formatBytes(bytesTotal) : '计算中...' }}
        </div>
      </div>

      <!-- 分割线 -->
      <div class="h-px bg-gradient-to-r from-gray-100 to-gray-200"></div>

      <!-- 剩余文件 -->
      <div>
        <div class="text-xs text-gray-500 mb-1">剩余文件</div>
        <div class="text-lg font-semibold text-gray-900">{{ filesRemaining }}</div>
      </div>
    </div>
  </div>
</template>
