/**
 * 图片缓存事件监听 composable
 *
 * 监听后端 `image-cached` 事件，当远程图片下载完成缓存到本地后通知调用方。
 *
 * 基于 `useTauriEvent` 封装，自动管理 unlisten 句柄和 onUnmounted 清理。
 *
 * @example
 * ```ts
 * onImageCached((remoteUrl, localUrl) => {
 *   if (skinUrl.value === remoteUrl) {
 *     skinUrl.value = localUrl
 *   }
 * })
 * ```
 */

import { useTauriEvent } from '@/composables/useTauriEvent'

interface ImageCachedPayload {
  remote_url: string
  local_url: string
}

/**
 * 监听 image-cached 事件，通过回调通知调用方
 *
 * @param callback 收到事件时的回调，参数为 (remoteUrl, localUrl)
 */
export function onImageCached(callback: (remoteUrl: string, localUrl: string) => void): void {
  const { start } = useTauriEvent<ImageCachedPayload>('image-cached', (payload) => {
    callback(payload.remote_url, payload.local_url)
  })
  start()
}
