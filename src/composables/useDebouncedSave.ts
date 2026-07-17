/**
 * 防抖保存 composable
 *
 * 两种用法：
 *
 * 1. 简单模式：`useDebouncedSave(flushFn)` - 只做时间防抖，不追踪字段
 * 2. 字段追踪模式：`useDebouncedSave.patch()` - 自动追踪改变的字段，flush 时只传改变的字段
 *
 * 字段追踪模式解决的核心问题：
 * 用户只改了一项，但旧实现会把整个对象都传过去 apply，造成后端无意义的覆盖写入。
 * 新模式用 `markDirty(key, value)` 标记改变的字段，防抖触发后只把改过的字段传给后端。
 *
 * 即使跨组件累积：用户在侧栏 A 改了字段 X，切到侧栏 B 改了字段 Y，
 * 只要两个侧栏共用同一个 patchSaver（或累积到同一份 patch），flush 时只会传 X 和 Y。
 */
import { onScopeDispose } from 'vue'
import type { ConfigPatch } from '@/utils/api/config'

/** 简单模式：只做时间防抖 */
export function useDebouncedSave(
  flushFn: () => Promise<void> | void,
  delay?: number,
): { scheduleSave: () => void; flushSave: () => void }

/** 字段追踪模式：自动追踪改变的字段，flush 时只传改变的字段 */
export function useDebouncedSave(
  pattern: 'patch',
  flushFn: (patch: ConfigPatch) => Promise<void> | void,
  delay?: number,
): {
  /** 标记字段已改变，并防抖触发 flush */
  markDirty: (key: keyof ConfigPatch, value: unknown) => void
  /** 立即 flush，返回未保存的 patch */
  flushSave: () => ConfigPatch
  /** 防抖触发 flush */
  scheduleSave: () => void
  /** 是否有未保存的改动 */
  isDirty: () => boolean
}

export function useDebouncedSave(
  arg1: 'patch' | (() => Promise<void> | void),
  arg2?: (() => Promise<void> | void) | ((patch: ConfigPatch) => Promise<void> | void) | number,
  arg3?: number,
): any {
  // ============ 字段追踪模式 ============
  if (arg1 === 'patch') {
    const flushFn = arg2 as (patch: ConfigPatch) => Promise<void> | void
    const delay = arg3 ?? 800
    let timer: ReturnType<typeof setTimeout> | null = null
    // 累积的改动字段
    const dirtyPatch: ConfigPatch = {}

    function clearTimer() {
      if (timer) {
        clearTimeout(timer)
        timer = null
      }
    }

    function isDirty() {
      return Object.keys(dirtyPatch).length > 0
    }

    function markDirty(key: keyof ConfigPatch, value: unknown) {
      ;(dirtyPatch as Record<string, unknown>)[key as string] = value
      clearTimer()
      timer = setTimeout(() => {
        timer = null
        void flushFn({ ...dirtyPatch })
        for (const k of Object.keys(dirtyPatch)) delete (dirtyPatch as Record<string, unknown>)[k]
      }, delay)
    }

    function flushSave(): ConfigPatch {
      clearTimer()
      if (!isDirty()) return {}
      const patch = { ...dirtyPatch }
      for (const k of Object.keys(dirtyPatch)) delete (dirtyPatch as Record<string, unknown>)[k]
      void flushFn(patch)
      return patch
    }

    function scheduleSave() {
      // markDirty 已经触发了 schedule，这里用于外部主动触发（保留 dirty）
      clearTimer()
      timer = setTimeout(() => {
        timer = null
        if (isDirty()) {
          void flushFn({ ...dirtyPatch })
          for (const k of Object.keys(dirtyPatch)) delete (dirtyPatch as Record<string, unknown>)[k]
        }
      }, delay)
    }

    onScopeDispose(() => {
      flushSave()
    })

    return { markDirty, flushSave, scheduleSave, isDirty }
  }

  // ============ 简单模式（兼容旧用法） ============
  const flushFn = arg1 as () => Promise<void> | void
  const delay = (arg2 as number) ?? 800
  let timer: ReturnType<typeof setTimeout> | null = null
  let dirty = false

  function clearTimer() {
    if (timer) {
      clearTimeout(timer)
      timer = null
    }
  }

  function scheduleSave() {
    dirty = true
    clearTimer()
    timer = setTimeout(() => {
      timer = null
      dirty = false
      void flushFn()
    }, delay)
  }

  function flushSave() {
    clearTimer()
    if (!dirty) return
    dirty = false
    void flushFn()
  }

  onScopeDispose(() => {
    flushSave()
  })

  return { scheduleSave, flushSave }
}
