/**
 * 资源包/光影列表管理 composable
 *
 * 列表加载、筛选/搜索/计数、watcher 生命周期（kind 或版本切换时重建监听）。
 */

import { ref, computed, watch, onUnmounted, type Ref, type ComputedRef } from 'vue'
import * as tauri from '@/utils/tauri'
import { onGlobalEvent } from '@/composables/useGlobalTauriEvent'
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

  onGlobalEvent('packs-dir-changed', () => {
    loadPacks(true)
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

  async function loadPacks(silent = false) {
    if (!selectedId.value) return
    if (!silent) packsLoading.value = true
    try {
      packs.value = await tauri.listPacks(selectedId.value, kind.value)
    } catch (e) {
      console.debug('[PackTab] 加载列表失败:', e)
    } finally {
      if (!silent) packsLoading.value = false
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

  // kind 或版本切换时：重建目录监听 + 重载列表 + 刷新可用性
  watch([selectedId, kind], async ([sid, k]) => {
    if (!sid) return
    await tauri.unwatchPacksDir().catch(() => {})
    await tauri.watchPacksDir(sid, k).catch(() => {})
    loadPacks()
    checkAvailable()
  }, { immediate: true })

  onUnmounted(() => {
    tauri.unwatchPacksDir().catch(() => {})
  })

  return {
    packs, packsLoading, packFilter, packSearch,
    available, checking,
    filteredPacks, filterOptions,
    loadPacks, checkAvailable,
  }
}
