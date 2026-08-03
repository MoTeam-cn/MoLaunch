/**
 * 全局文件拖拽 - 安装执行辅助
 *
 * formatToLabel（ModpackFormat 转中文）+ runModpackInstall（整合包安装流水线），
 * 供 handlers.ts 分发调用，避免业务分发文件过长。
 */
import router from '@/router'
import { installLocalModpack } from '@/utils/api/community'
import { installMerged } from '@/utils/api/loader'
import { showModal } from '@/utils/modal'
import { toastSuccess, toastInfo } from '@/utils/toast'
import { isCancelledError } from '@/utils/async'
import { useVersionStore } from '@/stores/version'
import type { InstallModpackResult, ModpackPreview } from '@/types/community'

/** ModpackFormat 枚举转中文标签 */
export function formatToLabel(format: ModpackPreview['format']): string {
  switch (format) {
    case 'curseforge':
      return 'CurseForge'
    case 'modrinth':
      return 'Modrinth'
    case 'hmcl':
      return 'HMCL'
    case 'mmc':
      return 'MultiMC'
    case 'mcbbs':
      return 'MCBBS'
    case 'launcherpack':
      return '带启动器整合包'
    case 'compress':
      return '普通压缩包'
    default:
      return '未知'
  }
}

/**
 * 执行整合包安装流程（install_local_modpack → install_merged）
 *
 * 拖拽安装时弹窗询问用户是否下载可选 Mod（CF required=false / MR env.client=optional）；
 * HMCL/MMC/MCBBS 无可选概念，直接安装。进度通过 download_state 推送，
 * DownloadPanel 自动展示，完成后跳转下载页轮询 install_merged 进度。
 */
export async function runModpackInstall(
  filePath: string,
  instanceName: string,
  includeOptional?: boolean,
): Promise<void> {
  const versionStore = useVersionStore()
  // 跳转到下载页，让用户看到进度
  router.push({ name: 'downloads' })

  let result: InstallModpackResult
  try {
    result = await installLocalModpack({ filePath, instanceName, includeOptional })
  } catch (err) {
    const msg = err instanceof Error ? err.message : String(err)
    // 用户主动取消：仅 toast 提示并退出下载页，不弹错误窗
    if (isCancelledError(err)) {
      toastInfo('下载已取消')
      versionStore.finishDownload()
      return
    }
    // 真实失败：后端已 mark_failed 重置 is_active，前端需 finishDownload 让 Downloads.vue watch 触发 router.back()
    showModal({
      type: 'error',
      title: '整合包安装失败',
      message: msg,
      onConfirm: () => {
        versionStore.finishDownload()
      },
    })
    return
  }

  // 整合包专属部分完成，紧接着调用 install_merged 安装游戏本体
  toastSuccess(`整合包解析完成，开始安装 MC ${result.gameVersion}...`)

  try {
    await installMerged(
      result.gameVersion,
      result.loader === 'forge' ? result.loaderVersion : undefined,
      result.loader === 'neoforge' ? result.loaderVersion : undefined,
      result.loader === 'fabric' ? result.loaderVersion : undefined,
      result.loader === 'optifine' ? result.loaderVersion : undefined,
      undefined,
      instanceName,
    )
    toastSuccess(`整合包 ${instanceName} 安装完成`)
  } catch (err) {
    const msg = err instanceof Error ? err.message : String(err)
    // 用户主动取消：仅 toast 提示并退出下载页，不弹错误窗
    if (isCancelledError(err)) {
      toastInfo('下载已取消')
      versionStore.finishDownload()
      return
    }
    // 同上：后端已 mark_failed，前端用 showModal + onConfirm 让用户点击确定后退出下载页
    showModal({
      type: 'error',
      title: '游戏本体安装失败',
      message: `整合包已解压，但游戏本体安装失败：${msg}`,
      onConfirm: () => {
        versionStore.finishDownload()
      },
    })
  }
}