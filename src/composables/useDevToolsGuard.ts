/**
 * 全局快捷键防护 composable（App.vue 挂载，全局生效）
 *
 * 禁用 F 键 / Ctrl+字母 / Ctrl+Shift+字母 / Alt+字母（保留 Ctrl+C/V/X/Z/Y/A 编辑键），
 * 拦截右键菜单与拖拽开窗；仅「提升门槛」的初级防护，真正的开发者模式校验在后端 require_dev_mode()。
 */

import { onMounted, onUnmounted } from 'vue'

/** 允许的编辑快捷键（Ctrl/Cmd + 字母），不影响输入框使用 */
const ALLOWED_EDIT_KEYS = new Set(['c', 'v', 'x', 'z', 'y', 'a'])

/** 拦截右键菜单 */
function onContextMenu(e: MouseEvent) {
  e.preventDefault()
}

/**
 * 拦截所有非编辑类快捷键
 *
 * 拦截范围：
 * - F1~F12：浏览器/系统功能键（F1 帮助、F3 搜索、F5 刷新、F11 全屏、F12 DevTools 等）
 * - Ctrl/Cmd + 字母（除 c/v/x/z/y/a 编辑键外）：浏览器快捷键（Ctrl+O 打开、Ctrl+S 保存等）
 * - Ctrl/Cmd + Shift + 字母：浏览器/DevTools 快捷键（Ctrl+Shift+I/J/C/K DevTools、Ctrl+Shift+D 书签等）
 * - Ctrl/Cmd + U：查看源代码
 * - Alt + 字母：避免激活菜单栏（Alt+F 文件菜单等）
 *
 * 保留：
 * - Ctrl+C/V/X/Z/Y/A：文本编辑键，在 input/textarea/contenteditable 中正常工作
 * - 不带修饰键的字母/数字键：正常文本输入
 * - 方向键、退格、删除等编辑键
 *
 * 兼容开发者页面独占快捷键：
 * - `useDevShortcuts` 在 capture 阶段调用 `stopImmediatePropagation` 抢先消费
 * - 本监听器在同一 capture 阶段被触发时事件已被停止传播，不会误拦截
 */
function onKeyDown(e: KeyboardEvent) {
  const ctrl = e.ctrlKey || e.metaKey
  const shift = e.shiftKey
  const alt = e.altKey
  const key = e.key.toLowerCase()

  // F1~F12 全部拦截
  if (/^f([1-9]|1[0-2])$/.test(e.key.toLowerCase())) {
    e.preventDefault()
    e.stopPropagation()
    return
  }

  // Ctrl/Cmd + Shift + 任意字母/数字：全部拦截
  if (ctrl && shift && /^[a-z0-9]$/.test(key)) {
    e.preventDefault()
    e.stopPropagation()
    return
  }

  // Ctrl/Cmd + Alt + 任意字母：拦截（避免 AltGr 等组合触发系统快捷键）
  if (ctrl && alt && /^[a-z]$/.test(key)) {
    e.preventDefault()
    e.stopPropagation()
    return
  }

  // Ctrl/Cmd + 字母：除允许的编辑键外全部拦截
  if (ctrl && !shift && !alt && /^[a-z]$/.test(key) && !ALLOWED_EDIT_KEYS.has(key)) {
    e.preventDefault()
    e.stopPropagation()
    return
  }

  // Ctrl/Cmd + U（查看源代码，U 不在编辑键白名单内，已被上面规则拦截，此处显式列出便于阅读）
  if (ctrl && !shift && !alt && key === 'u') {
    e.preventDefault()
    e.stopPropagation()
    return
  }

  // Alt + 字母：拦截（避免激活菜单栏，Edge/Chrome 在 Windows 下会触发）
  if (alt && !ctrl && !shift && /^[a-z]$/.test(key)) {
    e.preventDefault()
    e.stopPropagation()
    return
  }
}

/** 拦截拖拽新窗口打开（部分浏览器 Alt+Click 拖拽链接触发 devtools） */
function onDragStart(e: DragEvent) {
  // 仅阻止链接拖拽到新窗口场景，不影响业务拖拽（业务拖拽使用 dataTransfer 自定义数据）
  if (!e.dataTransfer || e.dataTransfer.types.length === 0) {
    e.preventDefault()
  }
}

export function useDevToolsGuard() {
  onMounted(() => {
    document.addEventListener('contextmenu', onContextMenu, { capture: true })
    document.addEventListener('keydown', onKeyDown, { capture: true })
    document.addEventListener('dragstart', onDragStart, { capture: true })
  })

  onUnmounted(() => {
    document.removeEventListener('contextmenu', onContextMenu, { capture: true } as EventListenerOptions)
    document.removeEventListener('keydown', onKeyDown, { capture: true } as EventListenerOptions)
    document.removeEventListener('dragstart', onDragStart, { capture: true } as EventListenerOptions)
  })
}
