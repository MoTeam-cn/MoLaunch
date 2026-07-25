/**
 * 系统模块统一 API 入口
 *
 * 后端 `system_manager` IPC 命令通过 `action` 字段分发到不同子模块
 * （参照 `meta_manager` / `image_cache_manager` / `config_manager` 模式）。
 *
 * 字段名约定：后端 Params 结构体使用 `#[serde(rename_all = "camelCase")]`，
 * 故前端 params 对象的字段名一律使用 camelCase。
 *
 * 注册的 action（18 个）：
 * - game_dir（7 个）：`open_game_dir` / `open_path` / `reveal_in_explorer`
 *   / `get_game_dir` / `write_text_file` / `get_system_memory` / `set_game_dir`
 * - config（2 个）：`get_config_path` / `save_config_to_file`
 * - developer（5 个）：`is_developer_unlocked` / `unlock_developer_mode`
 *   / `get_storage_dirs` / `get_system_info` / `get_cache_stats`
 * - about（1 个）：`get_about_data`
 * - logger（3 个）：`get_log_path` / `list_log_files` / `read_log_file`
 */

import { invoke } from '@tauri-apps/api/core'

/**
 * 调用 system_manager IPC
 * @param action 操作名称（取自 SYSTEM_ACTIONS 常量）
 * @param params 参数对象（字段名使用 camelCase）
 */
export async function systemManager<T = unknown>(
  action: string,
  params?: unknown,
): Promise<T> {
  return invoke<T>('system_manager', { req: { action, params: params ?? null } })
}

/**
 * 所有可用的 action 名称
 *
 * 与后端 `utils::system_manager::DISPATCHER` 注册的 action 一一对应。
 * 业务代码应优先使用此常量而非裸字符串，避免拼写错误。
 */
export const SYSTEM_ACTIONS = {
  // game_dir（7 个）
  OPEN_GAME_DIR: 'open_game_dir',
  OPEN_PATH: 'open_path',
  REVEAL_IN_EXPLORER: 'reveal_in_explorer',
  GET_GAME_DIR: 'get_game_dir',
  WRITE_TEXT_FILE: 'write_text_file',
  GET_SYSTEM_MEMORY: 'get_system_memory',
  SET_GAME_DIR: 'set_game_dir',
  // config（2 个）
  GET_CONFIG_PATH: 'get_config_path',
  SAVE_CONFIG_TO_FILE: 'save_config_to_file',
  // developer（5 个）
  IS_DEVELOPER_UNLOCKED: 'is_developer_unlocked',
  UNLOCK_DEVELOPER_MODE: 'unlock_developer_mode',
  GET_STORAGE_DIRS: 'get_storage_dirs',
  GET_SYSTEM_INFO: 'get_system_info',
  GET_CACHE_STATS: 'get_cache_stats',
  // about（1 个）
  GET_ABOUT_DATA: 'get_about_data',
  // logger（3 个）
  GET_LOG_PATH: 'get_log_path',
  LIST_LOG_FILES: 'list_log_files',
  READ_LOG_FILE: 'read_log_file',
} as const

/** action 名称类型 */
export type SystemAction = typeof SYSTEM_ACTIONS[keyof typeof SYSTEM_ACTIONS]
