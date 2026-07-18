/**
 * 版本个性化、Mod 管理、选中版本、文件补全、脚本导出 API
 */

import { invoke } from '@tauri-apps/api/core'
import type { ResourceProject } from '@/types/community'

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
  // ===== 高级选项开关（参考 PCL2 PageInstanceSetup 高级选项）=====
  advance_disable_mod_update: boolean
  advance_ignore_java_warning: boolean
  advance_disable_assets_verify: boolean
  advance_disable_jlw: boolean
  advance_disable_lua: boolean
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
  // ===== 高级选项开关 =====
  advanceDisableModUpdate?: boolean
  advanceIgnoreJavaWarning?: boolean
  advanceDisableAssetsVerify?: boolean
  advanceDisableJlw?: boolean
  advanceDisableLua?: boolean
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
  /** Mod 描述（来自 jar 内 metadata，可能为空） */
  description?: string
  /** Mod 版本号（来自 jar 内 metadata，可能为空） */
  version?: string
  /** Mod 图标（base64 data URL，从 jar 内 logo 文件提取，可能为空） */
  logo_data?: string
  /** Mod slug（来自 jar 内 metadata，用于关联 CF/MR 平台工程和查 mcmod.cn 直链） */
  slug: string
  /**
   * 预加载到的平台工程详情（由 `preload_mods_detail_cmd` 后台批量查询填充）。
   *
   * 参考 PCL2 `LocalResourceFile.Project`：
   * - `list_mods` 返回时为空（同步阶段不联网）
   * - 后台预加载完成后通过 `mods-preload-update` 事件推送，前端按 file_name 匹配更新此字段
   * - 详情按钮点击时判断此字段是否就绪，就绪直接弹 ResourceDetail（零延迟）
   */
  project?: ResourceProject
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
 *
 * 返回重命名后的新文件名（前端据此原地更新 mod 字段，避免重新加载列表丢失预加载的 project 等信息）。
 */
export async function toggleMod(
  versionId: string,
  fileName: string,
  enable: boolean,
): Promise<string> {
  return await invoke<string>('toggle_mod', { versionId, fileName, enable })
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
 * 在资源管理器中打开并选中指定 Mod 文件（参考 PCL2 Open_Click）
 */
export async function revealModFile(versionId: string, fileName: string): Promise<void> {
  return await invoke<void>('reveal_mod_file', { versionId, fileName })
}

/**
 * 获取版本的 mods 目录路径（自动创建，不打开）
 *
 * 用于资源详情弹窗点击"下载"按钮时默认保存到 mods 文件夹。
 */
export async function getVersionModsDir(versionId: string): Promise<string> {
  return await invoke<string>('get_version_mods_dir', { versionId })
}

/**
 * 获取版本对应的 Minecraft 游戏版本号（如 "1.20.1"）
 *
 * 用于从 ModTab 打开资源详情弹窗时，自动选中整合包对应的版本筛选 tag。
 * 返回 null 表示无法识别（JSON 缺失或所有策略都未命中）。
 */
export async function getVersionGameVersion(versionId: string): Promise<string | null> {
  return await invoke<string | null>('get_version_game_version', { versionId })
}

/**
 * 触发 mod 详情预加载（后台异步，立即返回）
 *
 * 参考 PCL2 `LocalResourceOnlineLoader`：在 `list_mods` 返回后立即调用本函数，
 * 后台并发批量查询每个 mod 的 CF/MR 工程详情：
 * - 命中持久化缓存（6h TTL）→ 直接 emit `mods-preload-update` 事件，不联网
 * - 未命中 → 计算文件 hash → CF `/fingerprints` + MR `/version_files` 批量查询 → emit 事件
 *
 * 前端监听 `mods-preload-update` 事件，按 `file_name` 匹配更新对应 mod 的 `project` 字段。
 * 详情按钮点击时判断 `mod.project` 是否就绪，就绪直接弹 ResourceDetail（零延迟）。
 */
export async function preloadModsDetail(versionId: string): Promise<void> {
  return await invoke<void>('preload_mods_detail_cmd', { versionId })
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
