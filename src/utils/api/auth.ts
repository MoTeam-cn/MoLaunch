/**
 * 认证相关 API（离线登录 + 微软登录 + 账号管理）
 *
 * 后端原 23 个分散的 auth Tauri 命令已聚合为 1 个 `meta_manager` IPC 入口，
 * 本文件通过 `metaManager(action, params)` 调用，字段名使用 camelCase
 * （后端 Params 结构体使用 `#[serde(rename_all = "camelCase")]` 反序列化）。
 */

import { metaManager, META_ACTIONS } from '@/utils/api/meta-manager'
import type { AuthResult, MsAccountInfo, OfflineAccountInfo, DeviceCodeInfo, PollResult, LoginConfig } from '@/types/auth'

/**
 * 离线登录
 */
export async function loginOffline(username: string): Promise<AuthResult> {
  return await metaManager<AuthResult>(META_ACTIONS.LOGIN_OFFLINE, { username })
}

/**
 * 获取登录状态
 */
export async function getLoginStatus(): Promise<AuthResult | null> {
  return await metaManager<AuthResult | null>(META_ACTIONS.GET_LOGIN_STATUS)
}

/**
 * 登出
 */
export async function logout(): Promise<void> {
  return await metaManager<void>(META_ACTIONS.LOGOUT)
}

// ============================================================
// 微软登录相关（支持 Web Auth Code Flow 和 Device Code Flow）
// ============================================================

/** 获取登录流程配置 */
export async function msLoginGetConfig(): Promise<LoginConfig> {
  return await metaManager<LoginConfig>(META_ACTIONS.MS_LOGIN_GET_CONFIG)
}

/** Web Auth Code Flow：打开 Webview 窗口 */
export async function msLoginWebStart(): Promise<void> {
  return await metaManager<void>(META_ACTIONS.MS_LOGIN_WEB_START)
}

/** Web Auth Code Flow：用授权码完成登录 */
export async function msLoginWebExchange(code: string): Promise<PollResult> {
  return await metaManager<PollResult>(META_ACTIONS.MS_LOGIN_WEB_EXCHANGE, { code })
}

/** Device Code Flow：请求设备码 */
export async function msLoginRequestDeviceCode(): Promise<DeviceCodeInfo> {
  return await metaManager<DeviceCodeInfo>(META_ACTIONS.MS_LOGIN_REQUEST_DEVICE_CODE)
}

/** Device Code Flow：轮询授权状态 */
export async function msLoginPoll(deviceCode: string): Promise<PollResult> {
  return await metaManager<PollResult>(META_ACTIONS.MS_LOGIN_POLL, { deviceCode })
}

/**
 * 微软登录：使用 Refresh Token 静默刷新
 */
export async function msLoginRefresh(): Promise<AuthResult> {
  return await metaManager<AuthResult>(META_ACTIONS.MS_LOGIN_REFRESH)
}

/**
 * 获取已存储的微软账号列表
 */
export async function getMsAccounts(): Promise<MsAccountInfo[]> {
  return await metaManager<MsAccountInfo[]>(META_ACTIONS.GET_MS_ACCOUNTS)
}

/**
 * 删除已存储的微软账号
 */
export async function removeMsAccount(uuid: string): Promise<void> {
  return await metaManager<void>(META_ACTIONS.REMOVE_MS_ACCOUNT, { uuid })
}

/**
 * 切换到已存储的微软账号
 */
export async function switchMsAccount(uuid: string): Promise<AuthResult> {
  return await metaManager<AuthResult>(META_ACTIONS.SWITCH_MS_ACCOUNT, { uuid })
}

/**
 * 获取已存储的离线账号列表
 */
export async function getOfflineAccounts(): Promise<OfflineAccountInfo[]> {
  return await metaManager<OfflineAccountInfo[]>(META_ACTIONS.GET_OFFLINE_ACCOUNTS)
}

/**
 * 删除已存储的离线账号
 */
export async function removeOfflineAccount(uuid: string): Promise<void> {
  return await metaManager<void>(META_ACTIONS.REMOVE_OFFLINE_ACCOUNT, { uuid })
}

/**
 * 切换到已存储的离线账号
 */
export async function switchOfflineAccount(uuid: string): Promise<AuthResult> {
  return await metaManager<AuthResult>(META_ACTIONS.SWITCH_OFFLINE_ACCOUNT, { uuid })
}

/**
 * 设置离线账号的皮肤选择
 */
export async function setOfflineSkin(uuid: string, skin: string | null): Promise<void> {
  return await metaManager<void>(META_ACTIONS.SET_OFFLINE_SKIN, { uuid, skin })
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
  return await metaManager<string>(META_ACTIONS.SAVE_CUSTOM_SKIN, { uuid, filePath, variant })
}
