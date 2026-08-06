/**
 * 点击外部关闭弹层 / 下拉菜单的公共工具
 *
 * 项目内 Select / ColorPicker 原各自内联实现 document click 监听，现提取为
 * 统一封装，避免重复。用法：
 *
 * ```ts
 * const rootRef = ref<HTMLElement | null>(null)
 * onClickOutside(rootRef, () => (menuOpen.value = false))
 * ```
 *
 * @param elRef 容器 ref，点击其内部（含子元素）不触发回调
 * @param handler 外部点击回调
 * @param extraRefs 额外视为"内部"的容器 ref（如 Teleport 到 body 的弹层）
 */
import { onMounted, onUnmounted, type Ref } from 'vue'

export function onClickOutside(
  elRef: Ref<HTMLElement | null>,
  handler: (e: MouseEvent) => void,
  extraRefs: Ref<HTMLElement | null>[] = [],
) {
  function handle(e: MouseEvent) {
    const target = e.target as Node | null
    if (!target) return
    if (elRef.value && elRef.value.contains(target)) return
    for (const r of extraRefs) {
      if (r.value && r.value.contains(target)) return
    }
    handler(e)
  }

  onMounted(() => document.addEventListener('click', handle))
  onUnmounted(() => document.removeEventListener('click', handle))
}

/** 按 ESC 键关闭弹层 / 下拉菜单（始终注册，handler 内部自行判断是否处于打开状态） */
export function onEscape(handler: () => void) {
  function handle(e: KeyboardEvent) {
    if (e.key === 'Escape') handler()
  }
  onMounted(() => document.addEventListener('keydown', handle))
  onUnmounted(() => document.removeEventListener('keydown', handle))
}
