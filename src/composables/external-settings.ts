/**
 * 外部下载工具 - 高级设置切片
 *
 * 管理自定义 UA / 并发线程数 / 分片数 / 限速，并通过 localStorage 持久化。
 * 所有字段均为空时，后端使用全局下载配置（不传覆盖参数）。
 */
import { ref, watch } from 'vue'
import type { DownloadFileSettings } from '@/utils/api/tools'

const STORAGE_KEY = 'molaunch.external-download.settings'

interface SettingsState {
  userAgent: string
  maxThreads: number
  chunkCount: number
  maxSpeed: number
}

const DEFAULTS: SettingsState = {
  userAgent: '',
  maxThreads: 0,
  chunkCount: 0,
  maxSpeed: 0,
}

function loadSettings(): SettingsState {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    if (!raw) return { ...DEFAULTS }
    const parsed = JSON.parse(raw) as Partial<SettingsState>
    return {
      userAgent: typeof parsed.userAgent === 'string' ? parsed.userAgent : DEFAULTS.userAgent,
      maxThreads: Number.isFinite(parsed.maxThreads) ? (parsed.maxThreads as number) : DEFAULTS.maxThreads,
      chunkCount: Number.isFinite(parsed.chunkCount) ? (parsed.chunkCount as number) : DEFAULTS.chunkCount,
      maxSpeed: Number.isFinite(parsed.maxSpeed) ? (parsed.maxSpeed as number) : DEFAULTS.maxSpeed,
    }
  } catch {
    return { ...DEFAULTS }
  }
}

export function useExternalSettings() {
  const saved = loadSettings()

  const userAgent = ref(saved.userAgent)
  const maxThreads = ref(saved.maxThreads)
  const chunkCount = ref(saved.chunkCount)
  /** 限速（MB/s，界面显示单位） */
  const maxSpeedMB = ref(saved.maxSpeed)

  watch([userAgent, maxThreads, chunkCount, maxSpeedMB], () => {
    try {
      localStorage.setItem(
        STORAGE_KEY,
        JSON.stringify({
          userAgent: userAgent.value,
          maxThreads: maxThreads.value,
          chunkCount: chunkCount.value,
          maxSpeed: maxSpeedMB.value,
        }),
      )
    } catch {
      // localStorage 不可用时静默忽略
    }
  })

  function resetSettings() {
    userAgent.value = DEFAULTS.userAgent
    maxThreads.value = DEFAULTS.maxThreads
    chunkCount.value = DEFAULTS.chunkCount
    maxSpeedMB.value = DEFAULTS.maxSpeed
  }

  /** 构造传给后端的下载设置（空值省略，走全局配置） */
  function toDownloadSettings(): DownloadFileSettings {
    const settings: DownloadFileSettings = {}
    if (userAgent.value.trim()) settings.userAgent = userAgent.value.trim()
    if (maxThreads.value > 0) settings.maxThreads = Math.floor(maxThreads.value)
    if (chunkCount.value > 0) settings.chunkCount = Math.floor(chunkCount.value)
    if (maxSpeedMB.value > 0) settings.maxSpeed = Math.floor(maxSpeedMB.value * 1024 * 1024)
    return settings
  }

  return {
    userAgent,
    maxThreads,
    chunkCount,
    maxSpeedMB,
    resetSettings,
    toDownloadSettings,
  }
}
