/**
 * 联机功能类型定义 - 大厅域（Scaffolding 收敛版）
 *
 * 大厅按整合包聚类（热度），展开查看该整合包下的公开房间摘要。
 * 房间摘要只含 N 段公开标识与基础信息，完整码经 room_join 闸门获取。
 */

/** 大厅聚合条目（按整合包分组） */
export interface LobbyPackageItem {
  /** 整合包记录主键（UUID） */
  modpackId: string
  /** 整合包名称 */
  name: string
  /** 来源平台（curseforge / modrinth） */
  source: string
  projectId: string
  /** 公开房间数 */
  roomCount: number
  /** 热度（服务端聚合排序） */
  heat: number
  mcVersion?: string
}

/** 大厅聚合响应 */
export interface LobbyPackagesResponse {
  total: number
  page: number
  pageSize: number
  items: LobbyPackageItem[]
}

/** 大厅房间列表查询参数（所有字段可选） */
export interface LobbyListQuery {
  /** 整合包 ID（仅返回该整合包下的公开房间） */
  packageId?: string
  /** 页码，默认 1 */
  page?: number
  /** 每页数量，默认 20，上限 50 */
  pageSize?: number
}

/** 大厅房间列表项（摘要，绝不包含完整码/密钥） */
export interface LobbyRoomItem {
  /** N 段公开标识（入房用） */
  publicIdentifier: string
  remark: string
  hasPassword: boolean
  playerCount: number
  maxPlayers: number
  hostMcVersion: string
  hostLoader?: string
  createdAt: number
}

/** 大厅房间列表响应 */
export interface LobbyListResponse {
  total: number
  page: number
  pageSize: number
  items: LobbyRoomItem[]
}
