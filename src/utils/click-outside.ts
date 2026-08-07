/** 点击外部或按 ESC 关闭弹层 / 下拉菜单的公共工具。 */
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
