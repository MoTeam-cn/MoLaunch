/**
 * 配置管理模块统一 API 入口
 *
 * 后端 `config_manager` IPC 命令通过 `action` 字段分发到不同子模块
 * （参照 `meta_manager` / `image_cache_manager` / `java_manager` 模式）。
 *
 * 字段名约定：后端 Params 结构体使用 `#[serde(rename_all = "camelCase")]`，
 * 故前端 params 对象的字段名一律使用 camelCase。
 *
 * 注册的 action（2 个）：
 * - `get_config`：读取配置（扁平化数组）
 * - `apply_config`：统一配置更新
 *
 * 注：`get_config_path` / `save_config_to_file` 不在本次聚合范围，仍走独立 invoke。
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
