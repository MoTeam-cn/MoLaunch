/**
 * Mod 详情预加载事件监听
 *
 * 后台加载完成后通过事件刷新 UI 的模式：
 * 后端 `preload_mods_detail_cmd` 命令在后台并发：
 *   1. 读 JAR 元数据（slug / 描述 / 版本 / logo / 译名）
 *   2. 批量查询 CF/MR 工程详情
 * 每完成一个就 emit `mods-preload-update` 事件，前端通过本 composable 监听并更新对应 mod。
 *
 * 全部完成后 emit `mods-preload-done` 事件，前端据此设置 `isPreloadDone = true`。
 * `handleShowInfo` 在 slug 为空时判断此标志：若预加载已完成则立即走本地信息弹窗，不等待。
 *
 * 事件可能分两次 emit：
 * - 元数据先到（slug/description/version/translated_name 字段非 null）
 * - project 后到（project 字段非 null，同时 cached_logo_url 一起到）
 * 也可能一次性 emit（缓存命中时所有字段一起到）。
 *
 * 使用全局单例 listener（`onGlobalEvent`），避免 Tauri 2.x `unlisten` 竞态
 * 导致的 "Couldn't find callback id xxx" 警告。组件卸载时 handler 自动从 Set 移除，
 * Tauri listener 永不 unlisten。
 *
 * 使用方式：
 * ```ts
 * const { isPreloadDone } = useModsPreload(mods)
 * // 事件监听在构造时自动注册，无需手动 startListener
 * // 组件卸载时 handler 自动移除
 * ```
 */

import { ref, type Ref } from 'vue'
import { onGlobalEvent } from '@/composables/useGlobalTauriEvent'
import type { ModInfo } from '@/utils/api/personalization'
import type { ResourceProject } from '@/types/community'

/** 预加载事件 payload（与后端 `PreloadUpdate` 结构对齐） */
interface PreloadUpdatePayload {
  file_name: string
  /** JAR 内读到的 slug（undefined 表示不更新） */
  slug?: string
  /** JAR 内读到的描述（undefined 表示不更新） */
  description?: string
  /** JAR 内读到的版本号（undefined 表示不更新） */
  version?: string
  /** 平台工程 logo_url 经 image_cache 处理后的缓存 URL（undefined 表示不更新） */
  cached_logo_url?: string
  /** mcmod 数据库查到的中文译名（undefined 表示不更新） */
  translated_name?: string
  /** CF/MR 查到的平台工程（undefined 表示尚未查询或未查到） */
  project?: ResourceProject
}

/**
 * 监听 `mods-preload-update` 事件，自动更新 mods 数组中对应 mod 的字段
 *
 * @param mods 响应式的 mod 列表 ref（按 file_name 匹配更新）
 * @returns `{ startListener, stopListener, isPreloadDone }`
 *   - `startListener`/`stopListener` 保留为 no-op 以兼容旧调用方，
 *     实际监听在构造时通过 `onGlobalEvent` 自动注册，`onUnmounted` 自动移除。
 */
export function useModsPreload(mods: Ref<ModInfo[]>) {
  /** 预加载是否已完成（后端 emit `mods-preload-done` 后置 true） */
  const isPreloadDone = ref(false)

  // 注册全局事件 handler（onUnmounted 自动移除 handler，Tauri listener 永不 unlisten）
  onGlobalEvent<PreloadUpdatePayload>('mods-preload-update', (payload) => {
    const { file_name } = payload
    if (!file_name) return

    // 在 mods 数组中查找匹配的 mod（按 file_name）
    // 用 for 循环 + 索引赋值确保 Vue 响应式触发更新
    for (let i = 0; i < mods.value.length; i++) {
      if (mods.value[i].file_name === file_name) {
        const old = mods.value[i]
        // 合并所有非 undefined 的字段
        const updated: ModInfo = {
          ...old,
          ...(payload.slug !== undefined ? { slug: payload.slug } : {}),
          ...(payload.description !== undefined ? { description: payload.description } : {}),
          ...(payload.version !== undefined ? { version: payload.version } : {}),
          ...(payload.cached_logo_url !== undefined ? { cached_logo_url: payload.cached_logo_url } : {}),
          ...(payload.translated_name !== undefined ? { translated_name: payload.translated_name } : {}),
          // project 仅在原值为空或新值非 null 时更新（避免覆盖已加载的更优结果）
          ...(payload.project && !old.project ? { project: payload.project } : {}),
        }
        mods.value[i] = updated
        break
      }
    }
  })

  onGlobalEvent<void>('mods-preload-done', () => {
    isPreloadDone.value = true
  })

  // 兼容旧 API：全局 listener 始终存活，无需手动 start/stop
  const startListener = async () => {}
  const stopListener = () => {}

  return { startListener, stopListener, isPreloadDone }
}
