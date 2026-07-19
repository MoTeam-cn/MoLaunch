/**
 * Fabric API 版本信息查询
 *
 * 用于 LoaderSelect 页面展示"将自动安装哪个 Fabric API 版本"。
 * 后端在 install_merged 时已自动安装最新版，此处仅做信息展示。
 *
 * 使用方式：
 *   const mcVersionRef = computed(() => props.mcVersion)
 *   const { fabricApiState, fabricApiLatest, fabricApiError, retry } = useFabricApi(mcVersionRef, selectedFabric)
 *
 * 选择 Fabric Loader（selected 非 null）后自动触发查询；
 * 出错时调用 retry() 可重置状态并重新查询。
 */
import { ref, watch, type Ref } from 'vue'
import { listFabricApiVersions, type FabricApiVersion } from '@/utils/api/loader'

export type FabricApiState = 'idle' | 'loading' | 'success' | 'empty' | 'error'

export function useFabricApi(
  mcVersion: Ref<string>,
  selected: Ref<string | null>,
) {
  const fabricApiState = ref<FabricApiState>('idle')
  const fabricApiLatest = ref<FabricApiVersion | null>(null)
  const fabricApiError = ref<string>('')

  async function fetchFabricApi() {
    // 正在查询中，跳过
    if (fabricApiState.value === 'loading') return
    // 已成功查询过（有数据或确认无兼容版本），不重复查询；出错则允许重试
    if (fabricApiState.value === 'success' || fabricApiState.value === 'empty') return

    fabricApiState.value = 'loading'
    try {
      const versions = await listFabricApiVersions(mcVersion.value)
      if (versions.length > 0) {
        // 列表已按发布日期降序排序，取第一个即最新版
        fabricApiLatest.value = versions[0]
        fabricApiState.value = 'success'
      } else {
        fabricApiLatest.value = null
        fabricApiState.value = 'empty'
      }
      fabricApiError.value = ''
    } catch (e: any) {
      console.error('Failed to load Fabric API versions:', e)
      fabricApiError.value = typeof e === 'string' ? e : (e?.message || String(e))
      fabricApiLatest.value = null
      fabricApiState.value = 'error'
    }
  }

  /** 重置状态并重新查询（供 error 状态下的"重试"按钮调用） */
  function retry() {
    fabricApiState.value = 'idle'
    fetchFabricApi()
  }

  // 选择 Fabric Loader 后触发查询 Fabric API 版本信息
  watch(() => selected.value, (newVal) => {
    if (newVal) fetchFabricApi()
  })

  return {
    fabricApiState,
    fabricApiLatest,
    fabricApiError,
    fetchFabricApi,
    retry,
  }
}
