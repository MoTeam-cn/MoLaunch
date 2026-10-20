/**
 * 地区判定 composable（中国大陆下载限制）
 *
 * 后端 utils::region 双层判定（机器环境 + 网络出口），结果进程内缓存；
 * 页面并发调用共享同一次 IPC 请求，避免重复判定与重复请求网络。
 */
import { computed, ref } from 'vue'
import { getRegionInfo } from '@/utils/api/system'
import type { RegionInfo } from '@/types/region'

/** 最近一次判定结果（null = 尚未判定） */
const regionInfo = ref<RegionInfo | null>(null)
/** 判定请求是否进行中 */
const loading = ref(false)
/** 进行中的请求（并发调用共享同一 Promise） */
let pending: Promise<RegionInfo | null> | null = null

/** 是否命中中国大陆下载限制 */
const isRestricted = computed(() => regionInfo.value?.isRestricted === true)

/**
 * 拉取地区判定结果
 *
 * 已有结果时直接复用（force = true 时强制重新判定），
 * 判定失败不清空已有结果，返回 null 由调用方决定降级行为。
 */
export function loadRegionInfo(force = false): Promise<RegionInfo | null> {
  if (!force && regionInfo.value) return Promise.resolve(regionInfo.value)
  if (pending) return pending
  loading.value = true
  pending = getRegionInfo()
    .then((info) => {
      regionInfo.value = info
      return info
    })
    .catch((e: unknown) => {
      console.error('[Region] 地区判定失败', e)
      return null
    })
    .finally(() => {
      loading.value = false
      pending = null
    })
  return pending
}

/** 地区判定状态（模块级单例，跨页面共享） */
export function useRegion() {
  return { regionInfo, loading, isRestricted, loadRegionInfo }
}
