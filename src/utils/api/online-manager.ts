/**
 * 联机功能统一 API 入口
 *
 * 后端 `online_manager` IPC 命令通过 `action` 字段分发到不同子模块
 * （参照 `community_manager` / `meta_manager` 模式）。
 *
 * 字段名约定：后端 Params 结构体使用 `#[serde(rename_all = "camelCase")]`，
 * 故前端 params 对象的字段名一律使用 camelCase。
 *
 * 阶段一注册的 6 个 action（认证相关）：
 * - `auth_status`：查询当前设备状态（不发网络请求）
 * - `auth_get_server_time`：获取服务器时间（测试连通性 + 校准时间）
 * - `auth_register`：注册新设备（生成密钥对 + 上报公钥 + 获取 JWT）
 * - `auth_login`：登录设备（用本地密钥签名换取新 JWT）
 * - `auth_logout`：登出设备（撤销 JWT，保留密钥）
 * - `auth_clear`：清除设备凭证（注销设备，删除本地密钥）
 *
 * 阶段二/三补充：房间管理、信令、WebRTC、虚拟网卡、MC 端口探测。
 */

import { invoke } from '@tauri-apps/api/core'
import type { DeviceStatus, ServerTimeInfo } from '@/types/online'

/**
 * 调用 online_manager IPC
 * @param action 操作名称（取自 ONLINE_ACTIONS 常量）
 * @param params 参数对象（字段名使用 camelCase）
 */
export async function onlineManager<T = unknown>(
  action: string,
  params?: unknown,
): Promise<T> {
  return invoke<T>('online_manager', { req: { action, params: params ?? null } })
}

/**
 * 所有可用的 action 名称
 *
 * 与后端 `utils::online_manager::DISPATCHER` 注册的 action 一一对应。
 */
export const ONLINE_ACTIONS = {
  AUTH_STATUS: 'auth_status',
  AUTH_GET_SERVER_TIME: 'auth_get_server_time',
  AUTH_REGISTER: 'auth_register',
  AUTH_LOGIN: 'auth_login',
  AUTH_LOGOUT: 'auth_logout',
  AUTH_CLEAR: 'auth_clear',
} as const

/** action 名称类型 */
export type OnlineAction = typeof ONLINE_ACTIONS[keyof typeof ONLINE_ACTIONS]

// ============================================================
// 类型安全的便捷封装（每个 action 一个函数）
// ============================================================

/** 查询当前设备状态（不发起网络请求，仅读本地凭证） */
export function getAuthStatus(): Promise<DeviceStatus> {
  return onlineManager<DeviceStatus>(ONLINE_ACTIONS.AUTH_STATUS)
}

/** 获取服务器时间（用于测试 api-server 连通性 + 校准本地时间） */
export function getServerTime(): Promise<ServerTimeInfo> {
  return onlineManager<ServerTimeInfo>(ONLINE_ACTIONS.AUTH_GET_SERVER_TIME)
}

/** 注册新设备（生成密钥对 + 上报公钥 + 获取 JWT） */
export function registerDevice(): Promise<DeviceStatus> {
  return onlineManager<DeviceStatus>(ONLINE_ACTIONS.AUTH_REGISTER)
}

/** 登录设备（用本地密钥签名换取新 JWT） */
export function loginDevice(): Promise<DeviceStatus> {
  return onlineManager<DeviceStatus>(ONLINE_ACTIONS.AUTH_LOGIN)
}

/** 登出设备（撤销 JWT，保留密钥） */
export function logoutDevice(): Promise<{ success: boolean }> {
  return onlineManager<{ success: boolean }>(ONLINE_ACTIONS.AUTH_LOGOUT)
}

/** 清除设备凭证（注销设备，删除本地密钥） */
export function clearDevice(): Promise<{ success: boolean }> {
  return onlineManager<{ success: boolean }>(ONLINE_ACTIONS.AUTH_CLEAR)
}
