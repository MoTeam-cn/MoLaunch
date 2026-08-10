/**
 * 资源包/光影管理 API 封装
 * 参数统一携带 `kind`（resourcepack / shader），字段名 camelCase。
 */

import { VERSION_PACKS_ACTIONS, versionPacksManager } from '@/utils/api/version-packs-manager'

/** 内容类型：资源包 / 光影 */
export type PackKind = 'resourcepack' | 'shader'

/** 单个资源包/光影信息（与后端 packs::types::PackInfo 对应） */
export interface PackInfo {
  file_name: string
  enabled_name: string
  is_enabled: boolean
  is_folder: boolean
  size: number
}

/** kind + versionId 参数 */
interface KindVersionIdParams {
  kind: PackKind
  versionId: string
}

/** kind + versionId + fileName 参数 */
interface KindFileParams extends KindVersionIdParams {
  fileName: string
}

/** 判断版本是否可安装资源包/光影 */
export async function isPacksAvailable(versionId: string, kind: PackKind): Promise<boolean> {
  return versionPacksManager<boolean>(VERSION_PACKS_ACTIONS.IS_PACKS_AVAILABLE, {
    kind,
    versionId,
  })
}

/** 列出版本的资源包/光影 */
export async function listPacks(versionId: string, kind: PackKind): Promise<PackInfo[]> {
  return versionPacksManager<PackInfo[]>(VERSION_PACKS_ACTIONS.LIST_PACKS, { kind, versionId })
}

/** 启用/禁用资源包/光影，返回重命名后的新文件名 */
export async function togglePack(
  versionId: string,
  fileName: string,
  enable: boolean,
  kind: PackKind,
): Promise<string> {
  return versionPacksManager<string>(VERSION_PACKS_ACTIONS.TOGGLE_PACK, {
    kind,
    versionId,
    fileName,
    enable,
  })
}

/** 删除资源包/光影 */
export async function deletePack(
  versionId: string,
  fileName: string,
  kind: PackKind,
): Promise<void> {
  const params: KindFileParams = { kind, versionId, fileName }
  await versionPacksManager(VERSION_PACKS_ACTIONS.DELETE_PACK, params)
}

/** 从外部路径安装资源包/光影 */
export async function installPack(
  versionId: string,
  sourcePath: string,
  kind: PackKind,
): Promise<void> {
  await versionPacksManager(VERSION_PACKS_ACTIONS.INSTALL_PACK, {
    kind,
    versionId,
    sourcePath,
  })
}

/** 打开内容目录（自动创建） */
export async function openPacksDir(versionId: string, kind: PackKind): Promise<void> {
  await versionPacksManager(VERSION_PACKS_ACTIONS.OPEN_PACKS_DIR, { kind, versionId })
}

/** 在资源管理器中定位文件 */
export async function revealPackFile(
  versionId: string,
  fileName: string,
  kind: PackKind,
): Promise<void> {
  await versionPacksManager(VERSION_PACKS_ACTIONS.REVEAL_PACK_FILE, {
    kind,
    versionId,
    fileName,
  })
}

/** 提取包内图标为 base64 data URL（无图标返回 null） */
export async function getPackIcon(
  versionId: string,
  fileName: string,
  kind: PackKind,
): Promise<string | null> {
  return versionPacksManager<string | null>(VERSION_PACKS_ACTIONS.GET_PACK_ICON, {
    kind,
    versionId,
    fileName,
  })
}

/** 开始监听内容目录变化 */
export async function watchPacksDir(versionId: string, kind: PackKind): Promise<void> {
  await versionPacksManager(VERSION_PACKS_ACTIONS.WATCH_PACKS_DIR, { kind, versionId })
}

/** 停止监听内容目录变化 */
export async function unwatchPacksDir(): Promise<void> {
  await versionPacksManager(VERSION_PACKS_ACTIONS.UNWATCH_PACKS_DIR)
}
