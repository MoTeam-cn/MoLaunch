import type { ResourceProject } from '@/types/community'
import { VERSION_MODS_ACTIONS, versionModsManager } from '../version-mods-manager'

/**
 * 单个 Mod 信息
 */
export interface ModInfo {
  /** 文件名（含扩展名） */
  file_name: string
  /** 启用时的文件名（去除 .disabled / .old 后缀） */
  enabled_name: string
  /** 是否启用 */
  is_enabled: boolean
  /** 文件大小（字节） */
  size: number
  /** 加载器类型（forge/fabric/neoforge/liteloader/unknown） */
  loader_type: string
  /** 中文译名（来自 mcmod 数据库，可能为空） */
  translated_name: string
  /** Mod 描述（来自 jar 内 metadata，可能为空） */
  description?: string
  /** Mod 版本号（来自 jar 内 metadata，可能为空） */
  version?: string
  /**
   * Mod 图标缓存 URL（由预加载阶段填充）
   */
  cached_logo_url?: string
  /** Mod slug（来自 jar 内 metadata） */
  slug: string
  /** 预加载到的平台工程详情（由 `preload_mods_detail_cmd` 后台批量查询填充） */
  project?: ResourceProject
}

/**
 * 判断版本是否可安装 Mod（含 Forge/Fabric/NeoForge/LiteLoader 或个性化分类为"可安装Mod"）
 */
export async function isVersionModable(versionId: string): Promise<boolean> {
  return versionModsManager<boolean>(VERSION_MODS_ACTIONS.IS_VERSION_MODABLE, { versionId })
}

/**
 * 列出版本的 Mod
 */
export async function listMods(versionId: string): Promise<ModInfo[]> {
  return versionModsManager<ModInfo[]>(VERSION_MODS_ACTIONS.LIST_MODS, { versionId })
}

/**
 * 启用/禁用 Mod
 *
 * 返回重命名后的新文件名（前端据此原地更新 mod 字段，避免重新加载列表丢失预加载的 project 等信息）。
 */
export async function toggleMod(
  versionId: string,
  fileName: string,
  enable: boolean,
): Promise<string> {
  return versionModsManager<string>(VERSION_MODS_ACTIONS.TOGGLE_MOD, {
    versionId,
    fileName,
    enable,
  })
}

/**
 * 删除 Mod
 */
export async function deleteMod(versionId: string, fileName: string): Promise<void> {
  return versionModsManager<void>(VERSION_MODS_ACTIONS.DELETE_MOD, { versionId, fileName })
}

/**
 * 从外部文件安装 Mod（复制到 mods 目录）
 */
export async function installMod(versionId: string, sourcePath: string): Promise<void> {
  return versionModsManager<void>(VERSION_MODS_ACTIONS.INSTALL_MOD, {
    versionId,
    sourcePath,
  })
}

/**
 * 打开版本的 mods 目录（自动创建）
 */
export async function openModsDir(versionId: string): Promise<void> {
  return versionModsManager<void>(VERSION_MODS_ACTIONS.OPEN_MODS_DIR, { versionId })
}

/**
 * 在资源管理器中打开并选中指定 Mod 文件
 */
export async function revealModFile(versionId: string, fileName: string): Promise<void> {
  return versionModsManager<void>(VERSION_MODS_ACTIONS.REVEAL_MOD_FILE, {
    versionId,
    fileName,
  })
}

/**
 * 获取版本的 mods 目录路径（自动创建，不打开）
 */
export async function getVersionModsDir(versionId: string): Promise<string> {
  return versionModsManager<string>(VERSION_MODS_ACTIONS.GET_VERSION_MODS_DIR, { versionId })
}

/**
 * 原子化更新 Mod（下载新版本 + 删旧版本）
 *
 * 下载失败时不删旧文件，确保用户不会因更新失败失去原有 mod。
 * 进度通过 DownloadSession 统一推送，前端下载管理页可见（分组"Mod 更新"）。
 */
export async function updateMod(
  versionId: string,
  oldFileName: string,
  downloadUrl: string,
  newFileName: string,
  expectedSize: number,
): Promise<void> {
  return versionModsManager<void>(VERSION_MODS_ACTIONS.UPDATE_MOD, {
    versionId,
    oldFileName,
    downloadUrl,
    newFileName,
    expectedSize,
  })
}

/**
 * 开始监听版本 mods 目录的文件变化
 */
export async function watchModsDir(versionId: string): Promise<void> {
  return versionModsManager<void>(VERSION_MODS_ACTIONS.WATCH_MODS_DIR, { versionId })
}

/**
 * 停止监听 mods 目录（ModTab 组件卸载时调用）
 */
export async function unwatchModsDir(): Promise<void> {
  return versionModsManager<void>(VERSION_MODS_ACTIONS.UNWATCH_MODS_DIR)
}