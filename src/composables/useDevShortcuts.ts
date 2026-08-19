/**
 * 开发者页面独占快捷键 composable（仅 SettingsDeveloper.vue 挂载）
 *
 * capture 阶段 stopImmediatePropagation 抢占事件流，绕过 useDevToolsGuard 全局防护；
 * Ctrl/Cmd+Shift+D 切换 DevTools、Alt+1~9 切换子页签。DevTools 切换仍需后端 require_dev_mode() 校验。
 */

import { onMounted, onUnmounted } from 'vue'
import { closeDevTools, isDevToolsOpen, openDevTools } from '@/utils/api/developer'
import { toastError, toastInfo, toastSuccess } from '@/utils/toast'

interface DevShortcutsOptions {
  /** 切换子页签回调（Alt+1~6） */
  onSwitchTab: (index: number) => void
}

/** 切换 DevTools 打开/关闭 */
async function toggleDevTools() {
  try {
    const open = await isDevToolsOpen()
    if (open) {
      await closeDevTools()
      toastSuccess('DevTools 已关闭')
    } else {
      await openDevTools()
      toastSuccess('DevTools 已打开')
    }
  } catch (e) {
    toastError('DevTools 操作失败：' + e)
  }
}

function createHandler(opts: DevShortcutsOptions) {
  return (e: KeyboardEvent) => {
    const ctrl = e.ctrlKey || e.metaKey
    const shift = e.shiftKey
    const alt = e.altKey
    const key = e.key.toLowerCase()

    // Ctrl/Cmd + Shift + D：切换 DevTools
    if (ctrl && shift && key === 'd') {
      e.preventDefault()
      e.stopPropagation()
      e.stopImmediatePropagation()
      void toggleDevTools()
      return
    }

    // Alt + 1~9：切换子页签
    if (alt && !ctrl && !shift && /^[1-9]$/.test(key)) {
      e.preventDefault()
      e.stopPropagation()
      e.stopImmediatePropagation()
      opts.onSwitchTab(parseInt(key, 10) - 1)
      return
    }

    // 其他快捷键不消费，交由全局防护处理
    // 显式提示：开发者页面其他被拦截的快捷键无法触发，避免用户疑惑
    if (ctrl && shift && /^[a-z0-9]$/.test(key) && key !== 'd') {
      e.preventDefault()
      e.stopPropagation()
      e.stopImmediatePropagation()
      toastInfo(`快捷键 Ctrl+Shift+${key.toUpperCase()} 在开发者页面未绑定动作`)
      return
    }
  }
}

export function useDevShortcuts(opts: DevShortcutsOptions) {
  const handler = createHandler(opts)

  onMounted(() => {
    // 使用 capture 阶段，优先于 useDevToolsGuard 的全局防护消费事件
    document.addEventListener('keydown', handler, { capture: true })
  })

  onUnmounted(() => {
    document.removeEventListener('keydown', handler, { capture: true } as EventListenerOptions)
  })
}
