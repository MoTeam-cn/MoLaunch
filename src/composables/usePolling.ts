/**
 * 轮询 composable
 *
 * 统一封装 `setInterval`/`clearInterval` 样板：onUnmounted 自动清理、防止重复启动。
 *
 * 参考 `MemorySection.vue`、`SettingsLaunch.vue`、`useDownloadPolling.ts` 的模式提取。
 *
 * @example
 * // 每 1 秒拉取系统内存
 * const { start, stop } = usePolling(async () => {
 *   memory.value = await tauri.getSystemMemory()
 * }, 1000)
 * onMounted(() => start())
 */
import { onUnmounted } from 'vue'

export function usePolling(
  callback: () => void | Promise<void>,
  intervalMs: number,
) {
  let timer: ReturnType<typeof setInterval> | null = null

  function start() {
    if (timer) return
    timer = setInterval(() => {
      // 异步回调内的错误吞掉（与原 MemorySection.vue 一致：静默失败）
      Promise.resolve(callback()).catch(() => { /* ignore */ })
    }, intervalMs)
  }

  function stop() {
    if (timer) {
      clearInterval(timer)
      timer = null
    }
  }

  onUnmounted(() => stop())

  return { start, stop }
}
