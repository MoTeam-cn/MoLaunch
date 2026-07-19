/**
 * 开发者模式 API
 *
 * 触发流程：
 * 1. SettingsOther.vue 版本号连续点击 5 次 → unlockDeveloperMode()
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
 */

import { invoke } from '@tauri-apps/api/core'

// ==================== 解锁 ====================

/** 查询开发者模式是否已解锁（用户连续点击版本号 5 次后解锁） */
export async function isDeveloperUnlocked(): Promise<boolean> {
  return await invoke<boolean>('is_developer_unlocked')
}

/** 解锁开发者模式（连续点击版本号 5 次后调用） */
export async function unlockDeveloperMode(): Promise<void> {
  return await invoke<void>('unlock_developer_mode')
}

// ==================== 存储目录 ====================

export interface StorageDirs {
  /** 数据根目录（.Molaunch） */
  base: string
  /** 配置文件（config.ini 完整路径） */
  config: string
  /** 日志目录 */
  logs: string
  /** 缓存目录 */
  cache: string
  /** 临时目录 */
  temp: string
}

/** 获取所有存储目录路径 */
export async function getStorageDirs(): Promise<StorageDirs> {
  return await invoke<StorageDirs>('get_storage_dirs')
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
  return await invoke<SystemInfo>('get_system_info')
}

// ==================== 日志查看 ====================

/** 获取今日日志文件完整路径 */
export async function getLogPath(): Promise<string> {
  return await invoke<string>('get_log_path')
}

/** 获取所有日志文件名列表（最新的在前） */
export async function listLogFiles(): Promise<string[]> {
  return await invoke<string[]>('list_log_files')
}

/** 读取指定日志文件内容 */
export async function readLogFile(filename: string): Promise<string> {
  return await invoke<string>('read_log_file', { filename })
}
