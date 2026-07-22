/**
 * Mod 管理操作 composable（编排层）
 *
 * 本文件为编排层，将原 useModOperations 的职责拆分为三个子 composable：
 * - useModList：列表加载 / 过滤 / 单个 Mod 操作 / 预加载 / 详情查询 / 版本上下文 / 文件监听
 * - useModBatchOps：多选状态 + 批量启用/禁用 / 删除
 * - useModUpdateDialog：更新对话框状态 + 打开/批量更新/安装完成回调
 *
 * 本文件仅负责组合三个子 composable，对外保持原有 API 不变。
 *
 * 设计原则：
 * - 接收所需的 ref/computed 作为参数（selectedId / isModable / modLocalNameStyle）
 * - 返回 handler 函数和状态
 * - handler 内部的 toast/modal 调用保持原 ModTab.vue 行为不变
 * - 模板中的事件绑定保持不变
 */
import { type Ref, type ComputedRef } from 'vue'
import { useModList } from './useModList'
import { useModBatchOps } from './useModBatchOps'
import { useModUpdateDialog } from './useModUpdateDialog'

interface UseModOperationsOptions {
  /** 当前选中的版本 ID（来自 useVersionSettings） */
  selectedId: ComputedRef<string | null>
  /** 版本是否可安装 Mod（来自 useVersionSettings 的 isModable computed） */
  isModable: ComputedRef<boolean>
  /** Mod 本地名称显示风格（0=文件名 1=译名 2=译名+文件名，由父组件持有以便其他子组件共用） */
  modLocalNameStyle: Ref<number>
}

/**
 * Mod 管理操作 composable
 *
 * 使用方式：
 * ```ts
 * const { selectedId, isModable } = useVersionSettings()
 * const modLocalNameStyle = ref(0)
 * const { mods, filteredMods, handleToggleMod, init, ... } = useModOperations({
 *   selectedId, isModable, modLocalNameStyle,
 * })
 * onMounted(init)
 * onUnmounted(stopPreloadListener)
 * ```
 */
export function useModOperations(options: UseModOperationsOptions) {
  // 1. 列表管理（加载 / 过滤 / 单 Mod 操作 / 预加载 / 详情查询 / 版本上下文 / 文件监听）
  const list = useModList(options)

  // 2. 多选批量操作（依赖 list.mods / list.filteredMods / list.loadMods）
  const batch = useModBatchOps({
    selectedId: options.selectedId,
    mods: list.mods,
    filteredMods: list.filteredMods,
    loadMods: list.loadMods,
  })

  // 3. 更新对话框状态（依赖 batch.selectedIds / batch.getSelectedItems / list.loadMods）
  const update = useModUpdateDialog({
    selectedIds: batch.selectedIds,
    getSelectedItems: batch.getSelectedItems,
    loadMods: list.loadMods,
  })

  return {
    // ===== 状态（来自 list） =====
    mods: list.mods,
    modsLoading: list.modsLoading,
    modFilter: list.modFilter,
    modSearch: list.modSearch,
    isModableVersion: list.isModableVersion,
    checkingModable: list.checkingModable,
    versionGameVersion: list.versionGameVersion,
    versionModsDir: list.versionModsDir,
    disableModUpdate: list.disableModUpdate,
    isPreloadDone: list.isPreloadDone,
    // 多选状态（来自 batch）
    selectedIds: batch.selectedIds,
    batchProcessing: batch.batchProcessing,
    hasSelection: batch.hasSelection,
    selectedCount: batch.selectedCount,
    // 按钮可用性判断（来自 batch）
    hasEnabledSelected: batch.hasEnabledSelected,
    hasDisabledSelected: batch.hasDisabledSelected,
    hasUpdatableSelected: batch.hasUpdatableSelected,
    // Mod 更新对话框状态（来自 update）
    updateDialogVisible: update.updateDialogVisible,
    updateTargetMod: update.updateTargetMod,
    // 详情弹窗（来自 list）
    detailVisible: list.detailVisible,
    detailProject: list.detailProject,
    detailLoadingFor: list.detailLoadingFor,
    // computed（来自 list）
    filteredMods: list.filteredMods,
    filterOptions: list.filterOptions,
    // 生命周期（来自 list）
    startPreloadListener: list.startPreloadListener,
    stopPreloadListener: list.stopPreloadListener,
    init: list.init,
    // handler（来自 list）
    checkModable: list.checkModable,
    loadMods: list.loadMods,
    prefetchVersionContext: list.prefetchVersionContext,
    handleToggleMod: list.handleToggleMod,
    handleDeleteMod: list.handleDeleteMod,
    handleInstallMod: list.handleInstallMod,
    handleOpenModsDir: list.handleOpenModsDir,
    handleOpenFile: list.handleOpenFile,
    onShowInfo: list.onShowInfo,
    handleOpenWiki: list.handleOpenWiki,
    // 多选操作（来自 batch）
    toggleSelect: batch.toggleSelect,
    selectAll: batch.selectAll,
    invertSelection: batch.invertSelection,
    clearSelection: batch.clearSelection,
    checkSelected: batch.checkSelected,
    getSelectedItems: batch.getSelectedItems,
    handleMultiSelectKeydown: batch.handleMultiSelectKeydown,
    // 批量业务 handler
    batchToggle: batch.batchToggle,
    batchDelete: batch.batchDelete,
    batchUpdate: update.batchUpdate,
    openUpdateDialog: update.openUpdateDialog,
    onModUpdated: update.onModUpdated,
  }
}
