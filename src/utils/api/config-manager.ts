/**
 * 配置管理模块统一 API 入口
 *
 * 后端 config_manager IPC 按 action 分发（get_config / apply_config）；
 * params 字段一律 camelCase。get_config_path / save_config_to_file 仍走独立 invoke。
 */

import { invoke } from '@tauri-apps/api/core'

/**
 * 调用 config_manager IPC
 * @param action 操作名称（取自 CONFIG_ACTIONS 常量）
 * @param params 参数对象（字段名使用 camelCase）
 */
export async function configManager<T = unknown>(
  action: string,
  params?: unknown,
): Promise<T> {
  return invoke<T>('config_manager', { req: { action, params: params ?? null } })
}

/**
 * 所有可用的 action 名称
 *
 * 与后端 `utils::config_manager::DISPATCHER` 注册的 action 一一对应。
 * 业务代码应优先使用此常量而非裸字符串，避免拼写错误。
 */
export const CONFIG_ACTIONS = {
  GET_CONFIG: 'get_config',
  APPLY_CONFIG: 'apply_config',
} as const

/** action 名称类型 */
export type ConfigAction = typeof CONFIG_ACTIONS[keyof typeof CONFIG_ACTIONS]
