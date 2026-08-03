/**
 * 图片缓存事件监听 composable
 *
 * 监听后端 image-cached 事件；基于 onGlobalEvent 全局单例 listener（永不 unlisten），
 * 避免图片下载任务在组件卸载后仍 emit 导致 Tauri 2.x "Couldn't find callback id" 竞态警告。
 */

import { onGlobalEvent } from '@/composables/useGlobalTauriEvent'

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
  onGlobalEvent<ImageCachedPayload>('image-cached', (payload) => {
    callback(payload.remote_url, payload.local_url)
  })
}
