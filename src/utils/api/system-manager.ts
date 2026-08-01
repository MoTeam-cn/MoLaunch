/**
 * 系统模块统一 API 入口
 *
 * 后端 `system_manager` IPC 命令通过 `action` 字段分发到不同子模块
 * （参照 `meta_manager` / `image_cache_manager` / `config_manager` 模式）。
 *
 * 字段名约定：后端 Params 结构体使用 `#[serde(rename_all = "camelCase")]`，
 * 故前端 params 对象的字段名一律使用 camelCase。
 *
 * 注册的 action（29 个）：
 * - game_dir（7 个）：`open_game_dir` / `open_path` / `reveal_in_explorer`
 *   / `get_game_dir` / `write_text_file` / `get_system_memory` / `set_game_dir`
 * - config（2 个）：`get_config_path` / `save_config_to_file`
 * - developer（6 个）：`is_developer_unlocked` / `unlock_developer_mode`
 *   / `lock_developer_mode` / `get_storage_dirs` / `get_system_info` / `get_cache_stats`
 * - devtools（3 个）：`open_devtools` / `close_devtools` / `is_devtools_open`
 *   （开发者模式解锁且开启时可调出 WebView2 DevTools）
 * - about（1 个）：`get_about_data`
 * - logger（3 个）：`get_log_path` / `list_log_files` / `read_log_file`
 * - http_log（2 个）：`read_http_logs` / `list_http_log_files`
 * - updater（2 个）：`check_update` / `download_and_install_update`
 * - certs（3 个）：`list_custom_certs` / `add_custom_cert` / `remove_custom_cert`
 * - ws（1 个）：`get_ws_port`（下载进度推送 WS 端口）
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
  // developer（6 个）
  IS_DEVELOPER_UNLOCKED: 'is_developer_unlocked',
  UNLOCK_DEVELOPER_MODE: 'unlock_developer_mode',
  LOCK_DEVELOPER_MODE: 'lock_developer_mode',
  GET_STORAGE_DIRS: 'get_storage_dirs',
  GET_SYSTEM_INFO: 'get_system_info',
  GET_CACHE_STATS: 'get_cache_stats',
  // devtools（3 个）—— 调出/关闭/查询 WebView2 开发者工具
  // 后端 require_dev_mode() 校验 DeveloperUnlocked && DeveloperMode，普通用户无法触发
  OPEN_DEVTOOLS: 'open_devtools',
  CLOSE_DEVTOOLS: 'close_devtools',
  IS_DEVTOOLS_OPEN: 'is_devtools_open',
  // about（1 个）
  GET_ABOUT_DATA: 'get_about_data',
  // logger（3 个）
  GET_LOG_PATH: 'get_log_path',
  LIST_LOG_FILES: 'list_log_files',
  READ_LOG_FILE: 'read_log_file',
  // http_log（2 个）—— 联机 API 调用追踪
  READ_HTTP_LOGS: 'read_http_logs',
  LIST_HTTP_LOG_FILES: 'list_http_log_files',
  // updater（4 个）—— Windows 便携版自实现 + macOS/Linux 转发官方 plugin
  CHECK_UPDATE: 'check_update',
  DOWNLOAD_AND_INSTALL_UPDATE: 'download_and_install_update',
  // Windows 后台静默下载新版本到 appdata/last.exe + 退出时应用替换
  DOWNLOAD_UPDATE_TO_APPDATA: 'download_update_to_appdata',
  APPLY_PENDING_UPDATE: 'apply_pending_update',
  // certs（3 个）—— 自定义 TLS 证书管理
  LIST_CUSTOM_CERTS: 'list_custom_certs',
  ADD_CUSTOM_CERT: 'add_custom_cert',
  REMOVE_CUSTOM_CERT: 'remove_custom_cert',
  // deeplink（3 个）—— molaunch:// 协议注册状态查询/注册/卸载（便携版用）
  GET_DEEPLINK_STATUS: 'get_deeplink_status',
  REGISTER_DEEPLINK: 'register_deeplink',
  UNREGISTER_DEEPLINK: 'unregister_deeplink',
  // ws（1 个）—— 下载进度推送 WebSocket 端口
  GET_WS_PORT: 'get_ws_port',
} as const

/** action 名称类型 */
export type SystemAction = typeof SYSTEM_ACTIONS[keyof typeof SYSTEM_ACTIONS]
