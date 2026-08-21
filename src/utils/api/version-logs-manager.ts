/**
 * 实例日志读取模块统一 API 入口
 * `version_logs_manager` IPC 经 `action` 分发（list_instance_logs / read_instance_log 共 2 个 action）。
 * params 字段名一律 camelCase（如 `dir` / `name`）。
 */

import { invoke } from '@tauri-apps/api/core'

/**
 * 调用 version_logs_manager IPC
 * @param action 操作名称（取自 VERSION_LOGS_ACTIONS 常量）
 * @param params 参数对象（字段名使用 camelCase）
 */
export async function versionLogsManager<T = unknown>(
  action: string,
  params?: unknown,
): Promise<T> {
  return invoke<T>('version_logs_manager', { req: { action, params: params ?? null } })
}

/**
 * 所有可用的 action 名称
 *
 * 与后端 `version::logs::DISPATCHER` 注册的 action 一一对应。
 */
export const VERSION_LOGS_ACTIONS = {
  LIST_INSTANCE_LOGS: 'list_instance_logs',
  READ_INSTANCE_LOG: 'read_instance_log',
} as const

/** action 名称类型 */
export type VersionLogsAction = typeof VERSION_LOGS_ACTIONS[keyof typeof VERSION_LOGS_ACTIONS]
