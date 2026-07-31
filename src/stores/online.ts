/**
 * 联机功能状态管理（聚合入口）
 *
 * 管理设备认证状态 + api-server 地址 + 房间状态（阶段二）。
 *
 * 设计：
 * - `deviceStatus`：本地缓存的上次查询到的设备状态（null 表示未查询）
 * - `apiServerUrl`：从后端配置同步的 api-server 地址（与 settings store 解耦，避免循环依赖）
 * - `refreshStatus()`：拉取最新设备状态（不发网络请求，仅读本地凭证 + 后端配置）
 * - 所有写操作（register/login/logout/clear）成功后自动 refreshStatus()
 *
 * 阶段二扩展房间状态：
 * - `roomState`：当前房间状态（角色 + 房间码 + 参与者列表）
 * - `roomAction(action, payload)`：统一调度信令 action 的薄封装
 *   （WebRTC 协商、keepalive 定时器等业务逻辑放在 RoomManager.vue 中，
 *    store 只持久化房间元信息，避免与组件生命周期耦合）
 *
 * 按职责拆分到 `./online/` 子文件，本文件仅做 re-export + 切片组合以保持
 * `@/stores/online` 路径对调用方完全兼容：
 * - `types.ts`：RoomRole / RoomCreateStep / RoomState / emptyRoom
 * - `authSlice.ts`：设备认证 state + actions
 * - `roomSlice.ts`：房间 state + actions（不含白名单）
 * - `whitelistSlice.ts`：白名单 state + actions
 * - `natSlice.ts`：NAT 检测 state + actions
 */

import { defineStore } from 'pinia'

// re-export 类型定义，保持调用方路径兼容
export * from './online/types'

// 切片
import { useOnlineAuthSlice } from './online/authSlice'
import { useOnlineRoomSlice } from './online/roomSlice'
import { useOnlineWhitelistSlice } from './online/whitelistSlice'
import { useOnlineNatSlice } from './online/natSlice'

export const useOnlineStore = defineStore('online', () => {
  // 组合各切片
  const authSlice = useOnlineAuthSlice()
  const roomSlice = useOnlineRoomSlice()
  const whitelistSlice = useOnlineWhitelistSlice(roomSlice.roomState)
  const natSlice = useOnlineNatSlice()

  /**
   * 重置房间状态（不调用后端，仅清空本地）
   *
   * 同时重置 roomState 与白名单条目，保持与拆分前行为一致。
   */
  function resetRoomState(): void {
    roomSlice.resetRoomState()
    whitelistSlice.resetWhitelistEntries()
  }

  return {
    // 设备认证状态
    ...authSlice,
    // 房间状态
    ...roomSlice,
    // 白名单状态
    ...whitelistSlice,
    // NAT 检测
    ...natSlice,
    // 顶层方法
    resetRoomState,
  }
})
