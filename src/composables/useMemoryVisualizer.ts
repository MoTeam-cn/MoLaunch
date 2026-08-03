/**
 * 内存可视化 composable：系统内存 1s 轮询 + 6 个可视化 computed + 自动分配
 *
 * applyAutoMemory 按可用内存 75% 分配（上限 8GB，下限 512MB）；
 * 消除 SettingsLaunch.vue 与 MemorySection.vue 的重复逻辑。
 */
import { ref, computed } from 'vue'
import type { Ref, ComputedRef } from 'vue'
import * as tauri from '@/utils/tauri'
import { usePolling } from '@/composables/usePolling'

interface SystemMemory {
  total: number
  available: number
  usage_percent: number
}

interface UseMemoryVisualizerReturn {
  /** 系统内存原始数据（轮询更新） */
  systemMemory: Ref<SystemMemory | null>
  /** 系统总内存（MB） */
  totalMemoryMB: ComputedRef<number>
  /** 系统已用内存（MB） */
  usedMemoryMB: ComputedRef<number>
  /** 游戏分配内存（MB，等于 maxMemory） */
  gameMemoryMB: ComputedRef<number>
  /** 其他占用内存（MB，= 总内存 - 已用 - 游戏） */
  otherMemoryMB: ComputedRef<number>
  /** 已用内存占比（%） */
  usedPercent: ComputedRef<number>
  /** 游戏内存占比（%） */
  gamePercent: ComputedRef<number>
  /** 按可用内存的 75% 自动分配游戏内存（上限 8GB，下限 512MB） */
  applyAutoMemory: () => void
  /** 启动 1 秒间隔的内存轮询 */
  startMemoryPolling: () => void
  /** 停止内存轮询 */
  stopMemoryPolling: () => void
}

/**
 * 封装系统内存轮询 + 内存可视化计算属性
 *
 * @param maxMemoryRef 最大内存的 ref（可写，applyAutoMemory 会修改它）
 * @param minMemoryRef 最小内存的 ref（可写，applyAutoMemory 会修改它）
 */
export function useMemoryVisualizer(
  maxMemoryRef: Ref<number>,
  minMemoryRef: Ref<number>,
): UseMemoryVisualizerReturn {
  const systemMemory = ref<SystemMemory | null>(null)

  const { start: startMemoryPolling, stop: stopMemoryPolling } = usePolling(async () => {
    systemMemory.value = await tauri.getSystemMemory()
  }, 1000)

  const totalMemoryMB = computed(() =>
    systemMemory.value ? Math.round(systemMemory.value.total / 1024 / 1024) : 0,
  )
  const usedMemoryMB = computed(() =>
    systemMemory.value
      ? Math.round(
          (systemMemory.value.total - systemMemory.value.available) / 1024 / 1024,
        )
      : 0,
  )
  const gameMemoryMB = computed(() => maxMemoryRef.value)
  const otherMemoryMB = computed(() =>
    Math.max(0, totalMemoryMB.value - usedMemoryMB.value - gameMemoryMB.value),
  )

  const usedPercent = computed(() =>
    totalMemoryMB.value > 0 ? (usedMemoryMB.value / totalMemoryMB.value) * 100 : 0,
  )
  const gamePercent = computed(() =>
    totalMemoryMB.value > 0 ? (gameMemoryMB.value / totalMemoryMB.value) * 100 : 0,
  )

  function applyAutoMemory() {
    if (!systemMemory.value) return
    const availableMB = systemMemory.value.available / 1024 / 1024
    const suggested = Math.min(Math.round(availableMB * 0.75), 8192)
    maxMemoryRef.value = Math.max(suggested, 512)
    minMemoryRef.value = Math.round(maxMemoryRef.value / 2)
  }

  return {
    systemMemory,
    totalMemoryMB,
    usedMemoryMB,
    gameMemoryMB,
    otherMemoryMB,
    usedPercent,
    gamePercent,
    applyAutoMemory,
    startMemoryPolling,
    stopMemoryPolling,
  }
}
