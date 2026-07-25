/**
 * 版本列表与文件夹管理 API
 *
 * 注：底层已聚合为 `version_list_manager` / `version_install_manager` 两个 IPC 入口，
 * 通过 `action` 字段分发。
 */

import type { VersionList } from '@/types/version'
import { VERSION_INSTALL_ACTIONS, versionInstallManager } from './version-install-manager'
import { VERSION_LIST_ACTIONS, versionListManager } from './version-list-manager'

/**
 * 获取版本列表
 */
export async function listVersions(): Promise<VersionList> {
  return versionListManager<VersionList>(VERSION_LIST_ACTIONS.LIST_VERSIONS)
}

/**
 * 下载版本
 */
export async function downloadVersion(versionId: string): Promise<void> {
  return versionInstallManager<void>(VERSION_INSTALL_ACTIONS.DOWNLOAD_VERSION, { versionId })
}

/**
 * 获取已安装版本列表
 */
export async function listInstalledVersions(): Promise<string[]> {
  return versionListManager<string[]>(VERSION_LIST_ACTIONS.LIST_INSTALLED_VERSIONS)
}

export interface InstalledVersionInfo {
  id: string
  version_type: string
  /** 自定义图标文件名（空=自动判断） */
  logo: string
}

/**
 * 获取已安装版本列表（包含类型信息）
 */
export async function listInstalledVersionsWithType(): Promise<InstalledVersionInfo[]> {
  return versionListManager<InstalledVersionInfo[]>(
    VERSION_LIST_ACTIONS.LIST_INSTALLED_VERSIONS_WITH_TYPE,
  )
}

/**
 * Minecraft 文件夹项
 */
export interface McFolder {
  name: string
  path: string
}

/** 列出所有 Minecraft 文件夹 */
export async function listMcFolders(): Promise<McFolder[]> {
  return versionListManager<McFolder[]>(VERSION_LIST_ACTIONS.LIST_MC_FOLDERS)
}

/** 添加 Minecraft 文件夹（自动去重） */
export async function addMcFolder(name: string, path: string): Promise<McFolder[]> {
  return versionListManager<McFolder[]>(VERSION_LIST_ACTIONS.ADD_MC_FOLDER, { name, path })
}

/** 移除 Minecraft 文件夹 */
export async function removeMcFolder(path: string): Promise<McFolder[]> {
  return versionListManager<McFolder[]>(VERSION_LIST_ACTIONS.REMOVE_MC_FOLDER, { path })
}

/** 切换当前 Minecraft 文件夹 */
export async function switchMcFolder(path: string): Promise<string> {
  return versionListManager<string>(VERSION_LIST_ACTIONS.SWITCH_MC_FOLDER, { path })
}

/** 重命名 Minecraft 文件夹 */
export async function renameMcFolder(path: string, newName: string): Promise<McFolder[]> {
  return versionListManager<McFolder[]>(VERSION_LIST_ACTIONS.RENAME_MC_FOLDER, { path, newName })
}

/**
 * 卸载版本
 */
export async function uninstallVersion(versionId: string): Promise<void> {
  return versionListManager<void>(VERSION_LIST_ACTIONS.UNINSTALL_VERSION, { versionId })
}

/**
 * 获取版本的有效游戏目录（考虑版本隔离）
 * 隔离时返回 `{game_dir}/versions/{version_id}/`，非隔离时返回 `{game_dir}/`
 */
export async function getVersionEffectiveDir(versionId: string): Promise<string> {
  return versionListManager<string>(VERSION_LIST_ACTIONS.GET_VERSION_EFFECTIVE_DIR, { versionId })
}
