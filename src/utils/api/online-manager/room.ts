/**
 * 联机 API - 房间信令便捷封装（阶段二）
 *
 * 所有信令 action 返回 `BusinessResult<T>`（含 code/data/msg/time/req_id），
 * 调用方需检查 `code === 1` 后取 `data` 字段使用。
 */

import type {
  BusinessResult,
  CreateRoomParams,
  CreateRoomResponse,
  JoinRoomResponse,
  KeepaliveResponse,
  ListAnswersResponse,
  ListBansResponse,
  ListParticipantsResponse,
  RoomInfoResponse,
  StunServersResponse,
} from '@/types/online'
import { ONLINE_ACTIONS, onlineManager } from './core'

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
