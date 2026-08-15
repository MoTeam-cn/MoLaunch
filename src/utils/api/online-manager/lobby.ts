/**
 * 大厅 IPC 封装（Scaffolding 收敛版：packages 聚合 + rooms 列表）
 */
import type {
  BusinessResult,
  LobbyListQuery,
  LobbyListResponse,
  LobbyPackagesResponse,
} from '@/types/online'
import { onlineManager, ONLINE_ACTIONS } from './core'

/** 查询大厅聚合（按整合包分组，含热度/房间数） */
export function listLobbyPackages(): Promise<BusinessResult<LobbyPackagesResponse>> {
  return onlineManager<BusinessResult<LobbyPackagesResponse>>(ONLINE_ACTIONS.LOBBY_LIST_PACKAGES, {})
}

/** 查询某整合包下的公开房间摘要列表 */
export function listLobbyRooms(query: LobbyListQuery = {}): Promise<BusinessResult<LobbyListResponse>> {
  return onlineManager<BusinessResult<LobbyListResponse>>(ONLINE_ACTIONS.LOBBY_LIST_ROOMS, {
    packageId: query.packageId,
    page: query.page,
    pageSize: query.pageSize,
  })
}
