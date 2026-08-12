import { systemManager, SYSTEM_ACTIONS } from '@/utils/api/system-manager'
import { toastError, toastInfo } from '@/utils/toast'
import { updateState, updaterFlags, type UpdateInfo } from './state'

/** 检测当前是否 Windows 平台（用于区分自实现 updater 与官方 plugin 流程） */
function isWindows(): boolean {
  return navigator.userAgent.includes('Windows')
}

/**
 * 检查更新
 *
 * @param opts.silent 静默模式（启动时/定时触发）：
 *   - true：仅在发现可用更新时弹窗，无更新/检查失败均不打扰用户
 *   - false：手动触发，无论结果都给用户反馈（toast 或弹窗）
 */
export async function checkForUpdate(opts: { silent?: boolean } = {}): Promise<void> {
  const { silent = false } = opts
  if (updaterFlags.checking) return
  updaterFlags.checking = true

  if (!silent) {
    updateState.status = 'checking'
    updateState.showDialog = true
    updateState.error = ''
  }

  try {
    const info = await systemManager<UpdateInfo>(SYSTEM_ACTIONS.CHECK_UPDATE)

    if (info?.available) {
      updateState.status = 'available'
      updateState.version = info.version
      updateState.notes = info.notes ?? ''
      updateState.forceUpdate = info.forceUpdate === true
      updateState.downloaded = 0
      updateState.total = 0
      updateState.error = ''
      updateState.showDialog = true
      updaterFlags.pendingUpdate = info
    } else {
      updateState.status = 'no-update'
      updateState.showDialog = false
      if (!silent) {
        toastInfo('当前已是最新版本')
      }
    }
  } catch (e) {
    updateState.status = 'error'
    updateState.error = String(e)
    updateState.showDialog = false
    if (!silent) {
      toastError(`检查更新失败：${String(e)}`)
    }
    console.error('[Updater] checkForUpdate error:', e)
  } finally {
    updaterFlags.checking = false
  }
}

/**
 * 初始化自动检查
 *
 * 在 App.vue onMounted 中调用一次：
 * - **Windows**：启动 5s + 每 10 分钟静默检查，发现新版本自动下载到 appdata/last.exe，
 *   退出时由 `applyPendingUpdate` 触发替换（无需用户干预）
 * - **macOS/Linux**：启动 5s + 每 6 小时静默检查，发现更新弹窗让用户手动安装
 *
 * 开发模式（vite dev）下跳过自动检查，避免 dev 版本号低于发布版本时反复触发更新。
 */
export function initAutoCheck(): void {
  if (import.meta.env.DEV) {
    console.info('[Updater] Dev mode, skip auto check')
    return
  }

  const win = isWindows()
  const interval = win ? 10 * 60 * 1000 : 6 * 60 * 60 * 1000

  setTimeout(() => {
    if (win) {
      silentCheckAndDownload().catch((e) => {
        console.error('[Updater] startup auto check failed:', e)
      })
    } else {
      checkForUpdate({ silent: true }).catch((e) => {
        console.error('[Updater] startup auto check failed:', e)
      })
    }
  }, 5000)

  setInterval(() => {
    if (win) {
      silentCheckAndDownload().catch((e) => {
        console.error('[Updater] scheduled auto check failed:', e)
      })
    } else {
      checkForUpdate({ silent: true }).catch((e) => {
        console.error('[Updater] scheduled auto check failed:', e)
      })
    }
  }, interval)
}

/**
 * Windows 专属：静默检查 + 自动下载到 appdata
 *
 * 定时触发（10 分钟），发现新版本后：
 * 1. 调用 `download_update_to_appdata` 下载到 `%APPDATA%/.Molaunch/last.exe`
 * 2. 记录已下载版本号，避免定时重复下载
 * 3. 不弹窗打扰用户，等用户退出时由 `applyPendingUpdate` 触发替换
 *
 * 已下载过同一版本时跳过（`appdataDownloadedVersion` 去重）。
 */
async function silentCheckAndDownload(): Promise<void> {
  if (updaterFlags.silentChecking) return
  updaterFlags.silentChecking = true

  try {
    const info = await systemManager<UpdateInfo>(SYSTEM_ACTIONS.CHECK_UPDATE)

    if (info?.available && info.version) {
      // 已下载过同一版本，跳过
      if (updaterFlags.appdataDownloadedVersion === info.version) {
        console.debug('[Updater] 版本 %s 已下载到 appdata，跳过', info.version)
        return
      }

      console.info('[Updater] 发现新版本 %s，开始后台下载到 appdata', info.version)
      const downloaded = await systemManager<boolean>(
        SYSTEM_ACTIONS.DOWNLOAD_UPDATE_TO_APPDATA,
        info,
      )

      if (downloaded) {
        updaterFlags.appdataDownloadedVersion = info.version
        console.info('[Updater] 新版本 %s 已下载到 appdata/last.exe，退出时将自动替换', info.version)
        // 更新状态供 UI 展示（如有更新提示组件）
        updateState.version = info.version
        updateState.notes = info.notes ?? ''
      }
    }
  } catch (e) {
    console.error('[Updater] silentCheckAndDownload error:', e)
  } finally {
    updaterFlags.silentChecking = false
  }
}