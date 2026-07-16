/**
 * 版本个性化、Mod 管理、选中版本、文件补全、脚本导出 API
 */

import { invoke } from '@tauri-apps/api/core'

/** 版本个性化信息 */
export interface VersionPersonalization {
  logo: string
  custom_info: string
  display_type: number
  is_star: boolean
  indie_type: number
  version_type: string
  original_version: string
  window_title: string
  server_enter: string
  advance_jvm_args: string
  advance_game_args: string
  advance_run_cmd: string
  java_path: string
  /** Java 选择模式：空/auto=自动选择, "auto_version"=自动选择指定版本范围, "folder"=使用版本文件夹中的 Java, "custom"=使用指定的 Java */
  java_mode: string
  /** 自动选择时的最小 Java 主版本（仅 auto_version 模式生效，0=不限） */
  java_version_min: number
  /** 自动选择时的最大 Java 主版本（仅 auto_version 模式生效，0=不限） */
  java_version_max: number
  /** 内存模式（空=跟随全局, "auto"=自动, "custom"=自定义） */
  memory_mode: string
  /** 版本独立最小内存（MB，仅 custom 模式生效，0 表示未设置） */
  min_memory: number
  /** 版本独立最大内存（MB，仅 custom 模式生效，0 表示未设置） */
  max_memory: number
}

/** 版本个性化字段更新（undefined 的字段不会被修改） */
export interface PersonalizationUpdate {
  logo?: string
  customInfo?: string
  displayType?: number
  isStar?: boolean
  indieType?: number
  windowTitle?: string
  serverEnter?: string
  advanceJvmArgs?: string
  advanceGameArgs?: string
  advanceRunCmd?: string
  javaPath?: string
  /** Java 选择模式：空/auto=自动选择, "auto_version"=自动选择指定版本范围, "folder"=使用版本文件夹中的 Java, "custom"=使用指定的 Java */
  javaMode?: string
  /** 自动选择时的最小 Java 主版本（仅 auto_version 模式生效，0=不限） */
  javaVersionMin?: number
  /** 自动选择时的最大 Java 主版本（仅 auto_version 模式生效，0=不限） */
  javaVersionMax?: number
  /** 内存模式：传空字符串=跟随全局, "auto"=自动, "custom"=自定义 */
  memoryMode?: string
  /** 版本独立最小内存（MB） */
  minMemory?: number
  /** 版本独立最大内存（MB） */
  maxMemory?: number
}

/**
 * 获取版本个性化设置
 */
export async function getVersionPersonalization(versionId: string): Promise<VersionPersonalization> {
  return await invoke<VersionPersonalization>('get_version_personalization', { versionId })
}

/**
 * 更新版本个性化字段（传 undefined 表示不修改该字段）
 */
export async function updateVersionPersonalization(
  versionId: string,
  update: PersonalizationUpdate,
): Promise<void> {
  return await invoke<void>('update_version_personalization', { versionId, update })
}

/**
 * 导出启动脚本（.bat，使用绝对路径 Java + 版权信息）
 *
 * @param javaPath 用户指定的 Java 路径（可选，为空时后端按 MC 版本自动检测）
 */
export async function exportLaunchScript(
  versionId: string,
  username: string,
  uuid: string,
  accessToken: string,
  loginType: string,
  javaPath: string | null,
  savePath: string,
): Promise<void> {
  return await invoke<void>('export_launch_script', {
    versionId,
    username,
    uuid,
    accessToken,
    loginType,
    javaPath,
    savePath,
  })
}

/**
 * 补全版本文件（校验并下载缺失的 libraries/assets）
 */
export async function fixVersionFiles(versionId: string): Promise<void> {
  return await invoke<void>('fix_version_files', { versionId })
}

// ==================== Mod 管理 ====================

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
}

/**
 * 判断版本是否可安装 Mod（含 Forge/Fabric/NeoForge/LiteLoader 或个性化分类为"可安装Mod"）
 */
export async function isVersionModable(versionId: string): Promise<boolean> {
  return await invoke<boolean>('is_version_modable', { versionId })
}

/**
 * 列出版本的 Mod
 */
export async function listMods(versionId: string): Promise<ModInfo[]> {
  return await invoke<ModInfo[]>('list_mods', { versionId })
}

/**
 * 启用/禁用 Mod
 */
export async function toggleMod(
  versionId: string,
  fileName: string,
  enable: boolean,
): Promise<void> {
  return await invoke<void>('toggle_mod', { versionId, fileName, enable })
}

/**
 * 删除 Mod
 */
export async function deleteMod(versionId: string, fileName: string): Promise<void> {
  return await invoke<void>('delete_mod', { versionId, fileName })
}

/**
 * 从外部文件安装 Mod（复制到 mods 目录）
 */
export async function installMod(versionId: string, sourcePath: string): Promise<void> {
  return await invoke<void>('install_mod', { versionId, sourcePath })
}

/**
 * 打开版本的 mods 目录（自动创建）
 */
export async function openModsDir(versionId: string): Promise<void> {
  return await invoke<void>('open_mods_dir', { versionId })
}

/**
 * 重命名版本
 */
export async function renameVersion(versionId: string, newName: string): Promise<void> {
  return await invoke<void>('rename_version', { versionId, newName })
}

/**
 * 获取上次选中的版本（持久化）
 */
export async function getSelectedVersion(): Promise<string | null> {
  return await invoke<string | null>('get_selected_version')
}

/**
 * 保存当前选中的版本（持久化到 config.ini）
 */
export async function setSelectedVersion(versionId: string | null): Promise<void> {
  return await invoke<void>('set_selected_version', { versionId })
}
