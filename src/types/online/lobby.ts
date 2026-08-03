/**
 * 联机功能类型定义 - 大厅浏览域（联机大厅阶段 5）
 *
 * 对应后端 signaling::Lobby* 类型；列表接口不返回 SDP/ICE/room_key 等敏感字段，
 * 加入方需调 POST /v1/signaling/rooms/{code}/join 走完整加入流程。
 */

/** 大厅房间列表查询参数 */
export interface LobbyListQuery {
  /** 大厅分类 ID，默认 `global` */
  lobbyId?: string
  /** 页码，默认 1 */
  page?: number
  /** 每页数量，默认 20，上限 50 */
  pageSize?: number
  /** `true` 仅返回有整合包的房间；`false` 仅返回无整合包房间；不传则不过滤 */
  hasModpack?: boolean
  /** 按房主加载器过滤（`forge` / `fabric` / `neoforge` / `quilt` / `vanilla`） */
  loader?: string
  /** 按房主 MC 版本或整合包 MC 版本过滤 */
  gameVersion?: string
  /** 模糊匹配房主 MC 版本或整合包名称 */
  keyword?: string
}

/**
 * 大厅整合包摘要（列表页轻量版）
 *
 * 与 `ModpackMeta` 的差异：
 * - 多出 `modpackId`（服务端主键）
 * - 缺少 `manifestHash` / `loaderVersion`（减少列表页载荷）
 */
export interface LobbyModpackSummary {
  /** 整合包记录主键（UUID） */
  modpackId: string
  name: string
  modpackVersion?: string
  /** 来源平台（`curseforge` / `modrinth`） */
  source: string
  projectId: string
  fileId: string
  mcVersion: string
  loader?: string
  fileSize?: number
  fileCount?: number
}

/** 大厅房间列表项 */
export interface LobbyRoomItem {
  roomCode: string
  hostDevicePk: string
  hostMcVersion: string
  hostLoader?: string
  hostLoaderVersion?: string
  maxPlayers: number
  playerCount: number
  hasPassword: boolean
  status: 'waiting' | 'active' | 'closed'
  createdAt: number
  expiresAt: number
  /** 整合包摘要，`undefined` 表示纯原版房间 */
  modpack?: LobbyModpackSummary
}

/** 大厅房间列表响应 */
export interface LobbyListResponse {
  total: number
  page: number
  pageSize: number
  items: LobbyRoomItem[]
}

/** 大厅分类条目 */
export interface LobbyCategory {
  id: string
  name: string
  roomCount: number
}

/** 大厅分类列表响应 */
export interface LobbyCategoriesResponse {
  categories: LobbyCategory[]
}
