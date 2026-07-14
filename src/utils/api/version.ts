/**
 * 版本列表与文件夹管理 API
 */

import { invoke } from '@tauri-apps/api/core'
import type { VersionList } from '@/types/version'

/**
 * 获取版本列表
 */
export async function listVersions(): Promise<VersionList> {
  return await invoke<VersionList>('list_versions')
}

/**
 * 下载版本
 */
export async function downloadVersion(versionId: string): Promise<void> {
  return await invoke<void>('download_version', { versionId })
}

/**
 * 获取已安装版本列表
 */
export async function listInstalledVersions(): Promise<string[]> {
  return await invoke<string[]>('list_installed_versions')
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
  return await invoke<InstalledVersionInfo[]>('list_installed_versions_with_type')
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
  return await invoke<McFolder[]>('list_mc_folders')
}

/** 添加 Minecraft 文件夹（自动去重） */
export async function addMcFolder(name: string, path: string): Promise<McFolder[]> {
  return await invoke<McFolder[]>('add_mc_folder', { name, path })
}

/** 移除 Minecraft 文件夹 */
export async function removeMcFolder(path: string): Promise<McFolder[]> {
  return await invoke<McFolder[]>('remove_mc_folder', { path })
}

/** 切换当前 Minecraft 文件夹 */
export async function switchMcFolder(path: string): Promise<string> {
  return await invoke<string>('switch_mc_folder', { path })
}

/** 重命名 Minecraft 文件夹 */
export async function renameMcFolder(path: string, newName: string): Promise<McFolder[]> {
  return await invoke<McFolder[]>('rename_mc_folder', { path, newName })
}

/**
 * 卸载版本
 */
export async function uninstallVersion(versionId: string): Promise<void> {
  return await invoke<void>('uninstall_version', { versionId })
}

/**
 * 获取版本的有效游戏目录（考虑版本隔离）
 * 隔离时返回 `{game_dir}/versions/{version_id}/`，非隔离时返回 `{game_dir}/`
 */
export async function getVersionEffectiveDir(versionId: string): Promise<string> {
  return await invoke<string>('get_version_effective_dir', { versionId })
}
