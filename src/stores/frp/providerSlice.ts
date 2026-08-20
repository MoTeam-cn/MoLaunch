/**
 * Frp store 厂商切片（阶段三）
 *
 * 所有厂商 action 直接调用 frp-manager IPC 封装；安装/卸载/启禁后统一 loadProviders 刷新；
 * providers 引用对外暴露，供 authSlice 迭代判断认证类型。
 */

import { ref } from 'vue'
import type { ProviderInfo } from '@/types/frp'
import {
  listProviders,
  ensureFrpc,
  installProviderFromDir as apiInstallFromDir,
  installProviderFromZip as apiInstallFromZip,
  installProviderFromUrl as apiInstallFromUrl,
  uninstallProvider as apiUninstallProvider,
  enableProvider as apiEnableProvider,
  disableProvider as apiDisableProvider,
} from '@/utils/api/frp-manager'
import { toastSuccess, toastError } from '@/utils/toast'

/** 创建 Frp 厂商切片（无外部依赖，providers 由本切片自持） */
export function useFrpProviderSlice() {
  /** 厂商列表 */
  const providers = ref<ProviderInfo[]>([])
  /** 厂商列表加载中 */
  const providersLoading = ref(false)
  /** frpc 下载中 */
  const frpcDownloading = ref(false)
  /** 厂商安装/卸载/启禁操作中 */
  const providerActionLoading = ref(false)

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

  /** 下载/确保 frpc 二进制就绪；force=true 强制重新下载（「有新版本」更新按钮） */
  async function downloadFrpc(force = false): Promise<boolean> {
    frpcDownloading.value = true
    try {
      await ensureFrpc(force)
      toastSuccess(force ? 'frpc 更新完成' : 'frpc 下载完成')
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

  /** 从 URL 下载并安装厂商（仅支持 HTTPS） */
  async function installProviderFromUrl(url: string): Promise<boolean> {
    providerActionLoading.value = true
    try {
      await apiInstallFromUrl(url)
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

  return {
    // state
    providers,
    providersLoading,
    frpcDownloading,
    providerActionLoading,
    // actions
    loadProviders,
    downloadFrpc,
    installProviderFromDir,
    installProviderFromZip,
    installProviderFromUrl,
    uninstallProvider,
    toggleProvider,
  }
}
