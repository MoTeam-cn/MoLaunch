/**
 * 加载器版本查询与合并安装 API
 */

import { invoke } from '@tauri-apps/api/core'

/**
 * 查询 Forge 版本列表
 */
export async function listForgeVersions(mcVersion: string): Promise<{ version: string; is_recommended: boolean; release_time: string }[]> {
  const json = await invoke<string>('list_forge_versions', { mcVersion })
  try { return JSON.parse(json) } catch { return [] }
}

/**
 * 查询 NeoForge 版本列表
 */
export async function listNeoforgeVersions(mcVersion: string): Promise<{ version: string; recommended: boolean }[]> {
  const json = await invoke<string>('list_neoforge_versions', { mcVersion })
  try { return JSON.parse(json) } catch { return [] }
}

/**
 * 查询 Fabric 版本列表
 */
export async function listFabricVersions(): Promise<{ version: string; stable: boolean }[]> {
  const json = await invoke<string>('list_fabric_versions')
  try { return JSON.parse(json) } catch { return [] }
}

/**
 * 查询 OptiFine 版本列表
 */
export async function listOptifineVersions(): Promise<{ display_name: string; is_preview: boolean }[]> {
  const json = await invoke<string>('list_optifine_versions')
  try { return JSON.parse(json) } catch { return [] }
}

/**
 * 查询 LiteLoader 版本列表
 */
export async function listLiteloaderVersions(mcVersion: string): Promise<string[]> {
  const json = await invoke<string>('list_liteloader_versions', { mcVersion })
  try { return JSON.parse(json) } catch { return [] }
}

/**
 * 校验加载器兼容性
 */
export async function validateLoaders(mcVersion: string, forge?: string, neoforge?: string, fabric?: string, optifine?: string): Promise<boolean> {
  return await invoke<boolean>('validate_loaders', { mcVersion, forgeVersion: forge, neoforgeVersion: neoforge, fabricVersion: fabric, optifineVersion: optifine })
}

/**
 * 合并安装（MC + 加载器）
 */
export async function installMerged(mcVersion: string, forge?: string, neoforge?: string, fabric?: string, optifine?: string, liteloader?: string, instanceName?: string): Promise<void> {
  return await invoke('install_merged', { mcVersion, forgeVersion: forge, neoforgeVersion: neoforge, fabricVersion: fabric, optifineVersion: optifine, liteloaderVersion: liteloader, instanceName })
}

/**
 * Fabric API 版本信息（来自 Modrinth）
 */
export interface FabricApiVersion {
  version_id: string       // Modrinth version ID
  version_number: string   // 版本号（如 0.92.2+1.20.4）
  display_name: string     // 显示名
  game_versions: string[]  // 支持的 MC 版本
  release_date: string     // 发布日期
  download_url: string     // 下载 URL
  file_name: string        // 文件名
  size: number             // 文件大小（字节）
  hash: string | null      // SHA1
}

/**
 * 查询指定 MC 版本可用的 Fabric API 版本列表
 * 返回的列表已按发布日期降序排序（最新版在前）
 */
export async function listFabricApiVersions(mcVersion: string): Promise<FabricApiVersion[]> {
  return await invoke<FabricApiVersion[]>('list_fabric_api_versions', { mcVersion })
}

/**
 * 为已安装的版本手动安装指定 Fabric API
 * （install_merged 已自动安装最新版，此命令用于手动更换版本）
 */
export async function installFabricApiForVersion(versionId: string, downloadUrl: string, fileName: string, hash?: string | null): Promise<void> {
  return await invoke('install_fabric_api_for_version', { versionId, downloadUrl, fileName, hash: hash ?? null })
}
