import { VERSION_INSTALL_ACTIONS, versionInstallManager } from '../version-install-manager'

/**
 * 触发 mod 详情预加载（后台异步，立即返回）
 *
 * 调用时机：在 `list_mods` 返回后立即调用本函数。
 * 前端监听 `mods-preload-update` 事件，按 `file_name` 匹配更新对应 mod 的 `project` 字段。
 */
export async function preloadModsDetail(versionId: string): Promise<void> {
  return versionInstallManager<void>(VERSION_INSTALL_ACTIONS.PRELOAD_MODS_DETAIL_CMD, { versionId })
}

/**
 * 取消当前正在运行的 mod 详情预加载 task
 *
 * ModTab 组件卸载时调用，abort 后台 spawn 的预加载 task，
 * 避免 task 继续 emit `mods-preload-update` 给已注销的前端 listener。
 */
export async function cancelPreloadModsDetail(): Promise<void> {
  return versionInstallManager<void>(VERSION_INSTALL_ACTIONS.CANCEL_PRELOAD_MODS_DETAIL_CMD)
}