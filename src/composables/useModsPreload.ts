/**
 * Mod 详情预加载事件监听
 *
 * 参考 PCL2 `LocalResourceOnlineLoader` 后台加载完成后通过事件刷新 UI 的模式：
 * 后端 `preload_mods_detail_cmd` 命令在后台并发：
 *   1. 读 JAR 元数据（slug / 描述 / 版本 / logo / 译名）
 *   2. 批量查询 CF/MR 工程详情
 * 每完成一个就 emit `mods-preload-update` 事件，前端通过本 composable 监听并更新对应 mod。
 *
 * 全部完成后 emit `mods-preload-done` 事件，前端据此设置 `isPreloadDone = true`。
 * `handleShowInfo` 在 slug 为空时判断此标志：若预加载已完成则立即走本地信息弹窗，不等待。
 *
 * 事件可能分两次 emit：
 * - 元数据先到（slug/description/version/logo_data/translated_name 字段非 null）
 * - project 后到（project 字段非 null）
 * 也可能一次性 emit（缓存命中时所有字段一起到）。
 *
 * 使用方式：
 * ```ts
 * const { startListener, stopListener, isPreloadDone } = useModsPreload(mods)
 * onMounted(() => startListener())
 * onUnmounted(() => stopListener())
 * ```
 */

import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { ref, type Ref } from 'vue'
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
  /** JAR 内提取的 logo（base64 data URL，undefined 表示不更新） */
  logo_data?: string
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
 */
export function useModsPreload(mods: Ref<ModInfo[]>) {
  let unlisten: UnlistenFn | null = null
  let unlistenDone: UnlistenFn | null = null
  /** 预加载是否已完成（后端 emit `mods-preload-done` 后置 true） */
  const isPreloadDone = ref(false)

  const startListener = async () => {
    if (unlisten && unlistenDone) return // 防止重复监听

    unlisten = await listen<PreloadUpdatePayload>(
      'mods-preload-update',
      (event) => {
        const payload = event.payload
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
              ...(payload.logo_data !== undefined ? { logo_data: payload.logo_data } : {}),
              ...(payload.translated_name !== undefined ? { translated_name: payload.translated_name } : {}),
              // project 仅在原值为空或新值非 null 时更新（避免覆盖已加载的更优结果）
              ...(payload.project && !old.project ? { project: payload.project } : {}),
            }
            mods.value[i] = updated
            break
          }
        }
      },
    )

    unlistenDone = await listen<void>('mods-preload-done', () => {
      isPreloadDone.value = true
    })
  }

  const stopListener = () => {
    if (unlisten) {
      unlisten()
      unlisten = null
    }
    if (unlistenDone) {
      unlistenDone()
      unlistenDone = null
    }
  }

  return { startListener, stopListener, isPreloadDone }
}
