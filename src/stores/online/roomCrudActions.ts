import type {
  CreateRoomResponse,
  IceServerEntry,
  JoinRoomResponse,
  ModpackMeta,
  RoomInfoResponse,
  StunServersResponse,
} from '@/types/online'
import {
  closeRoom,
  createRoom,
  getStunServers,
  joinRoom,
  leaveRoom,
} from '@/utils/api/online-manager'
import { toastSuccess } from '@/utils/toast'
import { buildIceServers, resolveIceServers } from '@/utils/online/webrtc-helpers'
import { emptyRoom } from './types'
import type { RoomActionDeps } from './roomActionTypes'

export function useRoomCrudActions(
  deps: RoomActionDeps,
  refreshRoomInfo: () => Promise<RoomInfoResponse | null>,
) {
  const { roomState, roomLoading, roomCreateStep, stunServers, customTurnServers } = deps

  async function fetchStunServers(): Promise<string[]> {
    const result = await getStunServers()
    if (result.code !== 1 || !result.data) throw new Error(result.msg || '获取 STUN 服务器列表失败')
    const list = (result.data as StunServersResponse).servers ?? []
    stunServers.value = list
    return list
  }

  async function hostCreateRoom(
    sdpOffer: string,
    iceCandidates: string[],
    maxPlayers: number,
    password: string,
    hostMcVersion: string,
    hostMcPort: number,
    preloadedStun?: string[],
    whitelistEnabled = false,
    whitelist: string[] = [],
    hostLoader = '',
    hostLoaderVersion = '',
    roomType: 'private' | 'lobby' = 'private',
    lobbyId?: string,
    modpack?: ModpackMeta,
  ): Promise<CreateRoomResponse> {
    roomLoading.value = true
    try {
      const stun = preloadedStun ?? (await fetchStunServers())
      const iceServers: IceServerEntry[] = buildIceServers({
        stunServers: stun,
        customTurnServers: customTurnServers.value,
      })
      // ice_servers 仅存 TURN，STUN 由 stun_servers 列承载（读取侧统一回退）
      const turnOnlyIceServers = [...customTurnServers.value]
      const result = await createRoom({
        sdpOffer,
        iceCandidates,
        maxPlayers,
        password,
        stunServers: stun,
        iceServers: turnOnlyIceServers,
        hostMcVersion,
        hostMcPort,
        hostLoader: hostLoader || undefined,
        hostLoaderVersion: hostLoaderVersion || undefined,
        roomType,
        lobbyId: roomType === 'lobby' ? (lobbyId ?? 'global') : undefined,
        whitelistEnabled,
        whitelist,
        modpack,
      })
      if (result.code !== 1 || !result.data) throw new Error(result.msg || '创建房间失败')
      const data = result.data
      roomState.value = {
        role: 'host', roomCode: data.roomCode, hostVirtualIp: data.hostVirtualIp,
        selfVirtualIp: data.hostVirtualIp, subnet: data.subnet, maxPlayers,
        expiresAt: data.expiresAt, stunServers: stun, iceServers, hostMcVersion,
        hostMcPort, hostMcPortManual: false, participants: [], participantId: null,
        whitelistEnabled, roomKey: data.roomKey ?? '', hostModpack: modpack,
      }
      toastSuccess(`房间已创建：${data.roomCode}`)
      return data
    } finally {
      roomLoading.value = false
      roomCreateStep.value = null
    }
  }

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

  async function guestJoinRoom(roomCode: string, password: string): Promise<JoinRoomResponse> {
    roomLoading.value = true
    try {
      const result = await joinRoom(roomCode, password)
      if (result.code !== 1 || !result.data) throw new Error(result.msg || '加入房间失败')
      const data = result.data
      roomState.value = {
        role: 'guest', roomCode, hostVirtualIp: '', selfVirtualIp: data.playerVirtualIp,
        subnet: data.subnet, maxPlayers: 0, expiresAt: 0,
        stunServers: data.stunServers ?? [], iceServers: resolveIceServers(data.iceServers, data.stunServers),
        hostMcVersion: '', hostMcPort: 0, hostMcPortManual: false, participants: [],
        participantId: data.participantId, whitelistEnabled: false, roomKey: data.roomKey ?? '',
        hostModpack: undefined,
      }
      await refreshRoomInfo()
      toastSuccess(`已加入房间：${roomCode}`)
      return data
    } finally {
      roomLoading.value = false
    }
  }

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

  return { fetchStunServers, hostCreateRoom, hostCloseRoom, guestJoinRoom, guestLeaveRoom }
}
