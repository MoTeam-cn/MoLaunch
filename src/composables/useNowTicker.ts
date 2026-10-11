/**
 * 定时刷新的响应式当前时间戳（默认 200ms）
 *
 * 用于下载进度的「伪进度」补丁：后端 ticker 到 95% 后卡住，
 * 前端基于 now 继续让小数点上涨到 99.9%。
 * 组件卸载时自动清理定时器，调用方无需自己管理。
 */
import { onUnmounted, ref } from 'vue'

export function useNowTicker(intervalMs = 200) {
  const now = ref(Date.now())
  const timer = setInterval(() => {
    now.value = Date.now()
  }, intervalMs)
  onUnmounted(() => clearInterval(timer))
  return now
}