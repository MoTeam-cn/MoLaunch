/**
 * 红石联机（hongshi 内核）IPC 封装：`redstone_manager` 经 action 分发
 *
 * params 字段名与后端契约一致（`redstone_start` 为 `mc_port`）。
 * 错误经 invoke reject 透出，message 为后端中文文案。
 */
import { invoke } from '@tauri-apps/api/core'
import type {
  RedStoneGetServersResult,
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
} as const

/** 红石联机统一调用入口 */
export function redstoneManager<T = unknown>(action: string, params?: unknown): Promise<T> {
  return invoke<T>('redstone_manager', { action, params: params ?? null })
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

export default {
  redstoneGetServers,
  redstoneStart,
  redstoneStatus,
  redstoneStop,
}