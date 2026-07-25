/**
 * 开发者模式 API
 *
 * 触发流程：
 * 1. SystemInfoTab.vue 版本号连续点击 5 次 → unlockDeveloperMode()
 * 2. SettingsAdvanced.vue 显示开关卡片（仅在已解锁时）→ applyConfig({ developerMode: true/false })
 * 3. Settings.vue 侧边菜单出现「开发者」项（仅在开关开启时）
 * 4. SettingsDeveloper.vue 展示日志/缓存/存储/系统信息
 *
 * 存储位置：Windows 注册表 HKCU\Software\MoLaunch 下的两个布尔值
 * - DeveloperUnlocked：是否已解锁（决定开关卡片是否显示）
 * - DeveloperMode：开关是否开启（决定侧边菜单 developer 项是否显示）
 *
 * 注意：开发者模式的「获取/修改」已统一到 get_config / apply_config
 * （ConfigSnapshot.developerMode / ConfigPatch.developerMode），
 * 此文件仅保留解锁触发动作和日志/缓存/存储/系统信息查询。
 *
 * 注：8 个原 Tauri 命令（is_developer_unlocked / unlock_developer_mode
 * / get_storage_dirs / get_system_info / get_cache_stats / get_log_path
 * / list_log_files / read_log_file）已聚合为 `system_manager` 单一 IPC 入口，
 * 通过 `action` 字段分发。
 */

import { SYSTEM_ACTIONS, systemManager } from './system-manager'

// ==================== 解锁 ====================

/** 查询开发者模式是否已解锁（用户连续点击版本号 5 次后解锁） */
export async function isDeveloperUnlocked(): Promise<boolean> {
  return systemManager<boolean>(SYSTEM_ACTIONS.IS_DEVELOPER_UNLOCKED)
}

/** 解锁开发者模式（连续点击版本号 5 次后调用） */
export async function unlockDeveloperMode(): Promise<void> {
  return systemManager<void>(SYSTEM_ACTIONS.UNLOCK_DEVELOPER_MODE)
}

// ==================== 存储目录 ====================

export interface StorageDirs {
  /** 数据根目录（.Molaunch） */
  base: string
  /** 配置文件（config.ini 完整路径） */
  config: string
  /** 日志目录 */
  logs: string
  /** 运行路径缓存目录（.Molaunch/cache/） */
  cache: string
  /** 运行路径临时目录（.Molaunch/temp/） */
  temp: string
  /** 系统临时目录缓存（<temp>/MoLaunch/，含 TaskTemp 和 sdk） */
  cacheTemp: string
  /** AppData 缓存目录（%APPDATA%/.minecraft/，Java Runtime） */
  cacheApp: string
}

/** 获取所有存储目录路径 */
export async function getStorageDirs(): Promise<StorageDirs> {
  return systemManager<StorageDirs>(SYSTEM_ACTIONS.GET_STORAGE_DIRS)
}

// ==================== 系统信息 ====================

export interface SystemInfo {
  /** 应用版本 */
  appVersion: string
  /** 操作系统（windows/macos/linux） */
  os: string
  /** 系统架构（x86_64/aarch64） */
  arch: string
  /** 是否 64 位 */
  is64bit: boolean
  /** 总内存（字节） */
  totalMemory: number
  /** 已用内存（字节） */
  usedMemory: number
  /** 可用内存（字节） */
  availableMemory: number
  /** 内存使用率（百分比） */
  memoryUsagePercent: number
}

/** 获取系统信息 */
export async function getSystemInfo(): Promise<SystemInfo> {
  return systemManager<SystemInfo>(SYSTEM_ACTIONS.GET_SYSTEM_INFO)
}

// ==================== 缓存统计 ====================

/** 单个缓存子目录的统计信息 */
export interface CacheStat {
  /** 显示名称（如 "图片缓存"） */
  name: string
  /** 所属类别（"cache" / "cacheTemp" / "cacheApp"） */
  category: string
  /** 子目录相对路径（如 "images" / "TaskTemp" / "runtime"） */
  subDir: string
  /** 完整路径 */
  path: string
  /** 文件数量（递归统计） */
  fileCount: number
  /** 占用字节数（递归统计） */
  totalSize: number
  /** 自动清理 TTL（小时），null 表示不清理 */
  ttlHours: number | null
}

/** 缓存统计结果（按类别分组） */
export interface CacheStatsResult {
  /** 运行路径缓存（.Molaunch/cache/） */
  cache: CacheStat[]
  /** 系统临时目录缓存（<temp>/MoLaunch/） */
  cacheTemp: CacheStat[]
  /** AppData 缓存（%APPDATA%/.minecraft/） */
  cacheApp: CacheStat[]
}

/** 获取所有缓存目录的统计信息（文件数、占用大小、TTL） */
export async function getCacheStats(): Promise<CacheStatsResult> {
  return systemManager<CacheStatsResult>(SYSTEM_ACTIONS.GET_CACHE_STATS)
}

// ==================== 日志查看 ====================

/** 获取今日日志文件完整路径 */
export async function getLogPath(): Promise<string> {
  return systemManager<string>(SYSTEM_ACTIONS.GET_LOG_PATH)
}

/** 获取所有日志文件名列表（最新的在前） */
export async function listLogFiles(): Promise<string[]> {
  return systemManager<string[]>(SYSTEM_ACTIONS.LIST_LOG_FILES)
}

/** 读取指定日志文件内容 */
export async function readLogFile(filename: string): Promise<string> {
  return systemManager<string>(SYSTEM_ACTIONS.READ_LOG_FILE, { filename })
}
