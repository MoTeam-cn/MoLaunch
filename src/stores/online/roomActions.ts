/**
 * 联机 store 房间动作切片（Scaffolding 方案）
 *
 * 房主：本地生成完整码 → room_create 登记 → 拉起联机中心/easytier；
 *      关闭：room_close + 停 easytier（由 useRoomHost 编排）。
 * 房客：room_join 拿完整码 → scaffolding_client_probe 探测进服地址（由 onlineSession 编排）。
 * 说明：easytier/scaffolding 的拉起与停止由 composable 负责，此处只管房间状态。
 */
import type { Ref } from 'vue'
import type { CreateRoomResponse, JoinRoomResponse, ModpackMeta, RoomInfoResponse } from '@/types/online'
import { parseScaffoldingCode } from '@/types/online'
import { closeRoom, createRoom, getRoomInfo, joinRoom } from '@/utils/api/online-manager'
import { toastSuccess } from '@/utils/toast'
import type { RoomCreateStep, RoomState } from './types'
import { emptyRoom } from './types'

/** 房间动作切片依赖（由 useRoomStateSlice 提供） */
export interface RoomActionDeps {
  roomState: Ref<RoomState>
  roomLoading: Ref<boolean>
  roomCreateStep: Ref<RoomCreateStep>
}

/** 房间已关闭错误（旧 keepalive 轮询兼容，已无定时器使用） */
export class RoomClosedError extends Error {}

/** 创建房间动作切片 */
export function useRoomActionsSlice(deps: RoomActionDeps) {
  const { roomState, roomLoading, roomCreateStep } = deps

  function applyRoomState(data: Partial<RoomState>): void {
    roomState.value = { ...emptyRoom(), ...data }
  }

  /** 房主创建房间：登记完整码并填充房主侧房间状态 */
  async function hostCreateRoom(params: {
    roomCode: string
    remark: string
    isPublic: boolean
    password: string
    hostMcVersion: string
    hostMcPort: number
    hostLoader?: string
    hostLoaderVersion?: string
    modpack?: ModpackMeta
  }): Promise<CreateRoomResponse> {
    roomLoading.value = true
    roomCreateStep.value = 'create'
    try {
      const result = await createRoom(params)
      if (result.code !== 1 || !result.data) throw new Error(result.msg || '创建房间失败')
      const parsed = parseScaffoldingCode(params.roomCode)
      applyRoomState({
        role: 'host',
        roomCode: params.roomCode,
        publicIdentifier: parsed?.publicId ?? '',
        remark: params.remark,
        isPublic: params.isPublic,
        hasPassword: Boolean(params.password),
        hostMcVersion: params.hostMcVersion,
        hostMcPort: params.hostMcPort,
        hostLoader: params.hostLoader ?? '',
        hostLoaderVersion: params.hostLoaderVersion ?? '',
        hostModpack: params.modpack,
      })
      toastSuccess(`房间已创建：${parsed?.publicId ?? params.roomCode}`)
      return result.data
    } finally {
      roomLoading.value = false
      roomCreateStep.value = null
    }
  }

  /** 房主关闭房间（room_close + 清空本地状态） */
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

  /** 加入方凭房间码/密码进房（join 闸门返回完整码） */
  async function guestJoinRoom(roomCode: string, password: string): Promise<JoinRoomResponse> {
    roomLoading.value = true
    try {
      const result = await joinRoom(roomCode, password)
      if (result.code !== 1 || !result.data) throw new Error(result.msg || '加入房间失败')
      const data = result.data
      const parsed = parseScaffoldingCode(data.roomCode)
      applyRoomState({
        role: 'guest',
        roomCode: data.roomCode,
        publicIdentifier: parsed?.publicId ?? data.publicIdentifier,
        remark: data.remark,
        hasPassword: data.hasPassword,
        hostMcVersion: data.hostMcVersion,
        hostMcPort: data.hostMcPort,
        hostLoader: data.hostLoader ?? '',
        hostLoaderVersion: data.hostLoaderVersion ?? '',
        hostModpack: data.modpack,
      })
      toastSuccess(`已加入房间：${parsed?.publicId ?? roomCode}`)
      return data
    } finally {
      roomLoading.value = false
    }
  }

  /** 刷新房间公开信息（房主/房客共用） */
  async function refreshRoomInfo(): Promise<RoomInfoResponse | null> {
    const roomCode = roomState.value.roomCode
    if (!roomCode) return null
    const result = await getRoomInfo(roomCode)
    if (result.code !== 1 || !result.data) return null
    const data = result.data
    roomState.value = {
      ...roomState.value,
      publicIdentifier: data.publicIdentifier || roomState.value.publicIdentifier,
      remark: data.remark || roomState.value.remark,
      hasPassword: data.hasPassword,
      expiresAt: data.expiresAt,
      hostMcVersion: data.hostMcVersion || roomState.value.hostMcVersion,
      hostMcPort: data.hostMcPort || roomState.value.hostMcPort,
      hostLoader: data.hostLoader ?? roomState.value.hostLoader,
      hostLoaderVersion: data.hostLoaderVersion ?? roomState.value.hostLoaderVersion,
      hostModpack: data.modpack ?? roomState.value.hostModpack,
    }
    return data
  }

  return { hostCreateRoom, hostCloseRoom, guestJoinRoom, refreshRoomInfo }
}
