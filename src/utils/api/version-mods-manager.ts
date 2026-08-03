/**
 * 版本 Mod 管理模块统一 API 入口
 * `version_mods_manager` IPC 经 `action` 分发；params 字段名一律 camelCase。
 */

import { invoke } from '@tauri-apps/api/core'

/**
 * 调用 version_mods_manager IPC
 * @param action 操作名称（取自 VERSION_MODS_ACTIONS 常量）
 * @param params 参数对象（字段名使用 camelCase），unwatch_mods_dir 无参数
 */
export async function versionModsManager<T = unknown>(
  action: string,
  params?: unknown,
): Promise<T> {
  return invoke<T>('version_mods_manager', { req: { action, params: params ?? null } })
}

/**
 * 所有可用的 action 名称
 *
 * 与后端 `utils::version_mods_manager::DISPATCHER` 注册的 action 一一对应。
 */
export const VERSION_MODS_ACTIONS = {
  // list.rs（2 个）
  IS_VERSION_MODABLE: 'is_version_modable',
  LIST_MODS: 'list_mods',
  // manage.rs（2 个）
  TOGGLE_MOD: 'toggle_mod',
  DELETE_MOD: 'delete_mod',
  // install.rs（4 个）
  INSTALL_MOD: 'install_mod',
  OPEN_MODS_DIR: 'open_mods_dir',
  REVEAL_MOD_FILE: 'reveal_mod_file',
  GET_VERSION_MODS_DIR: 'get_version_mods_dir',
  // update.rs（1 个，阶段 4 新增）
  UPDATE_MOD: 'update_mod',
  // watcher.rs（2 个）
  WATCH_MODS_DIR: 'watch_mods_dir',
  UNWATCH_MODS_DIR: 'unwatch_mods_dir',
  // dependency_resolver.rs（2 个，前置 mod 检查与安装）
  CHECK_MOD_DEPENDENCIES: 'check_mod_dependencies',
  INSTALL_MOD_WITH_DEPENDENCIES: 'install_mod_with_dependencies',
} as const

/** action 名称类型 */
export type VersionModsAction = typeof VERSION_MODS_ACTIONS[keyof typeof VERSION_MODS_ACTIONS]
