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
import { RoomClosedError } from './online/roomActions'
import { toastError } from '@/utils/toast'

/** 全局保活定时器：房主每 30s 上报一次，避免切页/组件卸载导致漏报被判失联 */
const GLOBAL_KEEPALIVE_INTERVAL = 30_000

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

  // ============================================================
  // 全局保活定时器（脱离页面生命周期）
  // ============================================================
  // 背景：keepalive 原本绑定在 RoomHostPanel 的 useRoomHost 定时器上，
  // 从联机页切到其他页面（Online.vue 卸载）会 stopTimers()，超过服务端
  // keepalive_timeout(120s) 后房间被判失联销毁。此定时器在 store 层运行，
  // 不依赖任何组件，房主在任意页面时都会持续上报。
  //
  // 注意：不使用 onUnmounted 清理——Pinia store 是全局单例，生命周期与
  // 应用一致；且 keepalive() 内部有 role='host' 守卫，非房主时直接返回
  // 空请求，不产生网络开销。
  const keepaliveTimer = globalThis.setInterval(() => {
    void (async () => {
      if (roomSlice.roomState.value.role !== 'host') return
      try {
        await roomSlice.keepalive()
      } catch (e) {
        if (e instanceof RoomClosedError) {
          resetRoomState()
          toastError(`房间已关闭：${e.message}`)
        }
        // 其他保活失败静默（网络抖动常见），下一周期重试
      }
    })()
  }, GLOBAL_KEEPALIVE_INTERVAL)

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
    // 供测试/调试暴露的定时器句柄
    keepaliveTimer,
  }
})
