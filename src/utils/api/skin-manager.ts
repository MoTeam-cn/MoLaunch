/**
 * 皮肤模块统一 API 入口
 *
 * 后端 skin_manager IPC 按 action 分发；params 字段一律 camelCase。
 */

import { invoke } from '@tauri-apps/api/core'

/**
 * 调用 skin_manager IPC
 * @param action 操作名称（取自 SKIN_ACTIONS 常量）
 * @param params 参数对象（字段名使用 camelCase）
 */
export async function skinManager<T = unknown>(
  action: string,
  params?: unknown,
): Promise<T> {
  return invoke<T>('skin_manager', { req: { action, params: params ?? null } })
}

/**
 * 所有可用的 action 名称
 *
 * 与后端 `utils::skin_manager::DISPATCHER` 注册的 action 一一对应。
 */
export const SKIN_ACTIONS = {
  GET_SKIN_CAPE_INFO: 'get_skin_cape_info',
  GET_SKIN_URL: 'get_skin_url',
  GET_CAPE_URL: 'get_cape_url',
  UPLOAD_SKIN: 'upload_skin',
  EQUIP_CAPE: 'equip_cape',
  UNEQUIP_CAPE: 'unequip_cape',
  DOWNLOAD_URL_TO_FILE: 'download_url_to_file',
} as const

/** action 名称类型 */
export type SkinAction = typeof SKIN_ACTIONS[keyof typeof SKIN_ACTIONS]
