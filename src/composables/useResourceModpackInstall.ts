/**
 * 资源详情页整合包安装 composable
 *
 * 弹窗询问安装名称 → installModpack（下载/解析/依赖 mods/overrides）→ installMerged（MC 本体 + 加载器）。
 * 共享 useResourceDownload 的 downloading 状态；入参为 props（reactive proxy），
 * 内部经 options.project 访问拿最新值，不得解构后传入。
 */
import type { Ref } from 'vue'
import type { ResourceProject, ResourceVersion } from '@/types/community'
import { installModpack } from '@/utils/api/community'
import { installMerged } from '@/utils/api/loader'
import { useVersionStore } from '@/stores/version'
import { showPrompt, showModal } from '@/utils/modal'
import { toastSuccess, toastInfo } from '@/utils/toast'
import { isCancelledError } from '@/utils/async'

export function useResourceModpackInstall(
  /** ResourceDetail 的 props（reactive，内部访问 options.project 拿最新值） */
  options: { project: ResourceProject | null },
  /** 共享 useResourceDownload 的 downloading 状态 */
  downloading: Ref<string | null>,
) {
  const versionStore = useVersionStore()

  /**
   * 弹窗询问整合包安装名称
   *
   * 将 callback 风格的 showPrompt 包装为 Promise，便于在 async 流程中 await。
   * 用户确认返回 trim 后的名称（空则回退默认名）；取消返回 null。
   */
  function promptForInstanceName(defaultName: string): Promise<string | null> {
    return new Promise((resolve) => {
      showPrompt(
        '安装整合包',
        '请输入整合包的安装名称：',
        (value: string) => {
          const trimmed = value.trim()
          resolve(trimmed || defaultName)
        },
        {
          defaultValue: defaultName,
          placeholder: '请输入安装名称',
          onCancel: () => resolve(null),
        },
      )
    })
  }

  /**
   * 安装整合包：下载原始包 + 解析 + 下载依赖 mods + 复制 overrides → 安装游戏本体 + 加载器
   * 进度走 download_state，复用 DownloadPanel
   *
   * 点击下载后先弹窗询问安装名称，用户可自定义；取消则中止安装。
   */
  async function handleInstallModpack(v: ResourceVersion) {
    if (!options.project) return
    const { platform, resource_type, translated_name, raw_name, id: projectId } = options.project
    if (resource_type !== 'ModPack') return

    const defaultName = translated_name || raw_name || v.file_name.replace(/\.(zip|mrpack)$/i, '')

    // 弹窗询问安装名称，允许用户自定义（取消则中止）
    const instanceName = await promptForInstanceName(defaultName)
    if (!instanceName) return

    downloading.value = v.id
    versionStore.startDownload(instanceName)

    try {
      // 联机大厅阶段 3：传入平台来源元数据，后端安装完成后写入 modpack.meta.json
      // 用于创建联机房间时上报整合包信息，加入方据此判断是否需要一键安装
      const result = await installModpack({
        platform,
        downloadUrl: v.download_url,
        fileName: v.file_name,
        instanceName,
        projectId,
        fileId: v.id,
        modpackVersion: v.version,
        fileSize: v.size,
        name: translated_name || raw_name,
      })
      const loader = result.loader
      const lv = result.loaderVersion
      await installMerged(
        result.gameVersion,
        loader === 'forge' ? lv : undefined,
        loader === 'neoforge' ? lv : undefined,
        loader === 'fabric' || loader === 'quilt' ? lv : undefined,
        undefined, undefined, instanceName,
      )
      toastSuccess(`整合包 ${instanceName} 安装完成`)
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e)
      // 用户主动取消：仅 toast 提示并退出下载页，不弹错误窗
      if (isCancelledError(e)) {
        toastInfo('下载已取消')
        versionStore.finishDownload()
        return
      }
      // 真实失败：后端已 mark_failed 重置 is_active，前端用 showModal + onConfirm 让用户点击确定后退出下载页
      showModal({
        type: 'error',
        title: '整合包安装失败',
        message: msg,
        onConfirm: () => {
          versionStore.finishDownload()
        },
      })
    } finally {
      downloading.value = null
    }
  }

  return { handleInstallModpack }
}
