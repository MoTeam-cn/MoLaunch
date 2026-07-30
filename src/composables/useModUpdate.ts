/**
 * Mod 版本更新/更改逻辑 composable
 *
 * 从 ModUpdateDialog.vue 抽离，封装版本列表查询、过滤、选中、安装等全部状态与逻辑。
 * 调用方传入响应式 props 与 emit，即可获得所有派生状态与操作函数。
 */
import { ref, computed, watch } from 'vue'
import { getProjectVersions } from '@/utils/api/community'
import { updateMod, type ModInfo } from '@/utils/api/personalization'
import { versionChangeType, type VersionChangeType } from '@/utils/version'
import { formatBytes } from '@/utils/format'
import { toastSuccess, toastInfo } from '@/utils/toast'
import { showConfirm, showModal } from '@/utils/modal'
import { isCancelledError } from '@/utils/async'
import { useVersionStore } from '@/stores/version'
import type { ResourceVersion, Platform } from '@/types/community'

interface UseModUpdateProps {
  mod: ModInfo | null
  mcVersion: string
  versionId: string
  visible: boolean
}

/** 加载器名称转 flag（用于按加载器过滤版本列表） */
function loaderToFlag(loader: string): number {
  const flags: Record<string, number> = {
    forge: 1,
    liteloader: 2,
    fabric: 4,
    quilt: 8,
    neoforge: 16,
  }
  return flags[loader] || 0
}

/** 发布类型样式 */
export function releaseTypeClass(type: string): string {
  switch (type) {
    case 'Release': return 'bg-green-100 text-green-700'
    case 'Beta': return 'bg-blue-100 text-blue-700'
    case 'Alpha': return 'bg-yellow-100 text-yellow-700'
    default: return 'bg-gray-100 text-gray-600'
  }
}

export function useModUpdate(
  props: UseModUpdateProps,
  emit: (event: 'installed' | 'update:visible', ...args: any[]) => void,
) {
  const versionStore = useVersionStore()
  const loading = ref(false)
  const versions = ref<ResourceVersion[]>([])
  const error = ref('')
  const installing = ref(false)

  // 过滤器状态（用户不可切换，由当前整合包的 MC 版本和加载器自动确定）
  const selectedGameVersion = ref<string>('')
  const selectedLoader = ref<string>('')

  // 选中的版本
  const selectedVersionId = ref<string | null>(null)

  // 过滤后的版本列表（按当前整合包的 MC 版本 + 加载器自动筛选）
  const filteredVersions = computed(() => {
    let result = versions.value
    if (selectedGameVersion.value) {
      result = result.filter(v => v.game_versions.includes(selectedGameVersion.value))
    }
    if (selectedLoader.value) {
      const loaderNum = loaderToFlag(selectedLoader.value)
      if (loaderNum > 0) {
        result = result.filter(v => (v.mod_loaders & loaderNum) !== 0)
      }
    }
    return result
  })

  // 选中的版本对象
  const selectedVersion = computed(() =>
    filteredVersions.value.find(v => v.id === selectedVersionId.value) || null,
  )

  // 平台（优先 CurseForge，回退 Modrinth）
  const platform = computed<Platform | null>(() => {
    if (!props.mod?.project) return null
    return props.mod.project.platform
  })

  // 当前 mod 的加载器类型
  const modLoaderType = computed(() => props.mod?.loader_type || 'unknown')

  /**
   * 选中版本相对于当前 mod 版本的变化类型
   *
   * 使用语义化版本比较（而非字符串相等），正确识别升级/降级/同版本。
   * 当 mod.version 或 selectedVersion 为空时返回 'unknown'。
   */
  const versionChange = computed<VersionChangeType>(() => {
    if (!props.mod?.version || !selectedVersion.value?.version) return 'unknown'
    return versionChangeType(props.mod.version, selectedVersion.value.version)
  })

  // 查询版本列表
  async function loadVersions() {
    if (!props.mod?.project || !platform.value) {
      error.value = '此 Mod 没有关联的平台工程信息，无法查询版本'
      return
    }

    loading.value = true
    error.value = ''
    versions.value = []

    try {
      const result = await getProjectVersions(platform.value, props.mod.project.id)
      versions.value = result

      // 自动用当前整合包的 MC 版本和加载器过滤（用户不可切换）
      if (props.mcVersion) {
        selectedGameVersion.value = props.mcVersion
      }
      if (modLoaderType.value !== 'unknown') {
        selectedLoader.value = modLoaderType.value
      }

      // 自动选中第一个（最新版本）
      if (filteredVersions.value.length > 0) {
        selectedVersionId.value = filteredVersions.value[0].id
      }
    } catch (e: any) {
      error.value = typeof e === 'string' ? e : (e?.message || String(e))
    } finally {
      loading.value = false
    }
  }

  // 安装选中的版本（使用 showConfirm 回调模式）
  function installSelected() {
    if (!selectedVersion.value || !props.mod) return

    const version = selectedVersion.value
    const mod = props.mod
    const oldFileName = mod.file_name

    showConfirm(
      '确认安装',
      `将下载 ${version.version} 并替换当前文件 ${oldFileName}。\n\n新文件名：${version.file_name}\n大小：${formatBytes(version.size)}`,
      async () => {
        installing.value = true
        try {
          // 原子化更新：后端封装"下载新版本 → 删旧版本"为单一 IPC
          // 下载失败时不删旧文件（后端保证原子性）
          // 进度通过 DownloadSession 统一推送（分组"Mod 更新"）
          versionStore.startDownload(version.file_name)
          toastInfo(`开始下载: ${version.file_name}`)
          await updateMod(
            props.versionId,
            oldFileName,
            version.download_url,
            version.file_name,
            version.size,
          )

          toastSuccess(`已安装 ${version.version}`)
          emit('installed')
          emit('update:visible', false)
        } catch (e: any) {
          const msg = typeof e === 'string' ? e : (e?.message || String(e))
          // 用户主动取消：仅 toast 提示并退出下载页，不弹错误窗
          if (isCancelledError(e)) {
            toastInfo('下载已取消')
            versionStore.finishDownload()
            return
          }
          // 真实失败：后端已 mark_failed 重置 is_active，前端用 showModal + onConfirm 让用户点击确定后退出下载页
          showModal({
            type: 'error',
            title: 'Mod 安装失败',
            message: msg,
            onConfirm: () => {
              versionStore.finishDownload()
            },
          })
        } finally {
          installing.value = false
        }
      },
    )
  }

  // 监听 visible 变化，打开时加载版本
  watch(() => props.visible, async (val) => {
    if (val && props.mod) {
      await loadVersions()
    } else {
      // 关闭时重置状态
      versions.value = []
      error.value = ''
      selectedVersionId.value = null
      selectedGameVersion.value = ''
      selectedLoader.value = ''
    }
  })

  return {
    loading,
    versions,
    error,
    installing,
    selectedVersionId,
    filteredVersions,
    selectedVersion,
    platform,
    modLoaderType,
    versionChange,
    installSelected,
  }
}
