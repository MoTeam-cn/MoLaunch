<script setup lang="ts">
/**
 * 下载管理页面
 * 左右分栏布局
 * - 左侧：统计面板（DownloadStatsPanel 子组件）
 * - 右侧：任务卡片（TaskGroupCard 子组件，含分组列表与暂停/取消按钮）
 * - 无任务时：空状态（DownloadEmptyState 子组件）
 */

import { computed, onMounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { useVersionStore } from '@/stores/version'
import { showConfirm } from '@/utils/modal'
import { pauseDownload, resumeDownload, cancelDownload, getDownloadProgress, isDownloading } from '@/utils/tauri'
import type { RawDownloadProgress } from '@/types/download'
import DownloadStatsPanel from './downloads/DownloadStatsPanel.vue'
import TaskGroupCard from '@/components/downloads/TaskGroupCard.vue'
import { initDownloadPolling } from '@/composables/useDownloadPolling'

const versionStore = useVersionStore()
const router = useRouter()

// 检查中状态：3 秒重试期间为 true，避免页面空白
const checking = ref(true)

// 进入页面时恢复下载状态
// 首次进入时延迟重试检查（给后端异步启动下载任务的时间，避免双击下载按钮进入页面就被赶回去）
onMounted(async () => {
  initDownloadPolling()
  const maxRetries = 6 // 最多重试 6 次，每次 500ms，共 3 秒
  for (let attempt = 0; attempt < maxRetries; attempt++) {
    try {
      const active = await isDownloading()
      if (active) {
        // 有下载任务，恢复状态
        const raw = await getDownloadProgress()
        if (raw && raw.stages && raw.stages.length > 0) {
          versionStore.startDownload(raw.version_name || '')
          let weightedProgress = 0
          let totalWeight = 0
          for (const s of raw.stages) {
            totalWeight += s.weight
            weightedProgress += s.progress * s.weight
          }
          const percentage = totalWeight > 0
            ? Math.min(100, parseFloat(((weightedProgress / totalWeight) * 100).toFixed(1)))
            : 0
          const isPaused = raw.stages.some((s) => s.is_paused === true)
          versionStore.updateProgress({
            stages: raw.stages.map((s) => ({
              name: s.name,
              progress: s.progress,
              weight: s.weight,
              status: s.status,
              bytes_downloaded: s.bytes_downloaded,
              bytes_total: s.bytes_total,
              files_downloaded: s.files_downloaded || 0,
              files_total: s.files_total || 0,
              group: s.group ?? null,
            })),
            current_stage_index: raw.current_stage_index ?? 0,
            global_speed: raw.global_speed ?? 0,
            global_bytes_downloaded: raw.global_bytes_downloaded ?? 0,
            global_bytes_total: raw.global_bytes_total ?? 0,
            percentage,
            isPaused,
          })
        }
        checking.value = false
        return // 成功恢复，不需要返回
      }
    } catch (e) {
      console.error('Failed to check download state:', e)
    }
    // 等待 500ms 后重试
    if (attempt < maxRetries - 1) {
      await new Promise(resolve => setTimeout(resolve, 500))
    }
  }
  // 重试完毕仍无任务，显示暂无任务极简画面，1.5 秒后返回上一页
  checking.value = false
  await new Promise(resolve => setTimeout(resolve, 1500))
  router.back()
})

const hasActiveDownload = computed(() => versionStore.downloading)

// 下载完成/取消后自动返回上一页（hasActiveDownload 从 true 变为 false 时触发）
watch(hasActiveDownload, (active, wasActive) => {
  if (!active && wasActive) {
    router.back()
  }
})
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
        // 确保取消后立即触发返回（避免轮询已停止时卡在空白页）
        versionStore.finishDownload()
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
      <!-- 有下载任务时 -->
      <template v-if="hasActiveDownload">
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

      <!-- 无任务时：极简占位画面（检查中 / 暂无任务） -->
      <div
        v-else
        class="flex-1 flex flex-col items-center justify-center bg-gray-50"
      >
        <!-- 检查中：旋转加载圈 -->
        <div v-if="checking" class="flex flex-col items-center gap-3">
          <svg class="h-7 w-7 animate-spin text-primary-400" viewBox="0 0 24 24" fill="none">
            <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="3" class="opacity-25" />
            <path d="M12 2a10 10 0 0 1 10 10" stroke="currentColor" stroke-width="3" stroke-linecap="round" />
          </svg>
          <p class="text-sm text-gray-500">正在检查下载任务...</p>
        </div>

        <!-- 暂无任务：极简空状态 -->
        <div v-else class="flex flex-col items-center gap-2">
          <svg class="h-10 w-10 text-gray-300" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <path stroke-linecap="round" stroke-linejoin="round" d="M9 12h6m-6 4h6m-6-8h6M6 4h12a1 1 0 011 1v14a1 1 0 01-1 1H6a1 1 0 01-1-1V5a1 1 0 011-1z" />
          </svg>
          <p class="text-sm text-gray-400">暂无下载任务</p>
          <p class="text-xs text-gray-300">即将返回上一页...</p>
        </div>
      </div>
    </div>
  </div>
</template>
