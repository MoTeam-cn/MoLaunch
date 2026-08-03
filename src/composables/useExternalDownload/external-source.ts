/**
 * 外部下载 - 来源解析切片
 *
 * URL 输入 + 自动获取文件名（防抖 500ms）+ 用户手动编辑标记。
 * 独立于下载任务逻辑，供 useExternalDownload 组合。
 */
import { ref, watch, onUnmounted } from 'vue'
import { fetchFilename } from '@/utils/api/tools'

/** 校验 URL 是否以 http(s):// 开头 */
export function isValidUrl(urlStr: string): boolean {
  const lower = urlStr.toLowerCase()
  return lower.startsWith('http://') || lower.startsWith('https://')
}

/** 外部下载来源（URL + 文件名输入） */
export function useExternalDownloadSource() {
  const url = ref('')
  const fileName = ref('')
  const isFetchingFilename = ref(false)
  const userEditedFilename = ref(false)
  let filenameDebounce: ReturnType<typeof setTimeout> | null = null

  // URL 变化 → 自动获取文件名（防抖 500ms，用户手动编辑后不再覆盖）
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

  onUnmounted(() => {
    if (filenameDebounce) clearTimeout(filenameDebounce)
  })

  return {
    url,
    fileName,
    isFetchingFilename,
    onFileNameInput,
  }
}