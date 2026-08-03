/**
 * Frp store 认证切片（阶段三）
 *
 * 接收主 store 的 providers 引用作为依赖（仅 loadAuthStatuses 迭代判断 authType !== 'none'），
 * 其余认证 action 自持并直接调用 frp-manager IPC 封装，由主 store 解构合并。
 */

import type { Ref } from 'vue'
import { ref } from 'vue'
import type { AuthStatus, DeviceCodeResult, ProviderInfo } from '@/types/frp'
import {
  getAuthStatus as apiGetAuthStatus,
  startOAuth2 as apiStartOAuth2,
  startDeviceCode as apiStartDeviceCode,
  pollDeviceCode as apiPollDeviceCode,
  refreshToken as apiRefreshToken,
  revokeAuth as apiRevokeAuth,
  saveApiKey as apiSaveApiKey,
} from '@/utils/api/frp-manager'
import { toastSuccess, toastError } from '@/utils/toast'

/**
 * 创建 Frp 认证切片
 *
 * @param providers 厂商列表引用（loadAuthStatuses 迭代使用）
 */
export function useFrpAuthSlice(providers: Ref<ProviderInfo[]>) {
  // ============================================================
  // 认证状态
  // ============================================================

  /** 各厂商认证状态（key = providerId） */
  const authStatuses = ref<Record<string, AuthStatus>>({})
  /** 认证状态批量加载中 */
  const authLoading = ref(false)
  /** 单个厂商认证操作中（OAuth2/刷新/撤销等） */
  const authActionLoading = ref<Record<string, boolean>>({})
  /** Device Code 流程信息（key = providerId） */
  const deviceCodeInfos = ref<Record<string, DeviceCodeResult>>({})
  /** Device Code 轮询中（key = providerId） */
  const deviceCodePolling = ref<Record<string, boolean>>({})
  /** API Key 输入值（key = providerId） */
  const apiKeyInputs = ref<Record<string, string>>({})

  // ============================================================
  // 内部 helper
  // ============================================================

  /** 设置单个厂商的认证操作 loading */
  function setAuthActionLoading(providerId: string, loading: boolean): void {
    authActionLoading.value[providerId] = loading
  }

  /** 刷新单个厂商的认证状态 */
  async function refreshAuthStatus(providerId: string): Promise<void> {
    try {
      const status = await apiGetAuthStatus(providerId)
      authStatuses.value[providerId] = status
    } catch {
      // 静默失败
    }
  }

  // ============================================================
  // 认证 Actions
  // ============================================================

  /** 批量加载所有需认证厂商的认证状态 */
  async function loadAuthStatuses(): Promise<void> {
    authLoading.value = true
    try {
      for (const p of providers.value) {
        if (p.authType === 'none') continue
        try {
          const status = await apiGetAuthStatus(p.id)
          authStatuses.value[p.id] = status
        } catch {
          // 单个厂商失败不影响其他厂商
        }
      }
    } finally {
      authLoading.value = false
    }
  }

  /** 启动 OAuth2 授权流程 */
  async function startOAuth2Auth(providerId: string): Promise<boolean> {
    setAuthActionLoading(providerId, true)
    try {
      await apiStartOAuth2(providerId)
      toastSuccess('OAuth2 认证成功')
      await refreshAuthStatus(providerId)
      return true
    } catch (e) {
      toastError('OAuth2 认证失败：' + e)
      return false
    } finally {
      setAuthActionLoading(providerId, false)
    }
  }

  /** 启动 Device Code 流程 */
  async function startDeviceCodeAuth(providerId: string): Promise<boolean> {
    setAuthActionLoading(providerId, true)
    try {
      const info = await apiStartDeviceCode(providerId)
      deviceCodeInfos.value[providerId] = info
      return true
    } catch (e) {
      toastError('启动 Device Code 失败：' + e)
      return false
    } finally {
      setAuthActionLoading(providerId, false)
    }
  }

  /** 轮询 Device Code token（前端按 interval 调用，返回是否应继续轮询） */
  async function pollDeviceCodeAuth(providerId: string): Promise<boolean> {
    deviceCodePolling.value[providerId] = true
    try {
      const result = await apiPollDeviceCode(providerId)
      if (result.status === 'success') {
        toastSuccess('Device Code 认证成功')
        delete deviceCodeInfos.value[providerId]
        await refreshAuthStatus(providerId)
        return false
      }
      if (result.status === 'expired' || result.status === 'declined') {
        const msg = result.status === 'expired' ? '设备码已过期，请重新认证' : '用户拒绝授权'
        toastError(msg)
        delete deviceCodeInfos.value[providerId]
        return false
      }
      // pending / slow_down -> 继续轮询
      return true
    } catch (e) {
      toastError('轮询 Device Code 失败：' + e)
      delete deviceCodeInfos.value[providerId]
      return false
    } finally {
      deviceCodePolling.value[providerId] = false
    }
  }

  /** 取消 Device Code 流程（清除前端状态） */
  function cancelDeviceCode(providerId: string): void {
    delete deviceCodeInfos.value[providerId]
  }

  /** 刷新 token */
  async function refreshTokenAuth(providerId: string): Promise<boolean> {
    setAuthActionLoading(providerId, true)
    try {
      await apiRefreshToken(providerId)
      toastSuccess('token 已刷新')
      await refreshAuthStatus(providerId)
      return true
    } catch (e) {
      toastError('刷新 token 失败：' + e)
      return false
    } finally {
      setAuthActionLoading(providerId, false)
    }
  }

  /** 撤销认证 */
  async function revokeAuthAuth(providerId: string): Promise<boolean> {
    setAuthActionLoading(providerId, true)
    try {
      await apiRevokeAuth(providerId)
      toastSuccess('已撤销认证')
      await refreshAuthStatus(providerId)
      return true
    } catch (e) {
      toastError('撤销认证失败：' + e)
      return false
    } finally {
      setAuthActionLoading(providerId, false)
    }
  }

  /** 保存 API Key */
  async function saveApiKeyAuth(providerId: string, apiKey: string): Promise<boolean> {
    setAuthActionLoading(providerId, true)
    try {
      await apiSaveApiKey({ providerId, apiKey })
      toastSuccess('API Key 已保存')
      await refreshAuthStatus(providerId)
      return true
    } catch (e) {
      toastError('保存 API Key 失败：' + e)
      return false
    } finally {
      setAuthActionLoading(providerId, false)
    }
  }

  return {
    // state
    authStatuses,
    authLoading,
    authActionLoading,
    deviceCodeInfos,
    deviceCodePolling,
    apiKeyInputs,
    // actions
    loadAuthStatuses,
    refreshAuthStatus,
    startOAuth2Auth,
    startDeviceCodeAuth,
    pollDeviceCodeAuth,
    cancelDeviceCode,
    refreshTokenAuth,
    revokeAuthAuth,
    saveApiKeyAuth,
  }
}
