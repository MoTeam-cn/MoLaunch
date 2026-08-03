/**
 * 联机 API - 房主白名单管理
 *
 * list / add / remove / set_enabled 4 个 action 仅房主可调用；
 * 启用白名单且条目为空时拒绝所有人加入（便于私密联机）。
 */

import type { BusinessResult, WhitelistResponse } from '@/types/online'
import { ONLINE_ACTIONS, onlineManager } from './core'

/** 查询当前房间的白名单启用状态与条目列表（房主） */
export function listWhitelist(
  roomCode: string,
): Promise<BusinessResult<WhitelistResponse>> {
  return onlineManager(ONLINE_ACTIONS.ROOM_LIST_WHITELIST, { roomCode })
}

/**
 * 添加白名单条目（房主）
 *
 * @param roomCode 房间码
 * @param deviceId 设备友好标识（`mcsdk-xxxx-xxxx-xxxx-xxxx`），服务端转换为 `device_pk` 落库
 */
export function addWhitelist(
  roomCode: string,
  deviceId: string,
): Promise<BusinessResult<unknown>> {
  return onlineManager(ONLINE_ACTIONS.ROOM_ADD_WHITELIST, { roomCode, deviceId })
}

/**
 * 移除白名单条目（房主）
 *
 * @param roomCode 房间码
 * @param deviceId 设备友好标识
 */
export function removeWhitelist(
  roomCode: string,
  deviceId: string,
): Promise<BusinessResult<unknown>> {
  return onlineManager(ONLINE_ACTIONS.ROOM_REMOVE_WHITELIST, { roomCode, deviceId })
}

/**
 * 修改白名单启用状态（房主）
 *
 * 关闭白名单不影响已落库的条目，再次启用时无需重新添加。
 * @param roomCode 房间码
 * @param enabled 是否启用白名单
 */
export function setWhitelistEnabled(
  roomCode: string,
  enabled: boolean,
): Promise<BusinessResult<unknown>> {
  return onlineManager(ONLINE_ACTIONS.ROOM_SET_WHITELIST_ENABLED, {
    roomCode,
    enabled,
  })
}
