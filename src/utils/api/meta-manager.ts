/**
 * 认证模块统一 API 入口
 *
 * 后端 `meta_manager` IPC 命令通过 `action` 字段分发到不同子模块（参照 `tools_manager` 模式）。
 * 本文件仅提供通用入口和 action 常量，具体业务函数仍由 `auth.ts` / `authlib.ts` 封装，
 * 这样业务调用点保持类型安全且字段名一致。
 *
 * 字段名约定：后端 Params 结构体使用 `#[serde(rename_all = "camelCase")]`，
 * 故前端 params 对象的字段名一律使用 camelCase（如 `serverUrl` / `filePath` / `deviceCode`）。
 */

import { invoke } from '@tauri-apps/api/core'

/**
 * 调用 meta_manager IPC
 * @param action 操作名称（取自 META_ACTIONS 常量）
 * @param params 参数对象（字段名使用 camelCase）
 */
export async function metaManager<T = unknown>(
  action: string,
  params?: unknown,
): Promise<T> {
  return invoke<T>('meta_manager', { req: { action, params: params ?? null } })
}

/**
 * 所有可用的 action 名称
 *
 * 与后端 `utils::meta_manager::DISPATCHER` 注册的 action 一一对应。
 * 业务代码应优先使用此常量而非裸字符串，避免拼写错误。
 */
export const META_ACTIONS = {
  // 离线登录
  LOGIN_OFFLINE: 'login_offline',
  // 离线账号管理
  GET_OFFLINE_ACCOUNTS: 'get_offline_accounts',
  REMOVE_OFFLINE_ACCOUNT: 'remove_offline_account',
  SWITCH_OFFLINE_ACCOUNT: 'switch_offline_account',
  SET_OFFLINE_SKIN: 'set_offline_skin',
  SAVE_CUSTOM_SKIN: 'save_custom_skin',
  // 微软登录
  MS_LOGIN_GET_CONFIG: 'ms_login_get_config',
  MS_LOGIN_WEB_START: 'ms_login_web_start',
  MS_LOGIN_WEB_EXCHANGE: 'ms_login_web_exchange',
  MS_LOGIN_REQUEST_DEVICE_CODE: 'ms_login_request_device_code',
  MS_LOGIN_POLL: 'ms_login_poll',
  MS_LOGIN_REFRESH: 'ms_login_refresh',
  // 微软账号管理
  GET_MS_ACCOUNTS: 'get_ms_accounts',
  REMOVE_MS_ACCOUNT: 'remove_ms_account',
  SWITCH_MS_ACCOUNT: 'switch_ms_account',
  // authlib 外置登录
  AUTHLIB_FETCH_SERVER_META: 'authlib_fetch_server_meta',
  AUTHLIB_LOGIN: 'authlib_login',
  AUTHLIB_SELECT_PROFILE: 'authlib_select_profile',
  SWITCH_AUTHLIB_ACCOUNT: 'switch_authlib_account',
  GET_AUTHLIB_ACCOUNTS: 'get_authlib_accounts',
  REMOVE_AUTHLIB_ACCOUNT: 'remove_authlib_account',
  // authlib 皮肤管理（yggdrasil 协议皮肤端点）
  AUTHLIB_GET_SKIN_INFO: 'authlib_get_skin_info',
  AUTHLIB_UPLOAD_SKIN: 'authlib_upload_skin',
  AUTHLIB_DELETE_SKIN: 'authlib_delete_skin',
  AUTHLIB_UPLOAD_CAPE: 'authlib_upload_cape',
  AUTHLIB_DELETE_CAPE: 'authlib_delete_cape',
  // 会话通用
  GET_LOGIN_STATUS: 'get_login_status',
  LOGOUT: 'logout',
} as const

/** action 名称类型（用于约束业务代码传值） */
export type MetaAction = typeof META_ACTIONS[keyof typeof META_ACTIONS]
