/**
 * SDK 系统域
 *
 * 通过 system_manager 查询系统内存与缓存统计。
 */
import { SYSTEM_ACTIONS, systemManager } from '@/utils/api/system-manager'
import type { CacheStatsResult } from './sdk-types'

/** 读取系统内存信息 */
export async function getSystemMemory(): Promise<{
  total: number
  used: number
  available: number
  usage_percent: number
}> {
  return systemManager<{
    total: number
    used: number
    available: number
    usage_percent: number
  }>(SYSTEM_ACTIONS.GET_SYSTEM_MEMORY)
}

/** 读取缓存统计信息 */
export async function getCacheStats(): Promise<CacheStatsResult> {
  return systemManager<CacheStatsResult>(SYSTEM_ACTIONS.GET_CACHE_STATS)
}