/**
 * 全局单例 Tauri 事件监听
 *
 * 解决 Tauri 2.x unlisten 竞态（_unlisten 先删前端 callback 再异步通知 Rust，
 * 期间 Rust 仍 emit 导致 "Couldn't find callback id xxx" 警告）：
 * 每个事件名维护全局单例 listener 且永不 unlisten，组件经 onGlobalEvent 注册 handler，
 * 卸载时自动从 Set 移除；适用后台异步事件，不适用需精确控制生命周期的一次性事件。
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
 * 必须在 Vue 组件 setup 阶段调用（内部使用 `onUnmounted`），
 * 或传 `autoRemove: false` 在组件上下文之外调用（handler 永久驻留，如全局联机会话）。
 *
 * @param eventName Tauri 事件名
 * @param handler 事件处理函数
 * @param options.autoRemove 组件卸载时自动移除 handler（默认 true）
 */
export function onGlobalEvent<T>(
  eventName: string,
  handler: Handler<T>,
  options?: { autoRemove?: boolean },
): void {
  const autoRemove = options?.autoRemove ?? true
  ensureListener<T>(eventName)
  const set = handlerSets.get(eventName)!
  set.add(handler as Handler<unknown>)
  if (autoRemove) {
    onUnmounted(() => {
      set.delete(handler as Handler<unknown>)
    })
  }
}
