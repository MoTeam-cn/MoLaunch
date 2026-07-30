/**
 * DevTools 防护与右键菜单禁用 composable
 *
 * 在 onMounted 中注册事件监听，onUnmounted 自动清理：
 * 1. 禁用浏览器右键菜单（防「检查元素」）
 * 2. 拦截 DevTools 快捷键：F12 / Ctrl+Shift+I / Ctrl+Shift+J / Ctrl+U / Ctrl+Shift+C
 * 3. 拦截 Ctrl+Shift+K（部分浏览器控制台快捷键）
 *
 * 安全约束：
 * - 这些只是「提升门槛」的初级防护，无法阻止已开启 devtools 的开发者操作 DOM
 * - 真正的防护在后端：devtools 调用 require_dev_mode() 校验
 * - 水印组件本身使用 pointer-events: none + 不可见隐写字段，DOM 移除不影响追溯
 *
 * 在 App.vue 中调用 `useDevToolsGuard()` 即可全局生效。
 */

import { onMounted, onUnmounted } from 'vue'

/** 拦截右键菜单 */
function onContextMenu(e: MouseEvent) {
  e.preventDefault()
}

/**
 * 拦截 DevTools 相关快捷键
 *
 * 仅拦截「可能用于打开 devtools」的快捷键，不影响正常编辑快捷键：
 * - F12：IE/Chrome/Edge/Firefox 通用的 devtools 快捷键
 * - Ctrl+Shift+I / Cmd+Opt+I：devtools
 * - Ctrl+Shift+J / Cmd+Opt+J：Console
 * - Ctrl+Shift+C / Cmd+Opt+C：Element Picker
 * - Ctrl+U：查看源代码（WebView2 内部已禁用，这里兜底）
 * - Ctrl+Shift+K / Cmd+Opt+K：Firefox Console
 */
function onKeyDown(e: KeyboardEvent) {
  const ctrl = e.ctrlKey || e.metaKey
  const shift = e.shiftKey
  const alt = e.altKey
  const key = e.key.toLowerCase()

  // F12
  if (e.key === 'F12') {
    e.preventDefault()
    e.stopPropagation()
    return
  }
  // Ctrl+Shift+I / Ctrl+Shift+J / Ctrl+Shift+C / Ctrl+Shift+K
  if (ctrl && shift && (key === 'i' || key === 'j' || key === 'c' || key === 'k')) {
    e.preventDefault()
    e.stopPropagation()
    return
  }
  // Cmd+Opt+I / Cmd+Opt+J / Cmd+Opt+C / Cmd+Opt+K（macOS）
  if (e.metaKey && alt && (key === 'i' || key === 'j' || key === 'c' || key === 'k')) {
    e.preventDefault()
    e.stopPropagation()
    return
  }
  // Ctrl+U（查看源代码）
  if (ctrl && !shift && key === 'u') {
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
