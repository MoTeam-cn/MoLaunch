/**
 * 联机 store 房间切片
 *
 * 从 stores/online.ts 抽取的房间 state + actions，按 Pinia setup store 的
 * composable 切片模式组织。切片内部闭环：
 * - fetchStunServers 被 hostCreateRoom 复用
 * - refreshRoomInfo 被 guestJoinRoom 复用
 * - resetRoomState 仅重置 roomState（白名单条目由 whitelistSlice 单独重置）
 *
 * 不依赖认证切片，所有房间操作直接调用 @/utils/api/online-manager 的 IPC 封装。
 * 白名单管理已进一步抽取到 stores/online/whitelistSlice.ts。
 */

import { ref } from 'vue'
import type {
  CreateRoomResponse,
  IceServerEntry,
  JoinRoomResponse,
  ListParticipantsResponse,
  ModpackMeta,
  ParticipantInfo,
  RoomInfoResponse,
  StunServersResponse,
  TurnServersResponse,
} from '@/types/online'
import {
  getStunServers,
  createRoom,
  getRoomInfo,
  closeRoom,
  joinRoom,
  leaveRoom,
  keepaliveRoom,
  listParticipants,
  getTurnServers,
} from '@/utils/api/online-manager'
import { toastSuccess } from '@/utils/toast'
import {
  buildIceServers,
  resolveIceServers,
} from '@/utils/online/webrtc-helpers'
import type { RoomCreateStep, RoomState } from './types'
import { emptyRoom } from './types'

/** 创建联机 store 房间切片 */
export function useOnlineRoomSlice() {
  // ===== 房间状态 =====
  /** 当前房间状态（null 表示未在房间） */
  const roomState = ref<RoomState>(emptyRoom())
  /** 是否正在执行房间操作 */
  const roomLoading = ref(false)
  /** 创建房间当前步骤（UI 进度反馈，null 表示未在创建中） */
  const roomCreateStep = ref<RoomCreateStep>(null)
  /** STUN 服务器列表缓存（房间创建/加入前预取） */
  const stunServers = ref<string[]>([])
  /**
   * 用户自定义 TURN 服务器列表
   *
   * 阶段三子任务 7 新增。由 SettingsOnline UI 配置，通过 `apply_config` 持久化
   * （阶段 I 接入）。房主创建房间时与 STUN 合并为 `iceServers` 上报后端；
   * 房主拉取系统 TURN 后再与此列表合并，通过 DataChannel 广播给参与者。
   * 加入方不能直接配置，仅接收房主广播。
   */
  const customTurnServers = ref<IceServerEntry[]>([])
  /**
   * 系统提供的 TURN 服务器快照（房主独占）
   *
   * 阶段三子任务 7 新增。房主调用 `fetchTurnServers` 后填充；包含负载过滤后的
   * 可用 TURN 列表 + 集群负载快照（用于 UI 调试展示）。
   */
  const systemTurnServers = ref<TurnServersResponse | null>(null)

  /** 拉取 STUN 服务器列表并缓存 */
  async function fetchStunServers(): Promise<string[]> {
    const result = await getStunServers()
    if (result.code !== 1 || !result.data) {
      throw new Error(result.msg || '获取 STUN 服务器列表失败')
    }
    const list = (result.data as StunServersResponse).servers ?? []
    stunServers.value = list
    return list
  }

  /**
   * 房主创建房间
   *
   * @param sdpOffer 房主本地 SDP Offer
   * @param iceCandidates 房主收集的 ICE candidate 数组
   * @param maxPlayers 最大人数（含房主）
   * @param password 房间密码（空字符串表示无密码）
   * @param hostMcVersion 房主 MC 版本
   * @param hostMcPort 房主 MC 端口
   * @param preloadedStun 可选，调用方已预取的 STUN 列表（避免重复获取）
   * @param whitelistEnabled 是否启用白名单（阶段三子任务 8，默认 false）
   * @param whitelist 初始白名单 `device_id` 数组（仅在 `whitelistEnabled=true` 时生效）
   * @param hostLoader 房主加载器类型（联机大厅阶段 1，如 `forge`/`fabric`，默认空字符串=未上报）
   * @param hostLoaderVersion 房主加载器版本号（联机大厅阶段 1，如 `47.3.0`，默认空字符串=未上报）
   * @param roomType 房间类型（联机大厅阶段 2，`private` 仅房间码 / `lobby` 加入大厅，默认 `private`）
   * @param lobbyId 大厅 ID（仅 `roomType='lobby'` 时生效，当前固定 `global`）
   * @param modpack 整合包元数据（联机大厅阶段 3，`undefined` 表示纯原版房间）
   * @returns 创建房间响应
   */
  async function hostCreateRoom(
    sdpOffer: string,
    iceCandidates: string[],
    maxPlayers: number,
    password: string,
    hostMcVersion: string,
    hostMcPort: number,
    preloadedStun?: string[],
    whitelistEnabled: boolean = false,
    whitelist: string[] = [],
    hostLoader: string = '',
    hostLoaderVersion: string = '',
    roomType: 'private' | 'lobby' = 'private',
    lobbyId?: string,
    modpack?: ModpackMeta,
  ): Promise<CreateRoomResponse> {
    roomLoading.value = true
    try {
      const stun = preloadedStun ?? (await fetchStunServers())
      // 阶段三子任务 7：合并 STUN + 用户自定义 TURN 为统一 iceServers
      const iceServers: IceServerEntry[] = buildIceServers({
        stunServers: stun,
        customTurnServers: customTurnServers.value,
      })
      const result = await createRoom({
        sdpOffer,
        iceCandidates,
        maxPlayers,
        password,
        stunServers: stun,
        iceServers,
        hostMcVersion,
        hostMcPort,
        // 联机大厅阶段 1：拆分 MC 版本上报，加载器类型 + 版本号
        // 空字符串视为未上报，转换为 undefined 让后端落库为 NULL（兼容旧客户端）
        hostLoader: hostLoader || undefined,
        hostLoaderVersion: hostLoaderVersion || undefined,
        // 联机大厅阶段 2：房间类型 + 大厅 ID
        // private 时 lobbyId 不传（后端忽略）；lobby 时必传（当前固定 global）
        roomType,
        lobbyId: roomType === 'lobby' ? (lobbyId ?? 'global') : undefined,
        whitelistEnabled,
        whitelist,
        // 联机大厅阶段 3：整合包元数据（undefined=纯原版房间，不上报）
        modpack,
      })
      if (result.code !== 1 || !result.data) {
        throw new Error(result.msg || '创建房间失败')
      }
      const data = result.data
      roomState.value = {
        role: 'host',
        roomCode: data.roomCode,
        hostVirtualIp: data.hostVirtualIp,
        selfVirtualIp: data.hostVirtualIp,
        subnet: data.subnet,
        maxPlayers,
        expiresAt: data.expiresAt,
        stunServers: stun,
        iceServers,
        hostMcVersion,
        hostMcPort,
        participants: [],
        participantId: null,
        whitelistEnabled,
        // 阶段三子任务 8：DataChannel 加密密钥（空字符串表示未启用）
        roomKey: data.roomKey ?? '',
        // 联机大厅阶段 4：房主创建房间时记录关联的整合包元数据
        hostModpack: modpack,
      }
      toastSuccess(`房间已创建：${data.roomCode}`)
      return data
    } finally {
      roomLoading.value = false
      roomCreateStep.value = null
    }
  }

  /** 房主关闭房间 */
  async function hostCloseRoom(): Promise<void> {
    if (roomState.value.role !== 'host' || !roomState.value.roomCode) return
    roomLoading.value = true
    try {
      await closeRoom(roomState.value.roomCode)
      roomState.value = emptyRoom()
      toastSuccess('房间已关闭')
    } finally {
      roomLoading.value = false
    }
  }

  /**
   * 加入方加入房间
   *
   * @param roomCode 房间码
   * @param password 房间密码（空字符串表示无密码）
   * @returns 加入房间响应（含 hostSdpOffer / hostIceCandidates 用于 WebRTC 协商）
   */
  async function guestJoinRoom(roomCode: string, password: string): Promise<JoinRoomResponse> {
    roomLoading.value = true
    try {
      const result = await joinRoom(roomCode, password)
      if (result.code !== 1 || !result.data) {
        throw new Error(result.msg || '加入房间失败')
      }
      const data = result.data
      // 阶段三子任务 7：优先使用 iceServers，旧房间回退到 stunServers
      const iceServers = resolveIceServers(data.iceServers, data.stunServers)
      roomState.value = {
        role: 'guest',
        roomCode,
        hostVirtualIp: '',
        selfVirtualIp: data.playerVirtualIp,
        subnet: data.subnet,
        maxPlayers: 0,
        expiresAt: 0,
        stunServers: data.stunServers ?? [],
        iceServers,
        hostMcVersion: '',
        hostMcPort: 0,
        participants: [],
        participantId: data.participantId,
        // 阶段三子任务 8：加入方初始白名单状态由后续 refreshRoomInfo() 同步
        whitelistEnabled: false,
        // 阶段三子任务 8：DataChannel 加密密钥（与房主一致，空字符串表示未启用）
        roomKey: data.roomKey ?? '',
        // 联机大厅阶段 4：加入方初始无整合包元数据，由后续 refreshRoomInfo() 同步
        hostModpack: undefined,
      }
      // 拉取房间公开信息补全元数据
      await refreshRoomInfo()
      toastSuccess(`已加入房间：${roomCode}`)
      return data
    } finally {
      roomLoading.value = false
    }
  }

  /** 加入方退出房间 */
  async function guestLeaveRoom(): Promise<void> {
    if (roomState.value.role !== 'guest' || !roomState.value.roomCode) return
    roomLoading.value = true
    try {
      await leaveRoom(roomState.value.roomCode)
      roomState.value = emptyRoom()
      toastSuccess('已退出房间')
    } finally {
      roomLoading.value = false
    }
  }

  /** 刷新房间公开信息（房主/加入方通用） */
  async function refreshRoomInfo(): Promise<RoomInfoResponse | null> {
    if (!roomState.value.roomCode) return null
    const result = await getRoomInfo(roomState.value.roomCode)
    if (result.code !== 1 || !result.data) return null
    const info = result.data
    roomState.value.maxPlayers = info.maxPlayers
    roomState.value.expiresAt = info.expiresAt
    roomState.value.hostMcVersion = info.hostMcVersion
    roomState.value.hostMcPort = info.hostMcPort
    // 阶段三子任务 8：同步白名单启用状态（房主/加入方均可见）
    roomState.value.whitelistEnabled = info.whitelistEnabled ?? false
    // 联机大厅阶段 4：同步房主整合包元数据（undefined=纯原版房间）
    roomState.value.hostModpack = info.modpack
    if (roomState.value.role === 'guest') {
      // 加入方需要 ICE 服务器与房主一致（优先 iceServers，回退 stunServers）
      roomState.value.stunServers = info.stunServers ?? roomState.value.stunServers
      const resolved = resolveIceServers(info.iceServers, info.stunServers)
      if (resolved.length > 0) {
        roomState.value.iceServers = resolved
      }
    }
    return info
  }

  /** 房主拉取参与者列表 */
  async function refreshParticipants(): Promise<ParticipantInfo[]> {
    if (roomState.value.role !== 'host' || !roomState.value.roomCode) return []
    const result = await listParticipants(roomState.value.roomCode)
    if (result.code !== 1 || !result.data) return []
    const list = (result.data as ListParticipantsResponse).participants ?? []
    roomState.value.participants = list
    return list
  }

  /** 房主保活 */
  async function keepalive(): Promise<{ expiresAt: number; serverTime: number } | null> {
    if (roomState.value.role !== 'host' || !roomState.value.roomCode) return null
    const result = await keepaliveRoom(roomState.value.roomCode)
    if (result.code !== 1 || !result.data) return null
    roomState.value.expiresAt = result.data.expiresAt
    return result.data
  }

  /**
   * 房主拉取系统提供的 TURN 服务器列表（房主独占接口）
   *
   * 阶段三子任务 7 新增。服务端基于全局开关、单机负载、集群总负载三层过滤后
   * 下发可用 TURN 服务器。结果缓存到 `systemTurnServers`，供房主在 UI 展示
   * 负载状况，并用于与 `customTurnServers` 合并后通过 DataChannel 广播给参与者。
   *
   * @returns TURN 服务器响应（含负载快照）；失败时返回 null
   */
  async function fetchTurnServers(): Promise<TurnServersResponse | null> {
    if (roomState.value.role !== 'host' || !roomState.value.roomCode) return null
    const result = await getTurnServers(roomState.value.roomCode)
    if (result.code !== 1 || !result.data) return null
    systemTurnServers.value = result.data
    return result.data
  }

  /**
   * 设置用户自定义 TURN 服务器列表
   *
   * 阶段三子任务 7 新增。由 SettingsOnline UI 调用，配置持久化由调用方负责
   * （阶段 I 接入 `apply_config`）。房主创建房间时会自动与 STUN 合并为 `iceServers`。
   *
   * @param servers TURN 服务器条目数组
   */
  function setCustomTurnServers(servers: IceServerEntry[]): void {
    customTurnServers.value = servers
  }

  // ============================================================
  // 房主白名单管理已抽取到 stores/online/whitelistSlice.ts
  // ============================================================

  /** 重置房间状态（不调用后端，仅清空本地 roomState） */
  function resetRoomState(): void {
    roomState.value = emptyRoom()
  }

  return {
    // 房间状态
    roomState,
    roomLoading,
    roomCreateStep,
    stunServers,
    // TURN 相关状态（阶段三子任务 7）
    customTurnServers,
    systemTurnServers,
    // 房间方法
    fetchStunServers,
    hostCreateRoom,
    hostCloseRoom,
    guestJoinRoom,
    guestLeaveRoom,
    refreshRoomInfo,
    refreshParticipants,
    keepalive,
    fetchTurnServers,
    setCustomTurnServers,
    resetRoomState,
  }
}
