/**
 * 联机功能统一 API 入口
 *
 * 后端 `online_manager` IPC 命令通过 `action` 字段分发到不同子模块
 * （参照 `community_manager` / `meta_manager` 模式）。
 *
 * 字段名约定：后端 Params 结构体使用 `#[serde(rename_all = "camelCase")]`，
 * 故前端 params 对象的字段名一律使用 camelCase。
 *
 * 阶段一注册的 6 个 action（认证相关）：
 * - `auth_status`：查询当前设备状态（不发网络请求）
 * - `auth_get_server_time`：获取服务器时间（测试连通性 + 校准时间）
 * - `auth_register`：注册新设备（生成密钥对 + 上报公钥 + 获取 JWT）
 * - `auth_login`：登录设备（用本地密钥签名换取新 JWT）
 * - `auth_logout`：登出设备（撤销 JWT，保留密钥）
 * - `auth_clear`：清除设备凭证（注销设备，删除本地密钥）
 *
 * 阶段二补充 13 个 action（信令相关）：
 * - `room_get_stun` / `room_create` / `room_get` / `room_close`
 * - `room_join` / `room_submit_answer` / `room_list_answers` / `room_confirm`
 * - `room_keepalive` / `room_leave` / `room_kick` / `room_unban` / `room_list_bans` / `room_list_participants`
 *
 * 阶段三子任务 7 补充 1 个 action（TURN 中继）：
 * - `room_get_turn`：房主独占，拉取经服务端负载过滤后的 TURN 服务器列表（含集群负载快照）
 *
 * 阶段三补充 2 个 action（mesh 拓扑：参与者级 SDP Offer）：
 * - `room_upload_participant_offer`：房主为指定参与者上传独立 SDP Offer + ICE
 * - `room_fetch_participant_offer`：参与者轮询拉取房主为自己生成的 SDP Offer
 *
 * 阶段三子任务 5 补充 3 个 action（TUN 桥接：数据分发打通）：
 * - `tun_start`：创建 TUN 接口 + 启动读写循环 + emit `online://tun-packet-out` 事件
 * - `tun_forward_to`：前端 DataChannel 收到消息后调用，base64 编码传入，后端解码并写入 TUN
 * - `tun_stop`：停止桥接，销毁 TUN 接口
 */

import { invoke } from '@tauri-apps/api/core'
import type {
  BusinessResult,
  CreateRoomParams,
  CreateRoomResponse,
  DeviceStatus,
  JoinRoomResponse,
  KeepaliveResponse,
  ListAnswersResponse,
  ListBansResponse,
  ListParticipantsResponse,
  LobbyCategoriesResponse,
  LobbyListQuery,
  LobbyListResponse,
  ParticipantOfferResponse,
  RoomInfoResponse,
  ServerTimeInfo,
  StunServersResponse,
  TurnServersResponse,
  TunForwardResponse,
  TunStartParams,
  TunStartResponse,
  WhitelistResponse,
} from '@/types/online'

/**
 * 调用 online_manager IPC
 * @param action 操作名称（取自 ONLINE_ACTIONS 常量）
 * @param params 参数对象（字段名使用 camelCase）
 */
export async function onlineManager<T = unknown>(
  action: string,
  params?: unknown,
): Promise<T> {
  return invoke<T>('online_manager', { req: { action, params: params ?? null } })
}

/**
 * 所有可用的 action 名称
 *
 * 与后端 `utils::online_manager::DISPATCHER` 注册的 action 一一对应。
 */
export const ONLINE_ACTIONS = {
  // 认证
  AUTH_STATUS: 'auth_status',
  AUTH_GET_SERVER_TIME: 'auth_get_server_time',
  AUTH_REGISTER: 'auth_register',
  AUTH_LOGIN: 'auth_login',
  AUTH_LOGOUT: 'auth_logout',
  AUTH_CLEAR: 'auth_clear',
  // 信令
  ROOM_GET_STUN: 'room_get_stun',
  ROOM_CREATE: 'room_create',
  ROOM_GET: 'room_get',
  ROOM_CLOSE: 'room_close',
  ROOM_JOIN: 'room_join',
  ROOM_SUBMIT_ANSWER: 'room_submit_answer',
  ROOM_LIST_ANSWERS: 'room_list_answers',
  ROOM_CONFIRM: 'room_confirm',
  ROOM_KEEPALIVE: 'room_keepalive',
  ROOM_LEAVE: 'room_leave',
  ROOM_KICK: 'room_kick',
  ROOM_UNBAN: 'room_unban',
  // 阶段 6.2：房主查询封禁列表
  ROOM_LIST_BANS: 'room_list_bans',
  ROOM_LIST_PARTICIPANTS: 'room_list_participants',
  // TURN 中继：房主独占（阶段三子任务 7）
  ROOM_GET_TURN: 'room_get_turn',
  // mesh 拓扑：参与者级 SDP Offer
  ROOM_UPLOAD_PARTICIPANT_OFFER: 'room_upload_participant_offer',
  ROOM_FETCH_PARTICIPANT_OFFER: 'room_fetch_participant_offer',
  // TUN 桥接：数据分发打通
  TUN_START: 'tun_start',
  TUN_FORWARD_TO: 'tun_forward_to',
  TUN_STOP: 'tun_stop',
  // TUN 权限不足时以管理员权限重启
  RESTART_AS_ADMIN: 'restart_as_admin',
  // 房主白名单管理（阶段三子任务 8 安全加强）
  ROOM_LIST_WHITELIST: 'room_list_whitelist',
  ROOM_ADD_WHITELIST: 'room_add_whitelist',
  ROOM_REMOVE_WHITELIST: 'room_remove_whitelist',
  ROOM_SET_WHITELIST_ENABLED: 'room_set_whitelist_enabled',
  // 大厅浏览（联机大厅阶段 5）
  LOBBY_LIST_ROOMS: 'lobby_list_rooms',
  LOBBY_LIST_CATEGORIES: 'lobby_list_categories',
} as const

/** action 名称类型 */
export type OnlineAction = typeof ONLINE_ACTIONS[keyof typeof ONLINE_ACTIONS]

// ============================================================
// 认证相关便捷封装
// ============================================================

/** 查询当前设备状态（不发起网络请求，仅读本地凭证） */
export function getAuthStatus(): Promise<DeviceStatus> {
  return onlineManager<DeviceStatus>(ONLINE_ACTIONS.AUTH_STATUS)
}

/** 获取服务器时间（用于测试 api-server 连通性 + 校准本地时间） */
export function getServerTime(): Promise<ServerTimeInfo> {
  return onlineManager<ServerTimeInfo>(ONLINE_ACTIONS.AUTH_GET_SERVER_TIME)
}

/** 注册新设备（生成密钥对 + 上报公钥 + 获取 JWT） */
export function registerDevice(): Promise<DeviceStatus> {
  return onlineManager<DeviceStatus>(ONLINE_ACTIONS.AUTH_REGISTER)
}

/** 登录设备（用本地密钥签名换取新 JWT） */
export function loginDevice(): Promise<DeviceStatus> {
  return onlineManager<DeviceStatus>(ONLINE_ACTIONS.AUTH_LOGIN)
}

/** 登出设备（撤销 JWT，保留密钥） */
export function logoutDevice(): Promise<{ success: boolean }> {
  return onlineManager<{ success: boolean }>(ONLINE_ACTIONS.AUTH_LOGOUT)
}

/** 清除设备凭证（注销设备，删除本地密钥） */
export function clearDevice(): Promise<{ success: boolean }> {
  return onlineManager<{ success: boolean }>(ONLINE_ACTIONS.AUTH_CLEAR)
}

// ============================================================
// 信令相关便捷封装（阶段二）
//
// 所有信令 action 返回 `BusinessResult<T>`（含 code/data/msg/time/req_id），
// 调用方需检查 `code === 1` 后取 `data` 字段使用。
// ============================================================

/** 获取 STUN 服务器列表 */
export function getStunServers(): Promise<BusinessResult<StunServersResponse>> {
  return onlineManager(ONLINE_ACTIONS.ROOM_GET_STUN)
}

/** 创建房间（房主上传 SDP Offer + ICE） */
export function createRoom(
  params: CreateRoomParams,
): Promise<BusinessResult<CreateRoomResponse>> {
  return onlineManager(ONLINE_ACTIONS.ROOM_CREATE, params)
}

/** 查询房间公开信息 */
export function getRoomInfo(
  roomCode: string,
): Promise<BusinessResult<RoomInfoResponse>> {
  return onlineManager(ONLINE_ACTIONS.ROOM_GET, { roomCode })
}

/** 关闭房间（仅房主） */
export function closeRoom(
  roomCode: string,
): Promise<BusinessResult<unknown>> {
  return onlineManager(ONLINE_ACTIONS.ROOM_CLOSE, { roomCode })
}

/** 加入房间 */
export function joinRoom(
  roomCode: string,
  password = '',
): Promise<BusinessResult<JoinRoomResponse>> {
  return onlineManager(ONLINE_ACTIONS.ROOM_JOIN, { roomCode, password })
}

/** 提交 SDP Answer（加入方） */
export function submitAnswer(
  roomCode: string,
  participantId: string,
  sdpAnswer: string,
  iceCandidates: string[],
): Promise<BusinessResult<unknown>> {
  return onlineManager(ONLINE_ACTIONS.ROOM_SUBMIT_ANSWER, {
    roomCode,
    participantId,
    sdpAnswer,
    iceCandidates,
  })
}

/** 拉取待确认 Answer 列表（房主轮询） */
export function listAnswers(
  roomCode: string,
): Promise<BusinessResult<ListAnswersResponse>> {
  return onlineManager(ONLINE_ACTIONS.ROOM_LIST_ANSWERS, { roomCode })
}

/** 确认/拒绝连接（房主） */
export function confirmParticipant(
  roomCode: string,
  participantId: string,
  accepted: boolean,
): Promise<BusinessResult<unknown>> {
  return onlineManager(ONLINE_ACTIONS.ROOM_CONFIRM, {
    roomCode,
    participantId,
    accepted,
  })
}

/** 房主保活（续期 + 更新保活时间戳） */
export function keepaliveRoom(
  roomCode: string,
): Promise<BusinessResult<KeepaliveResponse>> {
  return onlineManager(ONLINE_ACTIONS.ROOM_KEEPALIVE, { roomCode })
}

/** 退出房间（加入方） */
export function leaveRoom(
  roomCode: string,
): Promise<BusinessResult<unknown>> {
  return onlineManager(ONLINE_ACTIONS.ROOM_LEAVE, { roomCode })
}

/** 踢出参与者（房主，可选封禁） */
export function kickParticipant(
  roomCode: string,
  participantId: string,
  banDurationSeconds: number | null,
): Promise<BusinessResult<unknown>> {
  return onlineManager(ONLINE_ACTIONS.ROOM_KICK, {
    roomCode,
    participantId,
    banDurationSeconds,
  })
}

/** 解封参与者（房主） */
export function unbanParticipant(
  roomCode: string,
  devicePk: string,
): Promise<BusinessResult<unknown>> {
  return onlineManager(ONLINE_ACTIONS.ROOM_UNBAN, { roomCode, devicePk })
}

/**
 * 查询房间封禁列表（仅房主）
 *
 * 返回当前有效的封禁记录（永久 + 未过期临时），已过期的临时封禁不返回。
 * 同时返回服务端当前时间 `serverTime`，便于客户端计算剩余封禁时长。
 */
export function listBannedParticipants(
  roomCode: string,
): Promise<BusinessResult<ListBansResponse>> {
  return onlineManager(ONLINE_ACTIONS.ROOM_LIST_BANS, { roomCode })
}

/** 查询参与者列表（房主） */
export function listParticipants(
  roomCode: string,
): Promise<BusinessResult<ListParticipantsResponse>> {
  return onlineManager(ONLINE_ACTIONS.ROOM_LIST_PARTICIPANTS, { roomCode })
}

// ============================================================
// TURN 中继管理（阶段三子任务 7）
//
// 仅房主可调用 `room_get_turn`，服务端会基于全局开关、单机负载、集群总负载
// 三层过滤后下发可用的 TURN 服务器。房主拉取后通过 DataChannel 控制消息
// 0x05 广播给房间内所有参与者（加入方不能直接调用此接口）。
// ============================================================

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

// ============================================================
// mesh 拓扑：参与者级 SDP Offer（阶段三子任务 5）
//
// 房主为每个新加入的参与者单独创建 PeerConnection + Offer 后上传；
// 参与者轮询拉取自己的 Offer，ready=false 表示房主尚未生成。
// ============================================================

/** 房主为指定参与者上传 SDP Offer（mesh 拓扑） */
export function uploadParticipantOffer(
  roomCode: string,
  participantId: string,
  sdpOffer: string,
  iceCandidates: string[],
): Promise<BusinessResult<unknown>> {
  return onlineManager(ONLINE_ACTIONS.ROOM_UPLOAD_PARTICIPANT_OFFER, {
    roomCode,
    participantId,
    sdpOffer,
    iceCandidates,
  })
}

/** 参与者拉取房主为自己生成的 SDP Offer（mesh 拓扑，轮询用） */
export function fetchParticipantOffer(
  roomCode: string,
  participantId: string,
): Promise<BusinessResult<ParticipantOfferResponse>> {
  return onlineManager(ONLINE_ACTIONS.ROOM_FETCH_PARTICIPANT_OFFER, {
    roomCode,
    participantId,
  })
}

// ============================================================
// TUN 桥接管理（阶段三子任务 5：数据分发打通）
//
// 3 个 action 与后端 `utils::tun_manager` 注册一一对应。
// 数据流：
// - 后端 TUN 读包 → emit `online://tun-packet-out` 事件 → 前端 listen → DataChannel.send
// - 前端 DataChannel.onmessage → ArrayBuffer → base64 → invoke `tun_forward_to` → 写入 TUN
// ============================================================

/**
 * 启动 TUN 桥接（房主与加入方通用）
 *
 * 调用后后端会：
 * 1. 若已有 bridge，先停止（防止泄漏）
 * 2. 创建 TUN 接口（绑定 ipv4/prefix_len）
 * 3. 启动 select! 单读写循环 task
 * 4. emit `online://tun-packet-out` 事件给前端（每读到 IP 包就 emit）
 *
 * @param params 含虚拟 IP 与子网前缀长度
 * @returns 接口信息（接口名 / IP / 前缀长度 / MTU）
 */
export function tunStart(params: TunStartParams): Promise<TunStartResponse> {
  return onlineManager<TunStartResponse>(ONLINE_ACTIONS.TUN_START, params)
}

/**
 * 将 DataChannel 收到的消息转发到后端 TUN
 *
 * 前端从 `DataChannel.onmessage` 拿到 `ArrayBuffer` 后，转 base64 调用此函数。
 * 后端 base64 解码 → 协议帧 decode → 写入 TUN 接口。
 *
 * @param dataChannelMessage DataChannel 收到的二进制消息（协议帧编码后的字节）
 * @returns 是否为数据包 + IP 包字节数
 */
export function tunForwardTo(
  dataChannelMessage: ArrayBuffer | Uint8Array,
): Promise<TunForwardResponse> {
  // ArrayBuffer / Uint8Array → base64 字符串
  const bytes = dataChannelMessage instanceof Uint8Array
    ? dataChannelMessage
    : new Uint8Array(dataChannelMessage)
  // 分块处理避免 apply 参数上限（DataChannel 消息一般 < 64KB，但稳妥起见分块）
  let binary = ''
  const chunkSize = 0x8000
  for (let i = 0; i < bytes.length; i += chunkSize) {
    binary += String.fromCharCode.apply(null, bytes.subarray(i, i + chunkSize) as unknown as number[])
  }
  const messageBase64 = btoa(binary)
  return onlineManager<TunForwardResponse>(ONLINE_ACTIONS.TUN_FORWARD_TO, {
    messageBase64,
  })
}

/** 停止 TUN 桥接，销毁 TUN 接口（幂等） */
export function tunStop(): Promise<{ success: boolean }> {
  return onlineManager<{ success: boolean }>(ONLINE_ACTIONS.TUN_STOP)
}

/**
 * 以管理员权限重启启动器
 *
 * `tun_start` 返回 `TUN_PERMISSION_DENIED:` 前缀错误时，前端经用户确认后调用。
 * - release 模式：后端通过 `ShellExecuteW("runas")` 触发 UAC 提权并延迟退出当前进程
 * - dev 模式：后端返回 `dev_mode: true` 不重启（避免丢失 Vite dev server 连接），
 *   前端应提示用户用管理员权限终端运行 `npm run tauri dev`
 */
export interface RestartAsAdminResult {
  success: boolean
  /** dev 模式下为 true，表示未自动重启，需提示用户手动以管理员权限启动 */
  dev_mode?: boolean
  /** dev 模式下的提示文案 */
  message?: string
}

export function restartAsAdmin(): Promise<RestartAsAdminResult> {
  return onlineManager<RestartAsAdminResult>(ONLINE_ACTIONS.RESTART_AS_ADMIN)
}

// ============================================================
// 房主白名单管理（阶段三子任务 8 安全加强）
//
// 4 个 action 与后端 `utils::signaling_manager` 注册一一对应：
// - `room_list_whitelist`：查询当前房间的白名单启用状态 + 条目列表
// - `room_add_whitelist`：按 `device_id` 友好标识添加白名单条目
// - `room_remove_whitelist`：按 `device_id` 移除白名单条目
// - `room_set_whitelist_enabled`：动态开关白名单（不影响条目数据）
//
// 仅房主可调用，加入方无权管理白名单。
// 启用白名单且条目为空时，拒绝所有人加入（仅房主可进入，便于私密联机）。
// ============================================================

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

// ============================================================
// 大厅浏览（联机大厅阶段 5）
//
// 2 个 action 与后端 `utils::signaling_manager` 注册一一对应：
// - `lobby_list_rooms`：分页查询公开房间列表，支持加载器/版本/关键词过滤
// - `lobby_list_categories`：查询大厅分类列表（MVP 阶段仅 `global`）
//
// 列表接口不返回 SDP/ICE/room_key 等敏感字段，加入方需走完整 join 流程。
// ============================================================

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
