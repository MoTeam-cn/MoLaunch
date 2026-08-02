/**
 * Frp 管理 Pinia store
 *
 * 管理厂商列表、隧道列表、frpc 二进制状态。
 * 与 stores/online.ts 风格一致：state 用 ref，actions 直接调用 IPC API。
 *
 * 按职责拆分为独立切片（Pinia setup store 的 composable 切片模式），主文件仅组合：
 * - frp/providerSlice.ts（useFrpProviderSlice）：厂商列表 / frpc / 安装卸载启禁
 * - frp/tunnelSlice.ts（useFrpTunnelSlice）：隧道列表 / 状态同步 / 增删改启停
 * - frp/logsSlice.ts（useFrpLogsSlice）：日志行 / 日志文件 / 读取清空
 * - frp/authSlice.ts（useFrpAuthSlice）：认证状态与动作（依赖 providerSlice.providers）
 *
 * 主文件通过对象展开合并各切片，保持 useFrpStore() 对调用方完全兼容。
 */

import { defineStore } from 'pinia'
import { useFrpProviderSlice } from './frp/providerSlice'
import { useFrpAuthSlice } from './frp/authSlice'
import { useFrpTunnelSlice } from './frp/tunnelSlice'
import { useFrpLogsSlice } from './frp/logsSlice'

export const useFrpStore = defineStore('frp', () => {
  const providerSlice = useFrpProviderSlice()
  // 认证切片依赖厂商列表引用（loadAuthStatuses 迭代判断 authType !== 'none'）
  const authSlice = useFrpAuthSlice(providerSlice.providers)
  const tunnelSlice = useFrpTunnelSlice()
  const logsSlice = useFrpLogsSlice()

  return {
    ...providerSlice,
    ...authSlice,
    ...tunnelSlice,
    ...logsSlice,
  }
})
