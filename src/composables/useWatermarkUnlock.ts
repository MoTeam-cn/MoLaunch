/**
 * 水印解锁状态管理 composable
 *
 * 设计目标：
 * - 提供水印「解锁隐藏」状态管理，必须 DevTools 已打开才能解锁
 * - 状态仅存于 sessionStorage（重启后恢复显示，防止永久隐藏水印）
 * - DevTools 关闭时自动恢复水印显示（轮询 isDevToolsOpen 检测）
 *
 * 安全约束：
 * - 解锁前提：后端 `is_devtools_open()` 返回 true（后端用 AtomicBool 维护，
 *   修复了 Tauri 在 WebView2 上始终返回 false 的 bug）
 * - 解锁状态非持久化：sessionStorage 关闭应用即清除
 * - DevTools 关闭自动恢复：水印不能在 DevTools 关闭后继续隐藏
 *
 * 使用方式：
 * ```ts
 * const { unlocked, hide, show, syncWithDevTools } = useWatermarkUnlock()
 * // 隐藏水印（内部自动校验 DevTools 是否打开）
 * await hide()
 * // 显示水印
 * show()
 * // 持续同步（onMounted 启动轮询，onUnmounted 清理）
 * syncWithDevTools()
 * ```
 */

import { ref, onMounted, onUnmounted } from 'vue'
import { isDevToolsOpen } from '@/utils/api/developer'

const STORAGE_KEY = 'molaunch.watermark.unlocked'

/** 全局共享解锁状态（多个组件实例共用同一 ref） */
const unlocked = ref(false)

/** 从 sessionStorage 读取初始状态 */
function loadFromStorage() {
  try {
    unlocked.value = sessionStorage.getItem(STORAGE_KEY) === '1'
  } catch {
    // sessionStorage 不可用（隐私模式等），忽略
    unlocked.value = false
  }
}

/** 写入 sessionStorage */
function saveToStorage(v: boolean) {
  try {
    if (v) {
      sessionStorage.setItem(STORAGE_KEY, '1')
    } else {
      sessionStorage.removeItem(STORAGE_KEY)
    }
  } catch {
    // 忽略写入失败
  }
}

// 模块加载时同步一次 sessionStorage 状态
loadFromStorage()

/** DevTools 轮询定时器 */
let pollTimer: ReturnType<typeof setInterval> | null = null

/** 启动 DevTools 状态轮询：解锁状态下若 DevTools 关闭则自动恢复水印 */
function startPolling() {
  stopPolling()
  pollTimer = setInterval(async () => {
    if (!unlocked.value) return
    // 解锁状态下检查 DevTools 是否仍打开
    try {
      const open = await isDevToolsOpen()
      if (!open) {
        // DevTools 已关闭，自动恢复水印显示
        unlocked.value = false
        saveToStorage(false)
      }
    } catch {
      // 查询失败（开发者模式被关闭等），保守恢复水印
      unlocked.value = false
      saveToStorage(false)
    }
  }, 5000)
}

function stopPolling() {
  if (pollTimer !== null) {
    clearInterval(pollTimer)
    pollTimer = null
  }
}

export function useWatermarkUnlock() {
  /**
   * 隐藏水印（解锁）
   *
   * 前置条件：DevTools 已打开。若未打开则抛出错误，调用方提示用户。
   */
  async function hide(): Promise<void> {
    const open = await isDevToolsOpen()
    if (!open) {
      throw new Error('DevTools 未打开，无法隐藏水印')
    }
    unlocked.value = true
    saveToStorage(true)
    // 启动轮询监控 DevTools 状态
    startPolling()
  }

  /** 显示水印（重新锁定） */
  function show() {
    unlocked.value = false
    saveToStorage(false)
    stopPolling()
  }

  /** 同步状态：onMounted 启动轮询，onUnmounted 停止 */
  function syncWithDevTools() {
    onMounted(() => {
      // 若启动时已处于解锁状态，恢复轮询
      if (unlocked.value) {
        startPolling()
      }
    })
    onUnmounted(() => {
      // 注意：此处仅停止当前组件的轮询生命周期管理
      // 全局 unlocked 状态保留（其他组件仍可读取），由水印组件持续监听
      // 不主动 stopPolling()，避免在多组件场景下误停止
    })
  }

  return {
    unlocked,
    hide,
    show,
    syncWithDevTools,
  }
}
