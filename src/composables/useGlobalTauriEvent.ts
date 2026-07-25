/**
 * 全局单例 Tauri 事件监听
 *
 * 解决 Tauri 2.x `unlisten` 的固有竞态：
 * `_unlisten` 先同步删除前端 callback，再异步通知 Rust 删除 listener，
 * 期间 Rust 仍在 emit 事件 → 触发 "Couldn't find callback id xxx" 警告。
 *
 * 本 composable 为每个事件名维护一个全局单例 Tauri listener（永不 unlisten），
 * 组件通过 `onGlobalEvent` 注册 handler，`onUnmounted` 自动从 Set 中移除 handler，
 * 完全绕开 Tauri 的 callback 删除竞态。
 *
 * 适用：后台异步任务 emit 的事件（image-cached、mods-preload-update、mods-dir-changed 等），
 * 这些事件的 Rust emit 时机不受前端组件生命周期控制。
 *
 * 不适用：需要精确控制 Rust listener 生命周期的场景（如一次性事件）。
 *
 * @example
 * ```ts
 * onGlobalEvent<MyPayload>('my-event', (payload) => {
 *   console.log('收到:', payload)
 * })
 * // 组件卸载时 handler 自动移除，Tauri listener 保留
 * ```
 */
import { onUnmounted } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

type Handler<T> = (payload: T) => void

/** 事件名 → Tauri listener 的 unlisten 句柄 Promise（单例，永不 unlisten） */
const listenerPromises = new Map<string, Promise<UnlistenFn>>()

/** 事件名 → 已注册的 handler 集合 */
const handlerSets = new Map<string, Set<Handler<unknown>>>()

/** 确保事件名对应的全局 Tauri listener 已创建（单例，仅创建一次） */
function ensureListener<T>(eventName: string): void {
  if (listenerPromises.has(eventName)) return

  const handlers = new Set<Handler<T>>()
  handlerSets.set(eventName, handlers as Set<Handler<unknown>>)

  const promise = listen<T>(eventName, (event) => {
    const set = handlerSets.get(eventName)
    if (!set) return
    for (const h of set) {
      try {
        (h as Handler<T>)(event.payload)
      } catch (e) {
        console.error(`[GlobalTauriEvent] ${eventName} handler error:`, e)
      }
    }
  })
  listenerPromises.set(eventName, promise)
}

/**
 * 注册全局事件 handler（组件卸载时自动移除 handler）
 *
 * 底层使用单例 Tauri listener，永不 unlisten，避免 Tauri 2.x 的 callback 删除竞态。
 * handler 仅从本地 Set 中增删，不影响 Tauri listener 生命周期。
 *
 * 必须在 Vue 组件 setup 阶段调用（内部使用 `onUnmounted`）。
 *
 * @param eventName Tauri 事件名
 * @param handler 事件处理函数
 */
export function onGlobalEvent<T>(eventName: string, handler: Handler<T>): void {
  ensureListener<T>(eventName)
  const set = handlerSets.get(eventName)!
  set.add(handler as Handler<unknown>)
  onUnmounted(() => {
    set.delete(handler as Handler<unknown>)
  })
}
