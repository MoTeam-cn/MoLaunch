/**
 * 房间 IPC 封装（Scaffolding 收敛版，仅保留 create/get/join/close）
 */
import type {
  BusinessResult,
  CreateRoomParams,
  CreateRoomResponse,
  JoinRoomResponse,
  RoomInfoResponse,
} from '@/types/online'
import { onlineManager, ONLINE_ACTIONS } from './core'

/** 创建房间（房主本地生成完整 Scaffolding 码后登记） */
export function createRoom(params: CreateRoomParams): Promise<BusinessResult<CreateRoomResponse>> {
  return onlineManager<BusinessResult<CreateRoomResponse>>(ONLINE_ACTIONS.ROOM_CREATE, params)
}

/** 查询房间公开信息（roomCode 支持完整码或 N 段公开标识） */
export function getRoomInfo(roomCode: string): Promise<BusinessResult<RoomInfoResponse>> {
  return onlineManager<BusinessResult<RoomInfoResponse>>(ONLINE_ACTIONS.ROOM_GET, { roomCode })
}

/** 加入房间（join 闸门通过后返回完整码，供解析组网） */
export function joinRoom(roomCode: string, password: string): Promise<BusinessResult<JoinRoomResponse>> {
  return onlineManager<BusinessResult<JoinRoomResponse>>(ONLINE_ACTIONS.ROOM_JOIN, {
    roomCode,
    password,
  })
}

/** 关闭房间 */
export function closeRoom(roomCode: string): Promise<BusinessResult<unknown>> {
  return onlineManager<BusinessResult<unknown>>(ONLINE_ACTIONS.ROOM_CLOSE, { roomCode })
}

/** 房主心跳上报（每 3 分钟一次，防止房间被云端超时清理） */
export function heartbeatRoom(roomCode: string): Promise<BusinessResult<unknown>> {
  return onlineManager<BusinessResult<unknown>>(ONLINE_ACTIONS.ROOM_HEARTBEAT, { roomCode })
}
