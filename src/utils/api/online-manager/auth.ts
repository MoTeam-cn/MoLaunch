/**
 * 联机 API - 设备认证便捷封装
 *
 * 提供 auth_status / auth_get_server_time / auth_register / auth_login / auth_logout /
 * auth_clear 6 个 action + 启动静默认证 + 手动刷新 token。
 */

import type { AuthInitResult, DeviceStatus, ServerTimeInfo } from '@/types/online'
import { ONLINE_ACTIONS, onlineManager } from './core'

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

/**
 * 启动静默认证（程序启动时调用一次）
 *
 * 决策链：
 * 1. 无本地凭证 → 静默注册
 * 2. access token 未过期 → 直接返回
 * 3. access token 过期 + refresh_token 有效 → 自动 refresh 续期
 * 4. refresh 失败或双 token 过期 → 自动重新登录（ECDH）
 *
 * @returns 认证结果（status + error；error 非 null 表示云端连接失败需降级）
 */
export function initAuth(): Promise<AuthInitResult> {
  return onlineManager<AuthInitResult>(ONLINE_ACTIONS.AUTH_INIT)
}

/**
 * 手动刷新 access token（用 refresh_token 换新 token 对）
 *
 * 供设置页"重新连接"按钮调用，或 token 过期时业务调用方主动续期。
 */
export function refreshAuth(): Promise<DeviceStatus> {
  return onlineManager<DeviceStatus>(ONLINE_ACTIONS.AUTH_REFRESH)
}
