/**
 * 社区资源模块统一 API 入口
 *
 * 后端 community_manager IPC 按 action 分发；params 字段一律 camelCase。
 */

import { invoke } from '@tauri-apps/api/core'

/**
 * 调用 community_manager IPC
 * @param action 操作名称（取自 COMMUNITY_ACTIONS 常量）
 * @param params 参数对象（字段名使用 camelCase）
 */
export async function communityManager<T = unknown>(
  action: string,
  params?: unknown,
): Promise<T> {
  return invoke<T>('community_manager', { req: { action, params: params ?? null } })
}

/**
 * 所有可用的 action 名称
 *
 * 与后端 `utils::community_manager::DISPATCHER` 注册的 action 一一对应。
 */
export const COMMUNITY_ACTIONS = {
  SEARCH_RESOURCES: 'search_resources',
  GET_CATEGORY_TAGS: 'get_category_tags',
  GET_PROJECT_DETAIL: 'get_project_detail',
  GET_PROJECT_VERSIONS: 'get_project_versions',
  GET_MCMOD_URL: 'get_mcmod_url',
  DOWNLOAD_RESOURCE: 'download_resource',
  DOWNLOAD_RESOURCE_TO_PATH: 'download_resource_to_path',
  FORMAT_DOWNLOAD_FILENAME: 'format_download_filename',
  INSTALL_RESOURCE: 'install_resource',
  GET_RESOURCE_INSTALL_PATH: 'get_resource_install_path',
  INSTALL_MODPACK: 'install_modpack',
  INSTALL_LOCAL_MODPACK: 'install_local_modpack',
  PREVIEW_LOCAL_MODPACK: 'preview_local_modpack',
} as const

/** action 名称类型 */
export type CommunityAction = typeof COMMUNITY_ACTIONS[keyof typeof COMMUNITY_ACTIONS]
