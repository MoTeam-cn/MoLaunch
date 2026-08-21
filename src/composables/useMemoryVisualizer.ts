/**
 * 内存可视化 composable：系统内存事件推送订阅 + 6 个可视化 computed + 自动分配
 *
 * 数据来源改为后端 1s emit 的 `memory-info` 事件（前端订阅/退订控制，无页面打开时后端零开销）；
 * applyAutoMemory 按可用内存 75% 分配（上限 8GB，下限 512MB）；
 * 消除 SettingsLaunch.vue 与 MemorySection.vue 的重复逻辑。
 */
import { ref, computed, onUnmounted } from 'vue'
import type { Ref, ComputedRef } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import * as tauri from '@/utils/tauri'

/** 后端内存推送事件名（与 src-tauri/src/commands/system/memory_push.rs 约定一致） */
const MEMORY_PUSH_EVENT = 'memory-info'

interface SystemMemory {
  total: number
  used: number
  available: number
  usage_percent: number
}

interface UseMemoryVisualizerReturn {
  /** 系统内存原始数据（事件推送更新） */
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
  /** 订阅后端内存推送并监听 memory-info 事件（页面挂载时调用，幂等） */
  startMemoryPolling: () => void
  /** 退订内存推送并移除事件监听（页面卸载时调用，组件卸载自动执行） */
  stopMemoryPolling: () => void
}

/**
 * 封装系统内存事件订阅 + 内存可视化计算属性
 *
 * @param maxMemoryRef 最大内存的 ref（可写，applyAutoMemory 会修改它）
 * @param minMemoryRef 最小内存的 ref（可写，applyAutoMemory 会修改它）
 */
export function useMemoryVisualizer(
  maxMemoryRef: Ref<number>,
  minMemoryRef: Ref<number>,
): UseMemoryVisualizerReturn {
  const systemMemory = ref<SystemMemory | null>(null)

  // 订阅/退订状态（与后端 memory_subscribe / memory_unsubscribe 严格配对）
  let unlisten: UnlistenFn | null = null
  let subscribed = false

  /** 订阅后端内存推送并监听 memory-info 事件（页面挂载时调用） */
  async function startMemoryPolling() {
    if (subscribed) return
    subscribed = true
    try {
      await tauri.memorySubscribe()
    } catch {
      // 后端不可用时静默降级（保留单次数据，不再刷新）
    }
    if (!unlisten) {
      unlisten = await listen<SystemMemory>(MEMORY_PUSH_EVENT, (e) => {
        systemMemory.value = e.payload
      }).catch(() => null)
    }
  }

  /** 退订内存推送并移除事件监听（页面卸载时调用） */
  async function stopMemoryPolling() {
    subscribed = false
    if (unlisten) {
      unlisten()
      unlisten = null
    }
    try {
      await tauri.memoryUnsubscribe()
    } catch {
      // 忽略：后端不可用时无需退订
    }
  }

  // 组件卸载自动退订（与 usePolling 原行为一致，调用方无需手动 stop）
  onUnmounted(() => {
    stopMemoryPolling()
  })

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
