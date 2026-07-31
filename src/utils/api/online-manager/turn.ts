/**
 * 联机 API - TURN 中继管理（阶段三子任务 7）
 *
 * 仅房主可调用 `room_get_turn`，服务端会基于全局开关、单机负载、集群总负载
 * 三层过滤后下发可用的 TURN 服务器。房主拉取后通过 DataChannel 控制消息
 * 0x05 广播给房间内所有参与者（加入方不能直接调用此接口）。
 */

import type { BusinessResult, TurnServersResponse } from '@/types/online'
import { ONLINE_ACTIONS, onlineManager } from './core'

/**
 * 房主拉取 TURN 服务器列表
 *
 * @param roomCode 房间码
 * @returns TURN 服务器列表 + 全局开关 + 集群负载快照
 */
export function getTurnServers(
  roomCode: string,
): Promise<BusinessResult<TurnServersResponse>> {
  return onlineManager(ONLINE_ACTIONS.ROOM_GET_TURN, { roomCode })
}
