/**
 * Frp 管理 Pinia store
 *
 * 管理厂商列表、隧道列表、frpc 二进制状态。
 * 与 stores/online.ts 风格一致：state 用 ref，actions 直接调用 IPC API。
 *
 * 隧道状态同步：监听 `frp-tunnel-status` Tauri event，frpc 进程退出时
 * 自动刷新 tunnels 列表，避免列表显示"运行中"但实际已停止。
 */

import { ref } from 'vue'
import { defineStore } from 'pinia'
import type {
  AuthStatus,
  CreateTunnelParams,
  DeviceCodeResult,
  LogFileInfo,
  ProviderInfo,
  TunnelWithStatus,
  FrpTunnelStatusEvent,
  UpdateTunnelParams,
} from '@/types/frp'
import {
  listProviders,
  ensureFrpc,
  listTunnels,
  createTunnel as apiCreateTunnel,
  updateTunnel as apiUpdateTunnel,
  deleteTunnel as apiDeleteTunnel,
  startTunnel as apiStartTunnel,
  stopTunnel as apiStopTunnel,
  installProviderFromDir as apiInstallFromDir,
  installProviderFromZip as apiInstallFromZip,
  uninstallProvider as apiUninstallProvider,
  enableProvider as apiEnableProvider,
  disableProvider as apiDisableProvider,
  listLogFiles as apiListLogFiles,
  readLogFile as apiReadLogFile,
  getAuthStatus as apiGetAuthStatus,
  startOAuth2 as apiStartOAuth2,
  startDeviceCode as apiStartDeviceCode,
  pollDeviceCode as apiPollDeviceCode,
  refreshToken as apiRefreshToken,
  revokeAuth as apiRevokeAuth,
  saveApiKey as apiSaveApiKey,
} from '@/utils/api/frp-manager'
import { useTauriEvent } from '@/composables/useTauriEvent'
import { toastSuccess, toastError } from '@/utils/toast'

export const useFrpStore = defineStore('frp', () => {
  // ============================================================
  // State
  // ============================================================

  /** 厂商列表 */
  const providers = ref<ProviderInfo[]>([])
  /** 隧道列表（含运行状态） */
  const tunnels = ref<TunnelWithStatus[]>([])
  /** 厂商列表加载中 */
  const providersLoading = ref(false)
  /** 隧道列表加载中 */
  const tunnelsLoading = ref(false)
  /** frpc 下载中 */
  const frpcDownloading = ref(false)
  /** 隧道操作中（创建/删除/启动/停止） */
  const tunnelActionLoading = ref(false)
  /** 厂商安装/卸载/启禁操作中 */
  const providerActionLoading = ref(false)
  /** 当前日志页显示的日志行（实时事件 + 历史读取合并） */
  const logs = ref<string[]>([])
  /** 日志读取中 */
  const logsLoading = ref(false)
  /** 日志页选中的隧道 ID（空字符串=全部） */
  const selectedLogTunnelId = ref('')
  /** 所有隧道的日志文件信息（用于日志页隧道下拉选项） */
  const logFiles = ref<LogFileInfo[]>([])
  /** 日志读取时后端返回的 hasMore（指示是否还有更早的历史日志） */
  const logsHasMore = ref(false)
  /** 最近一次隧道状态变更事件（用于 TunnelManager 显示异常退出提示） */
  const lastTunnelStatus = ref<FrpTunnelStatusEvent | null>(null)
  /** 隧道状态事件监听器是否已启动（避免重复注册） */
  const statusListenerStarted = ref(false)

  // ============================================================
  // 认证状态（阶段三）
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
  // Actions
  // ============================================================

  /** 加载厂商列表 */
  async function loadProviders(): Promise<void> {
    providersLoading.value = true
    try {
      providers.value = await listProviders()
    } catch (e) {
      toastError('加载厂商列表失败：' + e)
    } finally {
      providersLoading.value = false
    }
  }

  /** 下载/确保 frpc 二进制就绪 */
  async function downloadFrpc(): Promise<boolean> {
    frpcDownloading.value = true
    try {
      await ensureFrpc()
      toastSuccess('frpc 下载完成')
      // 刷新厂商列表以更新 frpcReady 状态
      await loadProviders()
      return true
    } catch (e) {
      toastError('frpc 下载失败：' + e)
      return false
    } finally {
      frpcDownloading.value = false
    }
  }

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
    }
  }

  /** 启动隧道 */
  async function startTunnel(id: string): Promise<boolean> {
    tunnelActionLoading.value = true
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
    }
  }

  /** 停止隧道 */
  async function stopTunnel(id: string): Promise<boolean> {
    tunnelActionLoading.value = true
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
    }
  }

  /** 从目录安装厂商（manifest.toml + frpc 二进制） */
  async function installProviderFromDir(sourceDir: string): Promise<boolean> {
    providerActionLoading.value = true
    try {
      await apiInstallFromDir(sourceDir)
      toastSuccess('厂商安装成功')
      await loadProviders()
      return true
    } catch (e) {
      toastError('安装厂商失败：' + e)
      return false
    } finally {
      providerActionLoading.value = false
    }
  }

  /** 从 ZIP 包安装厂商（sourceDir 复用为 zipPath） */
  async function installProviderFromZip(zipPath: string): Promise<boolean> {
    providerActionLoading.value = true
    try {
      await apiInstallFromZip(zipPath)
      toastSuccess('厂商安装成功')
      await loadProviders()
      return true
    } catch (e) {
      toastError('安装厂商失败：' + e)
      return false
    } finally {
      providerActionLoading.value = false
    }
  }

  /** 卸载外部厂商（内置厂商会被后端拒绝） */
  async function uninstallProvider(providerId: string): Promise<boolean> {
    providerActionLoading.value = true
    try {
      await apiUninstallProvider(providerId)
      toastSuccess('厂商已卸载')
      await loadProviders()
      return true
    } catch (e) {
      toastError('卸载厂商失败：' + e)
      return false
    } finally {
      providerActionLoading.value = false
    }
  }

  /** 启用/禁用厂商（内置厂商不可禁用，会被后端拒绝） */
  async function toggleProvider(providerId: string, enabled: boolean): Promise<boolean> {
    providerActionLoading.value = true
    try {
      if (enabled) {
        await apiEnableProvider(providerId)
        toastSuccess('厂商已启用')
      } else {
        await apiDisableProvider(providerId)
        toastSuccess('厂商已禁用')
      }
      await loadProviders()
      return true
    } catch (e) {
      toastError((enabled ? '启用厂商失败：' : '禁用厂商失败：') + e)
      return false
    } finally {
      providerActionLoading.value = false
    }
  }

  /** 加载所有隧道的日志文件信息（用于日志页隧道下拉选项） */
  async function loadLogFiles(): Promise<void> {
    try {
      logFiles.value = await apiListLogFiles()
    } catch (e) {
      toastError('加载日志文件列表失败：' + e)
    }
  }

  /** 读取指定隧道的历史日志（tunnelId 为空时后端返回空数组） */
  async function readLogs(tunnelId: string, maxLines?: number): Promise<void> {
    logsLoading.value = true
    try {
      const content = await apiReadLogFile(tunnelId, maxLines)
      logs.value = content.lines
      logsHasMore.value = content.hasMore
    } catch (e) {
      toastError('读取日志失败：' + e)
      logs.value = []
      logsHasMore.value = false
    } finally {
      logsLoading.value = false
    }
  }

  /** 清空当前日志页显示的日志行（仅清前端缓存，不删后端文件） */
  function clearLogs(): void {
    logs.value = []
    logsHasMore.value = false
  }

  // ============================================================
  // 认证 Actions（阶段三）
  // ============================================================

  /** 设置单个厂商的认证操作 loading */
  function setAuthActionLoading(providerId: string, loading: boolean): void {
    authActionLoading.value[providerId] = loading
  }

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

  /** 刷新单个厂商的认证状态 */
  async function refreshAuthStatus(providerId: string): Promise<void> {
    try {
      const status = await apiGetAuthStatus(providerId)
      authStatuses.value[providerId] = status
    } catch {
      // 静默失败
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
    providers,
    tunnels,
    providersLoading,
    tunnelsLoading,
    frpcDownloading,
    tunnelActionLoading,
    providerActionLoading,
    logs,
    logsLoading,
    selectedLogTunnelId,
    logFiles,
    logsHasMore,
    lastTunnelStatus,
    authStatuses,
    authLoading,
    authActionLoading,
    deviceCodeInfos,
    deviceCodePolling,
    apiKeyInputs,
    // actions
    loadProviders,
    downloadFrpc,
    loadTunnels,
    refreshTunnelsSilent,
    startTunnelStatusListener,
    createTunnel,
    updateTunnel,
    deleteTunnel,
    startTunnel,
    stopTunnel,
    installProviderFromDir,
    installProviderFromZip,
    uninstallProvider,
    toggleProvider,
    loadLogFiles,
    readLogs,
    clearLogs,
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
})
