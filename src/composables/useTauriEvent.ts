/**
 * Tauri 事件监听 composable
 *
 * 统一封装 `listen`/`unlisten` 样板：自动管理 unlisten 句柄、onUnmounted 自动清理。
 *
 * 参考 `useCommunityDownload.ts` 和 `JavaDownloadBar.vue` 的模式提取。
 *
 * 竞态保护：Tauri 2.x 的 `_unlisten` 是"先同步删前端 callback、再异步通知 Rust 删 listener"，
 * 如果 `listen` 的 await 期间组件已卸载，`stop()` 看到 `unlisten === null` 直接返回，
 * listen 完成后注册的 listener 永远不会被取消 → listener 泄漏 + Rust 后台 task 持续 emit
 * 打到已删除的 callback id 触发 "Couldn't find callback id xxx" 警告。
 * 这里用 `isMounted` 标志确保 await 期间卸载时立即 unlisten 新拿到的句柄。
 *
 * @example
 * const { start, stop } = useTauriEvent<MyPayload>('my-event', (payload) => {
 *   console.log('收到:', payload)
 * })
 * start() // 在 onMounted 中启动
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
