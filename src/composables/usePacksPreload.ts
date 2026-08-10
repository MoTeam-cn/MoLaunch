/**
 * 资源包/光影详情预加载事件监听
 *
 * 后端后台并发预加载，每查到一个 project emit `packs-preload-update`，全部完成
 * emit `packs-preload-done`。packs 为 zip 无 JAR 元数据，事件仅携带 logo 与 project。
 * 使用 onGlobalEvent 全局单例 listener（永不 unlisten），卸载时自动移除 handler。
 */

import { ref, type Ref } from 'vue'
import { onGlobalEvent } from '@/composables/useGlobalTauriEvent'
import type { PackInfo } from '@/utils/api/personalization'
import type { ResourceProject } from '@/types/community'

/** 预加载事件 payload（与后端 `PreloadUpdate` 结构对齐，packs 仅用 logo/project） */
interface PackPreloadUpdatePayload {
  file_name: string
  /** 平台工程 logo_url 经 image_cache 处理后的缓存 URL（undefined 表示不更新） */
  cached_logo_url?: string
  /** CF/MR 查到的平台工程（undefined 表示尚未查询或未查到） */
  project?: ResourceProject
}

/**
 * 监听 `packs-preload-update` 事件，自动更新 packs 数组中对应包的字段
 *
 * @param packs 响应式的包列表 ref（按 file_name 匹配更新）
 * @returns `{ isPreloadDone }`（start/stop 为 no-op 以兼容旧调用方）
 */
export function usePacksPreload(packs: Ref<PackInfo[]>) {
  /** 预加载是否已完成（后端 emit `packs-preload-done` 后置 true） */
  const isPreloadDone = ref(false)

  onGlobalEvent<PackPreloadUpdatePayload>('packs-preload-update', (payload) => {
    const { file_name } = payload
    if (!file_name) return

    for (let i = 0; i < packs.value.length; i++) {
      if (packs.value[i].file_name === file_name) {
        const old = packs.value[i]
        const updated: PackInfo = {
          ...old,
          ...(payload.cached_logo_url !== undefined ? { cached_logo_url: payload.cached_logo_url } : {}),
          // project 仅在原值为空或新值非 null 时更新（避免覆盖已加载的更优结果）
          ...(payload.project && !old.project ? { project: payload.project } : {}),
        }
        packs.value[i] = updated
        break
      }
    }
  })

  onGlobalEvent<void>('packs-preload-done', () => {
    isPreloadDone.value = true
  })

  const startListener = async () => {}
  const stopListener = () => {}

  return { startListener, stopListener, isPreloadDone }
}
