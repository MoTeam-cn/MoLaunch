/**
 * 联机功能状态管理
 *
 * 管理设备认证状态 + api-server 地址，提供统一的注册/登录/登出/清除接口。
 * 阶段二会扩展房间状态、WebRTC PeerConnection 等。
 *
 * 设计：
 * - `deviceStatus`：本地缓存的上次查询到的设备状态（null 表示未查询）
 * - `apiServerUrl`：从后端配置同步的 api-server 地址（与 settings store 解耦，避免循环依赖）
 * - `refreshStatus()`：拉取最新设备状态（不发网络请求，仅读本地凭证 + 后端配置）
 * - 所有写操作（register/login/logout/clear）成功后自动 refreshStatus()
 */

import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { DeviceStatus } from '@/types/online'
import {
  getAuthStatus,
  registerDevice,
  loginDevice,
  logoutDevice,
  clearDevice,
} from '@/utils/api/online-manager'
import { applyConfig, getConfigMap } from '@/utils/api/config'
import { toastSuccess, toastError } from '@/utils/toast'
import { safeCall } from '@/utils/async'

export const useOnlineStore = defineStore('online', () => {
  /** 设备认证状态（null 表示未查询） */
  const deviceStatus = ref<DeviceStatus | null>(null)
  /** 是否正在执行写操作（注册/登录/登出/清除） */
  const loading = ref(false)
  /** 是否正在拉取状态 */
  const refreshing = ref(false)
  /** 当前 api-server 地址（从后端配置同步） */
  const apiServerUrl = ref('')

  /**
   * 拉取最新设备状态（不发起网络请求，仅读本地凭证 + 后端配置）
   *
   * 同时同步 apiServerUrl（用于 SettingsOnline 页显示）。
   */
  async function refreshStatus(): Promise<void> {
    refreshing.value = true
    await safeCall(async () => {
      const status = await getAuthStatus()
      deviceStatus.value = status
      apiServerUrl.value = status.api_server_url
    }, '[Online] refresh status')
    refreshing.value = false
  }

  /**
   * 更新 api-server 地址（写入后端 INI，不立即触发设备状态刷新）
   *
   * 后端 `apply_online` 会忽略空字符串，避免误清空。
   * @returns 是否保存成功
   */
  async function setApiServerUrl(url: string): Promise<boolean> {
    const trimmed = url.trim()
    if (!trimmed) {
      toastError('api-server 地址不能为空')
      return false
    }
    const ok = await safeCall(
      async () => {
        await applyConfig({ onlineApiServerUrl: trimmed })
        apiServerUrl.value = trimmed
      },
      '[Online] set api server url',
    )
    if (ok !== undefined) {
      toastSuccess('api-server 地址已保存')
      return true
    }
    return false
  }

  /** 从后端配置同步 apiServerUrl（不拉取设备状态，用于 SettingsOnline 初始化） */
  async function syncApiServerUrlFromConfig(): Promise<void> {
    await safeCall(async () => {
      const cfg = await getConfigMap()
      apiServerUrl.value = cfg.onlineApiServerUrl
    }, '[Online] sync api server url from config')
  }

  /** 注册新设备 */
  async function register(): Promise<boolean> {
    loading.value = true
    const ok = await safeCall(
      async () => {
        const status = await registerDevice()
        deviceStatus.value = status
      },
      '[Online] register device',
    )
    loading.value = false
    if (ok !== undefined) {
      toastSuccess('设备注册成功')
      return true
    }
    return false
  }

  /** 登录设备（刷新 JWT） */
  async function login(): Promise<boolean> {
    loading.value = true
    const ok = await safeCall(
      async () => {
        const status = await loginDevice()
        deviceStatus.value = status
      },
      '[Online] login device',
    )
    loading.value = false
    if (ok !== undefined) {
      toastSuccess('设备登录成功')
      return true
    }
    return false
  }

  /** 登出设备（撤销 JWT，保留密钥） */
  async function logout(): Promise<boolean> {
    loading.value = true
    const ok = await safeCall(
      async () => {
        await logoutDevice()
        await refreshStatus()
      },
      '[Online] logout device',
    )
    loading.value = false
    if (ok !== undefined) {
      toastSuccess('设备已登出')
      return true
    }
    return false
  }

  /** 清除设备凭证（注销设备，删除本地密钥） */
  async function clear(): Promise<boolean> {
    loading.value = true
    const ok = await safeCall(
      async () => {
        await clearDevice()
        await refreshStatus()
      },
      '[Online] clear device',
    )
    loading.value = false
    if (ok !== undefined) {
      toastSuccess('设备凭证已清除')
      return true
    }
    return false
  }

  return {
    deviceStatus,
    loading,
    refreshing,
    apiServerUrl,
    refreshStatus,
    syncApiServerUrlFromConfig,
    setApiServerUrl,
    register,
    login,
    logout,
    clear,
  }
})
