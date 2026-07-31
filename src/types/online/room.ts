/**
 * 联机功能类型定义 - 房间信令域
 *
 * 与后端 `minecraft::online::signaling` 及 `utils::signaling_handler` 对应。
 * 包含房间创建/加入/查询/保活/参与者管理/封禁等全部房间相关类型。
 */
import type { IceServerEntry } from './signaling'
import type { ModpackMeta } from './modpack'

/** 创建房间请求参数 */
export interface CreateRoomParams {
  sdpOffer: string
  iceCandidates: string[]
  maxPlayers: number
  password: string
  /** 兼容字段：旧客户端仅传 STUN URL 字符串数组 */
  stunServers: string[]
  /**
   * ICE 服务器列表（新客户端优先，可含 TURN 凭据）
   *
   * 阶段三子任务 7 新增。非空时后端优先落库；为空时后端将 `stunServers` 转换为
   * `IceServerEntry` 后落库，保证旧客户端兼容。
   */
  iceServers?: IceServerEntry[]
  hostMcVersion: string
  hostMcPort: number
  /**
   * 房主加载器类型（联机大厅阶段 1 新增）
   *
   * 客户端从 `setup.ini` 的 `Type` 字段读取，值为 `forge` / `fabric` / `neoforge` /
   * `quilt` / `optifine` / `liteloader` / `release` / `snapshot` / `old` / `unknown`。
   * 服务端可据此在大厅列表展示加载器图标，加入方据此判断兼容性。
   * 未传时后端落库为 NULL（兼容旧客户端）。
   */
  hostLoader?: string
  /**
   * 房主加载器版本号（联机大厅阶段 1 新增）
   *
   * 客户端从 `setup.ini` 的 `ForgeVersion` / `FabricVersion` / ... 字段读取，
   * 如 `47.3.0`。无加载器（原版）或 setup.ini 缺失时为空字符串。
   */
  hostLoaderVersion?: string
  /**
   * 房间类型（联机大厅阶段 2 新增）
   *
   * - `private`：仅房间码加入（默认，兼容旧客户端）
   * - `lobby`：加入大厅，可被大厅浏览页检索到
   *
   * 未传时后端默认 `private`。
   */
  roomType?: 'private' | 'lobby'
  /**
   * 大厅 ID（联机大厅阶段 2 新增）
   *
   * 仅当 `roomType = 'lobby'` 时生效，标识房间归属的大厅。
   * 当前固定为 `global`（全球大厅），阶段 5 大厅浏览页支持多大厅选择后扩展。
   * `private` 房间忽略此字段。
   */
  lobbyId?: string
  /**
   * 是否启用白名单（阶段三子任务 8 安全加强）
   *
   * `true` 时仅 `whitelist` 数组中的设备可加入；
   * `true` 且 `whitelist` 为空 = 拒绝所有人加入（仅房主可进入）。
   * 默认 `false`（不启用白名单，允许任何已注册设备加入）。
   */
  whitelistEnabled?: boolean
  /**
   * 初始白名单设备 `device_id` 数组（阶段三子任务 8 安全加强）
   *
   * 仅当 `whitelistEnabled = true` 时生效；未启用时此字段被忽略。
   * 房主可在房间运行期间通过 `room_list_whitelist` / `room_add_whitelist` /
   * `room_remove_whitelist` / `room_set_whitelist_enabled` 动态管理。
   */
  whitelist?: string[]
  /**
   * 整合包元数据（联机大厅阶段 3 新增）
   *
   * `undefined` 表示无整合包（纯原版房间）；传入时服务端 UPSERT 到
   * `room_modpacks` 表并关联到 `rooms.modpack_id`。
   * 前端从 `versions/{id}/modpack.meta.json` 读取后填充（见 `readLocalModpackMeta`）。
   */
  modpack?: ModpackMeta
}

/** 创建房间响应 */
export interface CreateRoomResponse {
  roomCode: string
  hostVirtualIp: string
  subnet: string
  createdAt: number
  expiresAt: number
  /**
   * DataChannel 加密密钥（Base64Url 编码的 32 字节 AES-256 密钥）
   *
   * 阶段三子任务 8 新增。空字符串表示服务器未启用加密（兼容旧服务器）。
   */
  roomKey: string
}

/** 房间公开信息 */
export interface RoomInfoResponse {
  roomCode: string
  hostDevicePk: string
  maxPlayers: number
  playerCount: number
  hasPassword: boolean
  /** 兼容字段：旧房间仅含 STUN URL 数组 */
  stunServers: string[]
  /**
   * ICE 服务器列表（含 STUN + TURN 凭据）
   *
   * 阶段三子任务 7 新增。新房间非空；旧房间可能为空数组，需回退到 `stunServers`。
   */
  iceServers: IceServerEntry[]
  status: 'waiting' | 'active' | 'closed'
  createdAt: number
  expiresAt: number
  hostMcVersion: string
  hostMcPort: number
  /**
   * 是否启用白名单（阶段三子任务 8 安全加强）
   *
   * `true` 时仅白名单内设备可加入；`false` 时允许任何已注册设备加入。
   * 加入方据此判断是否提示房主将自己加入白名单。
   */
  whitelistEnabled: boolean
  /**
   * 房间类型（联机大厅阶段 2，`private` / `public`，旧服务器缺省空字符串）
   */
  roomType?: string
  /**
   * 房主加载器类型（联机大厅阶段 1，如 `forge` / `fabric`，旧服务器缺省 undefined）
   */
  hostLoader?: string
  /**
   * 房主加载器版本号（联机大厅阶段 1，如 `47.3.0`，旧服务器缺省 undefined）
   */
  hostLoaderVersion?: string
  /**
   * 整合包元数据（联机大厅阶段 3，`undefined` 表示纯原版房间）
   *
   * 加入方据此判断是否需要一键安装，通过 `checkLocalModpack` IPC 校验本地是否已装同款。
   */
  modpack?: ModpackMeta
}

/** 加入房间响应 */
export interface JoinRoomResponse {
  participantId: string
  hostSdpOffer: string
  hostIceCandidates: string[]
  /** 兼容字段：旧房间仅含 STUN URL 数组 */
  stunServers: string[]
  /**
   * ICE 服务器列表（含 STUN + TURN 凭据）
   *
   * 阶段三子任务 7 新增。新房间非空；旧房间可能为空数组，需回退到 `stunServers`。
   */
  iceServers: IceServerEntry[]
  playerVirtualIp: string
  subnet: string
  /**
   * DataChannel 加密密钥（Base64Url 编码的 32 字节 AES-256 密钥，与房主一致）
   *
   * 阶段三子任务 8 新增。空字符串表示服务器未启用加密（兼容旧服务器）。
   */
  roomKey: string
}

/** 待确认 Answer */
export interface PendingAnswer {
  participantId: string
  devicePk: string
  sdpAnswer: string
  iceCandidates: string[]
  playerVirtualIp: string
  joinedAt: number
}

/** 待确认 Answer 列表响应 */
export interface ListAnswersResponse {
  answers: PendingAnswer[]
}

/** 参与者信息 */
export interface ParticipantInfo {
  participantId: string
  devicePk: string
  virtualIp: string
  status: 'joined' | 'answered' | 'confirmed' | 'rejected' | 'kicked' | 'left'
  joinedAt: number
  confirmedAt: number
  /** 房主是否已为该参与者生成 SDP Offer（mesh 拓扑，true 表示 offer 已就绪） */
  hostOfferReady: boolean
}

/** 参与者列表响应 */
export interface ListParticipantsResponse {
  participants: ParticipantInfo[]
}

/** 房间封禁记录（房主查询用） */
export interface RoomBan {
  id: string
  roomCode: string
  devicePk: string
  /** 0=永久封禁；>0=解封 Unix 秒时间戳 */
  bannedUntil: number
  /** 封禁发起 Unix 秒时间戳 */
  createdAt: number
}

/** 封禁列表响应（仅房主） */
export interface ListBansResponse {
  /** 当前有效封禁记录（永久 + 未过期临时），已过期的不返回 */
  bans: RoomBan[]
  /** 服务端当前 Unix 秒，便于客户端计算剩余封禁时长 */
  serverTime: number
}

/** keepalive 响应 */
export interface KeepaliveResponse {
  expiresAt: number
  serverTime: number
}

// ============================================================
// mesh 拓扑：参与者级 SDP Offer（阶段三子任务 5）
//
// 房主为每个新加入的参与者单独创建 PeerConnection + Offer 后上传；
// 参与者轮询拉取自己的 Offer，ready=false 表示房主尚未生成。
// ============================================================

/** 房主上传 SDP Offer 请求参数 */
export interface UploadParticipantOfferParams {
  roomCode: string
  participantId: string
  sdpOffer: string
  iceCandidates: string[]
}

/** 参与者拉取 SDP Offer 响应 */
export interface ParticipantOfferResponse {
  /** Offer 是否已就绪（等价于 sdpOffer 非空） */
  ready: boolean
  /** SDP Offer（未就绪时为空字符串） */
  sdpOffer: string
  /** ICE Candidates 数组（未就绪时为空数组） */
  iceCandidates: string[]
}
