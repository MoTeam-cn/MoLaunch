/**
 * 通用多选 composable
 *
 * 多选交互：
 * - 点击列表项切换选中状态（非长按）
 * - Shift+点击 范围选择
 * - 滑动拖拽选择可后续扩展
 * - ESC 清空选中
 * - 当 selectedCount > 0 时，外部组件显示 MultiSelectBar
 *
 * 设计原则：
 * - 只管理选中状态，不涉及业务逻辑（启用/禁用/删除等由调用方实现）
 * - 泛型 T 支持任意列表项类型
 * - 通过 getId 函数提取唯一标识
 *
 * 使用方式：
 * ```ts
 * const { selectedIds, selectedCount, hasSelection, toggle, selectAll, ... } = useMultiSelect({
 *   items: filteredList,
 *   getId: (item) => item.id,
 * })
 * ```
 */
import { ref, computed, unref, type Ref, type ComputedRef, type ToRef } from 'vue'

export interface UseMultiSelectOptions<T> {
  /** 可选的列表项（支持 ref 或 computed） */
  items: ComputedRef<T[]> | Ref<T[]> | ToRef<T[]>
  /** 从列表项提取唯一标识 */
  getId: (item: T) => string
}

export function useMultiSelect<T>(options: UseMultiSelectOptions<T>) {
  const { items, getId } = options

  /** 已选中的 ID 集合（用 Set 保证唯一性） */
  const selectedIds = ref<Set<string>>(new Set())
  /** 最后一次点击的项（用于 Shift 范围选择） */
  const lastClickedItem = ref<T | null>(null)
  /** 批量操作进行中（由调用方控制） */
  const batchProcessing = ref(false)

  /** 是否有选中项（控制 MultiSelectBar 显隐） */
  const hasSelection = computed(() => selectedIds.value.size > 0)
  /** 选中数量 */
  const selectedCount = computed(() => selectedIds.value.size)

  /**
   * 切换某项的选中状态
   *
   * @param item 要切换的项
   * @param shiftKey 是否按住 Shift（范围选择）
   *
   * Checked 的 Set 逻辑 + Swipe 范围选择
   */
  function toggle(item: T, shiftKey = false) {
    const id = getId(item)

    if (shiftKey && lastClickedItem.value) {
      // Shift 范围选择：选中上次点击到当前点击之间的所有项
      const list = unref(items)
      const startIdx = list.findIndex(i => getId(i) === getId(lastClickedItem.value!))
      const endIdx = list.findIndex(i => getId(i) === id)
      if (startIdx !== -1 && endIdx !== -1) {
        const [from, to] = startIdx <= endIdx ? [startIdx, endIdx] : [endIdx, startIdx]
        for (let i = from; i <= to; i++) {
          selectedIds.value.add(getId(list[i]))
        }
      }
    } else {
      // 单击切换选中
      if (selectedIds.value.has(id)) {
        selectedIds.value.delete(id)
      } else {
        selectedIds.value.add(id)
      }
    }
    // Set 的 add/delete 不触发响应式，需重新赋值
    selectedIds.value = new Set(selectedIds.value)
    lastClickedItem.value = item
  }

  /** 全选/取消全选当前列表 */
  function selectAll() {
    const list = unref(items)
    if (selectedIds.value.size === list.length) {
      selectedIds.value = new Set()
    } else {
      selectedIds.value = new Set(list.map(getId))
    }
  }

  /** 反选 */
  function invertSelection() {
    const list = unref(items)
    const newSet = new Set<string>()
    list.forEach(item => {
      if (!selectedIds.value.has(getId(item))) {
        newSet.add(getId(item))
      }
    })
    selectedIds.value = newSet
  }

  /** 清空选中（退出多选状态） */
  function clearSelection() {
    selectedIds.value = new Set()
    lastClickedItem.value = null
  }

  /** 判断某项是否被选中 */
  function checkSelected(id: string): boolean {
    return selectedIds.value.has(id)
  }

  /** 获取选中的项列表 */
  function getSelectedItems(): T[] {
    const list = unref(items)
    return list.filter(item => selectedIds.value.has(getId(item)))
  }

  /** ESC 清空选中 */
  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape' && selectedIds.value.size > 0) {
      clearSelection()
    }
  }

  return {
    // 状态
    selectedIds,
    batchProcessing,
    // computed
    hasSelection,
    selectedCount,
    // 操作
    toggle,
    selectAll,
    invertSelection,
    clearSelection,
    checkSelected,
    getSelectedItems,
    handleKeydown,
  }
}
