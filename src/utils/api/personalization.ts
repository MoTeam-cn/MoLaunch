/**
 * 版本个性化、Mod 管理、选中版本、文件补全、脚本导出 API
 *
 * 注：底层命令已聚合为多个统一 IPC 入口，通过 `action` 字段分发：
 * - 个性化 / 选中版本 / 文件补全 / 重命名版本 / 游戏版本号 → `version_list_manager`
 * - Mod 管理 → `version_mods_manager`
 * - 启动脚本导出 → `version_launch_manager`
 * - mod 详情预加载 → `version_install_manager`
 */

import type { ResourceProject } from '@/types/community'
import { VERSION_INSTALL_ACTIONS, versionInstallManager } from './version-install-manager'
import { VERSION_MODS_ACTIONS, versionModsManager } from './version-mods-manager'
import { VERSION_LAUNCH_ACTIONS, versionLaunchManager } from './version-launch-manager'
import { VERSION_LIST_ACTIONS, versionListManager } from './version-list-manager'

/** 版本个性化信息 */
export interface VersionPersonalization {
  logo: string
  customInfo: string
  displayType: number
  isStar: boolean
  indieType: number
  versionType: string
  originalVersion: string
  windowTitle: string
  serverEnter: string
  advanceJvmArgs: string
  advanceGameArgs: string
  advanceRunCmd: string
  javaPath: string
  /** Java 选择模式：空/auto=自动选择, "auto_version"=自动选择指定版本范围, "folder"=使用版本文件夹中的 Java, "custom"=使用指定的 Java */
  javaMode: string
  /** 自动选择时的最小 Java 主版本（仅 auto_version 模式生效，0=不限） */
  javaVersionMin: number
  /** 自动选择时的最大 Java 主版本（仅 auto_version 模式生效，0=不限） */
  javaVersionMax: number
  /** 内存模式（空=跟随全局, "auto"=自动, "custom"=自定义） */
  memoryMode: string
  /** 版本独立最小内存（MB，仅 custom 模式生效，0 表示未设置） */
  minMemory: number
  /** 版本独立最大内存（MB，仅 custom 模式生效，0 表示未设置） */
  maxMemory: number
  // ===== 高级选项开关 =====
  advanceDisableModUpdate: boolean
  advanceIgnoreJavaWarning: boolean
  advanceDisableAssetsVerify: boolean
  advanceDisableJlw: boolean
  advanceDisableLua: boolean
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
  return versionListManager<VersionPersonalization>(VERSION_LIST_ACTIONS.GET_VERSION_PERSONALIZATION, { versionId })
}

/**
 * 更新版本个性化字段（传 undefined 表示不修改该字段）
 */
export async function updateVersionPersonalization(
  versionId: string,
  update: PersonalizationUpdate,
): Promise<void> {
  return versionListManager<void>(VERSION_LIST_ACTIONS.UPDATE_VERSION_PERSONALIZATION, { versionId, update })
}

/**
 * 导出启动脚本（.bat，使用绝对路径 Java + 版权信息）
 *
 * 安全修复：移除 accessToken 参数，后端根据 uuid 自行从 auth_storage 获取 token
 *
 * @param javaPath 用户指定的 Java 路径（可选，为空时后端按 MC 版本自动检测）
 *
 * 注：底层已聚合为 `version_launch_manager` 单一 IPC 入口，通过 `action` 字段分发。
 */
export async function exportLaunchScript(
  versionId: string,
  username: string,
  uuid: string,
  loginType: string,
  javaPath: string | null,
  savePath: string,
): Promise<void> {
  return versionLaunchManager<void>(VERSION_LAUNCH_ACTIONS.EXPORT_LAUNCH_SCRIPT, {
    versionId,
    username,
    uuid,
    loginType,
    javaPath,
    savePath,
  })
}

/**
 * 补全版本文件（校验并下载缺失的 libraries/assets）
 */
export async function fixVersionFiles(versionId: string): Promise<void> {
  return versionListManager<void>(VERSION_LIST_ACTIONS.FIX_VERSION_FILES, { versionId })
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
  /**
   * Mod 图标缓存 URL（由预加载阶段填充）
   *
   * 设计：本项目皮肤/披风 cached_url：
   * - 后端预加载查到平台工程后，调用 `image_cache::get_image_url` 处理 `project.logo_url`
   * - 命中缓存：返回 `cache-image://{hash}.png`，零网络请求
   * - 未命中：返回原始远程 URL，后端异步下载，完成后 emit `image-cached` 事件通知前端刷新
   * - 列表初始加载时为空，显示默认图标；预加载完成后自动更新为真实 logo
   */
  cached_logo_url?: string
  /** Mod slug（来自 jar 内 metadata：fabric.mod.json 的 id / mods.toml 的 modId / mcmod.info 的 modid） */
  slug: string
  /**
   * 预加载到的平台工程详情（由 `preload_mods_detail_cmd` 后台批量查询填充）。
   *
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
 *
 * 用于资源详情弹窗点击"下载"按钮时默认保存到 mods 文件夹。
 */
export async function getVersionModsDir(versionId: string): Promise<string> {
  return versionModsManager<string>(VERSION_MODS_ACTIONS.GET_VERSION_MODS_DIR, { versionId })
}

/**
 * 原子化更新 Mod（下载新版本 + 删旧版本）
 *
 * 阶段 4 新增：后端封装"下载新版本 → 删旧版本"为原子操作。
 * 下载失败时不删旧文件，确保用户不会因更新失败失去原有 mod。
 * 进度通过 DownloadSession 统一推送，前端下载管理页可见（分组"Mod 更新"）。
 *
 * @param versionId 版本 ID
 * @param oldFileName 旧 mod 文件名（可能是 .disabled 后缀）
 * @param downloadUrl 新版本下载 URL
 * @param newFileName 新版本文件名
 * @param expectedSize 新版本预期大小（字节）
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
 *
 * 后台启动 `notify` 文件监听，当 mods 目录中的文件被创建/修改/删除时，
 * 通过 `mods-dir-changed` 事件通知前端。前端应监听此事件并调用 `listMods` 刷新列表。
 *
 * 如果已有监听中的 watcher，会自动停止旧的再启动新的（同一时间只有一个监听）。
 * 文件变化事件经过 500ms 防抖处理，避免文件还在写入时触发过早刷新。
 *
 * 应在 ModTab 组件 `onMounted` 时调用，配合 `unwatchModsDir` 在 `onUnmounted` 时清理。
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
 * 触发 mod 详情预加载（后台异步，立即返回）
 *
 * 调用时机：在 `list_mods` 返回后立即调用本函数，
 * 后台并发批量查询每个 mod 的 CF/MR 工程详情：
 * - 命中持久化缓存（6h TTL）→ 直接 emit `mods-preload-update` 事件，不联网
 * - 未命中 → 计算文件 hash → CF `/fingerprints` + MR `/version_files` 批量查询 → emit 事件
 *
 * 前端监听 `mods-preload-update` 事件，按 `file_name` 匹配更新对应 mod 的 `project` 字段。
 * 详情按钮点击时判断 `mod.project` 是否就绪，就绪直接弹 ResourceDetail（零延迟）。
 */
export async function preloadModsDetail(versionId: string): Promise<void> {
  return versionInstallManager<void>(VERSION_INSTALL_ACTIONS.PRELOAD_MODS_DETAIL_CMD, { versionId })
}

/**
 * 取消当前正在运行的 mod 详情预加载 task
 *
 * ModTab 组件卸载时调用，abort 后台 spawn 的预加载 task，
 * 避免 task 继续 emit `mods-preload-update` 给已注销的前端 listener
 * 触发 `Couldn't find callback id xxx` 警告。
 */
export async function cancelPreloadModsDetail(): Promise<void> {
  return versionInstallManager<void>(VERSION_INSTALL_ACTIONS.CANCEL_PRELOAD_MODS_DETAIL_CMD)
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
