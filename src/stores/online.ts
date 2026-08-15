/**
 * 联机功能状态管理（聚合入口）
 *
 * 管理设备认证状态 + api-server 地址 + 房间状态（roomState / roomAction 薄封装）
 * + easytier 运行时状态。按职责拆分到 ./online/ 子文件
 * （types/authSlice/roomSlice/easytierSlice），本文件仅 re-export + 组合切片。
 */

import { defineStore } from 'pinia'

// re-export 类型定义，保持调用方路径兼容
export * from './online/types'
export type { EasyTierRuntime } from './online/easytierSlice'

// 切片
import { useOnlineAuthSlice } from './online/authSlice'
import { useOnlineRoomSlice } from './online/roomSlice'
import { useOnlineEasyTierSlice } from './online/easytierSlice'

export const useOnlineStore = defineStore('online', () => {
  // 组合各切片
  const authSlice = useOnlineAuthSlice()
  const roomSlice = useOnlineRoomSlice()
  const easytierSlice = useOnlineEasyTierSlice()

  /**
   * 重置房间状态（不调用后端，仅清空本地）
   */
  function resetRoomState(): void {
    roomSlice.resetRoomState()
  }

  return {
    // 设备认证状态
    ...authSlice,
    // 房间状态
    ...roomSlice,
    // easytier 运行时
    ...easytierSlice,
    // 顶层方法
    resetRoomState,
  }
})
