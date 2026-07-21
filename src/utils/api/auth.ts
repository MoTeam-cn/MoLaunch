/**
 * 认证相关 API（离线登录 + 微软登录 + 账号管理）
 */

import { invoke } from '@tauri-apps/api/core'
import type { AuthResult, MsAccountInfo, OfflineAccountInfo, DeviceCodeInfo, PollResult, LoginConfig } from '@/types/auth'

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

// ============================================================
// 微软登录相关（支持 Web Auth Code Flow 和 Device Code Flow）
// ============================================================

/** 获取登录流程配置 */
export async function msLoginGetConfig(): Promise<LoginConfig> {
  return await invoke<LoginConfig>('ms_login_get_config')
}

/** Web Auth Code Flow：打开 Webview 窗口 */
export async function msLoginWebStart(): Promise<void> {
  return await invoke<void>('ms_login_web_start')
}

/** Web Auth Code Flow：用授权码完成登录 */
export async function msLoginWebExchange(code: string): Promise<PollResult> {
  return await invoke<PollResult>('ms_login_web_exchange', { code })
}

/** Device Code Flow：请求设备码 */
export async function msLoginRequestDeviceCode(): Promise<DeviceCodeInfo> {
  return await invoke<DeviceCodeInfo>('ms_login_request_device_code')
}

/** Device Code Flow：轮询授权状态 */
export async function msLoginPoll(deviceCode: string): Promise<PollResult> {
  return await invoke<PollResult>('ms_login_poll', { deviceCode })
}

/**
 * 微软登录：使用 Refresh Token 静默刷新
 */
export async function msLoginRefresh(): Promise<AuthResult> {
  return await invoke<AuthResult>('ms_login_refresh')
}

/**
 * 获取已存储的微软账号列表
 */
export async function getMsAccounts(): Promise<MsAccountInfo[]> {
  return await invoke<MsAccountInfo[]>('get_ms_accounts')
}

/**
 * 删除已存储的微软账号
 */
export async function removeMsAccount(uuid: string): Promise<void> {
  return await invoke<void>('remove_ms_account', { uuid })
}

/**
 * 切换到已存储的微软账号
 */
export async function switchMsAccount(uuid: string): Promise<AuthResult> {
  return await invoke<AuthResult>('switch_ms_account', { uuid })
}

/**
 * 获取已存储的离线账号列表
 */
export async function getOfflineAccounts(): Promise<OfflineAccountInfo[]> {
  return await invoke<OfflineAccountInfo[]>('get_offline_accounts')
}

/**
 * 删除已存储的离线账号
 */
export async function removeOfflineAccount(uuid: string): Promise<void> {
  return await invoke<void>('remove_offline_account', { uuid })
}

/**
 * 切换到已存储的离线账号
 */
export async function switchOfflineAccount(uuid: string): Promise<AuthResult> {
  return await invoke<AuthResult>('switch_offline_account', { uuid })
}

/**
 * 设置离线账号的皮肤选择
 */
export async function setOfflineSkin(uuid: string, skin: string | null): Promise<void> {
  return await invoke<void>('set_offline_skin', { uuid, skin })
}

/**
 * 保存自定义皮肤文件并设置到离线账号
 *
 * 将用户选择的 PNG 文件复制到 app data 目录，返回 skin 字段值（custom:path|variant）
 */
export async function saveCustomSkin(
  uuid: string,
  filePath: string,
  variant?: 'classic' | 'slim',
): Promise<string> {
  return await invoke<string>('save_custom_skin', { uuid, filePath, variant })
}
