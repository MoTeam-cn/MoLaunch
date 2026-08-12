/**
 * 下载进度 Tauri 事件流（替代 WebSocket/轮询方案）
 *
 * 订阅后端 `app.emit("download-progress")` 推送的进度 snapshot 更新 versionStore；
 * `is_complete=true` 时触发 finishDownload + toastSuccess。
 * 模块级单例 listener，App 生命周期内常驻，无需按下载状态建连/断开。
 */

import { listen } from '@tauri-apps/api/event'
import { useVersionStore } from '@/stores/version'
import { toastSuccess } from '@/utils/toast'
import { isDownloading, getDownloadProgress } from '@/utils/api/system'
import type { DownloadStage, RawDownloadProgress, RawDownloadStage } from '@/types/download'
import { safeCall } from '@/utils/async'

/** 防止 initDownloadStream 被多次调用注册重复 listener */
let registered = false

/**
 * 处理 download-progress 事件 payload
 *
 * 逻辑与原 WS handleProgress 一致：
 * 1. 错误码检查 → 忽略（失败 UI 由调用方 catch 统一处理）
 * 2. stages 映射 + 加权百分比计算
 * 3. 暂停状态检测
 * 4. 完成检查 → finishDownload + toastSuccess
 */
function handleProgress(progress: RawDownloadProgress) {
  if (!progress || !progress.stages || progress.stages.length === 0) return

  const versionStore = useVersionStore()

  // 错误码检查：仅忽略事件，不 finishDownload / toastError
  // 失败的 UI 提示由调用方的 catch 统一处理
  if (progress.error_code && progress.error_code !== 0) {
    if (import.meta.env.DEV) {
      console.debug('[Download] Failed with error_code=', progress.error_code, '（交给调用方 catch 处理）')
    }
    return
  }

  const stages: DownloadStage[] = progress.stages.map((s: RawDownloadStage) => ({
    name: s.name,
    progress: s.progress,
    weight: s.weight,
    status: s.status,
    bytes_downloaded: s.bytes_downloaded,
    bytes_total: s.bytes_total,
    files_downloaded: s.files_downloaded || 0,
    files_total: s.files_total || 0,
    group: s.group ?? null,
  }))

  // 检测暂停状态：任意 stage 携带 is_paused=true 即表示全局暂停
  const isPaused = progress.stages.some((s: RawDownloadStage) => s.is_paused === true)

  let weightedProgress = 0
  let totalWeight = 0
  for (const stage of stages) {
    totalWeight += stage.weight
    weightedProgress += stage.progress * stage.weight
  }
  const percentage = totalWeight > 0
    ? Math.min(100, parseFloat(((weightedProgress / totalWeight) * 100).toFixed(1)))
    : 0

  versionStore.updateProgress({
    stages,
    current_stage_index: progress.current_stage_index ?? 0,
    global_speed: progress.global_speed ?? 0,
    global_bytes_downloaded: progress.global_bytes_downloaded ?? 0,
    global_bytes_total: progress.global_bytes_total ?? 0,
    percentage,
    isPaused,
  })

  if (progress.is_complete) {
    if (import.meta.env.DEV) {
      console.debug('[Download] Download complete')
    }
    const completedName = versionStore.downloadingVersion || '下载任务'
    versionStore.finishDownload()
    toastSuccess(`${completedName} 下载完成`)
  }
}

/**
 * 初始化下载进度 Tauri 事件流
 *
 * 在 `App.vue` 的 `onMounted` 中调用一次即可。订阅后端 `download-progress`
 * 事件实时更新 versionStore。启动时主动检查后端是否有进行中的下载任务：
 * - 应用刷新/重启后 `versionStore.downloading` 为 false，但后端可能仍在下载
 * - 主动调用 `isDownloading` 检查，若仍在下载则 `startDownload` 恢复状态
 *
 * 内部有 guard 防止多次调用注册重复 listener。
 */
export function initDownloadStream() {
  if (registered) return
  registered = true

  const versionStore = useVersionStore()

  // 启动时主动检查后端是否有进行中的下载任务
  void (async () => {
    const active = await safeCall(() => isDownloading(), 'init check downloading')
    if (active) {
      const raw = await safeCall(() => getDownloadProgress(), 'init get progress')
      versionStore.startDownload(raw?.version_name || '')
    }
  })()

  // 订阅后端下载面板显隐事件
  // 下载批次开始时 DownloadManager emit {visible:true}，全部结束后 emit {visible:false}
  // 静默下载（Java / 程序更新 / 启动补全）不触发，面板保持隐藏
  void listen<{ visible: boolean }>('download-panel-state', (event) => {
    versionStore.setDownloading(event.payload.visible === true)
  })

  // 订阅后端下载进度事件（模块级单例，App 生命周期内不注销）
  void listen<RawDownloadProgress>('download-progress', (event) => {
    handleProgress(event.payload)
  })
}