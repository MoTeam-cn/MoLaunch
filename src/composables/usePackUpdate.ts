/**
 * 资源包/光影版本更新/更改逻辑 composable
 *
 * 从 ModUpdateDialog 的 useModUpdate 简化而来：
 * - 无加载器过滤（资源包/光影不分加载器）
 * - 无依赖检查（zip 包无前置依赖）
 * - 当前版本未知（zip 内无版本号），版本变化恒为 unknown
 */
import { ref, computed, watch } from 'vue'
import { getProjectVersions } from '@/utils/api/community'
import { updatePack, type PackInfo, type PackKind } from '@/utils/api/personalization'
import { formatBytes } from '@/utils/format'
import { toastSuccess, toastInfo, toastError } from '@/utils/toast'
import { showConfirm, showModal } from '@/utils/modal'
import { isCancelledError } from '@/utils/async'
import { useVersionStore } from '@/stores/version'
import type { ResourceVersion, Platform } from '@/types/community'

interface UsePackUpdateProps {
  pack: PackInfo | null
  kind: PackKind
  mcVersion: string
  versionId: string
  visible: boolean
}

export function usePackUpdate(
  props: UsePackUpdateProps,
  emit: ((event: 'update:visible', val: boolean) => void) & ((event: 'installed') => void),
) {
  const versionStore = useVersionStore()
  const loading = ref(false)
  const versions = ref<ResourceVersion[]>([])
  const error = ref('')
  const installing = ref(false)

  // 按当前整合包的 MC 版本过滤
  const selectedGameVersion = ref<string>('')
  const selectedVersionId = ref<string | null>(null)

  const filteredVersions = computed(() => {
    let result = versions.value
    if (selectedGameVersion.value) {
      result = result.filter(v => v.game_versions.includes(selectedGameVersion.value))
    }
    return result
  })

  const selectedVersion = computed(() =>
    filteredVersions.value.find(v => v.id === selectedVersionId.value) || null,
  )

  const platform = computed<Platform | null>(() => {
    if (!props.pack?.project) return null
    return props.pack.project.platform
  })

  // zip 包内无版本号，当前版本未知
  const versionChange = computed<'unknown'>(() => 'unknown')

  async function loadVersions() {
    if (!props.pack?.project || !platform.value) {
      error.value = '此内容没有关联的平台工程信息，无法查询版本'
      return
    }

    loading.value = true
    error.value = ''
    versions.value = []

    try {
      const result = await getProjectVersions(platform.value, props.pack.project.id)
      versions.value = result

      if (props.mcVersion) {
        selectedGameVersion.value = props.mcVersion
      }

      if (filteredVersions.value.length > 0) {
        selectedVersionId.value = filteredVersions.value[0].id
      }
    } catch (e: any) {
      error.value = typeof e === 'string' ? e : (e?.message || String(e))
      toastError('查询版本列表失败：' + String(e))
    } finally {
      loading.value = false
    }
  }

  function installSelected() {
    if (!selectedVersion.value || !props.pack) return

    const version = selectedVersion.value
    const oldFileName = props.pack.file_name

    showConfirm(
      '确认安装',
      `将下载 ${version.version} 并替换当前文件 ${oldFileName}。\n\n新文件名：${version.file_name}\n大小：${formatBytes(version.size)}`,
      async () => {
        installing.value = true
        try {
          versionStore.startDownload(version.file_name)
          toastInfo(`开始下载: ${version.file_name}`)
          await updatePack(
            props.versionId,
            oldFileName,
            version.download_url,
            version.file_name,
            version.size,
            props.kind,
          )

          toastSuccess(`已安装 ${version.version}`)
          emit('installed')
          emit('update:visible', false)
        } catch (e: any) {
          const msg = typeof e === 'string' ? e : (e?.message || String(e))
          if (isCancelledError(e)) {
            toastInfo('下载已取消')
            versionStore.finishDownload()
            return
          }
          showModal({
            type: 'error',
            title: '安装失败',
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
    if (val && props.pack) {
      await loadVersions()
    } else {
      versions.value = []
      error.value = ''
      selectedVersionId.value = null
      selectedGameVersion.value = ''
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
    versionChange,
    installSelected,
  }
}
