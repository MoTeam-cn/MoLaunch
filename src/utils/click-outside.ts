/** 点击外部或按 ESC 关闭弹层 / 下拉菜单的公共工具。 */
import { onMounted, onUnmounted } from 'vue'

/** 按 ESC 键关闭弹层 / 下拉菜单（始终注册，handler 内部自行判断是否处于打开状态） */
export function onEscape(handler: () => void) {
  function handle(e: KeyboardEvent) {
    if (e.key === 'Escape') handler()
  }
  onMounted(() => document.addEventListener('keydown', handle))
  onUnmounted(() => document.removeEventListener('keydown', handle))
}
