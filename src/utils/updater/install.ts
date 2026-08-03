import { systemManager, SYSTEM_ACTIONS } from '@/utils/api/system-manager'
import { toastError } from '@/utils/toast'
import { updateState, updaterFlags } from './state'

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
  if (updaterFlags.installing) return
  if (!updaterFlags.pendingUpdate) {
    toastError('无可用的更新，请先检查更新')
    return
  }
  updaterFlags.installing = true

  updateState.status = 'downloading'
  updateState.downloaded = 0
  updateState.total = 0
  updateState.error = ''

  try {
    // 后端调用成功后主程序会退出，不会执行到这里
    await systemManager(SYSTEM_ACTIONS.DOWNLOAD_AND_INSTALL_UPDATE, updaterFlags.pendingUpdate)

    updateState.status = 'installing'
    updateState.status = 'done'
  } catch (e) {
    updateState.status = 'error'
    updateState.error = String(e)
    console.error('[Updater] downloadAndInstall error:', e)
    toastError(`更新失败：${String(e)}`)
  } finally {
    updaterFlags.installing = false
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