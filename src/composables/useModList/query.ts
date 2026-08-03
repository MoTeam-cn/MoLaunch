/**
 * Mod 列表 - 过滤 / 搜索 / 计数切片
 *
 * 纯计算逻辑，不持有业务状态；mods/modFilter/modSearch 由主 composable 传入。
 */
import { computed } from 'vue'
import type { Ref } from 'vue'
import type { ModInfo } from '@/utils/tauri'

export interface UseModListQueryDeps {
  /** 完整 Mod 列表 */
  mods: Ref<ModInfo[]>
  /** 当前过滤条件（all / enabled / disabled） */
  modFilter: Ref<'all' | 'enabled' | 'disabled'>
  /** 搜索关键字 */
  modSearch: Ref<string>
}

export function useModListQuery(deps: UseModListQueryDeps) {
  const { mods, modFilter, modSearch } = deps

  /** 按过滤条件 + 搜索关键字过滤后的 Mod 列表 */
  const filteredMods = computed(() => {
    let list = mods.value
    if (modFilter.value === 'enabled') list = list.filter(m => m.is_enabled)
    else if (modFilter.value === 'disabled') list = list.filter(m => !m.is_enabled)
    if (modSearch.value.trim()) {
      const q = modSearch.value.toLowerCase()
      list = list.filter(m =>
        m.enabled_name.toLowerCase().includes(q) ||
        m.translated_name.toLowerCase().includes(q),
      )
    }
    return list
  })

  const enabledCount = computed(() => mods.value.filter(m => m.is_enabled).length)
  const disabledCount = computed(() => mods.value.filter(m => !m.is_enabled).length)

  /** 过滤选项卡（全部 / 已启用 / 已禁用 + 计数） */
  const filterOptions = computed(() => [
    { v: 'all' as const, l: '全部', count: mods.value.length },
    { v: 'enabled' as const, l: '已启用', count: enabledCount.value },
    { v: 'disabled' as const, l: '已禁用', count: disabledCount.value },
  ])

  return {
    filteredMods,
    enabledCount,
    disabledCount,
    filterOptions,
  }
}
