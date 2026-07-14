/**
 * SDK 相关 API
 */

import { invoke } from '@tauri-apps/api/core'
import type { SdkStatus } from '@/types/auth'

/**
 * 获取平台信息
 */
export async function getPlatformInfo(): Promise<SdkStatus> {
  return await invoke<SdkStatus>('get_platform_info')
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
