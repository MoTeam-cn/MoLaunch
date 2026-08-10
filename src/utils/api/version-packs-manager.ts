/**
 * 版本资源包/光影管理模块统一 API 入口
 * `version_packs_manager` IPC 经 `action` 分发；params 字段名一律 camelCase，均携带 `kind`。
 */

import { invoke } from '@tauri-apps/api/core'

/**
 * 调用 version_packs_manager IPC
 * @param action 操作名称（取自 VERSION_PACKS_ACTIONS 常量）
 * @param params 参数对象（字段名使用 camelCase），unwatch_packs_dir 无参数
 */
export async function versionPacksManager<T = unknown>(
  action: string,
  params?: unknown,
): Promise<T> {
  return invoke<T>('version_packs_manager', { req: { action, params: params ?? null } })
}

/**
 * 所有可用的 action 名称
 *
 * 与后端 `packs::manager::DISPATCHER` 注册的 action 一一对应。
 */
export const VERSION_PACKS_ACTIONS = {
  // list.rs（2 个）
  IS_PACKS_AVAILABLE: 'is_packs_available',
  LIST_PACKS: 'list_packs',
  // manage.rs（2 个）
  TOGGLE_PACK: 'toggle_pack',
  DELETE_PACK: 'delete_pack',
  // install.rs（5 个）
  INSTALL_PACK: 'install_pack',
  OPEN_PACKS_DIR: 'open_packs_dir',
  REVEAL_PACK_FILE: 'reveal_pack_file',
  GET_PACK_ICON: 'get_pack_icon',
  // update.rs（1 个）
  UPDATE_PACK: 'update_pack',
  // watcher.rs（2 个）
  WATCH_PACKS_DIR: 'watch_packs_dir',
  UNWATCH_PACKS_DIR: 'unwatch_packs_dir',
} as const

/** action 名称类型 */
export type VersionPacksAction = typeof VERSION_PACKS_ACTIONS[keyof typeof VERSION_PACKS_ACTIONS]
