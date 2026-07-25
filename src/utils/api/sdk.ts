/**
 * SDK 相关 API
 *
 * 注：底层已聚合为 `sdk_manager` 单一 IPC 入口，通过 `action` 字段分发。
 */

import { SDK_ACTIONS, sdkManager } from './sdk-manager'
import type { SdkStatus } from '@/types/auth'

/**
 * 获取平台信息
 */
export async function getPlatformInfo(): Promise<SdkStatus> {
  return sdkManager<SdkStatus>(SDK_ACTIONS.GET_PLATFORM_INFO)
}

/**
 * 获取 SDK 版本
 */
export async function getSdkVersion(): Promise<string | null> {
  return sdkManager<string | null>(SDK_ACTIONS.GET_SDK_VERSION)
}
