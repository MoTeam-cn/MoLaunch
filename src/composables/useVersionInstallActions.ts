/**
 * 版本下载/安装/卸载操作 composable（从 Versions.vue 抽出）
 *
 * 封装 Versions 下载页的全部业务逻辑：
 * - 加载已安装版本列表（含类型与 logo 映射）
 * - 刷新版本列表
 * - 安装请求（合并加载器版本，后台执行）
 * - 下载/卸载版本
 * - 打开游戏目录
 *
 * 设计原则：
 * - 不接收参数：versionStore 由 composable 内部获取（与原 Versions.vue 一致）
 * - 返回状态 ref 和 handler 函数
 * - handler 内部的 toast/modal 调用保持原 Versions.vue 行为不变
 * - 同时引入了 toast 和 modal，toast 函数使用 `toastXxx` 前缀，`showError`/`showConfirm` 从 modal 导入
 *
 * 使用方式：
 * ```ts
 * const {
 *   installedVersions, installedVersionTypes, installedVersionLogos,
 *   loadInstalledVersions, handleRefresh, onInstallRequest,
 *   handleDownload, handleUninstall, handleOpenGameDir,
 * } = useVersionInstallActions()
 * ```
 */
import { ref } from 'vue'
import { useVersionStore } from '@/stores/version'
import * as tauri from '@/utils/tauri'
import { showError, showConfirm, showModal } from '@/utils/modal'
import { toastSuccess, toastInfo, toastWarning } from '@/utils/toast'
import { safeCall } from '@/utils/async'

/** 安装请求参数（与 LoaderSelect.vue 的 install emit 类型对齐） */
export interface InstallOptions {
  mcVersion: string
  forge?: string
  neoforge?: string
  fabric?: string
  optifine?: string
  liteloader?: string
  instanceName: string
}

/**
 * 版本下载/安装/卸载操作 composable
 *
 * 注意：`onInstallRequest` 仅负责启动下载流程，
 * 调用方需在调用前/后自行清空 `selectedVersion` 等 UI 状态（属页面级 UI，不归属本 composable）。
 */
export function useVersionInstallActions() {
  const versionStore = useVersionStore()

  const installedVersions = ref<string[]>([])
  const installedVersionTypes = ref<Record<string, string>>({})
  const installedVersionLogos = ref<Record<string, string>>({})

  async function loadInstalledVersions() {
    try {
      const vwt = await tauri.listInstalledVersionsWithType()
      installedVersions.value = vwt.map(v => v.id)
      const typeMap: Record<string, string> = {}
      const logoMap: Record<string, string> = {}
      vwt.forEach(v => { typeMap[v.id] = v.version_type; logoMap[v.id] = v.logo || '' })
      installedVersionTypes.value = typeMap
      installedVersionLogos.value = logoMap
    } catch (e) {
      console.error(e)
      const fallback = await safeCall(() => tauri.listInstalledVersions(), 'list installed versions (fallback)')
      if (fallback) installedVersions.value = fallback
    }
  }

  async function handleRefresh() {
    toastInfo('正在刷新版本列表...')
    try {
      await versionStore.refreshVersions()
      await loadInstalledVersions()
      if (versionStore.versions.length === 0) {
        toastWarning('未获取到版本列表，请检查网络连接')
      } else {
        toastSuccess('版本列表已刷新')
      }
    } catch (e) {
      showError('获取版本列表失败', String(e))
    }
  }

  function onInstallRequest(options: InstallOptions) {
    // 设置下载状态，显示 DownloadPanel（会自动启动轮询）
    versionStore.startDownload(options.instanceName)
    // 后台执行安装
    tauri.installMerged(
      options.mcVersion,
      options.forge,
      options.neoforge,
      options.fabric,
      options.optifine,
      options.liteloader,
      options.instanceName,
    ).then(async () => {
      toastSuccess(`${options.instanceName} 安装完成`)
      await loadInstalledVersions()
    }).catch((e) => {
      // 后端已 mark_failed 重置 is_active，前端用 showModal + onConfirm 让用户点击确定后退出下载页
      showModal({
        type: 'error',
        title: '安装失败',
        message: String(e),
        onConfirm: () => {
          versionStore.finishDownload()
        },
      })
    })
    // 不在这里调用 finishDownload，由轮询统一管理生命周期
  }

  async function handleDownload(versionId: string) {
    versionStore.startDownload(versionId)
    toastInfo(`开始下载 ${versionId}`)
    try {
      await tauri.downloadVersion(versionId)
      await loadInstalledVersions()
      toastSuccess(`${versionId} 下载完成`)
    } catch (e) {
      // 后端已 mark_failed 重置 is_active，前端用 showModal + onConfirm 让用户点击确定后退出下载页
      showModal({
        type: 'error',
        title: '下载失败',
        message: `无法下载版本 ${versionId}`,
        details: String(e),
        onConfirm: () => {
          versionStore.finishDownload()
        },
      })
    }
    // 不在这里调用 finishDownload，由轮询统一管理生命周期
  }

  function handleUninstall(versionId: string) {
    showConfirm('卸载版本', `确定要卸载版本 ${versionId} 吗？此操作不可撤销。`, async () => {
      try {
        await tauri.uninstallVersion(versionId)
        installedVersions.value = installedVersions.value.filter(v => v !== versionId)
        toastSuccess(`${versionId} 已卸载`)
      } catch (e) { showError('卸载失败', `无法卸载版本 ${versionId}`, String(e)) }
    })
  }

  async function handleOpenGameDir() {
    toastInfo('正在打开游戏目录...')
    try {
      await tauri.openGameDir()
      // 随机延迟 1-2 秒，增加真实感
      const delay = 1000 + Math.random() * 1000
      setTimeout(() => {
        toastSuccess('已打开游戏目录，请自行浏览哈')
      }, delay)
    } catch (e) {
      showError('打开失败', '无法打开游戏目录', String(e))
    }
  }

  return {
    installedVersions,
    installedVersionTypes,
    installedVersionLogos,
    loadInstalledVersions,
    handleRefresh,
    onInstallRequest,
    handleDownload,
    handleUninstall,
    handleOpenGameDir,
  }
}
