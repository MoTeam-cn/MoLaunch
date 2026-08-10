/**
 * 资源包/光影列表管理 composable
 *
 * 列表加载（合并预加载数据）、筛选/搜索/计数、watcher 生命周期（kind 或版本切换时重建监听）、
 * 详情预加载事件监听（isPreloadDone + packs-preload-update 合并）。
 */

import { ref, computed, watch, onUnmounted, type Ref, type ComputedRef } from 'vue'
import * as tauri from '@/utils/tauri'
import { onGlobalEvent } from '@/composables/useGlobalTauriEvent'
import { usePacksPreload } from '@/composables/usePacksPreload'
import { onImageCached } from '@/composables/useImageCache'
import type { PackInfo, PackKind } from '@/utils/tauri'

export interface UsePackListOptions {
  /** 当前选中的版本 ID（来自 useVersionSettings） */
  selectedId: ComputedRef<string | null>
  /** 内容类型（资源包 / 光影） */
  kind: Ref<PackKind>
}

export function usePackList(options: UsePackListOptions) {
  const { selectedId, kind } = options

  const packs = ref<PackInfo[]>([])
  const packsLoading = ref(false)
  const packFilter = ref<'all' | 'enabled' | 'disabled'>('all')
  const packSearch = ref('')
  const available = ref(true)
  const checking = ref(false)
  /** 组件是否仍挂载（异步链中检查，卸载后不再触发 fire-and-forget invoke） */
  let isMounted = true

  // 版本上下文（详情弹窗 / 更新对话框使用）
  const versionGameVersion = ref<string | null>(null)

  // 预加载事件监听（isPreloadDone + packs-preload-update 自动合并）
  const { isPreloadDone } = usePacksPreload(packs)

  /**
   * 图片缓存完成事件监听
   *
   * 后端 `image_cache::get_image_url` 缓存未命中时返回远程 URL，并异步下载；
   * 完成后 emit `image-cached`，本监听在列表中原地替换为本地缓存 URL。
   */
  onImageCached((remoteUrl, localUrl) => {
    for (let i = 0; i < packs.value.length; i++) {
      if (packs.value[i].cached_logo_url === remoteUrl) {
        packs.value[i] = { ...packs.value[i], cached_logo_url: localUrl }
      }
    }
  })

  /**
   * 内容目录文件变化监听
   *
   * 后端 watcher 在目录文件变化时 emit `packs-dir-changed`（500ms 防抖），
   * 收到后重载列表（合并预加载数据）并重新触发后台预加载。
   */
  onGlobalEvent('packs-dir-changed', () => {
    loadPacks(true)
    if (selectedId.value) {
      tauri.preloadPacksDetail(selectedId.value, kind.value).catch(e => {
        console.debug('[PackTab] 文件变化后预加载启动失败:', e)
      })
    }
  })

  async function checkAvailable() {
    if (!selectedId.value) return
    checking.value = true
    try {
      available.value = await tauri.isPacksAvailable(selectedId.value, kind.value)
    } catch {
      available.value = true
    } finally {
      checking.value = false
    }
  }

  /**
   * 加载包列表（合并预加载数据）
   *
   * `list_packs` 返回的 project / cached_logo_url 为空，由 `preload_packs_detail` 后台补全；
   * 重新加载时按 `enabled_name` 合并已加载的预加载数据，避免刷新后丢失平台工程信息。
   */
  async function loadPacks(silent = false) {
    if (!selectedId.value) return
    const savedData = new Map<string, Partial<PackInfo>>()
    for (const pack of packs.value) {
      savedData.set(pack.enabled_name, {
        project: pack.project,
        cached_logo_url: pack.cached_logo_url,
      })
    }
    if (!silent) packsLoading.value = true
    try {
      const fresh = await tauri.listPacks(selectedId.value, kind.value)
      packs.value = fresh.map(pack => {
        const saved = savedData.get(pack.enabled_name)
        return saved ? { ...pack, ...saved } : pack
      })
    } catch (e) {
      console.debug('[PackTab] 加载列表失败:', e)
    } finally {
      if (!silent) packsLoading.value = false
    }
  }

  /** 预取整合包 MC 版本号（不阻塞 UI） */
  async function prefetchVersionContext() {
    if (!selectedId.value) return
    try {
      versionGameVersion.value = await tauri.getVersionGameVersion(selectedId.value)
    } catch (e) {
      console.debug('[PackTab] 获取版本号失败:', e)
      versionGameVersion.value = null
    }
  }

  const filteredPacks = computed(() => {
    let list = packs.value
    if (packFilter.value === 'enabled') list = list.filter(p => p.is_enabled)
    if (packFilter.value === 'disabled') list = list.filter(p => !p.is_enabled)
    const q = packSearch.value.trim().toLowerCase()
    if (q) list = list.filter(p => p.enabled_name.toLowerCase().includes(q))
    return list
  })

  const filterOptions = computed(() => [
    { v: 'all' as const, l: '全部', count: packs.value.length },
    { v: 'enabled' as const, l: '已启用', count: packs.value.filter(p => p.is_enabled).length },
    { v: 'disabled' as const, l: '已禁用', count: packs.value.filter(p => !p.is_enabled).length },
  ])

  // kind 或版本切换时：重建目录监听 + 重载列表 + 刷新可用性 + 预取版本上下文 + 触发预加载
  watch([selectedId, kind], async ([sid, k]) => {
    if (!sid) return
    await tauri.unwatchPacksDir().catch(() => {})
    await tauri.watchPacksDir(sid, k).catch(() => {})
    if (!isMounted) return
    loadPacks()
    checkAvailable()
    prefetchVersionContext()
    tauri.preloadPacksDetail(sid, k).catch(e => {
      console.debug('[PackTab] 预加载启动失败:', e)
    })
  }, { immediate: true })

  onUnmounted(() => {
    isMounted = false
    tauri.unwatchPacksDir().catch(() => {})
    tauri.cancelPreloadPacksDetail().catch(() => {})
  })

  return {
    packs, packsLoading, packFilter, packSearch,
    available, checking,
    versionGameVersion, isPreloadDone,
    filteredPacks, filterOptions,
    loadPacks, checkAvailable,
  }
}
