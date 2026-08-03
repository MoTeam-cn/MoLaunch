/**
 * 联机 API - 大厅浏览
 *
 * 2 个 action：lobby_list_rooms（分页公开房间）+ lobby_list_categories（分类，MVP 仅 global）。
 * 列表接口不返回 SDP/ICE/room_key 等敏感字段，加入方需走完整 join 流程。
 */

import type {
  BusinessResult,
  LobbyCategoriesResponse,
  LobbyListQuery,
  LobbyListResponse,
} from '@/types/online'
import { ONLINE_ACTIONS, onlineManager } from './core'

/**
 * 查询大厅公开房间列表
 *
 * @param query 查询参数（页码/过滤/关键词），所有字段可选
 * @returns 房间列表 + 分页信息
 */
export function listLobbyRooms(
  query?: LobbyListQuery,
): Promise<BusinessResult<LobbyListResponse>> {
  return onlineManager(ONLINE_ACTIONS.LOBBY_LIST_ROOMS, query ?? {})
}

/**
 * 查询大厅分类列表
 *
 * MVP 阶段仅返回 `global` 一个分类，`roomCount` 实时统计。
 */
export function listLobbyCategories(): Promise<BusinessResult<LobbyCategoriesResponse>> {
  return onlineManager(ONLINE_ACTIONS.LOBBY_LIST_CATEGORIES)
}
