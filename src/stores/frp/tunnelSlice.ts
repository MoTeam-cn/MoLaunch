/**
 * Frp store 隧道切片（阶段三）
 *
 * 监听 frp-tunnel-status 事件，frpc 退出时静默刷新隧道列表并记录 lastTunnelStatus；
 * 增删改启停后统一 loadTunnels 刷新列表。
 */

import { ref } from 'vue'
import type {
  CreateTunnelParams,
  TunnelWithStatus,
  FrpTunnelStatusEvent,
  UpdateTunnelParams,
} from '@/types/frp'
import {
  listTunnels,
  createTunnel as apiCreateTunnel,
  updateTunnel as apiUpdateTunnel,
  deleteTunnel as apiDeleteTunnel,
  startTunnel as apiStartTunnel,
  stopTunnel as apiStopTunnel,
} from '@/utils/api/frp-manager'
import { useTauriEvent } from '@/composables/useTauriEvent'
import { toastSuccess, toastError } from '@/utils/toast'

/** 创建 Frp 隧道切片（无外部依赖） */
export function useFrpTunnelSlice() {
  /** 隧道列表（含运行状态） */
  const tunnels = ref<TunnelWithStatus[]>([])
  /** 隧道列表加载中 */
  const tunnelsLoading = ref(false)
  /** 隧道操作中（创建/删除/启动/停止） */
  const tunnelActionLoading = ref(false)
  /** 正在启动/停止的隧道 ID（为空表示非启停操作，用于按钮按隧道区分加载态） */
  const tunnelActionTunnelId = ref<string | null>(null)
  /** 最近一次隧道状态变更事件（用于 TunnelManager 显示异常退出提示） */
  const lastTunnelStatus = ref<FrpTunnelStatusEvent | null>(null)
  /** 隧道状态事件监听器是否已启动（避免重复注册） */
  const statusListenerStarted = ref(false)

  /** 加载隧道列表 */
  async function loadTunnels(): Promise<void> {
    tunnelsLoading.value = true
    try {
      tunnels.value = await listTunnels()
    } catch (e) {
      toastError('加载隧道列表失败：' + e)
    } finally {
      tunnelsLoading.value = false
    }
  }

  /**
   * 静默刷新隧道列表（不触发 loading，不影响 UI 抖动）
   *
   * 用于 `frp-tunnel-status` 事件回调：frpc 进程退出时后端会发事件，
   * 前端立即刷新列表保持状态同步。若用 loadTunnels 会闪一下 loading。
   */
  async function refreshTunnelsSilent(): Promise<void> {
    try {
      tunnels.value = await listTunnels()
    } catch {
      // 静默失败，不打扰用户
    }
  }

  /**
   * 启动隧道状态事件监听器
   *
   * 监听 `frp-tunnel-status` 事件，frpc 进程退出时：
   * 1. 自动刷新 tunnels 列表（状态同步）
   * 2. 记录 lastTunnelStatus 供 TunnelManager 显示提示
   *
   * 监听器在整个 app 生命周期内只需启动一次，使用 statusListenerStarted 防重复。
   */
  function startTunnelStatusListener(): void {
    if (statusListenerStarted.value) return
    statusListenerStarted.value = true
    const { start } = useTauriEvent<FrpTunnelStatusEvent>('frp-tunnel-status', (e) => {
      lastTunnelStatus.value = e
      // 异常退出（带 error 字段）时弹 toast 提示
      if (e.status === 'stopped' && e.error) {
        toastError(`隧道「${e.tunnelName}」已退出：${e.error}`)
      }
      // 自动刷新列表（静默）
      void refreshTunnelsSilent()
    })
    void start()
  }

  /** 创建隧道 */
  async function createTunnel(params: CreateTunnelParams): Promise<boolean> {
    tunnelActionLoading.value = true
    try {
      await apiCreateTunnel(params)
      toastSuccess('隧道创建成功')
      await loadTunnels()
      return true
    } catch (e) {
      toastError('创建隧道失败：' + e)
      return false
    } finally {
      tunnelActionLoading.value = false
      tunnelActionTunnelId.value = null
    }
  }

  /** 更新隧道配置 */
  async function updateTunnel(params: UpdateTunnelParams): Promise<boolean> {
    tunnelActionLoading.value = true
    try {
      await apiUpdateTunnel(params)
      toastSuccess('隧道配置已更新')
      await loadTunnels()
      return true
    } catch (e) {
      toastError('更新隧道失败：' + e)
      return false
    } finally {
      tunnelActionLoading.value = false
      tunnelActionTunnelId.value = null
    }
  }

  /** 删除隧道 */
  async function deleteTunnel(id: string): Promise<boolean> {
    tunnelActionLoading.value = true
    try {
      await apiDeleteTunnel(id)
      toastSuccess('隧道已删除')
      await loadTunnels()
      return true
    } catch (e) {
      toastError('删除隧道失败：' + e)
      return false
    } finally {
      tunnelActionLoading.value = false
      tunnelActionTunnelId.value = null
    }
  }

  /** 启动隧道 */
  async function startTunnel(id: string): Promise<boolean> {
    tunnelActionLoading.value = true
    tunnelActionTunnelId.value = id
    try {
      await apiStartTunnel(id)
      toastSuccess('隧道已启动')
      await loadTunnels()
      return true
    } catch (e) {
      toastError('启动隧道失败：' + e)
      return false
    } finally {
      tunnelActionLoading.value = false
      tunnelActionTunnelId.value = null
    }
  }

  /** 停止隧道 */
  async function stopTunnel(id: string): Promise<boolean> {
    tunnelActionLoading.value = true
    tunnelActionTunnelId.value = id
    try {
      await apiStopTunnel(id)
      toastSuccess('隧道已停止')
      await loadTunnels()
      return true
    } catch (e) {
      toastError('停止隧道失败：' + e)
      return false
    } finally {
      tunnelActionLoading.value = false
      tunnelActionTunnelId.value = null
    }
  }

  return {
    // state
    tunnels,
    tunnelsLoading,
    tunnelActionLoading,
    tunnelActionTunnelId,
    lastTunnelStatus,
    // actions
    loadTunnels,
    refreshTunnelsSilent,
    startTunnelStatusListener,
    createTunnel,
    updateTunnel,
    deleteTunnel,
    startTunnel,
    stopTunnel,
  }
}
