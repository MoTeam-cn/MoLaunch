/**
 * 外部下载来源 composable
 *
 * 负责 URL / 文件名输入（防抖自动获取）、下载目录管理与已下载文件列表。
 */
import { ref, computed, watch, onUnmounted } from 'vue'
import { toastSuccess, toastError, toastInfo } from '@/utils/toast'
import { showConfirm } from '@/utils/modal'
import { applyConfig, getConfigMap } from '@/utils/api/config'
import { openPath } from '@/utils/api/system'
import { pickDirectory } from '@/utils/fileDialog'
import {
  getDownloadDir,
  listDownloads,
  deleteDownload,
  fetchFilename,
} from '@/utils/api/tools'
import type { ExternalDownloadEntry } from '@/utils/api/tools'

export function useExternalSource() {
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
    const folder = await pickDirectory()
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
    showConfirm(
      '恢复默认目录',
      '确定要恢复使用默认下载目录（.Molaunch/Download/）吗？已下载的文件不会被删除。',
      async () => {
        try {
          await applyConfig({ externalDownloadDir: null })
          customDir.value = null
          await loadDownloadDir()
          await refreshFiles()
          toastInfo('已恢复默认下载目录')
        } catch (e) {
          toastError(`恢复失败: ${e instanceof Error ? e.message : String(e)}`)
        }
      },
    )
  }

  async function openDownloadDir() {
    if (!downloadDir.value) return
    try {
      await openPath(downloadDir.value)
    } catch (e) {
      toastError('打开目录失败: ' + (e instanceof Error ? e.message : String(e)))
    }
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

  // ==================== 文件列表 ====================
  async function refreshFiles() {
    try {
      files.value = await listDownloads()
    } catch {
      files.value = []
    }
  }

  async function deleteFile(name: string) {
    showConfirm(
      '删除文件',
      `确定要删除 "${name}" 吗？此操作不可恢复。`,
      async () => {
        try {
          await deleteDownload(name)
          toastSuccess(`已删除: ${name}`)
          await refreshFiles()
        } catch (e) {
          toastError(`删除失败: ${e instanceof Error ? e.message : String(e)}`)
        }
      },
    )
  }

  onUnmounted(() => {
    if (filenameDebounce) clearTimeout(filenameDebounce)
  })

  return {
    url,
    fileName,
    isFetchingFilename,
    onFileNameInput,
    isValidUrl,
    downloadDir,
    isCustomDir,
    files,
    selectDownloadDir,
    resetDownloadDir,
    openDownloadDir,
    loadDownloadDir,
    loadCustomDir,
    refreshFiles,
    deleteFile,
  }
}