import { watch } from 'vue'
import { useVersionStore } from '@/stores/version'
import { getDownloadProgress, isDownloading as checkIsDownloading } from '@/utils/tauri'

/**
 * 全局下载轮询服务
 * 当 store 中 downloading 为 true 时自动启动轮询
 * 不依赖组件生命周期
 */

// SDK 阶段映射（根据 poll.md 文档）
const STAGE_NAMES = [
  '版本清单',   // 0 - VersionManifest
  '版本信息',   // 1 - VersionJson
  '客户端',     // 2 - ClientJar
  '库文件',     // 3 - Libraries
  '资源文件',   // 4 - Assets
  '本地库',     // 5 - Natives
  '解压',       // 6 - ExtractNatives
  '模组',       // 7 - Mods
  '整合包',     // 8 - Modpack
]

let pollTimer: ReturnType<typeof setInterval> | null = null
let pollCount = 0

function startPolling(versionStore: ReturnType<typeof useVersionStore>) {
  if (pollTimer) return

  pollCount = 0
  console.log('[Polling] Starting download polling...')

  pollTimer = setInterval(async () => {
    pollCount++
    try {
      const [progress, downloading] = await Promise.all([
        getDownloadProgress(),
        checkIsDownloading()
      ])

      // 每 10 次轮询输出一次详细日志（约 3 秒）
      if (pollCount % 10 === 0) {
        console.log(`[Polling] #${pollCount} progress=`, progress, `downloading=${downloading}`)
      }

      if (progress) {
        // 计算百分比 - 使用字节数计算（SDK已修复bytes_total包含所有文件）
        let percentage = 0
        
        if (progress.bytes_total > 0) {
          percentage = Math.min(100, Math.round((progress.bytes_downloaded / progress.bytes_total) * 100))
        }

        // 只有SDK明确报告完成时才设为100%
        if (progress.is_complete) {
          percentage = 100
        }

        // 输出异常百分比警告
        if (percentage > 100) {
          console.warn(`[Polling] Abnormal percentage: ${percentage}%, current=${progress.current}, total=${progress.total}`)
        }

        // 获取阶段名称
        const stageIndex = progress.stage
        const stageName = STAGE_NAMES[stageIndex] || `阶段 ${stageIndex}`

        versionStore.updateProgress({
          stage: stageName,
          stageIndex: stageIndex,
          current: progress.current,
          total: progress.total,
          percentage: percentage,
          speed: progress.speed, // SDK 返回的是 bytes/sec
          bytesDownloaded: progress.bytes_downloaded,
          bytesTotal: progress.bytes_total,
          filesRemaining: progress.files_remaining
        })
      }

      // 判断是否应该停止轮询
      // 1. is_complete 为 true
      // 2. is_active 为 false 且 downloading 为 false
      const shouldStop = progress?.is_complete || (!progress?.is_active && !downloading)
      
      if (shouldStop) {
        console.log(`[Polling] Stopping polling: is_complete=${progress?.is_complete}, is_active=${progress?.is_active}, downloading=${downloading}`)
        // 延迟停止轮询，确保 UI 有时间更新
        setTimeout(() => {
          stopPolling()
          versionStore.finishDownload()
        }, 1500)
      }
    } catch (e) {
      console.error('[Polling] Error:', e)
    }
  }, 300)
}

function stopPolling() {
  if (pollTimer) {
    clearInterval(pollTimer)
    pollTimer = null
  }
  pollCount = 0
}

/**
 * 初始化全局轮询监听
 * 在 App.vue 中调用一次
 */
export function initDownloadPolling() {
  const versionStore = useVersionStore()

  // 监听 downloading 状态变化
  watch(
    () => versionStore.downloading,
    (isDownloading) => {
      if (isDownloading) {
        startPolling(versionStore)
      } else {
        stopPolling()
      }
    }
  )
}
