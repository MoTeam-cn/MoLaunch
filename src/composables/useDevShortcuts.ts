/**
 * 开发者页面独占快捷键 composable
 *
 * 设计目标：
 * - 仅在 SettingsDeveloper.vue 内 onMounted 时绑定，onUnmounted 自动解绑
 * - 在 capture 阶段调用 `stopImmediatePropagation` 抢先消费事件，绕过
 *   `useDevToolsGuard` 的全局防护
 *
 * 当前绑定快捷键：
 * - Ctrl/Cmd + Shift + D：切换 DevTools 打开/关闭
 * - Alt + 1~6：切换开发者页面子页签
 *   （1=实验性 / 2=DevTools / 3=证书 / 4=日志 / 5=存储 / 6=系统信息）
 *
 * 安全约束：
 * - 切换 DevTools 仍需后端 `require_dev_mode()` 校验通过，普通用户即使
 *   误入开发者页面也无法触发（侧边菜单在未开启开发者模式时不显示）
 * - 快捷键仅在该页面组件存活时生效，离开页面自动失效
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

    // Alt + 1~6：切换子页签
    if (alt && !ctrl && !shift && /^[1-6]$/.test(key)) {
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
