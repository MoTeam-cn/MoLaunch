/**
 * 加载器兼容性判断 composable
 *
 * 从 LoaderSelect.vue 抽离，封装：
 * - MC 版本类型判断（snapshot/fool/ancient + showXxx 标志）
 * - 加载器兼容性检查（互斥规则 + 禁用判断 + 选中判断）
 */
import { computed, type Ref } from 'vue'
import { useVersionStore } from '@/stores/version'

interface LoaderSelection {
  forge: Ref<string | null>
  neoforge: Ref<string | null>
  fabric: Ref<string | null>
  optifine: Ref<string | null>
  liteloader: Ref<string | null>
}

export function useLoaderCompatibility(
  mcVersion: Ref<string>,
  selected: LoaderSelection,
) {
  const versionStore = useVersionStore()

  // —— MC 版本数值化（用于数值比较） ——
  const mcNum = computed(() => {
    const parts = mcVersion.value.split('.')
    return (parseInt(parts[0]) || 0) * 10000 + (parseInt(parts[1]) || 0) * 100 + (parseInt(parts[2]) || 0)
  })

  const versionInfo = computed(() => versionStore.getVersionById(mcVersion.value))
  const isSnapshot = computed(() => versionInfo.value?.version_type === 'snapshot')
  const isFool = computed(() => versionInfo.value?.version_type === 'fool')
  const isAncient = computed(() => {
    const type = versionInfo.value?.version_type
    return type === 'old_beta' || type === 'old_alpha' || mcNum.value < 10000
  })

  const showForge = computed(() => !isSnapshot.value && !isAncient.value && !isFool.value && mcNum.value >= 10501)
  const showNeoforge = computed(() => !isSnapshot.value && !isAncient.value && !isFool.value && mcNum.value >= 12001)
  const showFabric = computed(() => !isAncient.value && !isFool.value && mcNum.value > 11300)
  const showLiteloader = computed(() => !isSnapshot.value && !isAncient.value && !isFool.value && mcNum.value <= 11202)
  const showOptifine = computed(() => !isAncient.value && !isFool.value)

  // —— 兼容性检查 ——
  function getLoaderError(loader: string): string | null {
    if (loader === 'forge') {
      if (selected.fabric.value) return '与 Fabric 不兼容'
      if (selected.neoforge.value) return '与 NeoForge 不兼容'
    }
    if (loader === 'neoforge') {
      if (selected.forge.value) return '与 Forge 不兼容'
      if (selected.fabric.value) return '与 Fabric 不兼容'
      if (selected.optifine.value) return '与 OptiFine 不兼容'
    }
    if (loader === 'fabric') {
      if (selected.forge.value) return '与 Forge 不兼容'
      if (selected.neoforge.value) return '与 NeoForge 不兼容'
    }
    if (loader === 'optifine') {
      if (selected.neoforge.value) return '与 NeoForge 不兼容'
    }
    return null
  }

  function isLoaderDisabled(loader: string): boolean {
    return getLoaderError(loader) !== null && !isLoaderSelected(loader)
  }

  function isLoaderSelected(loader: string): boolean {
    if (loader === 'forge') return !!selected.forge.value
    if (loader === 'neoforge') return !!selected.neoforge.value
    if (loader === 'fabric') return !!selected.fabric.value
    if (loader === 'optifine') return !!selected.optifine.value
    if (loader === 'liteloader') return !!selected.liteloader.value
    return false
  }

  return {
    mcNum,
    versionInfo,
    isSnapshot,
    isFool,
    isAncient,
    showForge,
    showNeoforge,
    showFabric,
    showLiteloader,
    showOptifine,
    getLoaderError,
    isLoaderDisabled,
    isLoaderSelected,
  }
}
