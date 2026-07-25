/**
 * 图片缓存事件监听 composable
 *
 * 监听后端 `image-cached` 事件，当远程图片下载完成缓存到本地后通知调用方。
 *
 * 基于 `onGlobalEvent` 封装（全局单例 listener），避免 Tauri 2.x unlisten 竞态
 * 导致的 "Couldn't find callback id xxx" 警告。
 *
 * 背景：`image_cache::spawn_download` 是独立的 `tokio::spawn` 任务，不受
 * `cancelPreloadModsDetail` 控制。当 ModTab 卸载后，已 spawn 的图片下载任务
 * 仍在运行并 emit `image-cached`。传统 `listen`/`unlisten` 模式下，前端
 * callback 已被同步删除，Rust listener 尚未异步注销 → 触发警告。
 * 全局单例 listener 永不 unlisten，彻底消除该竞态。
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
