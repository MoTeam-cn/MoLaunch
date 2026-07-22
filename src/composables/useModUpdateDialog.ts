/**
 * Mod 更新对话框状态 composable（从 useModOperations 拆出）
 *
 * 负责：
 * - 更新对话框可见状态 + 目标 Mod
 * - 单个 Mod 打开更新对话框（openUpdateDialog）
 * - 批量更新入口（batchUpdate）：从选中项找第一个可更新的 Mod 打开对话框
 * - 安装完成后的回调（onModUpdated）：刷新列表 + 清理选中状态
 *
 * 不负责：
 * - 列表加载（见 useModList）
 * - 多选状态（见 useModBatchOps，但批量更新依赖其 selectedIds 和 getSelectedItems）
 */
import { ref, type Ref } from 'vue'
import { toastError } from '@/utils/toast'
import type { ModInfo } from '@/utils/tauri'

export interface UseModUpdateDialogOptions {
  /** 多选选中集合（onModUpdated 后需清理已更新 mod 的选中状态） */
  selectedIds: Ref<Set<string>>
  /** 获取当前选中项数组（batchUpdate 用） */
  getSelectedItems: () => ModInfo[]
  /** 重新加载列表函数（onModUpdated 后调用） */
  loadMods: (silent?: boolean) => Promise<void>
}

export function useModUpdateDialog(options: UseModUpdateDialogOptions) {
  const { selectedIds, getSelectedItems, loadMods } = options

  /** Mod 更新对话框状态 */
  const updateDialogVisible = ref(false)
  /** 当前要更新/更改的 mod */
  const updateTargetMod = ref<ModInfo | null>(null)

  /** 批量更新（打开更新对话框逐个处理） */
  function batchUpdate() {
    if (selectedIds.value.size === 0) return
    const selected = getSelectedItems()
    // 找第一个有 project 信息的可更新 mod
    const updatable = selected.find(m => m.project)
    if (!updatable) {
      toastError('选中的 Mod 没有关联平台信息，无法更新')
      return
    }
    // 打开更新对话框
    updateTargetMod.value = updatable
    updateDialogVisible.value = true
  }

  /** 打开单个 mod 的更新/更改对话框 */
  function openUpdateDialog(mod: ModInfo) {
    if (!mod.project) {
      toastError('此 Mod 没有关联平台信息，无法更新')
      return
    }
    updateTargetMod.value = mod
    updateDialogVisible.value = true
  }

  /** Mod 更新对话框安装完成后的回调 */
  async function onModUpdated() {
    await loadMods()
    // 移除已更新 mod 的选中状态
    if (updateTargetMod.value) {
      selectedIds.value.delete(updateTargetMod.value.file_name)
      selectedIds.value = new Set(selectedIds.value)
    }
  }

  return {
    updateDialogVisible,
    updateTargetMod,
    batchUpdate,
    openUpdateDialog,
    onModUpdated,
  }
}
