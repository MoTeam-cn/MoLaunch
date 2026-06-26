/**
 * Tauri API 封装工具
 */

import { invoke } from '@tauri-apps/api/tauri'
import type { AuthResult, SdkStatus } from '@/types/auth'
import type { VersionList } from '@/types/version'
import type { JavaRuntime } from '@/types/java'

/**
 * 获取平台信息
 */
export async function getPlatformInfo(): Promise<SdkStatus> {
  return await invoke<SdkStatus>('get_platform_info')
}

/**
 * 初始化 SDK
 */
export async function initializeSdk(gameDir?: string): Promise<string> {
  return await invoke<string>('initialize_sdk', { gameDir })
}

/**
 * 获取 SDK 版本
 */
export async function getSdkVersion(): Promise<string | null> {
  return await invoke<string | null>('get_sdk_version')
}

/**
 * 检查 SDK 是否已初始化
 */
export async function isSdkInitialized(): Promise<boolean> {
  return await invoke<boolean>('is_sdk_initialized')
}

/**
 * 离线登录
 */
export async function loginOffline(username: string): Promise<AuthResult> {
  return await invoke<AuthResult>('login_offline', { username })
}

/**
 * 获取登录状态
 */
export async function getLoginStatus(): Promise<AuthResult | null> {
  return await invoke<AuthResult | null>('get_login_status')
}

/**
 * 登出
 */
export async function logout(): Promise<void> {
  return await invoke<void>('logout')
}

/**
 * 获取版本列表
 */
export async function listVersions(): Promise<VersionList> {
  return await invoke<VersionList>('list_versions')
}

/**
 * 下载版本
 */
export async function downloadVersion(versionId: string): Promise<void> {
  return await invoke<void>('download_version', { versionId })
}

/**
 * 获取已安装版本列表
 */
export async function listInstalledVersions(): Promise<string[]> {
  return await invoke<string[]>('list_installed_versions')
}

/**
 * 获取设备 ID
 */
export async function getDeviceId(): Promise<string> {
  return await invoke<string>('get_device_id')
}

/**
 * 检测 Java
 */
export async function detectJava(): Promise<JavaRuntime> {
  return await invoke<JavaRuntime>('detect_java')
}

/**
 * 列出所有 Java
 */
export async function listJava(): Promise<JavaRuntime[]> {
  return await invoke<JavaRuntime[]>('list_java')
}

/**
 * 打开游戏目录
 */
export async function openGameDir(): Promise<void> {
  return await invoke<void>('open_game_dir')
}

/**
 * 获取游戏目录
 */
export async function getGameDir(): Promise<string> {
  return await invoke<string>('get_game_dir')
}
