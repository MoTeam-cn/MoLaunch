/**
 * 版本列表与文件夹管理 API
 *
 * 注：底层已聚合为 `version_list_manager` / `version_install_manager` 两个 IPC 入口，
 * 通过 `action` 字段分发。
 */

import type { VersionList } from '@/types/version'
import type { CheckLocalModpackResult, ModpackMetaFile } from '@/types/online'
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

/**
 * 获取版本加载器信息（加载器类型 + 加载器版本号）
 *
 * 联机大厅阶段 1 新增。读取 `versions/{id}/setup.ini` 的 `Type` 字段和
 * 对应的 `XxxVersion` 字段，用于创建房间时上报 `host_loader` / `host_loader_version`。
 *
 * 注：`getVersionGameVersion`（获取纯 MC 版本号）已在 `./personalization.ts` 实现，
 * 此处不再重复导出，避免 `tauri.ts` re-export 时命名冲突。
 *
 * - `loaderType`：`forge` / `fabric` / `neoforge` / `quilt` / `optifine` /
 *   `liteloader` / `release` / `snapshot` / `old` / `unknown`
 * - `loaderVersion`：对应加载器的版本号（如 `47.3.0`），无加载器时为空字符串
 *
 * 读取优先级：`setup.ini` → `modpack.meta.json`（整合包，含 loader + loader_version）→
 * 版本 JSON 检测（仅 loaderType）；均无则兜底为原版 `{ loaderType: 'release', loaderVersion: '' }`。
 */
export async function getVersionLoaderInfo(
  versionId: string,
): Promise<{ loaderType: string; loaderVersion: string }> {
  return versionListManager<{ loaderType: string; loaderVersion: string }>(
    VERSION_LIST_ACTIONS.GET_VERSION_LOADER_INFO,
    { versionId },
  )
}

/**
 * 读取本地整合包元数据（联机大厅阶段 3 新增）
 *
 * 从 `versions/{versionId}/modpack.meta.json` 读取整合包来源元数据，
 * 用于创建联机房间时上报 `modpack` 字段。
 *
 * @returns 整合包元数据文件；`null` 表示该版本无整合包元数据（非平台安装或原版）
 */
export async function readLocalModpackMeta(
  versionId: string,
): Promise<ModpackMetaFile | null> {
  return versionListManager<ModpackMetaFile | null>(
    VERSION_LIST_ACTIONS.READ_LOCAL_MODPACK_META,
    { versionId },
  )
}

/**
 * 校验本地是否已安装指定整合包（联机大厅阶段 4 新增）
 *
 * 扫描所有已安装版本的 `versions/<id>/modpack.meta.json`，比对 manifest_hash
 *（优先）或 source + projectId + fileId 三元组（回退）判断是否已装同款。
 *
 * 加入方拉取房主整合包元数据后调用此 API，据此决定是直接进入房间还是触发一键安装。
 *
 * @param manifestHash manifest.json SHA-256（可选，优先匹配；undefined 时仅三元组匹配）
 * @param source 平台标识（curseforge / modrinth）
 * @param projectId 平台工程 ID
 * @param fileId 平台文件 ID（CF file id / MR version id）
 * @returns `{ installed, versionId? }`：installed=true 时 versionId 为匹配的本地版本 ID
 */
export async function checkLocalModpack(
  manifestHash: string | undefined,
  source: string,
  projectId: string,
  fileId: string,
): Promise<CheckLocalModpackResult> {
  return versionListManager<CheckLocalModpackResult>(
    VERSION_LIST_ACTIONS.CHECK_LOCAL_MODPACK,
    {
      manifestHash: manifestHash ?? null,
      source,
      projectId,
      fileId,
    },
  )
}
