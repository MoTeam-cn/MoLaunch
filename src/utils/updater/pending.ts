/**
 * 查询「已下载待重启」的更新包状态
 *
 * Windows 便携版会在后台把新版本下载到 `%APPDATA%/.Molaunch/last.exe`，
 * 下载完成后无需再下一次；前端据此把更新抽屉切到「立即重启」。
 * macOS / Linux 由官方 plugin 同步完成安装，恒定返回未就绪。
 */
import { systemManager, SYSTEM_ACTIONS } from '@/utils/api/system-manager'
import type { PendingUpdateStatus } from './state'

/** 查询待安装更新包；失败按「无待安装包」处理，不影响正常更新流程 */
export async function fetchPendingUpdateStatus(): Promise<PendingUpdateStatus | null> {
  try {
    return await systemManager<PendingUpdateStatus>(SYSTEM_ACTIONS.PENDING_UPDATE_STATUS)
  } catch (e) {
    console.error('[Updater] pending_update_status error:', e)
    return null
  }
}
