/**
 * 联机功能类型定义
 *
 * 与后端 `minecraft::online` 模块及 `utils::online_manager` 中注册的 action 对应。
 * 字段命名采用 camelCase（后端 `#[serde(rename_all = "camelCase")]` 或显式 `rename`）。
 */

/**
 * 设备认证状态
 *
 * 对应后端 `utils::online_manager::DeviceStatus`。
 * 不发起网络请求，仅读本地凭证 + 配置中的 api_server_url。
 */
export interface DeviceStatus {
  /** 是否已注册（device_pk + 三组密钥齐全） */
  registered: boolean
  /** 是否已登录（device_token 非空） */
  logged_in: boolean
  /** JWT 是否已过期（容差 60 秒） */
  token_expired: boolean
  /** 设备主键（UUID） */
  device_pk: string
  /** 设备友好标识（mcsdk-xxxx-xxxx-xxxx-xxxx） */
  device_id: string
  /** JWT 过期时间（Unix 秒） */
  token_expires_at: number
  /** 最后登录时间（Unix 秒） */
  last_login_at: number
  /** 当前配置的 api-server 地址 */
  api_server_url: string
}

/**
 * 启动静默认证结果
 *
 * 对应后端 `utils::online_manager::AuthInitResult`。
 * 由 `auth_init` action 返回，前端据此设置 `cloudConnected` 全局状态。
 */
export interface AuthInitResult {
  /** 设备认证状态快照 */
  status: DeviceStatus
  /** 错误信息（null 表示成功；非 null 表示云端连接失败，需降级） */
  error: string | null
}

/**
 * 服务器时间信息
 *
 * 对应后端 `utils::online_manager::ServerTimeInfo`。
 * 用于测试 api-server 连通性 + 校准本地时间。
 */
export interface ServerTimeInfo {
  /** 服务器 Unix 时间戳（秒） */
  server_time: number
  /** RFC3339 格式时间字符串 */
  rfc3339: string
  /** 服务器时区名称（如 "Asia/Shanghai"） */
  timezone: string
  /** 时区偏移秒数（如 +28800 表示 UTC+8） */
  offset_seconds: number
}

// ============================================================
// 信令相关类型（阶段二）
//
// 与后端 `minecraft::online::signaling` 及 `utils::signaling_handler` 对应。
// 字段命名 camelCase，与后端 `#[serde(rename_all = "camelCase")]` 一致。
// 业务响应统一为 `BusinessResult<T>`（含 code/data/msg 字段）。
// ============================================================

/** 统一业务响应（解密后） */
export interface BusinessResult<T = unknown> {
  /** 业务码：1=成功，0=系统错误，1001+=业务错误 */
  code: number
  /** 业务数据，失败时为 null */
  data: T | null
  /** 提示消息，成功默认 "ok" */
  msg: string
  /** 服务端时间（ISO 8601 UTC） */
  time: string
  /** 请求 ID */
  req_id: string
}

/** STUN 服务器列表响应 */
export interface StunServersResponse {
  servers: string[]
}

/**
 * ICE 服务器条目（统一 STUN + TURN）
 *
 * 阶段三子任务 7 新增。对应后端 `IceServerEntry`：
 * - `urls` 长度 ≥ 1，首项通常为 `stun:` / `turn:` / `turns:` 协议 URL
 * - `username` / `credential` 仅 TURN 需要填充（STUN 缺省）
 *
 * 浏览器侧 `RTCIceServer` 接口对齐：
 * ```ts
 * { urls: ['stun:stun.example.com:3478'] }
 * { urls: ['turn:turn.example.com:3478?transport=udp'], username: 'foo', credential: 'bar' }
 * ```
 */
export interface IceServerEntry {
  urls: string[]
  /** TURN 用户名（STUN 时缺省） */
  username?: string
  /** TURN 凭据（STUN 时缺省） */
  credential?: string
}

/**
 * TURN 服务器列表响应（房主独占接口）
 *
 * 阶段三子任务 7 新增。对应后端 `TurnServersResponse`：
 * - `servers`：经服务端负载过滤后的可下发 TURN 条目（可能为空数组）
 * - `enabled`：全局 TURN 开关（false 时始终不下发）
 * - `currentTotalLoad`：当前集群已用负载（用于客户端展示/调试）
 * - `loadThreshold`：集群负载阈值（≥ 时本次不下发新 TURN）
 *
 * 房主拉取后通过 DataChannel 控制消息 0x05 广播给房间内所有参与者。
 */
export interface TurnServersResponse {
  /** 过滤后的 TURN 服务器列表（可能为空） */
  servers: IceServerEntry[]
  /** TURN 全局开关 */
  enabled: boolean
  /** 当前集群总负载 */
  currentTotalLoad: number
  /** 集群负载阈值 */
  loadThreshold: number
}

/**
 * 整合包元数据（联机大厅阶段 3 新增）
 *
 * 房主创建房间时关联本地已安装整合包，上报给 api-server。
 * 加入方拉取房间详情后据此判断是否需要一键安装。
 *
 * **安全设计**：不包含 `downloadUrl` 字段。加入方通过现有 `getProjectVersions`
 * IPC 反查平台 API 获取下载链接，避免 api-server 成为 URL 分发中心。
 *
 * 对应后端 `minecraft::online::signaling::ModpackMeta`。
 */
export interface ModpackMeta {
  /** 来源平台（`curseforge` / `modrinth`） */
  source: string
  /** CF project id 或 MR project id */
  projectId: string
  /** CF file id 或 MR version id */
  fileId: string
  /** 整合包对应的 MC 版本（如 `1.12.2`） */
  mcVersion: string
  /** 整合包自身版本号（如 `2.9.3`） */
  modpackVersion?: string
  /** 整合包名称 */
  name: string
  /** 加载器类型（`forge` / `fabric` / `neoforge` / `quilt`） */
  loader?: string
  /** 加载器版本号 */
  loaderVersion?: string
  /** 整合包文件大小（字节，仅展示用） */
  fileSize?: number
  /** mods 文件数（仅展示用） */
  fileCount?: number
  /** manifest.json SHA-256，用于加入方校验本地是否已装同款 */
  manifestHash?: string
}

/**
 * 本地整合包元数据文件（`versions/{id}/modpack.meta.json`）
 *
 * 整合包安装完成时由后端写入，创建联机房间时读取并转换为 `ModpackMeta` 上报。
 * 与 `ModpackMeta` 字段一致，额外含 `installedAt` 本地记录（不上报）。
 *
 * 对应后端 `minecraft::version::modpack_meta::ModpackMetaFile`。
 */
export interface ModpackMetaFile {
  source: string
  projectId: string
  fileId: string
  mcVersion: string
  modpackVersion?: string
  name: string
  loader?: string
  loaderVersion?: string
  fileSize?: number
  fileCount?: number
  manifestHash?: string
  /** 安装时间（Unix 秒，仅本地记录，不上报） */
  installedAt: number
}

/**
 * 校验本地是否已安装指定整合包的结果（联机大厅阶段 4 新增）
 *
 * 对应后端 `commands::version::list::CheckLocalModpackResult`。
 * 加入方加入房间后据此判断是否需要一键安装房主要求的整合包。
 */
export interface CheckLocalModpackResult {
  /** 是否已安装 */
  installed: boolean
  /** 匹配的 version_id（`installed=false` 时为 undefined） */
  versionId?: string
}

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

// ============================================================
// TUN 桥接管理（阶段三子任务 5：数据分发打通）
//
// 后端 `utils::tun_manager` 注册 3 个 IPC action，前端通过 `onlineManager` 调用。
// - `tun_start`：创建 TUN 接口 + 启动读写循环 + emit `online://tun-packet-out` 事件
// - `tun_forward_to`：前端 DataChannel 收到消息后调用，base64 编码传入，后端解码并写入 TUN
// - `tun_stop`：停止桥接，销毁 TUN 接口
// ============================================================

/** `tun_start` 参数 */
export interface TunStartParams {
  /** 虚拟 IPv4 地址（如 `10.244.1.1`） */
  ipv4: string
  /** 子网前缀长度（如 24，对应 `10.244.1.0/24`） */
  prefixLen: number
}

/** `tun_start` 返回 */
export interface TunStartResponse {
  /** TUN 接口名（如 `tun-molaunch`） */
  interfaceName: string
  /** 虚拟 IP */
  ipv4: string
  /** 子网前缀长度 */
  prefixLen: number
  /** MTU */
  mtu: number
}

/** `tun_forward_to` 返回 */
export interface TunForwardResponse {
  /** 是否为数据包（true=已写入 TUN，false=控制/错误消息，未写入） */
  isData: boolean
  /** 解码出的 IP 包字节数（控制消息为 0） */
  packetLen: number
}

/** 后端 emit 给前端的 TUN 数据包事件名 */
export const EVENT_TUN_PACKET_OUT = 'online://tun-packet-out'

/** TUN 数据包事件 payload（后端 emit 的 `Vec<u8>` 协议帧，Tauri 序列化为 number[]） */
export type TunPacketPayload = number[]

// ============================================================
// NAT 类型定义
// ============================================================

/**
 * NAT 类型枚举
 *
 * 参考 RFC 3489 / STUN RFC 5389 的 NAT 分类：
 * - `Open`：公网 IP，无 NAT（罕见）
 * - `FullCone`：全锥 NAT，任意外部主机可访问映射端口（联机最佳）
 * - `RestrictedCone`：限制锥 NAT，仅允许联系过的外部 IP（联机可用）
 * - `PortRestrictedCone`：端口限制锥 NAT，仅允许联系过的外部 IP:Port（联机可用，但兼容性较差）
 * - `Symmetric`：对称 NAT，每个目标分配独立端口（无 STUN 中转无法 P2P）
 * - `Blocked`：UDP 被阻断（无法 P2P）
 * - `Unknown`：检测失败或浏览器不支持
 */
export type NatType =
  | 'Open'
  | 'FullCone'
  | 'RestrictedCone'
  | 'PortRestrictedCone'
  | 'Symmetric'
  | 'Blocked'
  | 'Unknown'

/** NAT 检测结果 */
export interface NatDetectionResult {
  /** NAT 类型 */
  type: NatType
  /** 检测耗时（毫秒） */
  durationMs: number
  /** 本地出口 IP（如有） */
  localIp?: string
  /** 公网 IP（如有） */
  publicIp?: string
  /** 检测错误信息（失败时） */
  error?: string
}

// ============================================================
// 房主白名单管理（阶段三子任务 8 安全加强）
//
// 对应后端 `minecraft::online::signaling::WhitelistEntry` / `WhitelistResponse`。
// 房主可启用白名单后指定允许加入的设备（按 `device_id` 友好标识），
// 启用且白名单为空 = 拒绝所有人加入（仅房主可进入）。
// ============================================================

/** 白名单条目（房主查询/管理用） */
export interface WhitelistEntry {
  /** 设备主键（UUID） */
  devicePk: string
  /** 设备友好标识（如 `mcsdk-xxxx-xxxx-xxxx-xxxx`） */
  deviceId: string
  /** 加入白名单时间（Unix 秒） */
  addedAt: number
}

/** 白名单列表响应 */
export interface WhitelistResponse {
  /** 是否启用白名单 */
  enabled: boolean
  /** 白名单条目数组 */
  entries: WhitelistEntry[]
}

// ============================================================
// 大厅浏览（联机大厅阶段 5）
//
// 对应后端 `minecraft::online::signaling::Lobby*` 类型。
// 大厅列表接口不返回 SDP/ICE/room_key 等敏感字段，
// 加入方需调用 `POST /v1/signaling/rooms/{code}/join` 走完整加入流程。
// ============================================================

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
