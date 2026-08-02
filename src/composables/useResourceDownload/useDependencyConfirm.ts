/**
 * 前置依赖确认弹窗切片（从 useResourceDownload.ts 抽取）
 *
 * 负责前置确认弹窗状态（showDependencyDialog / pendingMainVersion / pendingContext）
 * 与弹窗交互：
 * - openDependencyDialog：进入弹窗等待阶段（stage=waiting，保持下载 loading）
 * - handleDependencyConfirm：确认后走 install_mod_with_dependencies（选目录 → 安装）
 * - handleDependencyClose：取消（清空状态并复位下载进度）
 *
 * 依赖注入：
 * - progress（UseDownloadProgress）：下载进度 ref 与迁移 helper，弹窗等待期间保持 loading
 * - resetDeps / installDeps：复用 useDependencyCheck composable，不重复实现
 * - project：options.project（reactive），用于主 Mod 判空
 */
import { ref } from 'vue'
import type {
  ResourceProject,
  ResourceVersion,
  ResolvedDependency,
  DependencyInstallResult,
} from '@/types/community'
import { useVersionStore } from '@/stores/version'
import { pickDirectory } from '@/utils/fileDialog'
import { toastSuccess, toastInfo } from '@/utils/toast'
import { showModal } from '@/utils/modal'
import { isCancelledError } from '@/utils/async'
import type { InstallDepsParams } from '@/composables/useDependencyCheck'
import type { UseDownloadProgress } from './useDownloadProgress'

/** 前置检查解析出的上下文（versionId/gameVersion/modsDir/modLoader） */
export interface PendingContext {
  versionId: string
  gameVersion: string
  modsDir: string | undefined
  modLoader: number
}

export function useDependencyConfirm(deps: {
  /** 资源项目（props.project，reactive） */
  project: ResourceProject | null
  versionStore: ReturnType<typeof useVersionStore>
  progress: UseDownloadProgress
  /** 复位 useDependencyCheck 状态（关闭弹窗时调用） */
  resetDeps: () => void
  /** install_mod_with_dependencies IPC 调用 */
  installDeps: (params: InstallDepsParams) => Promise<DependencyInstallResult>
}) {
  const { project, versionStore, progress, resetDeps, installDeps } = deps

  /** 前置确认弹窗状态 */
  const showDependencyDialog = ref(false)
  /** 暂存待安装的主 Mod（弹窗确认后用于 install 调用） */
  const pendingMainVersion = ref<ResourceVersion | null>(null)
  /** 暂存前置检查解析出的上下文（versionId/gameVersion/modsDir/modLoader） */
  const pendingContext = ref<PendingContext | null>(null)

  /** 清空弹窗状态 + 复位下载进度 + 复位依赖检查（弹窗关闭/确认完成时统一调用） */
  function clearPendingState() {
    showDependencyDialog.value = false
    pendingMainVersion.value = null
    pendingContext.value = null
    progress.resetDownload()
    resetDeps()
  }

  /** 打开前置确认弹窗并进入等待阶段（runDependencyCheck 检出缺失时调用） */
  function openDependencyDialog(v: ResourceVersion) {
    pendingMainVersion.value = v
    showDependencyDialog.value = true
    // 进入等待阶段：等用户在弹窗确认（保持 downloading loading）
    progress.toWaiting()
  }

  /**
   * 用户在 DependencyConfirmDialog 点击"确认安装"后回调
   *
   * - 版本管理场景（有 versionId）：直接 install，下载到版本 mods 目录
   * - Community 场景（无 versionId）：先选保存文件夹，install 时传 targetDir
   *
   * 调用 install_mod_with_dependencies IPC，后端启动 DownloadSession 并发下载主 Mod + 勾选前置。
   * 进度通过 WS 推送到下载管理页。
   */
  async function handleDependencyConfirm(selectedDeps: ResolvedDependency[]) {
    const main = pendingMainVersion.value
    const ctx = pendingContext.value
    if (!main || !ctx || !project) return

    showDependencyDialog.value = false

    // Community 场景（无 versionId）：先选保存文件夹
    let targetDir: string | undefined
    if (!ctx.versionId) {
      const dir = await pickDirectory({
        title: '选择保存文件夹（主 Mod + 前置将下载到此目录）',
      })
      if (!dir) {
        // 用户取消，清空 loading 和状态
        clearPendingState()
        return
      }
      targetDir = dir
    }

    progress.downloading.value = main.id
    // 进入下载阶段
    progress.toDownloading()
    versionStore.startDownload(main.file_name)
    const totalFiles = 1 + selectedDeps.length
    toastInfo(`开始下载: ${main.file_name}（含 ${selectedDeps.length} 个前置，共 ${totalFiles} 个文件）`)

    try {
      const result = await installDeps({
        versionId: ctx.versionId || undefined,
        targetDir,
        mainVersion: main,
        deps: selectedDeps,
      })
      if (result.failedCount > 0) {
        // 部分失败：toast 警告，但仍由 WS 推送 mark_complete 触发退出
        toastInfo(`安装完成：成功 ${result.installedCount} / ${totalFiles}，失败 ${result.failedCount}`)
      } else {
        toastSuccess(`安装完成：共 ${result.installedCount} 个文件`)
      }
      // 兜底：WS 可能因时序未收到 is_complete，IPC 返回成功后直接 finishDownload
      // 若 WS 已 finishDownload（downloading=false），此处无副作用（finishDownload 幂等）
      if (versionStore.downloading) {
        versionStore.finishDownload()
      }
    } catch (e: any) {
      const msg = e?.message || String(e)
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
      clearPendingState()
    }
  }

  /** 用户在 DependencyConfirmDialog 点击取消（不下载） */
  function handleDependencyClose() {
    clearPendingState()
  }

  return {
    showDependencyDialog,
    pendingMainVersion,
    pendingContext,
    openDependencyDialog,
    handleDependencyConfirm,
    handleDependencyClose,
  }
}
