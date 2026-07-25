/**
 * 下载进度模块统一 API 入口
 *
 * 后端 `version_progress_manager` IPC 命令通过 `action` 字段分发到不同子模块
 * （参照 `image_cache_manager` / `meta_manager` 模式）。
 *
 * 字段名约定：后端 Params 结构体使用 `#[serde(rename_all = "camelCase")]`，
 * 故前端 params 对象的字段名一律使用 camelCase。本模块 6 个 action 均无参数。
 */

import { invoke } from '@tauri-apps/api/core'

/**
 * 调用 version_progress_manager IPC
 * @param action 操作名称（取自 VERSION_PROGRESS_ACTIONS 常量）
 * @param params 参数对象（字段名使用 camelCase），本模块 6 个 action 均无参数
 */
export async function versionProgressManager<T = unknown>(
  action: string,
  params?: unknown,
): Promise<T> {
  return invoke<T>('version_progress_manager', { req: { action, params: params ?? null } })
}

/**
 * 所有可用的 action 名称
 *
 * 与后端 `utils::version_progress_manager::DISPATCHER` 注册的 action 一一对应。
 */
export const VERSION_PROGRESS_ACTIONS = {
  GET_DOWNLOAD_PROGRESS: 'get_download_progress',
  IS_DOWNLOADING: 'is_downloading',
  RESET_DOWNLOAD_PROGRESS: 'reset_download_progress',
  CANCEL_DOWNLOAD: 'cancel_download',
  PAUSE_DOWNLOAD: 'pause_download',
  RESUME_DOWNLOAD: 'resume_download',
} as const

/** action 名称类型 */
export type VersionProgressAction = typeof VERSION_PROGRESS_ACTIONS[keyof typeof VERSION_PROGRESS_ACTIONS]
