import type { ProviderInfo, FetchTunnelsResult } from '@/types/frp'
import { FRP_ACTIONS, frpManager } from './core'

export function listProviders(): Promise<ProviderInfo[]> {
  return frpManager(FRP_ACTIONS.LIST_PROVIDERS)
}
/** 确保 frpc 就绪；force=true 跳过就绪检查强制重新下载（「有新版本」更新按钮） */
export function ensureFrpc(force = false): Promise<string> {
  return frpManager<string>(FRP_ACTIONS.ENSURE_FRPC, { force })
}
export function installProviderFromDir(sourceDir: string): Promise<ProviderInfo> {
  return frpManager<ProviderInfo>(FRP_ACTIONS.INSTALL_PROVIDER_FROM_DIR, { sourceDir })
}
export function installProviderFromZip(zipPath: string): Promise<ProviderInfo> {
  return frpManager<ProviderInfo>(FRP_ACTIONS.INSTALL_PROVIDER_FROM_ZIP, { sourceDir: zipPath })
}
export function installProviderFromUrl(url: string): Promise<ProviderInfo> {
  return frpManager<ProviderInfo>(FRP_ACTIONS.INSTALL_PROVIDER_FROM_URL, { url })
}
export function uninstallProvider(providerId: string): Promise<void> {
  return frpManager<void>(FRP_ACTIONS.UNINSTALL_PROVIDER, { providerId })
}
export function enableProvider(providerId: string): Promise<void> {
  return frpManager<void>(FRP_ACTIONS.ENABLE_PROVIDER, { providerId })
}
export function disableProvider(providerId: string): Promise<void> {
  return frpManager<void>(FRP_ACTIONS.DISABLE_PROVIDER, { providerId })
}
export function fetchTunnels(providerId: string): Promise<FetchTunnelsResult> {
  return frpManager<FetchTunnelsResult>(FRP_ACTIONS.FETCH_TUNNELS, { providerId })
}

export type PackageType = 'frp_provider' | 'modpack' | 'unknown'
export interface DetectPackageResult {
  type: PackageType
  providerId?: string
  providerName?: string
}
export function detectPackageType(path: string): Promise<DetectPackageResult> {
  return frpManager<DetectPackageResult>(FRP_ACTIONS.DETECT_PACKAGE_TYPE, { path })
}
