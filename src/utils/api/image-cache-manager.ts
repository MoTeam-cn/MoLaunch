/**
 * 图片缓存模块统一 API 入口
 *
 * 后端 `image_cache_manager` IPC 命令通过 `action` 字段分发到不同子模块
 * （参照 `meta_manager` / `tools_manager` 模式）。
 *
 * 字段名约定：后端 Params 结构体使用 `#[serde(rename_all = "camelCase")]`，
 * 故前端 params 对象的字段名一律使用 camelCase。
 */

import { invoke } from '@tauri-apps/api/core'

/**
 * 调用 image_cache_manager IPC
 * @param action 操作名称（取自 IMAGE_CACHE_ACTIONS 常量）
 * @param params 参数对象（字段名使用 camelCase）
 */
export async function imageCacheManager<T = unknown>(
  action: string,
  params?: unknown,
): Promise<T> {
  return invoke<T>('image_cache_manager', { req: { action, params: params ?? null } })
}

/**
 * 所有可用的 action 名称
 *
 * 与后端 `utils::image_cache_manager::DISPATCHER` 注册的 action 一一对应。
 */
export const IMAGE_CACHE_ACTIONS = {
  GET_CACHED_IMAGE_URL: 'get_cached_image_url',
  INVALIDATE_CACHED_IMAGE: 'invalidate_cached_image',
  CLEAR_IMAGE_CACHE: 'clear_image_cache',
} as const

/** action 名称类型 */
export type ImageCacheAction = typeof IMAGE_CACHE_ACTIONS[keyof typeof IMAGE_CACHE_ACTIONS]
