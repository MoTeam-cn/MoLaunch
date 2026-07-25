/**
 * 版本启动管理统一 API 入口
 *
 * 后端 `version_launch_manager` IPC 命令通过 `action` 字段分发到不同子模块
 * （launch + script_export 共 7 个 action），参照 `meta_manager` / `tools_manager` /
 * `image_cache_manager` 模式。
 *
 * 字段名约定：后端 Params 结构体使用 `#[serde(rename_all = "camelCase")]`，
 * 故前端 params 对象的字段名一律使用 camelCase。
 */

import { invoke } from '@tauri-apps/api/core'

/**
 * 调用 version_launch_manager IPC
 * @param action 操作名称（取自 VERSION_LAUNCH_ACTIONS 常量）
 * @param params 参数对象（字段名使用 camelCase）
 */
export async function versionLaunchManager<T = unknown>(
  action: string,
  params?: unknown,
): Promise<T> {
  return invoke<T>('version_launch_manager', { req: { action, params: params ?? null } })
}

/**
 * 所有可用的 action 名称
 *
 * 与后端 `utils::version_launch_manager::DISPATCHER` 注册的 action 一一对应。
 * 常量名使用大写蛇形，值使用小写下划线。
 */
export const VERSION_LAUNCH_ACTIONS = {
  LAUNCH_GAME: 'launch_game',
  GET_LAUNCH_PROGRESS: 'get_launch_progress',
  CANCEL_LAUNCH: 'cancel_launch',
  STOP_GAME: 'stop_game',
  GET_RUNNING_GAME: 'get_running_game',
  GET_LAUNCH_HISTORY: 'get_launch_history',
  EXPORT_LAUNCH_SCRIPT: 'export_launch_script',
} as const

/** action 名称类型 */
export type VersionLaunchAction = typeof VERSION_LAUNCH_ACTIONS[keyof typeof VERSION_LAUNCH_ACTIONS]
