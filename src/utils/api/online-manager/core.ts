/**
 * 联机功能统一 API - 核心入口（`online_manager` IPC 经 `action` 分发）
 *
 * params 字段名一律 camelCase（后端 `#[serde(rename_all = "camelCase")]`）。
 * 各 action 便捷封装拆分至同目录：auth / room / lobby / easytier / lan。
 */

import { invoke } from '@tauri-apps/api/core'

/**
 * 认证类 action 前缀，这些 action 本身就是认证相关的，1003 时不自动重试
 */
const AUTH_ACTION_PREFIX = 'auth_'

/** 防止并发触发多次静默重认证 */
let isReauthing = false

/**
 * 调用 online_manager IPC
 *
 * 检测到服务端返回 code=1003（未授权）时，自动静默走降级链重新认证：
 * 1. `auth_refresh`（用 refresh_token 换新 access_token）
 * 2. refresh 失败 → `auth_login`（用本地 ECDH 密钥签名换新 JWT）
 * 3. login 失败 → `auth_register`（重新注册设备，适用于云端 RSA 密钥变更）
 * 认证成功后重试原请求一次。认证类 action（`auth_*`）不自动重试，直接抛出错误。
 *
 * 注意：不调用 `auth_init`，因为它仅检查本地 token 过期时间，本地未过期时
 * 直接返回旧凭证，无法发现云端已撤销 token，导致重试请求仍然 1003。
 *
 * @param action 操作名称（取自 ONLINE_ACTIONS 常量）
 * @param params 参数对象（字段名使用 camelCase）
 */
export async function onlineManager<T = unknown>(
  action: string,
  params?: unknown,
): Promise<T> {
  try {
    return await invoke<T>('online_manager', { req: { action, params: params ?? null } })
  } catch (e) {
    const errMsg = e instanceof Error ? e.message : String(e)
    // 仅对非认证类 action 且错误包含 code=1003 时自动重试
    if (
      errMsg.includes('code=1003') &&
      !action.startsWith(AUTH_ACTION_PREFIX) &&
      !isReauthing
    ) {
      isReauthing = true
      try {
        console.warn('[OnlineManager] 检测到 1003 未授权，静默重新认证后重试')
        // 降级链：refresh → login → register
        try {
          await invoke('online_manager', { req: { action: 'auth_refresh', params: null } })
        } catch {
          try {
            await invoke('online_manager', { req: { action: 'auth_login', params: null } })
          } catch {
            await invoke('online_manager', { req: { action: 'auth_register', params: null } })
          }
        }
        return await invoke<T>('online_manager', { req: { action, params: params ?? null } })
      } finally {
        isReauthing = false
      }
    }
    throw e
  }
}

/**
 * 所有可用的 action 名称
 *
 * 与后端 `utils::online_manager::DISPATCHER` 注册的 action 一一对应。
 */
export const ONLINE_ACTIONS = {
  // 认证
  AUTH_STATUS: 'auth_status',
  AUTH_GET_SERVER_TIME: 'auth_get_server_time',
  AUTH_REGISTER: 'auth_register',
  AUTH_LOGIN: 'auth_login',
  AUTH_LOGOUT: 'auth_logout',
  AUTH_CLEAR: 'auth_clear',
  // 启动静默认证（refresh_token 自动续期 + 首次注册 + 重新登录）
  AUTH_INIT: 'auth_init',
  // 手动刷新 token（用 refresh_token 换新 access_token）
  AUTH_REFRESH: 'auth_refresh',
  // 房间（Scaffolding 方案：创建/查询/加入/关闭）
  ROOM_CREATE: 'room_create',
  ROOM_GET: 'room_get',
  ROOM_CLOSE: 'room_close',
  ROOM_JOIN: 'room_join',
  // MC 局域网伪装 + 端口探测
  LAN_FAKE_SERVER_START: 'lan_fake_server_start',
  LAN_FAKE_SERVER_STOP: 'lan_fake_server_stop',
  LAN_PORT_PROBE: 'lan_port_probe',
  GET_RUNNING_MC_PORT: 'get_running_mc_port',
  // 大厅浏览（按整合包聚合 + 公开房间列表）
  LOBBY_LIST_ROOMS: 'lobby_list_rooms',
  LOBBY_LIST_PACKAGES: 'lobby_list_packages',
  // easytier 虚拟组网 + Scaffolding 联机中心
  EASYTIER_JOIN: 'easytier_join',
  EASYTIER_STOP: 'easytier_stop',
  SCAFFOLDING_HOST_START: 'scaffolding_host_start',
  SCAFFOLDING_HOST_STOP: 'scaffolding_host_stop',
  SCAFFOLDING_CLIENT_PROBE: 'scaffolding_client_probe',
} as const

/** action 名称类型 */
export type OnlineAction = typeof ONLINE_ACTIONS[keyof typeof ONLINE_ACTIONS]
