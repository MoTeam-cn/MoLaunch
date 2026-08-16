/**
 * 红石联机（hongshi 内核）IPC 封装：`redstone_manager` 经 action 分发
 *
 * params 字段名与后端契约一致（`redstone_start` 为 `mc_port`）。
 * 错误经 invoke reject 透出，message 为后端中文文案。
 */
import { invoke } from '@tauri-apps/api/core'
import type {
  RedStoneGetServersResult,
  RedStoneLogFilesResult,
  RedStoneReadLogResult,
  RedStoneStartParams,
  RedStoneStartResult,
  RedStoneStatusResult,
  RedStoneStopResult,
} from '@/types/redstone'

/** 红石联机 action 名称常量 */
export const REDSTONE_ACTIONS = {
  GET_SERVERS: 'redstone_get_servers',
  START: 'redstone_start',
  STATUS: 'redstone_status',
  STOP: 'redstone_stop',
  LOG_FILES: 'redstone_log_files',
  READ_LOG: 'redstone_read_log',
} as const

/** 红石联机统一调用入口（action/params 放在 req 中，与后端 `req: ActionRequest` 契约一致） */
export function redstoneManager<T = unknown>(action: string, params?: unknown): Promise<T> {
  return invoke<T>('redstone_manager', { req: { action, params: params ?? null } })
}

/** 获取中转服务器列表 */
export function redstoneGetServers(): Promise<RedStoneGetServersResult> {
  return redstoneManager<RedStoneGetServersResult>(REDSTONE_ACTIONS.GET_SERVERS)
}

/** 创建隧道（内部释放内核并拉起 hongshi 进程） */
export function redstoneStart(params: RedStoneStartParams): Promise<RedStoneStartResult> {
  return redstoneManager<RedStoneStartResult>(REDSTONE_ACTIONS.START, params)
}

/** 查询隧道状态（读 tunnel.ini） */
export function redstoneStatus(): Promise<RedStoneStatusResult> {
  return redstoneManager<RedStoneStatusResult>(REDSTONE_ACTIONS.STATUS)
}

/** 停止隧道 */
export function redstoneStop(): Promise<RedStoneStopResult> {
  return redstoneManager<RedStoneStopResult>(REDSTONE_ACTIONS.STOP)
}

/** 列出红石内核日志文件（logs/ 目录，按时间倒序） */
export function redstoneLogFiles(): Promise<RedStoneLogFilesResult> {
  return redstoneManager<RedStoneLogFilesResult>(REDSTONE_ACTIONS.LOG_FILES)
}

/** 读取指定红石内核日志文件尾部内容（maxLines 默认 500） */
export function redstoneReadLog(fileName: string, maxLines?: number): Promise<RedStoneReadLogResult> {
  return redstoneManager<RedStoneReadLogResult>(REDSTONE_ACTIONS.READ_LOG, {
    file_name: fileName,
    max_lines: maxLines ?? 500,
  })
}

export default {
  redstoneGetServers,
  redstoneStart,
  redstoneStatus,
  redstoneStop,
  redstoneLogFiles,
  redstoneReadLog,
}