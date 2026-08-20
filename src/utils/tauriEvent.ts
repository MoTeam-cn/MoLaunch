/**
 * Tauri 事件监听工具：封装 listen/unlisten 样板，组件内 onUnmounted 自动清理
 *
 * 竞态保护：listen await 期间组件已卸载时立即 unlisten 新句柄，避免 Tauri 2.x
 * 异步删 listener 导致的泄漏与 "Couldn't find callback id" 警告。
 * 供 composables 与 stores 共同引用（stores 不反向依赖 composables）。
 */
import { onUnmounted } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

export function useTauriEvent<T>(
  eventName: string,
  handler: (payload: T) => void,
) {
  let unlisten: UnlistenFn | null = null
  let isMounted = true

  async function start() {
    if (unlisten) return
    const unlistenFn = await listen<T>(eventName, (event) => {
      if (isMounted) handler(event.payload)
    })
    // await 期间组件已卸载：立即 unlisten 刚拿到的句柄，避免泄漏
    if (!isMounted) {
      unlistenFn()
      return
    }
    unlisten = unlistenFn
  }

  function stop() {
    isMounted = false
    if (unlisten) {
      unlisten()
      unlisten = null
    }
  }

  onUnmounted(() => stop())

  return { start, stop }
}