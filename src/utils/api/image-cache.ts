/**
 * 通用图片缓存 API
 *
 * 将任意远程图片 URL 转为缓存 URL（方案 C：混合缓存）。
 * 适用于皮肤、披风、头像、缩略图等所有需要缓存的远程图片场景。
 *
 * - 缓存命中：返回 cache-image:// 本地 URL（cached: true），零网络请求
 * - 缓存未命中：返回原始远程 URL（cached: false），后端异步下载，完成后 emit 'image-cached' 事件
 */

import { invoke } from '@tauri-apps/api/core'

/**
 * 缓存图片结果
 * - url: 立即用于渲染的 URL（本地缓存或远程）
 * - cached: true 表示本地缓存命中，无需网络
 */
export interface CachedImage {
  url: string
  cached: boolean
}

/**
 * 获取图片的缓存 URL（通用接口）
 *
 * @param remoteUrl 远程图片 URL
 * @returns CachedImage，cached 为 true 时 url 是本地缓存 URL
 */
export async function getCachedImageUrl(remoteUrl: string): Promise<CachedImage> {
  return await invoke<CachedImage>('get_cached_image_url', { url: remoteUrl })
}

/**
 * 失效指定 URL 的图片缓存（强制刷新）
 *
 * @param remoteUrl 远程图片 URL
 */
export async function invalidateCachedImage(remoteUrl: string): Promise<void> {
  return await invoke<void>('invalidate_cached_image', { url: remoteUrl })
}

/**
 * 清空所有图片缓存
 */
export async function clearImageCache(): Promise<void> {
  return await invoke<void>('clear_image_cache')
}
