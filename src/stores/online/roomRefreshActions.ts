import type { Ref } from 'vue'
import type { RoomInfoResponse, TurnServersResponse } from '@/types/online'
import { getRoomInfo, keepaliveRoom, getTurnServers } from '@/utils/api/online-manager'
import { resolveIceServers } from '@/utils/online/webrtc-helpers'
import type { RoomState } from './types'

export class RoomClosedError extends Error {
  constructor(message: string) {
    super(message)
    this.name = 'RoomClosedError'
  }
}

export interface RoomRefreshDeps {
  roomState: Ref<RoomState>
  systemTurnServers: Ref<TurnServersResponse | null>
}

export function useRoomRefreshActions(deps: RoomRefreshDeps) {
  const { roomState, systemTurnServers } = deps

  async function refreshRoomInfo(): Promise<RoomInfoResponse | null> {
    if (!roomState.value.roomCode) return null
    const result = await getRoomInfo(roomState.value.roomCode)
    if (result.code !== 1 || !result.data) return null
    const info = result.data
    roomState.value.maxPlayers = info.maxPlayers
    roomState.value.expiresAt = info.expiresAt
    roomState.value.hostMcVersion = info.hostMcVersion
    // hostMcPort 以房主实时检测为准（GameWatcher 捕获后经 HOST_MC_PORT 控制消息下发），
    // 服务端元数据为创建时初始值，不做覆盖，避免拉取竞态覆盖掉实时广播的端口
    roomState.value.whitelistEnabled = info.whitelistEnabled ?? false
    roomState.value.hostModpack = info.modpack
    if (roomState.value.role === 'guest') {
      roomState.value.stunServers = info.stunServers ?? roomState.value.stunServers
      const resolved = resolveIceServers(info.iceServers, info.stunServers)
      if (resolved.length > 0) roomState.value.iceServers = resolved
    }
    return info
  }

  async function keepalive(): Promise<{ expiresAt: number; serverTime: number } | null> {
    if (roomState.value.role !== 'host' || !roomState.value.roomCode) return null
    const result = await keepaliveRoom(roomState.value.roomCode)
    if (result.code !== 1 || !result.data) {
      if (result.code === 1001) throw new RoomClosedError(result.msg || '房间已关闭')
      throw new Error(result.msg || '保活失败')
    }
    roomState.value.expiresAt = result.data.expiresAt
    return result.data
  }

  async function fetchTurnServers(): Promise<TurnServersResponse | null> {
    if (roomState.value.role !== 'host' || !roomState.value.roomCode) return null
    const result = await getTurnServers(roomState.value.roomCode)
    if (result.code !== 1 || !result.data) return null
    systemTurnServers.value = result.data
    return result.data
  }

  return { refreshRoomInfo, keepalive, fetchTurnServers }
}
