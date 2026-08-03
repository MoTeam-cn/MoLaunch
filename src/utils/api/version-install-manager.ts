/**
 * 版本安装管理统一 API 入口
 * `version_install_manager` IPC 经 `action` 分发（download + install + loaders + preload，共 11 个 action）。
 * params 字段名一律 camelCase（后端 `#[serde(rename_all = "camelCase")]`）。
 */

import { invoke } from '@tauri-apps/api/core'

/**
 * 调用 version_install_manager IPC
 * @param action 操作名称（取自 VERSION_INSTALL_ACTIONS 常量）
 * @param params 参数对象（字段名使用 camelCase），无参数 action 可省略
 */
export async function versionInstallManager<T = unknown>(
  action: string,
  params?: unknown,
): Promise<T> {
  return invoke<T>('version_install_manager', { req: { action, params: params ?? null } })
}

/**
 * 所有可用的 action 名称
 *
 * 与后端 `utils::version_install_manager::DISPATCHER` 注册的 action 一一对应。
 * 常量名使用大写蛇形，值使用小写下划线。
 */
export const VERSION_INSTALL_ACTIONS = {
  // download.rs（1 个）
  DOWNLOAD_VERSION: 'download_version',
  // install/mod.rs（1 个）
  INSTALL_MERGED: 'install_merged',
  // loaders.rs（8 个）
  LIST_FORGE_VERSIONS: 'list_forge_versions',
  LIST_NEOFORGE_VERSIONS: 'list_neoforge_versions',
  LIST_FABRIC_VERSIONS: 'list_fabric_versions',
  LIST_OPTIFINE_VERSIONS: 'list_optifine_versions',
  LIST_LITELOADER_VERSIONS: 'list_liteloader_versions',
  VALIDATE_LOADERS: 'validate_loaders',
  LIST_FABRIC_API_VERSIONS: 'list_fabric_api_versions',
  INSTALL_FABRIC_API_FOR_VERSION: 'install_fabric_api_for_version',
  // preload.rs（2 个）
  PRELOAD_MODS_DETAIL_CMD: 'preload_mods_detail_cmd',
  CANCEL_PRELOAD_MODS_DETAIL_CMD: 'cancel_preload_mods_detail_cmd',
} as const

/** action 名称类型 */
export type VersionInstallAction = typeof VERSION_INSTALL_ACTIONS[keyof typeof VERSION_INSTALL_ACTIONS]
