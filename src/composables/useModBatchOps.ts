/**
 * Mod 多选批量操作 composable（从 useModOperations 拆出）
 *
 * 基于 useMultiSelect 管理多选状态，提供批量启用/禁用、批量删除与按钮可用性判断；
 * 列表加载见 useModList，更新对话框见 useModUpdateDialog。
 */
import { computed, type ComputedRef, type Ref } from 'vue'
import * as tauri from '@/utils/tauri'
import { toastSuccess, toastError } from '@/utils/toast'
import { showConfirm } from '@/utils/modal'
import { useMultiSelect } from '@/composables/useMultiSelect'
import type { ModInfo } from '@/utils/tauri'

export interface UseModBatchOpsOptions {
  /** 当前选中的版本 ID */
  selectedId: ComputedRef<string | null>
  /** Mod 列表（批量操作需原地更新 file_name/is_enabled 字段） */
  mods: Ref<ModInfo[]>
  /** 过滤后的列表（多选范围） */
  filteredMods: ComputedRef<ModInfo[]>
  /** 重新加载列表函数（批量删除后调用） */
  loadMods: (silent?: boolean) => Promise<void>
}

export function useModBatchOps(options: UseModBatchOpsOptions) {
  const { selectedId, mods, filteredMods, loadMods } = options

  const {
    selectedIds,
    batchProcessing,
    hasSelection,
    selectedCount,
    toggle: toggleSelect,
    selectAll,
    invertSelection,
    clearSelection,
    checkSelected,
    getSelectedItems,
    handleKeydown: handleMultiSelectKeydown,
  } = useMultiSelect<ModInfo>({
    items: filteredMods,
    getId: (mod) => mod.file_name,
  })

  // ===== 按钮可用性判断 =====
  // 选中项中是否有已启用的 mod（控制"禁用"按钮）
  const hasEnabledSelected = computed(() =>
    getSelectedItems().some(m => m.is_enabled),
  )
  // 选中项中是否有已禁用的 mod（控制"启用"按钮）
  const hasDisabledSelected = computed(() =>
    getSelectedItems().some(m => !m.is_enabled),
  )
  // 选中项中是否有可更新的 mod（有关联平台工程信息）
  const hasUpdatableSelected = computed(() =>
    getSelectedItems().some(m => m.project),
  )

  /**
   * 批量启用/禁用
   *
   * 与单个 toggle 一致：原地更新 mod 字段，不重新加载列表。
   * 同步更新 selectedIds 集合的 file_name（toggleMod 会重命名文件）。
   */
  async function batchToggle(enable: boolean) {
    if (!selectedId.value || selectedIds.value.size === 0) return
    batchProcessing.value = true
    try {
      const selected = getSelectedItems()
      const toToggle = selected.filter(m => m.is_enabled !== enable)
      let success = 0
      let failed = 0
      for (const mod of toToggle) {
        try {
          const newFileName = await tauri.toggleMod(selectedId.value, mod.file_name, enable)
          const idx = mods.value.findIndex(m => m.file_name === mod.file_name)
          if (idx !== -1) {
            mods.value[idx] = {
              ...mods.value[idx],
              file_name: newFileName,
              is_enabled: enable,
            }
          }
          // 同步更新选中集合的 file_name（toggleMod 会重命名文件）
          selectedIds.value.delete(mod.file_name)
          selectedIds.value.add(newFileName)
          success++
        } catch {
          failed++
        }
      }
      selectedIds.value = new Set(selectedIds.value)
      if (failed === 0) {
        toastSuccess(`已${enable ? '启用' : '禁用'} ${success} 个 Mod`)
      } else {
        toastError(`${enable ? '启用' : '禁用'}完成，成功 ${success} 个，失败 ${failed} 个`)
      }
      // 操作完成后自动清空选中
      clearSelection()
    } finally {
      batchProcessing.value = false
    }
  }

  /** 批量删除（带二次确认） */
  function batchDelete() {
    if (!selectedId.value || selectedIds.value.size === 0) return
    const count = selectedIds.value.size
    showConfirm(
      '批量删除 Mod',
      `确定要删除选中的 ${count} 个 Mod 吗？此操作不可恢复。`,
      async () => {
        batchProcessing.value = true
        try {
          let success = 0
          let failed = 0
          const toDelete = getSelectedItems()
          for (const mod of toDelete) {
            try {
              await tauri.deleteMod(selectedId.value!, mod.file_name)
              selectedIds.value.delete(mod.file_name)
              success++
            } catch {
              failed++
            }
          }
          await loadMods()
          if (failed === 0) {
            toastSuccess(`已删除 ${success} 个 Mod`)
          } else {
            toastError(`删除完成，成功 ${success} 个，失败 ${failed} 个`)
          }
          // 操作完成后自动清空选中
          clearSelection()
        } finally {
          batchProcessing.value = false
        }
      },
    )
  }

  return {
    // 多选状态
    selectedIds,
    batchProcessing,
    hasSelection,
    selectedCount,
    // 按钮可用性判断
    hasEnabledSelected,
    hasDisabledSelected,
    hasUpdatableSelected,
    // 多选操作
    toggleSelect,
    selectAll,
    invertSelection,
    clearSelection,
    checkSelected,
    getSelectedItems,
    handleMultiSelectKeydown,
    // 批量业务 handler
    batchToggle,
    batchDelete,
  }
}
