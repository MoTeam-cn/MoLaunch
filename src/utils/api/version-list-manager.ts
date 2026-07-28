/**
 * 版本列表/文件夹/管理/个性化模块统一 API 入口
 *
 * 后端 `version_list_manager` IPC 命令通过 `action` 字段分发到不同子模块
 * （list 8 个 + folder 5 个 + manage 4 个 + personalization 2 个，共 19 个 action），
 * 参照 `image_cache_manager` / `meta_manager` / `version_mods_manager` 模式。
 *
 * 字段名约定：后端 Params 结构体使用 `#[serde(rename_all = "camelCase")]`，
 * 故前端 params 对象的字段名一律使用 camelCase（如 `versionId` / `newName` / `path`）。
 */

import { invoke } from '@tauri-apps/api/core'

/**
 * 调用 version_list_manager IPC
 * @param action 操作名称（取自 VERSION_LIST_ACTIONS 常量）
 * @param params 参数对象（字段名使用 camelCase），无参数的 action 可省略
 */
export async function versionListManager<T = unknown>(
  action: string,
  params?: unknown,
): Promise<T> {
  return invoke<T>('version_list_manager', { req: { action, params: params ?? null } })
}

/**
 * 所有可用的 action 名称
 *
 * 与后端 `utils::version_list_manager::DISPATCHER` 注册的 action 一一对应。
 * 常量名使用大写蛇形，值使用小写下划线。
 */
export const VERSION_LIST_ACTIONS = {
  // list.rs（8 个）
  LIST_VERSIONS: 'list_versions',
  LIST_INSTALLED_VERSIONS: 'list_installed_versions',
  LIST_INSTALLED_VERSIONS_WITH_TYPE: 'list_installed_versions_with_type',
  UNINSTALL_VERSION: 'uninstall_version',
  GET_VERSION_EFFECTIVE_DIR: 'get_version_effective_dir',
  GET_VERSION_GAME_VERSION: 'get_version_game_version',
  GET_VERSION_LOADER_INFO: 'get_version_loader_info',
  READ_LOCAL_MODPACK_META: 'read_local_modpack_meta',
  // 联机大厅阶段 4 新增：校验本地是否已安装指定整合包（加入方判断是否需要一键安装）
  CHECK_LOCAL_MODPACK: 'check_local_modpack',
  // folder.rs（5 个）
  LIST_MC_FOLDERS: 'list_mc_folders',
  ADD_MC_FOLDER: 'add_mc_folder',
  REMOVE_MC_FOLDER: 'remove_mc_folder',
  SWITCH_MC_FOLDER: 'switch_mc_folder',
  RENAME_MC_FOLDER: 'rename_mc_folder',
  // manage.rs（4 个）
  FIX_VERSION_FILES: 'fix_version_files',
  RENAME_VERSION: 'rename_version',
  GET_SELECTED_VERSION: 'get_selected_version',
  SET_SELECTED_VERSION: 'set_selected_version',
  // personalization.rs（2 个）
  GET_VERSION_PERSONALIZATION: 'get_version_personalization',
  UPDATE_VERSION_PERSONALIZATION: 'update_version_personalization',
} as const

/** action 名称类型 */
export type VersionListAction = typeof VERSION_LIST_ACTIONS[keyof typeof VERSION_LIST_ACTIONS]
