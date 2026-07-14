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
