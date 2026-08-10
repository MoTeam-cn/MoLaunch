/**
 * 资源包/光影图标加载 composable（模块级缓存）
 *
 * 后端 `get_pack_icon` 提取包内 pack.png 等为 base64 data URL；
 * 同名文件缓存避免列表刷新重复解压。
 */

import { ref, watchEffect, type Ref } from 'vue'
import * as tauri from '@/utils/tauri'
import type { PackKind } from '@/utils/tauri'

const iconCache = new Map<string, string | null>()

export function usePackIcon(
  selectedId: Ref<string | null>,
  kind: Ref<PackKind>,
  fileName: Ref<string>,
) {
  const iconUrl = ref<string | null>(null)

  watchEffect((onCleanup) => {
    const sid = selectedId.value
    const k = kind.value
    const name = fileName.value
    if (!sid || !name) return
    const req = { cancelled: false }
    onCleanup(() => { req.cancelled = true })

    const cacheKey = `${k}:${name}`
    if (iconCache.has(cacheKey)) {
      iconUrl.value = iconCache.get(cacheKey) ?? null
      return
    }
    tauri.getPackIcon(sid, name, k).then((dataUrl) => {
      if (req.cancelled) return
      iconCache.set(cacheKey, dataUrl)
      iconUrl.value = dataUrl
    }).catch(() => {
      if (req.cancelled) return
      iconCache.set(cacheKey, null)
      iconUrl.value = null
    })
  })

  return { iconUrl }
}
