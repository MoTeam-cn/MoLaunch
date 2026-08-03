/**
 * 通用图片缓存 API
 *
 * 将远程图片 URL 转为缓存 URL：命中返回 cache-image:// 本地 URL（cached: true），
 * 未命中返回原 URL（cached: false）由后端异步下载后 emit 'image-cached'。
 */

import { IMAGE_CACHE_ACTIONS, imageCacheManager } from './image-cache-manager'

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
  return imageCacheManager<CachedImage>(IMAGE_CACHE_ACTIONS.GET_CACHED_IMAGE_URL, { url: remoteUrl })
}

/**
 * 失效指定 URL 的图片缓存（强制刷新）
 *
 * @param remoteUrl 远程图片 URL
 */
export async function invalidateCachedImage(remoteUrl: string): Promise<void> {
  return imageCacheManager<void>(IMAGE_CACHE_ACTIONS.INVALIDATE_CACHED_IMAGE, { url: remoteUrl })
}

/**
 * 清空所有图片缓存
 */
export async function clearImageCache(): Promise<void> {
  return imageCacheManager<void>(IMAGE_CACHE_ACTIONS.CLEAR_IMAGE_CACHE)
}
