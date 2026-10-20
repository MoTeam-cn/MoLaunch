/**
 * 地区判定 composable（下载限制）
 *
 * 后端 utils::region 结合机器环境、网络出口与官网下发的下载策略判定；
 * 结果进程内缓存，页面并发调用共享同一次 IPC 请求。
 */
import { computed, ref } from 'vue'
import { getRegionInfo, setDownloadRestrictionBypass } from '@/utils/api/system'
import type { RegionInfo } from '@/types/region'

/** 最近一次判定结果（null = 尚未判定） */
const regionInfo = ref<RegionInfo | null>(null)
/** 判定请求是否进行中 */
const loading = ref(false)
/** 进行中的请求（并发调用共享同一 Promise） */
let pending: Promise<RegionInfo | null> | null = null
/** 本次运行是否已弹过限制提示（切换页面不重复弹） */
const noticeShown = ref(false)

/** 是否命中下载限制（已扣除开发者模式临时绕过） */
const isRestricted = computed(() => regionInfo.value?.isRestricted === true)
/** 官网下发的提示文案（未判定时为 null） */
const notice = computed(() => regionInfo.value?.notice ?? null)
/** 是否已临时关闭限制（仅本次运行有效） */
const bypassed = computed(() => regionInfo.value?.bypassed === true)
/** 是否提供「临时关闭限制」入口：策略允许 + 开发者模式开启 + 当前受限 */
const canBypass = computed(
  () =>
    regionInfo.value?.isRestricted === true &&
    regionInfo.value?.allowDeveloperModeBypass === true &&
    regionInfo.value?.developerMode === true,
)

/** 标记限制提示已展示 */
export function markNoticeShown(): void {
  noticeShown.value = true
}

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

/**
 * 开发者模式临时关闭 / 恢复下载限制
 *
 * 仅当官网策略允许且启动器处于开发者模式时生效；失败时抛出后端错误文案。
 */
export async function setBypass(enabled: boolean): Promise<void> {
  regionInfo.value = await setDownloadRestrictionBypass(enabled)
}

/** 地区判定状态（模块级单例，跨页面共享） */
export function useRegion() {
  return {
    regionInfo,
    loading,
    isRestricted,
    notice,
    bypassed,
    canBypass,
    noticeShown,
    markNoticeShown,
    loadRegionInfo,
    setBypass,
  }
}
