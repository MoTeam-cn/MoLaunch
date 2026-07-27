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

/** 创建房间请求参数 */
export interface CreateRoomParams {
  sdpOffer: string
  iceCandidates: string[]
  maxPlayers: number
  password: string
  stunServers: string[]
  hostMcVersion: string
  hostMcPort: number
}

/** 创建房间响应 */
export interface CreateRoomResponse {
  roomCode: string
  hostVirtualIp: string
  subnet: string
  createdAt: number
  expiresAt: number
}

/** 房间公开信息 */
export interface RoomInfoResponse {
  roomCode: string
  hostDevicePk: string
  maxPlayers: number
  playerCount: number
  hasPassword: boolean
  stunServers: string[]
  status: 'waiting' | 'active' | 'closed'
  createdAt: number
  expiresAt: number
  hostMcVersion: string
  hostMcPort: number
}

/** 加入房间响应 */
export interface JoinRoomResponse {
  participantId: string
  hostSdpOffer: string
  hostIceCandidates: string[]
  stunServers: string[]
  playerVirtualIp: string
  subnet: string
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
