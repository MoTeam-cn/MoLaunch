<script setup lang="ts">
/**
 * 下载管理页面
 * 左右分栏布局
 * - 左侧：统计面板（DownloadStatsPanel 子组件）
 * - 右侧：任务卡片（TaskGroupCard 子组件，含分组列表与暂停/取消按钮）
 * - 无任务时：空状态（DownloadEmptyState 子组件）
 */

import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { useVersionStore } from '@/stores/version'
import { showConfirm } from '@/utils/modal'
import { toastInfo, toastError, toastWarning } from '@/utils/toast'
import { pauseDownload, resumeDownload, cancelDownload, getDownloadProgress, isDownloading } from '@/utils/tauri'
import DownloadStatsPanel from './downloads/DownloadStatsPanel.vue'
import TaskGroupCard from '@/components/downloads/TaskGroupCard.vue'
import { safeCall } from '@/utils/async'
import { applyProgressPatch } from '@/utils/downloadProgress'

const versionStore = useVersionStore()
const router = useRouter()

// 检查中状态：3 秒重试期间为 true，避免页面空白
const checking = ref(true)

// 进入页面时恢复下载状态
// 首次进入时延迟重试检查（给后端异步启动下载任务的时间，避免双击下载按钮进入页面就被赶回去）
// 下载进度事件流由 App.vue 的 initDownloadStream 全局管理，这里只负责初始状态恢复
onMounted(async () => {
  const maxRetries = 6 // 最多重试 6 次，每次 500ms，共 3 秒
  for (let attempt = 0; attempt < maxRetries; attempt++) {
    const success = await safeCall(async () => {
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
        return true // 成功恢复，不需要返回
      }
      return false
    }, 'check download state')
    if (success === true) return
    // 等待 500ms 后重试
    if (attempt < maxRetries - 1) {
      await new Promise(resolve => setTimeout(resolve, 500))
    }
  }
  // 重试完毕仍无任务，显示暂无任务极简画面，1.5 秒后返回上一页
  checking.value = false
  toastWarning('未检测到下载任务，已返回')
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

// 定时刷新 now，驱动 percentage 伪进度补丁重算
// 后端 ticker 到 95% 后卡住，前端基于 now 继续小数点上涨到 99.9%
const now = ref(Date.now())
const percentageStartTime = ref<number | null>(null)
let nowTimer: ReturnType<typeof setInterval> | null = null
onMounted(() => {
  nowTimer = setInterval(() => { now.value = Date.now() }, 200)
})
onUnmounted(() => {
  if (nowTimer) clearInterval(nowTimer)
})

const realPercentage = computed(() => progress.value?.percentage || 0)

// 监听真实进度，在进入 95% 区间时记录起始时间（副作用放 watch，不放 computed）
watch(realPercentage, (val) => {
  const ratio = val / 100
  if (ratio >= 0.95 && ratio < 1) {
    if (percentageStartTime.value === null) {
      percentageStartTime.value = Date.now()
    }
  } else {
    percentageStartTime.value = null
  }
})

const percentage = computed(() => {
  const real = realPercentage.value
  const ratio = real / 100
  if (ratio < 0.95 || ratio >= 1) return real
  if (percentageStartTime.value === null) return real
  const patched = applyProgressPatch(ratio, percentageStartTime.value, now.value)
  return parseFloat((patched * 100).toFixed(1))
})
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
  await safeCall(async () => {
    if (isPaused.value) {
      await resumeDownload()
      toastInfo('下载已恢复')
    } else {
      await pauseDownload()
      toastInfo('下载已暂停')
    }
  }, 'toggle pause', () => {
    toastError('操作失败')
  })
  togglingPause.value = false
}

function handleCancel() {
  showConfirm(
    '取消下载',
    '确定要取消当前下载任务吗？已下载的文件将保留，但未完成的下载将被中止。',
    async () => {
      if (cancelling.value) return
      cancelling.value = true
      const ok = await safeCall(() => cancelDownload(), 'cancel download', () => {
        toastError('取消失败')
      })
      cancelling.value = false
      if (ok === undefined) return
      toastInfo('下载已取消')
      // 确保取消后立即触发返回（避免轮询已停止时卡在空白页）
      versionStore.finishDownload()
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
