/**
 * 联机功能状态管理（聚合入口）
 *
 * 管理设备认证状态 + api-server 地址 + 房间状态（roomState / roomAction 薄封装）。
 * 按职责拆分到 ./online/ 子文件（types/authSlice/roomSlice/whitelistSlice/natSlice），
 * 本文件仅 re-export + 组合切片，保持 @/stores/online 路径对调用方兼容。
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
