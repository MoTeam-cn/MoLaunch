/**
 * 联机 store 设备认证切片
 *
 * 从 stores/online.ts 抽取的设备认证 state + actions，按 Pinia setup store 的
 * composable 切片模式组织。切片内部闭环（refreshStatus 被 logout/clear 复用，
 * initAuth 被 reconnect 复用），不依赖房间切片。
 */

import { ref } from 'vue'
import type { DeviceStatus } from '@/types/online'
import {
  getAuthStatus,
  registerDevice,
  loginDevice,
  logoutDevice,
  clearDevice,
  initAuth as initAuthApi,
  refreshAuth as refreshAuthApi,
} from '@/utils/api/online-manager'
import { applyConfig, getConfigMap } from '@/utils/api/config'
import { toastSuccess, toastError } from '@/utils/toast'
import { safeCall } from '@/utils/async'

/** 创建联机 store 设备认证切片 */
export function useOnlineAuthSlice() {
  // ===== 设备认证 =====
  /** 设备认证状态（null 表示未查询） */
  const deviceStatus = ref<DeviceStatus | null>(null)
  /** 是否正在执行写操作（注册/登录/登出/清除） */
  const loading = ref(false)
  /** 是否正在拉取状态 */
  const refreshing = ref(false)
  /** 当前 api-server 地址（从后端配置同步） */
  const apiServerUrl = ref('')
  /**
   * 云端连接状态（全局降级开关）
   *
   * - `true`：云端 API 可用，联机功能正常
   * - `false`：云端 API 不可用（启动初始化失败），联机按钮禁用、相关功能降级
   *
   * 由 `initAuth()` 在启动时设置；`reconnect()` 可手动重试。
   */
  const cloudConnected = ref(false)
  /** 云端连接错误信息（cloudConnected=false 时非空，用于弹窗提示） */
  const cloudError = ref<string | null>(null)
  /**
   * 是否正在执行启动认证 / 重连
   *
   * 初始值为 `true`：App.vue 在 onMounted 中调用 `initAuth()` 之前，
   * Online.vue 可能已经挂载（路由为 /apps/online 时硬刷新场景）。
   * 若初始为 `false`，Online.vue 的 `!cloudConnected && !initializing` 判断为 true，
   * 会短暂显示"云端连接失败"遮罩，直到 initAuth 完成才消失（闪烁）。
   * 初始为 `true` 可让遮罩在 initAuth 完成前不显示，避免误判。
   */
  const initializing = ref(true)

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
      (e) => toastError('登出失败：' + String(e)),
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
      (e) => toastError('清除凭证失败：' + String(e)),
    )
    loading.value = false
    if (ok !== undefined) {
      toastSuccess('设备凭证已清除')
      return true
    }
    return false
  }

  /**
   * 启动静默认证（程序启动时调用一次）
   *
   * 调用后端 `auth_init` action，自动完成：
   * - 首次启动无凭证 → 静默注册
   * - access token 过期 → refresh_token 续期或重新登录
   *
   * 根据结果设置 `cloudConnected`：
   * - 成功 → `cloudConnected = true`，联机功能可用
   * - 失败 → `cloudConnected = false`，联机按钮禁用，`cloudError` 存错误信息
   *
   * @returns 是否成功连接云端
   */
  async function initAuth(): Promise<boolean> {
    initializing.value = true
    cloudError.value = null
    const result = await safeCall(
      async () => {
        const res = await initAuthApi()
        deviceStatus.value = res.status
        apiServerUrl.value = res.status.api_server_url
        if (res.error) {
          cloudConnected.value = false
          cloudError.value = res.error
          return false
        }
        cloudConnected.value = true
        return true
      },
      '[Online] init auth',
    )
    initializing.value = false
    // safeCall 返回 undefined 表示异常，返回 false 表示云端失败
    if (result === undefined) {
      cloudConnected.value = false
      cloudError.value = '联机服务初始化异常'
      return false
    }
    return result
  }

  /**
   * 手动重新连接云端（设置页"重新连接"按钮调用）
   *
   * 流程：
   * 1. 调用 `refreshAuth` 尝试用 refresh_token 换新 token
   * 2. refresh 失败 → 调用 `initAuth` 走完整初始化（含注册/登录）
   *
   * @returns 是否重连成功
   */
  async function reconnect(): Promise<boolean> {
    initializing.value = true
    cloudError.value = null
    // 先尝试 refresh，失败再走完整 initAuth
    const refreshed = await safeCall(
      async () => {
        const status = await refreshAuthApi()
        deviceStatus.value = status
        apiServerUrl.value = status.api_server_url
        return true
      },
      '[Online] reconnect via refresh',
    )
    if (refreshed) {
      cloudConnected.value = true
      initializing.value = false
      toastSuccess('已重新连接到云端')
      return true
    }
    // refresh 失败，走完整初始化
    const ok = await initAuth()
    initializing.value = false
    if (ok) {
      toastSuccess('已重新连接到云端')
    } else {
      toastError(cloudError.value || '重连失败，请检查网络或 api-server 地址')
    }
    return ok
  }

  return {
    // 设备认证状态
    deviceStatus,
    loading,
    refreshing,
    apiServerUrl,
    // 云端连接状态（全局降级开关）
    cloudConnected,
    cloudError,
    initializing,
    // 设备认证方法
    refreshStatus,
    syncApiServerUrlFromConfig,
    setApiServerUrl,
    register,
    login,
    logout,
    clear,
    // 启动静默认证 + 重连
    initAuth,
    reconnect,
  }
}
