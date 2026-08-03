import { VERSION_LIST_ACTIONS, versionListManager } from '../version-list-manager'

/**
 * 获取版本对应的 Minecraft 游戏版本号（如 "1.20.1"）
 *
 * 用于从 ModTab 打开资源详情弹窗时，自动选中整合包对应的版本筛选 tag。
 * 返回 null 表示无法识别（JSON 缺失或所有策略都未命中）。
 */
export async function getVersionGameVersion(versionId: string): Promise<string | null> {
  return versionListManager<string | null>(VERSION_LIST_ACTIONS.GET_VERSION_GAME_VERSION, { versionId })
}

/**
 * 重命名版本
 */
export async function renameVersion(versionId: string, newName: string): Promise<void> {
  return versionListManager<void>(VERSION_LIST_ACTIONS.RENAME_VERSION, { versionId, newName })
}

/**
 * 获取上次选中的版本（持久化）
 */
export async function getSelectedVersion(): Promise<string | null> {
  return versionListManager<string | null>(VERSION_LIST_ACTIONS.GET_SELECTED_VERSION)
}

/**
 * 保存当前选中的版本（持久化到 config.ini）
 */
export async function setSelectedVersion(versionId: string | null): Promise<void> {
  return versionListManager<void>(VERSION_LIST_ACTIONS.SET_SELECTED_VERSION, { versionId })
}