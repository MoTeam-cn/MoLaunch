/**
 * 资源下载进度状态切片（从 useResourceDownload.ts 抽取）
 *
 * 负责下载中标志与下载阶段两个核心状态及状态迁移 helper，
 * 供主文件 handleDownload 与确认切片 handleDependencyConfirm 共用，
 * 避免两处重复编写 downloading/downloadStage 的状态复位逻辑。
 */
import type { Ref } from 'vue'
import { ref } from 'vue'

/** 下载阶段：idle=空闲 / requesting=请求中（前置检查/准备）/ waiting=等待用户确认前置 / downloading=下载中 */
export type DownloadStage = 'idle' | 'requesting' | 'waiting' | 'downloading'

export interface UseDownloadProgress {
  /** 下载中标志（值=正在下载的 version_id，null=空闲） */
  downloading: Ref<string | null>
  /** 下载阶段（按钮文字分阶段显示） */
  downloadStage: Ref<DownloadStage>
  /** 进入请求阶段：设 downloading + stage=requesting（前置检查或直接下载准备） */
  startDownload: (versionId: string) => void
  /** 进入等待用户确认前置阶段 */
  toWaiting: () => void
  /** 进入实际下载阶段 */
  toDownloading: () => void
  /** 复位为空闲：downloading=null + stage=idle */
  resetDownload: () => void
}

export function useDownloadProgress(): UseDownloadProgress {
  const downloading = ref<string | null>(null)
  const downloadStage = ref<DownloadStage>('idle')

  function startDownload(versionId: string) {
    downloading.value = versionId
    downloadStage.value = 'requesting'
  }

  function toWaiting() {
    downloadStage.value = 'waiting'
  }

  function toDownloading() {
    downloadStage.value = 'downloading'
  }

  function resetDownload() {
    downloading.value = null
    downloadStage.value = 'idle'
  }

  return {
    downloading,
    downloadStage,
    startDownload,
    toWaiting,
    toDownloading,
    resetDownload,
  }
}
