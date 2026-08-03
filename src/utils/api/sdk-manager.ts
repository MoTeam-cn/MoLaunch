/**
 * SDK 模块统一 API 入口
 *
 * 后端 sdk_manager IPC 按 action 分发；params 字段一律 camelCase。
 */

import { invoke } from '@tauri-apps/api/core'

/**
 * 调用 sdk_manager IPC
 * @param action 操作名称（取自 SDK_ACTIONS 常量）
 * @param params 参数对象（字段名使用 camelCase）
 */
export async function sdkManager<T = unknown>(
  action: string,
  params?: unknown,
): Promise<T> {
  return invoke<T>('sdk_manager', { req: { action, params: params ?? null } })
}

/**
 * 所有可用的 action 名称
 *
 * 与后端 `utils::sdk_manager::DISPATCHER` 注册的 action 一一对应。
 */
export const SDK_ACTIONS = {
  GET_PLATFORM_INFO: 'get_platform_info',
  GET_SDK_VERSION: 'get_sdk_version',
  IS_SDK_INITIALIZED: 'is_sdk_initialized',
  GET_DEVICE_ID: 'get_device_id',
  CHECK_UPDATE_LITE: 'check_update_lite',
} as const

/** action 名称类型 */
export type SdkAction = typeof SDK_ACTIONS[keyof typeof SDK_ACTIONS]
