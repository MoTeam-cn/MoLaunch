/**
 * 防抖保存 composable
 *
 * 统一各设置页的 scheduleSave / flushSave 防抖保存模式：
 * - scheduleSave: 重置定时器，到时间调用 flushFn
 * - flushSave: 立即执行 flushFn 并清定时器
 * - 组件作用域销毁（onUnmounted）时自动 flushSave + 清定时器，
 *   避免丢失用户最后一次调整
 */
import { onScopeDispose } from 'vue'

export function useDebouncedSave(
  flushFn: () => Promise<void> | void,
  delay = 800,
) {
  let timer: ReturnType<typeof setTimeout> | null = null

  function clearTimer() {
    if (timer) {
      clearTimeout(timer)
      timer = null
    }
  }

  /** 重置定时器，到时间调用 flushFn */
  function scheduleSave() {
    clearTimer()
    timer = setTimeout(() => {
      timer = null
      void flushFn()
    }, delay)
  }

  /** 立即执行 flushFn 并清定时器 */
  function flushSave() {
    clearTimer()
    void flushFn()
  }

  // 组件卸载时自动 flush，避免丢失最后一次调整
  onScopeDispose(() => {
    flushSave()
  })

  return { scheduleSave, flushSave }
}
