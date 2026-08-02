/**
 * 水印解锁状态管理 composable
 *
 * 仅当后端 is_devtools_open() 为 true 时可解锁隐藏；状态纯内存（刷新即恢复），
 * 隐藏后轮询 DevTools 状态，关闭时自动恢复水印。unlocked 为全局共享 ref，
 * 轮询仅在 unlocked=true 时运行。
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
