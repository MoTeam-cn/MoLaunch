import type { Ref } from 'vue'
import type { IceServerEntry, RoomInfoResponse, TurnServersResponse } from '@/types/online'
import { getRoomInfo, keepaliveRoom, getTurnServers } from '@/utils/api/online-manager'
import { resolveIceServers, mergeIceServerEntries } from '@/utils/online/webrtc-helpers'
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
      if (resolved.length > 0) {
        // 保留参与者已自拉的系统 TURN（凭据绑定自身 IP/device，regionCode 为标记），
        // 避免刷新覆盖后中继失效；云端 STUN / 自定义 TURN 照常更新
        const ownTurn = roomState.value.iceServers.filter((e) => e.regionCode)
        roomState.value.iceServers = mergeIceServerEntries(ownTurn, resolved)
      }
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

  /**
   * 拉取服务端 TURN 服务器列表（房主或参与者均可调用）
   *
   * 服务端 `list_turn_servers` 对房间内所有成员签发**各自绑定**的凭证
   * （HMAC 含调用者 IP + device），因此参与者必须自拉而非依赖房主广播——
   * 房主广播的凭证对参与者无效（coturn 校验 IP/device 不匹配）。
   */
  async function fetchTurnServers(): Promise<TurnServersResponse | null> {
    if (!roomState.value.roomCode) return null
    const result = await getTurnServers(roomState.value.roomCode)
    if (result.code !== 1 || !result.data) return null
    systemTurnServers.value = result.data
    return result.data
  }

  /**
   * 参与者自拉系统 TURN 并合并进 roomState.iceServers
   *
   * 房主广播的 TURN 凭据绑定房主 IP+device，对参与者无效；参与者必须自拉
   * `/turn` 获取绑定自身 IP+device 的凭据（服务端已允许参与者调用），
   * P2P 打洞失败时浏览器才能成功分配 relay candidate 走中继。
   *
   * @returns 合并后的 ICE 服务器列表（供 fetchOfferAndAnswer 使用）
   */
  async function guestPullTurnServers(): Promise<IceServerEntry[]> {
    if (roomState.value.role !== 'guest' || !roomState.value.roomCode) {
      return roomState.value.iceServers
    }
    // 同一房间仅拉取一次：/turn 签发的凭证绑定自身 IP+device，房间不变可复用，
    // 避免 P2P 失败多次恢复时重复请求与重复求解 PoW
    let resp = systemTurnServers.value
    if (!resp) {
      resp = await fetchTurnServers()
      if (!resp || resp.servers.length === 0) return roomState.value.iceServers
    }
    roomState.value.iceServers = mergeIceServerEntries(
      roomState.value.iceServers,
      resp.servers,
    )
    return roomState.value.iceServers
  }

  return { refreshRoomInfo, keepalive, fetchTurnServers, guestPullTurnServers }
}
