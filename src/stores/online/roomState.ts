/**
 * 联机 store 房间切片 - 状态定义
 *
 * 持有房间相关全部响应式状态（roomState / loading / 步骤 / ICE 服务器缓存），
 * 以及不依赖后端的纯状态操作（resetRoomState / setCustomTurnServers）。
 * 房间动作见 roomActions.ts，由 roomSlice.ts 组合对外。
 */
import { ref } from 'vue'
import type { IceServerEntry, TurnServersResponse } from '@/types/online'
import type { RoomCreateStep, RoomState } from './types'
import { emptyRoom } from './types'

export function useRoomStateSlice() {
  /** 当前房间状态（null 表示未在房间） */
  const roomState = ref<RoomState>(emptyRoom())
  /** 是否正在执行房间操作 */
  const roomLoading = ref(false)
  /** 创建房间当前步骤（UI 进度反馈，null 表示未在创建中） */
  const roomCreateStep = ref<RoomCreateStep>(null)
  /** STUN 服务器列表缓存（房间创建/加入前预取） */
  const stunServers = ref<string[]>([])
  /** 用户自定义 TURN 服务器列表（SettingsOnline UI 配置，`apply_config` 持久化） */
  const customTurnServers = ref<IceServerEntry[]>([])
  /** 系统提供的 TURN 服务器快照（房主独占，fetchTurnServers 填充） */
  const systemTurnServers = ref<TurnServersResponse | null>(null)

  /** 设置用户自定义 TURN 服务器列表（持久化由调用方负责） */
  function setCustomTurnServers(servers: IceServerEntry[]): void {
    customTurnServers.value = servers
  }

  /** 重置房间状态（不调用后端，仅清空本地 roomState） */
  function resetRoomState(): void {
    roomState.value = emptyRoom()
  }

  return {
    roomState,
    roomLoading,
    roomCreateStep,
    stunServers,
    customTurnServers,
    systemTurnServers,
    setCustomTurnServers,
    resetRoomState,
  }
}
