/**
 * 外部下载工具的 composable
 *
 * 封装外部下载的全部状态与逻辑：
 * - URL 输入 + 自动获取文件名（防抖 500ms）
 * - 下载目录管理（自定义 / 默认）
 * - 下载启动 / 暂停 / 取消
 * - 已下载文件列表刷新
 */
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import { useVersionStore } from '@/stores/version'
import { toastSuccess, toastError, toastInfo } from '@/utils/toast'
import { showConfirm } from '@/utils/modal'
import { formatBytes } from '@/utils/format'
import { applyConfig, getConfigMap } from '@/utils/api/config'
import { selectFolder, openPath, pauseDownload, resumeDownload, cancelDownload } from '@/utils/api/system'
import {
  downloadFile,
  getDownloadDir,
  listDownloads,
  deleteDownload,
  fetchFilename,
} from '@/utils/api/tools'
import type { ExternalDownloadEntry } from '@/utils/api/tools'

export function useExternalDownload() {
  const versionStore = useVersionStore()

  // ==================== 表单状态 ====================
  const url = ref('')
  const fileName = ref('')
  const isFetchingFilename = ref(false)
  const userEditedFilename = ref(false)
  let filenameDebounce: ReturnType<typeof setTimeout> | null = null

  // ==================== 下载目录 ====================
  const downloadDir = ref('')
  const customDir = ref<string | null>(null)
  const isCustomDir = computed(() => !!customDir.value)

  // ==================== 文件列表 ====================
  const files = ref<ExternalDownloadEntry[]>([])

  // ==================== 下载状态（复用 versionStore） ====================
  const downloading = computed(() => versionStore.downloading)
  const downloadProgress = computed(() => versionStore.downloadProgress)
  const isPaused = computed(() => downloadProgress.value?.isPaused ?? false)

  const percentage = computed(() => downloadProgress.value?.percentage ?? 0)
  const speedFormatted = computed(() => formatBytes(downloadProgress.value?.global_speed ?? 0) + '/s')
  const downloadedFormatted = computed(() => {
    const d = downloadProgress.value?.global_bytes_downloaded ?? 0
    const t = downloadProgress.value?.global_bytes_total ?? 0
    return `${formatBytes(d)} / ${formatBytes(t)}`
  })
  const currentFileName = computed(() => versionStore.downloadingVersion ?? fileName.value)

  const canStartDownload = computed(() => {
    return url.value.trim() && fileName.value.trim() && !downloading.value
  })

  // ==================== 工具函数 ====================
  function isValidUrl(urlStr: string): boolean {
    const lower = urlStr.toLowerCase()
    return lower.startsWith('http://') || lower.startsWith('https://')
  }

  // ==================== 自动获取文件名 ====================
  watch(url, (newUrl) => {
    userEditedFilename.value = false

    if (!newUrl || !isValidUrl(newUrl)) {
      return
    }

    if (filenameDebounce) clearTimeout(filenameDebounce)
    filenameDebounce = setTimeout(async () => {
      if (userEditedFilename.value) return
      isFetchingFilename.value = true
      try {
        const result = await fetchFilename(newUrl)
        if (!userEditedFilename.value && result.filename) {
          fileName.value = result.filename
        }
      } catch {
        // 获取失败时静默处理，用户可手动输入
      } finally {
        isFetchingFilename.value = false
      }
    }, 500)
  })

  function onFileNameInput() {
    userEditedFilename.value = true
  }

  // ==================== 下载目录管理 ====================
  async function selectDownloadDir() {
    const folder = await selectFolder()
    if (!folder) return

    try {
      await applyConfig({ externalDownloadDir: folder })
      customDir.value = folder
      await loadDownloadDir()
      await refreshFiles()
      toastSuccess(`下载目录已更新`)
    } catch (e) {
      toastError(`设置目录失败: ${e instanceof Error ? e.message : String(e)}`)
    }
  }

  async function resetDownloadDir() {
    const confirmed = await showConfirm(
      '恢复默认目录',
      '确定要恢复使用默认下载目录（.Molaunch/Download/）吗？已下载的文件不会被删除。',
    )
    if (!confirmed) return

    try {
      await applyConfig({ externalDownloadDir: null })
      customDir.value = null
      await loadDownloadDir()
      await refreshFiles()
      toastInfo('已恢复默认下载目录')
    } catch (e) {
      toastError(`恢复失败: ${e instanceof Error ? e.message : String(e)}`)
    }
  }

  async function openDownloadDir() {
    if (!downloadDir.value) return
    await openPath(downloadDir.value)
  }

  async function loadDownloadDir() {
    try {
      downloadDir.value = await getDownloadDir()
    } catch {
      downloadDir.value = ''
    }
  }

  async function loadCustomDir() {
    try {
      const config = await getConfigMap()
      customDir.value = config.externalDownloadDir ?? null
    } catch {
      customDir.value = null
    }
  }

  // ==================== 下载逻辑 ====================
  async function startDownload() {
    const urlVal = url.value.trim()
    const nameVal = fileName.value.trim()

    if (!urlVal) {
      toastError('请输入下载地址')
      return
    }
    if (!nameVal) {
      toastError('请输入文件名')
      return
    }
    if (!isValidUrl(urlVal)) {
      toastError('下载地址必须以 http:// 或 https:// 开头')
      return
    }

    versionStore.startDownload(nameVal)
    toastInfo(`开始下载: ${nameVal}`)

    downloadFile(urlVal, nameVal)
      .then(async () => {
        await refreshFiles()
        url.value = ''
        fileName.value = ''
      })
      .catch((e) => {
        if (versionStore.downloading) {
          versionStore.finishDownload()
        }
        toastError(`下载失败: ${e instanceof Error ? e.message : String(e)}`)
      })
  }

  async function togglePause() {
    try {
      if (isPaused.value) {
        await resumeDownload()
        toastInfo('下载已恢复')
      } else {
        await pauseDownload()
        toastInfo('下载已暂停')
      }
    } catch (e) {
      toastError(`操作失败: ${e instanceof Error ? e.message : String(e)}`)
    }
  }

  async function cancelDownloadTask() {
    const confirmed = await showConfirm('取消下载', '确定要取消当前下载任务吗？已下载的部分文件将被删除。')
    if (!confirmed) return

    try {
      await cancelDownload()
      toastInfo('下载已取消')
    } catch (e) {
      toastError(`取消失败: ${e instanceof Error ? e.message : String(e)}`)
    }
  }

  // ==================== 文件列表 ====================
  async function refreshFiles() {
    try {
      files.value = await listDownloads()
    } catch {
      files.value = []
    }
  }

  async function deleteFile(name: string) {
    const confirmed = await showConfirm('删除文件', `确定要删除 "${name}" 吗？此操作不可恢复。`)
    if (!confirmed) return

    try {
      await deleteDownload(name)
      toastSuccess(`已删除: ${name}`)
      await refreshFiles()
    } catch (e) {
      toastError(`删除失败: ${e instanceof Error ? e.message : String(e)}`)
    }
  }

  // ==================== 监听下载完成（刷新文件列表） ====================
  watch(downloading, (isDownloading, wasDownloading) => {
    if (wasDownloading && !isDownloading) {
      refreshFiles()
    }
  })

  // ==================== 生命周期 ====================
  onMounted(async () => {
    await Promise.all([loadCustomDir(), loadDownloadDir(), refreshFiles()])
  })

  onUnmounted(() => {
    if (filenameDebounce) clearTimeout(filenameDebounce)
  })

  return {
    // 表单状态
    url,
    fileName,
    isFetchingFilename,
    onFileNameInput,
    // 下载目录
    downloadDir,
    isCustomDir,
    selectDownloadDir,
    resetDownloadDir,
    openDownloadDir,
    // 下载状态
    downloading,
    isPaused,
    percentage,
    speedFormatted,
    downloadedFormatted,
    currentFileName,
    canStartDownload,
    startDownload,
    togglePause,
    cancelDownloadTask,
    // 文件列表
    files,
    deleteFile,
  }
}
