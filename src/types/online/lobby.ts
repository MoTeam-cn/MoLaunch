/**
 * 联机功能类型定义 - 大厅域（Scaffolding 收敛版）
 *
 * 大厅按整合包聚类（热度），展开查看该整合包下的公开房间摘要。
 * 房间摘要只含 N 段公开标识与基础信息，完整码经 room_join 闸门获取。
 */

/** 大厅聚合条目（按整合包分组，字段对齐 api-server） */
export interface LobbyPackageItem {
  /** 整合包记录主键（UUID） */
  modpackId: string
  /** 整合包名称 */
  name: string
  /** 来源平台（curseforge / modrinth） */
  source: string
  projectId: string
  /** 平台文件 ID */
  fileId: string
  /** 整合包对应的 MC 版本 */
  mcVersion: string
  /** 整合包自身版本号 */
  modpackVersion?: string
  /** 加载器类型 */
  loader?: string
  /** 公开房间数 */
  roomCount: number
}

/** 大厅聚合响应（api-server 返回非分页结构） */
export interface LobbyPackagesResponse {
  packages: LobbyPackageItem[]
}

/** 大厅房间列表查询参数（仅 packageId 生效，服务端忽略分页参数） */
export interface LobbyListQuery {
  /** 整合包 ID（仅返回该整合包下的公开房间） */
  packageId?: string
  /** 页码（服务端暂不支持分页，保留兼容） */
  page?: number
  /** 每页数量（服务端暂不支持分页，保留兼容） */
  pageSize?: number
}

/** 大厅房间列表项（摘要，绝不包含完整码/密钥） */
export interface LobbyRoomItem {
  /** N 段公开标识（入房用） */
  publicIdentifier: string
  remark: string
  hasPassword: boolean
  /** 当前在线人数（服务端暂不返回，恒为 0） */
  playerCount: number
  maxPlayers: number
  hostMcVersion?: string
  hostLoader?: string
  createdAt: number
}

/** 大厅房间列表响应（api-server 返回非分页结构） */
export interface LobbyListResponse {
  rooms: LobbyRoomItem[]
}
