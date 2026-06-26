/**
 * Tauri API 封装工具
 */

import { invoke } from '@tauri-apps/api/tauri'
import type { AuthResult, SdkStatus } from '@/types/auth'
import type { VersionList } from '@/types/version'

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
