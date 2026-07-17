/**
 * Tauri 事件监听 composable
 *
 * 统一封装 `listen`/`unlisten` 样板：自动管理 unlisten 句柄、onUnmounted 自动清理。
 *
 * 参考 `useCommunityDownload.ts` 和 `JavaDownloadBar.vue` 的模式提取。
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

  async function start() {
    if (unlisten) return
    unlisten = await listen<T>(eventName, (event) => {
      handler(event.payload)
    })
  }

  function stop() {
    if (unlisten) {
      unlisten()
      unlisten = null
    }
  }

  onUnmounted(() => stop())

  return { start, stop }
}
