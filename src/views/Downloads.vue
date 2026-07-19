<script setup lang="ts">
/**
 * 下载管理页面
 * 参考 PCL2 设计：左右分栏布局
 * - 左侧：统计面板（DownloadStatsPanel 子组件）
 * - 右侧：任务卡片（TaskGroupCard 子组件，含分组列表与暂停/取消按钮）
 * - 无任务时：空状态（DownloadEmptyState 子组件）
 */

import { computed, ref } from 'vue'
import { useVersionStore } from '@/stores/version'
import { showConfirm } from '@/utils/modal'
import { pauseDownload, resumeDownload, cancelDownload } from '@/utils/tauri'
import DownloadEmptyState from './downloads/DownloadEmptyState.vue'
import DownloadStatsPanel from './downloads/DownloadStatsPanel.vue'
import TaskGroupCard from '@/components/downloads/TaskGroupCard.vue'

const versionStore = useVersionStore()

const hasActiveDownload = computed(() => versionStore.downloading)
const progress = computed(() => versionStore.downloadProgress)
const isPaused = computed(() => progress.value?.isPaused ?? false)

const percentage = computed(() => progress.value?.percentage || 0)
const speed = computed(() => progress.value?.global_speed || 0)
const bytesDownloaded = computed(() => progress.value?.global_bytes_downloaded || 0)
const bytesTotal = computed(() => progress.value?.global_bytes_total || 0)
const stages = computed(() => progress.value?.stages || [])
const currentStageIndex = computed(() => progress.value?.current_stage_index ?? 0)

const filesRemaining = computed(() => {
  if (!progress.value?.stages) return 0
  // 统计所有未完成阶段（loading + waiting）的剩余文件数
  let remaining = 0
  for (const s of progress.value.stages) {
    if (s.status === 'loading') {
      remaining += Math.max(0, (s.files_total || 0) - (s.files_downloaded || 0))
    } else if (s.status === 'waiting') {
      remaining += s.files_total || 0
    }
  }
  return remaining
})

const currentStageName = computed(() => {
  const s = stages.value
  const idx = currentStageIndex.value
  return idx < s.length ? s[idx].name : '准备中...'
})

// 按钮状态
const togglingPause = ref(false)
const cancelling = ref(false)

async function handleTogglePause() {
  if (togglingPause.value) return
  togglingPause.value = true
  try {
    if (isPaused.value) {
      await resumeDownload()
    } else {
      await pauseDownload()
    }
  } catch (e) {
    console.error('Failed to toggle pause:', e)
  } finally {
    togglingPause.value = false
  }
}

function handleCancel() {
  showConfirm(
    '取消下载',
    '确定要取消当前下载任务吗？已下载的文件将保留，但未完成的下载将被中止。',
    async () => {
      if (cancelling.value) return
      cancelling.value = true
      try {
        await cancelDownload()
      } catch (e) {
        console.error('Failed to cancel download:', e)
      } finally {
        cancelling.value = false
      }
    },
  )
}
</script>

<template>
  <div class="h-full flex flex-col">
    <!-- 页面标题 -->
    <div class="px-6 py-4 bg-white border-b border-gray-200 shrink-0">
      <h1 class="text-lg font-semibold text-gray-900">下载管理</h1>
      <p class="text-xs text-gray-500 mt-0.5">查看和管理下载任务</p>
    </div>

    <!-- 内容区域 -->
    <div class="flex-1 overflow-hidden flex">
      <!-- 无下载任务时的空状态 -->
      <DownloadEmptyState v-if="!hasActiveDownload" />

      <!-- 有下载任务时 -->
      <template v-else>
        <!-- 左侧：统计面板 -->
        <DownloadStatsPanel
          :current-stage-name="currentStageName"
          :percentage="percentage"
          :speed="speed"
          :bytes-downloaded="bytesDownloaded"
          :bytes-total="bytesTotal"
          :files-remaining="filesRemaining"
        />

        <!-- 右侧：任务列表 -->
        <div class="flex-1 overflow-y-auto p-6 bg-gray-50">
          <div class="max-w-2xl mx-auto">
            <!-- 任务卡片（含分组列表与暂停/取消按钮） -->
            <TaskGroupCard
              :version-name="versionStore.downloadingVersion"
              :percentage="percentage"
              :is-paused="isPaused"
              :toggling-pause="togglingPause"
              :cancelling="cancelling"
              :stages="stages"
              @toggle-pause="handleTogglePause"
              @cancel="handleCancel"
            />

            <!-- 提示信息 -->
            <div class="mt-4 text-center text-xs text-gray-400">
              {{ isPaused ? '下载已暂停，点击恢复按钮继续' : '下载完成后将自动返回' }}
            </div>
          </div>
        </div>
      </template>
    </div>
  </div>
</template>
