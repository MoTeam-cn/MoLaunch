/**
 * 轮询 composable：封装 setInterval/clearInterval 样板
 *
 * onUnmounted 自动清理；组件卸载后禁止再启动，避免异步回调注册的 interval 泄漏。
 */
import { onUnmounted } from 'vue'

export function usePolling(
  callback: () => void | Promise<void>,
  intervalMs: number,
) {
  let timer: ReturnType<typeof setInterval> | null = null
  // 组件卸载后禁止再启动：防止异步 onLoad 完成时（组件已卸载）
  // 调用 start() 注册的 interval 永远不会被清理（泄漏 IPC 轮询）
  let unmounted = false

  function start() {
    if (timer || unmounted) return
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

  onUnmounted(() => {
    unmounted = true
    stop()
  })

  return { start, stop }
}
