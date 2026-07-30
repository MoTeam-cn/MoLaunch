/**
 * 水印解锁状态管理 composable
 *
 * 设计目标：
 * - 提供水印「解锁隐藏」状态管理，必须 DevTools 已打开才能解锁
 * - 状态纯内存（不持久化），刷新页面/重启应用即恢复显示，防止外部持久关闭水印
 * - 隐藏后启动轮询检测 DevTools 状态，DevTools 关闭时自动恢复水印显示
 *   （用户原话："咋不轮询接口判断是否关闭devtools，然后自动切换水印显示了呢"）
 * - `unlocked` 为全局共享响应式 ref，DevToolsTab 按钮状态自动同步
 *
 * 安全约束：
 * - 解锁前提：后端 `is_devtools_open()` 返回 true（后端用 AtomicBool 维护，
 *   修复了 Tauri 在 WebView2 上始终返回 false 的 bug）
 * - 状态纯内存：刷新页面即恢复，攻击者无法通过 sessionStorage 持久关闭水印
 * - 轮询仅在 `unlocked=true` 时运行，DevTools 关闭即自动恢复并停止轮询
 * - 水印的追溯能力不依赖前端 DOM：屏印哈希已嵌入截图，水印被隐藏仍可追溯
 *
 * 使用方式：
 * ```ts
 * const { unlocked, hide, show, syncWithDevTools } = useWatermarkUnlock()
 * await hide()              // 隐藏水印（内部校验 DevTools 是否打开）
 * show()                    // 恢复水印显示
 * syncWithDevTools()        // 启动轮询（由 Watermark.vue onMounted 调用）
 * ```
 */

import { ref, watch } from 'vue'
import { isDevToolsOpen } from '@/utils/api/developer'

/**
 * 全局共享解锁状态（多个组件实例共用同一 ref）
 *
 * 纯内存状态：不写入 sessionStorage/localStorage，刷新页面即恢复 false。
 * 这样攻击者无法通过浏览器存储持久关闭水印，确保水印安全。
 */
const unlocked = ref(false)

/** 轮询定时器句柄 */
let pollTimer: ReturnType<typeof setInterval> | null = null

/** 轮询间隔（毫秒） */
const POLL_INTERVAL = 5000

/**
 * 启动 DevTools 状态轮询
 *
 * 仅在 `unlocked=true`（水印已隐藏）时轮询 `isDevToolsOpen()`，
 * 检测到 DevTools 关闭后自动恢复水印（`unlocked=false`）并停止轮询。
 *
 * 由 Watermark.vue onMounted 调用一次，内部通过 watch 自动管理轮询启停。
 */
function startPolling() {
  stopPolling()
  pollTimer = setInterval(async () => {
    if (!unlocked.value) {
      // 未隐藏水印时无需轮询，停止以节省 IPC 调用
      stopPolling()
      return
    }
    try {
      const open = await isDevToolsOpen()
      if (!open) {
        // DevTools 已关闭，自动恢复水印
        unlocked.value = false
        stopPolling()
      }
    } catch {
      // IPC 异常（如开发者模式被关闭），保守恢复水印
      unlocked.value = false
      stopPolling()
    }
  }, POLL_INTERVAL)
}

/** 停止轮询 */
function stopPolling() {
  if (pollTimer !== null) {
    clearInterval(pollTimer)
    pollTimer = null
  }
}

/**
 * 启动 DevTools 状态同步
 *
 * 由 Watermark.vue onMounted 调用。通过 watch 监听 `unlocked` 变化：
 * - `unlocked` 变 true（水印隐藏）时自动启动轮询
 * - `unlocked` 变 false（水印恢复）时自动停止轮询
 */
function syncWithDevTools() {
  watch(unlocked, (val) => {
    if (val) {
      startPolling()
    } else {
      stopPolling()
    }
  }, { immediate: true })
}

export function useWatermarkUnlock() {
  /**
   * 隐藏水印（解锁）
   *
   * 前置条件：DevTools 已打开。若未打开则抛出错误，调用方提示用户。
   * 隐藏后自动启动轮询，DevTools 关闭时自动恢复水印。
   */
  async function hide(): Promise<void> {
    const open = await isDevToolsOpen()
    if (!open) {
      throw new Error('DevTools 未打开，无法隐藏水印')
    }
    unlocked.value = true
  }

  /** 显示水印（重新锁定），由用户主动点击「恢复水印」按钮触发 */
  function show() {
    unlocked.value = false
  }

  return {
    unlocked,
    hide,
    show,
    syncWithDevTools,
  }
}
