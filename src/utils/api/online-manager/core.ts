/**
 * 联机功能统一 API - 核心入口（`online_manager` IPC 经 `action` 分发）
 *
 * params 字段名一律 camelCase（后端 `#[serde(rename_all = "camelCase")]`）。
 * 各 action 便捷封装拆分至同目录：auth / room / turn / mesh / tun / whitelist / lobby。
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
  // 信令
  ROOM_GET_STUN: 'room_get_stun',
  ROOM_CREATE: 'room_create',
  ROOM_GET: 'room_get',
  ROOM_CLOSE: 'room_close',
  ROOM_JOIN: 'room_join',
  ROOM_SUBMIT_ANSWER: 'room_submit_answer',
  ROOM_LIST_ANSWERS: 'room_list_answers',
  ROOM_CONFIRM: 'room_confirm',
  ROOM_KEEPALIVE: 'room_keepalive',
  ROOM_LEAVE: 'room_leave',
  ROOM_KICK: 'room_kick',
  ROOM_UNBAN: 'room_unban',
  // 阶段 6.2：房主查询封禁列表
  ROOM_LIST_BANS: 'room_list_bans',
  ROOM_LIST_PARTICIPANTS: 'room_list_participants',
  // TURN 中继：房主独占（阶段三子任务 7）
  ROOM_GET_TURN: 'room_get_turn',
  // mesh 拓扑：参与者级 SDP Offer
  ROOM_UPLOAD_PARTICIPANT_OFFER: 'room_upload_participant_offer',
  ROOM_FETCH_PARTICIPANT_OFFER: 'room_fetch_participant_offer',
  // TUN 桥接：数据分发打通
  TUN_START: 'tun_start',
  TUN_FORWARD_TO: 'tun_forward_to',
  TUN_STOP: 'tun_stop',
  LAN_FAKE_SERVER_START: 'lan_fake_server_start',
  LAN_FAKE_SERVER_STOP: 'lan_fake_server_stop',
  // MC 局域网端口探测：监听发现广播解析 [AD]port[/AD]
  LAN_PORT_PROBE: 'lan_port_probe',
  // MC 局域网端口回查：按当前游戏进程 PID 扫描监听端口（进房时补事件丢失）
  GET_RUNNING_MC_PORT: 'get_running_mc_port',
  // TUN 权限不足时以管理员权限重启
  RESTART_AS_ADMIN: 'restart_as_admin',
  // 房主白名单管理（阶段三子任务 8 安全加强）
  ROOM_LIST_WHITELIST: 'room_list_whitelist',
  ROOM_ADD_WHITELIST: 'room_add_whitelist',
  ROOM_REMOVE_WHITELIST: 'room_remove_whitelist',
  ROOM_SET_WHITELIST_ENABLED: 'room_set_whitelist_enabled',
  // 大厅浏览（联机大厅阶段 5）
  LOBBY_LIST_ROOMS: 'lobby_list_rooms',
  LOBBY_LIST_CATEGORIES: 'lobby_list_categories',
} as const

/** action 名称类型 */
export type OnlineAction = typeof ONLINE_ACTIONS[keyof typeof ONLINE_ACTIONS]
