/**
 * 客户端自动更新工具（module-level 单例）
 *
 * 调用后端 `system_manager` 的 `check_update` / `download_and_install_update` action，
 * 后端根据平台内部分流：
 * - **Windows 便携版**：自实现下载 + 启动 updater.exe 子进程替换 exe
 *   - 定时检查（10 分钟）发现新版本后静默下载到 `%APPDATA%/.Molaunch/last.exe`
 *   - 用户退出程序时调用 `apply_pending_update` 启动 updater.exe 替换主 exe
 *   - 下次启动即为新版本
 * - **macOS / Linux**：转发到 `tauri-plugin-updater` 官方 plugin（6h 定时 + 弹窗手动安装）
 *
 * 前端不关心平台差异，统一调用 `systemManager`。
 *
 * - 状态用 `reactive` 暴露，组件可直接 watch
 * - `checkForUpdate` / `downloadAndInstall` / `closeDialog` 为纯函数，可在任意位置调用
 * - `initAutoCheck` 在 App.vue 启动时调用一次，注册启动 5s + 定时检查
 *   - Windows：10 分钟间隔，静默下载到 appdata，退出时替换
 *   - macOS/Linux：6 小时间隔，发现更新弹窗让用户手动安装
 *
 * 配套组件：`src/components/about/UpdateDialog.vue`
 *
 * See: docs/updater/design.md §4
 */

import { reactive } from 'vue'
import { systemManager, SYSTEM_ACTIONS } from '@/utils/api/system-manager'
import { toastInfo, toastError } from '@/utils/toast'

/** 检测当前是否 Windows 平台（用于区分自实现 updater 与官方 plugin 流程） */
function isWindows(): boolean {
  return navigator.userAgent.includes('Windows')
}

export type UpdateStatus =
  | 'idle'
  | 'checking'
  | 'available'
  | 'no-update'
  | 'downloading'
  | 'installing'
  | 'done'
  | 'error'

/** 后端返回的更新信息（与 Rust `UpdateInfo` 结构对应） */
interface UpdateInfo {
  available: boolean
  version: string
  notes: string
  forceUpdate: boolean
  downloadUrl: string
  signature: string
}

export interface UpdateState {
  /** 当前状态 */
  status: UpdateStatus
  /** 新版本号 */
  version: string
  /** 更新日志 */
  notes: string
  /** 是否强制更新（来自 manifest 扩展字段） */
  forceUpdate: boolean
  /** 已下载字节数（当前版本不报告精确进度，保持 0） */
  downloaded: number
  /** 总字节数（当前版本不报告精确进度，保持 0） */
  total: number
  /** 错误信息 */
  error: string
  /** 是否显示弹窗（手动触发或发现更新时为 true） */
  showDialog: boolean
}

/** 全局更新状态（响应式，组件可直接 watch） */
export const updateState = reactive<UpdateState>({
  status: 'idle',
  version: '',
  notes: '',
  forceUpdate: false,
  downloaded: 0,
  total: 0,
  error: '',
  showDialog: false,
})

/** 当前待安装的更新信息（checkForUpdate 后缓存，downloadAndInstall 使用） */
let pendingUpdate: UpdateInfo | null = null

/** 防止并发检查 */
let checking = false

/** 防止并发下载 */
let installing = false

/** Windows 后台静默下载已完成的版本号（避免 10 分钟定时重复下载同一版本） */
let appdataDownloadedVersion: string | null = null

/**
 * 检查更新
 *
 * @param opts.silent 静默模式（启动时/定时触发）：
 *   - true：仅在发现可用更新时弹窗，无更新/检查失败均不打扰用户
 *   - false：手动触发，无论结果都给用户反馈（toast 或弹窗）
 */
export async function checkForUpdate(opts: { silent?: boolean } = {}): Promise<void> {
  const { silent = false } = opts
  if (checking) return
  checking = true

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
      pendingUpdate = info
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
    checking = false
  }
}

/**
 * 下载并安装更新
 *
 * 调用前提：updateState.status === 'available'（即 checkForUpdate 已发现更新）。
 * 后端行为：
 * - Windows：下载新 exe → 启动 updater.exe → 主程序退出 → updater 替换 exe → 启动新版
 * - macOS/Linux：转发官方 plugin → 下载/验签/替换 → relaunch
 *
 * 注意：后端调用成功返回后主程序会立即退出，前端不会收到响应。
 * 如果收到错误响应，说明下载或启动失败。
 */
export async function downloadAndInstall(): Promise<void> {
  if (installing) return
  if (!pendingUpdate) {
    toastError('无可用的更新，请先检查更新')
    return
  }
  installing = true

  updateState.status = 'downloading'
  updateState.downloaded = 0
  updateState.total = 0
  updateState.error = ''

  try {
    // 后端调用成功后主程序会退出，不会执行到这里
    await systemManager(SYSTEM_ACTIONS.DOWNLOAD_AND_INSTALL_UPDATE, pendingUpdate)

    updateState.status = 'installing'
    updateState.status = 'done'
  } catch (e) {
    updateState.status = 'error'
    updateState.error = String(e)
    console.error('[Updater] downloadAndInstall error:', e)
    toastError(`更新失败：${String(e)}`)
  } finally {
    installing = false
  }
}

/**
 * 关闭更新弹窗（用户点击"稍后"时调用）
 *
 * 强制更新时不允许关闭，调用方需自行判断（UpdateDialog 内也会二次校验）。
 */
export function closeDialog(): void {
  if (updateState.forceUpdate) return
  if (updateState.status === 'downloading' || updateState.status === 'installing') return
  updateState.showDialog = false
  updateState.status = 'idle'
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
  if (checking) return
  checking = true

  try {
    const info = await systemManager<UpdateInfo>(SYSTEM_ACTIONS.CHECK_UPDATE)

    if (info?.available && info.version) {
      // 已下载过同一版本，跳过
      if (appdataDownloadedVersion === info.version) {
        console.debug('[Updater] 版本 %s 已下载到 appdata，跳过', info.version)
        return
      }

      console.info('[Updater] 发现新版本 %s，开始后台下载到 appdata', info.version)
      const downloaded = await systemManager<boolean>(
        SYSTEM_ACTIONS.DOWNLOAD_UPDATE_TO_APPDATA,
        info,
      )

      if (downloaded) {
        appdataDownloadedVersion = info.version
        console.info('[Updater] 新版本 %s 已下载到 appdata/last.exe，退出时将自动替换', info.version)
        // 更新状态供 UI 展示（如有更新提示组件）
        updateState.version = info.version
        updateState.notes = info.notes ?? ''
      }
    }
  } catch (e) {
    console.error('[Updater] silentCheckAndDownload error:', e)
  } finally {
    checking = false
  }
}

/**
 * 退出时应用待安装更新
 *
 * 在窗口关闭事件中调用（TopNavLayout.vue `handleClose`）：
 * - 后端检查 `%APPDATA%/.Molaunch/last.exe` 是否存在
 * - 存在：启动 updater.exe 替换主 exe，返回 true（调用方随后退出主程序）
 * - 不存在：返回 false（正常退出）
 *
 * 仅 Windows 有效，其他平台直接返回 false。
 */
export async function applyPendingUpdate(): Promise<boolean> {
  try {
    return await systemManager<boolean>(SYSTEM_ACTIONS.APPLY_PENDING_UPDATE)
  } catch (e) {
    console.error('[Updater] applyPendingUpdate error:', e)
    return false
  }
}
