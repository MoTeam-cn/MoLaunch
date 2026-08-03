/**
 * 插件模块统一 API 入口
 *
 * 后端 plugins_manager IPC 按 action 分发；params 字段一律 camelCase。
 */

import { invoke } from '@tauri-apps/api/core'

/**
 * 调用 plugins_manager IPC
 * @param action 操作名称（取自 PLUGINS_ACTIONS 常量）
 * @param params 参数对象（字段名使用 camelCase）
 */
export async function pluginsManager<T = unknown>(
  action: string,
  params?: unknown,
): Promise<T> {
  return invoke<T>('plugins_manager', { req: { action, params: params ?? null } })
}

/**
 * 所有可用的 action 名称
 *
 * 与后端 `utils::plugins_manager::DISPATCHER` 注册的 action 一一对应。
 */
export const PLUGINS_ACTIONS = {
  LIST_EXTERNAL_PLUGINS: 'list_external_plugins',
  READ_EXTERNAL_PLUGIN_FILE: 'read_external_plugin_file',
  UNINSTALL_EXTERNAL_PLUGIN: 'uninstall_external_plugin',
  INSTALL_EXTERNAL_PLUGIN_FROM_DIR: 'install_external_plugin_from_dir',
  INSTALL_EXTERNAL_PLUGIN_FROM_ZIP: 'install_external_plugin_from_zip',
  PLUGIN_SPAWN_PROCESS: 'plugin_spawn_process',
  PLUGIN_CREATE_WINDOW: 'plugin_create_window',
  LOAD_CUSTOM_LAYOUT: 'load_custom_layout',
  READ_LAYOUT_SAMPLE: 'read_layout_sample',
  EXPORT_PLUGIN_SAMPLE: 'export_plugin_sample',
  READ_PERSONALIZATION: 'read_personalization',
  WRITE_PERSONALIZATION: 'write_personalization',
} as const

/** action 名称类型 */
export type PluginsAction = typeof PLUGINS_ACTIONS[keyof typeof PLUGINS_ACTIONS]
