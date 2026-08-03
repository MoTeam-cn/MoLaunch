/**
 * 联机 store 房间切片 - 动作实现
 *
 * 房间创建/关闭、加入/退出、信息刷新、参与者列表、保活、TURN 拉取。
 * 状态由 roomState.ts 注入，本切片只读取并按约定更新这些 ref。
 */
import type { Ref } from 'vue'
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

export interface RoomActionDeps {
  roomState: Ref<RoomState>
  roomLoading: Ref<boolean>
  roomCreateStep: Ref<RoomCreateStep>
  stunServers: Ref<string[]>
  customTurnServers: Ref<IceServerEntry[]>
  systemTurnServers: Ref<TurnServersResponse | null>
}

export function useRoomActionsSlice(deps: RoomActionDeps) {
  const { roomState, roomLoading, roomCreateStep, stunServers, customTurnServers, systemTurnServers } = deps

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
   * @param preloadedStun 可选，调用方已预取的 STUN 列表
   * @param whitelistEnabled 是否启用白名单
   * @param whitelist 初始白名单 device_id 数组
   * @param hostLoader 房主加载器类型
   * @param hostLoaderVersion 房主加载器版本号
   * @param roomType 房间类型（`private` / `lobby`）
   * @param lobbyId 大厅 ID（仅 lobby 生效）
   * @param modpack 整合包元数据（undefined=纯原版）
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
        // 联机大厅阶段 1：空字符串视为未上报，转 undefined 让后端落库为 NULL
        hostLoader: hostLoader || undefined,
        hostLoaderVersion: hostLoaderVersion || undefined,
        // 联机大厅阶段 2：private 时不传 lobbyId；lobby 时必传（固定 global）
        roomType,
        lobbyId: roomType === 'lobby' ? (lobbyId ?? 'global') : undefined,
        whitelistEnabled,
        whitelist,
        // 联机大厅阶段 3：整合包元数据（undefined=纯原版）
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
        // 阶段三子任务 8：白名单状态由后续 refreshRoomInfo() 同步
        whitelistEnabled: false,
        // 阶段三子任务 8：DataChannel 加密密钥（与房主一致）
        roomKey: data.roomKey ?? '',
        // 联机大厅阶段 4：加入方初始无整合包元数据，由 refreshRoomInfo() 同步
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
   * 结果缓存到 `systemTurnServers`，供房主 UI 展示并广播给参与者。
   * @returns TURN 服务器响应（含负载快照）；失败时返回 null
   */
  async function fetchTurnServers(): Promise<TurnServersResponse | null> {
    if (roomState.value.role !== 'host' || !roomState.value.roomCode) return null
    const result = await getTurnServers(roomState.value.roomCode)
    if (result.code !== 1 || !result.data) return null
    systemTurnServers.value = result.data
    return result.data
  }

  return {
    fetchStunServers,
    hostCreateRoom,
    hostCloseRoom,
    guestJoinRoom,
    guestLeaveRoom,
    refreshRoomInfo,
    refreshParticipants,
    keepalive,
    fetchTurnServers,
  }
}