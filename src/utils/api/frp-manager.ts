/**
 * Frp 管理 IPC API
 *
 * 所有调用走 `invoke('frp_manager', { req: { action, params } })` 单一入口，
 * 通过 `action` 字段分发到后端对应处理函数。
 * 与 online-manager.ts / plugins-manager.ts 模式一致。
 */

import { invoke } from '@tauri-apps/api/core'
import type {
  CreateTunnelParams,
  LogFileContent,
  LogFileInfo,
  ProviderInfo,
  Tunnel,
  TunnelStatus,
  TunnelWithStatus,
} from '@/types/frp'

/** Frp 管理 action 名称常量 */
export const FRP_ACTIONS = {
  /** 列出所有厂商 */
  LIST_PROVIDERS: 'list_providers',
  /** 下载/确保 frpc 二进制就绪 */
  ENSURE_FRPC: 'ensure_frpc',
  /** 列出所有隧道（含运行状态） */
  LIST_TUNNELS: 'list_tunnels',
  /** 创建隧道 */
  CREATE_TUNNEL: 'create_tunnel',
  /** 删除隧道 */
  DELETE_TUNNEL: 'delete_tunnel',
  /** 启动隧道 */
  START_TUNNEL: 'start_tunnel',
  /** 停止隧道 */
  STOP_TUNNEL: 'stop_tunnel',
  /** 查询隧道状态 */
  GET_TUNNEL_STATUS: 'get_tunnel_status',
  /** 从目录安装厂商（manifest.toml + frpc 二进制） */
  INSTALL_PROVIDER_FROM_DIR: 'install_provider_from_dir',
  /** 从 ZIP 包安装厂商（sourceDir 复用为 zipPath） */
  INSTALL_PROVIDER_FROM_ZIP: 'install_provider_from_zip',
  /** 卸载外部厂商 */
  UNINSTALL_PROVIDER: 'uninstall_provider',
  /** 启用厂商（内置厂商不可禁用，调用会被后端拒绝） */
  ENABLE_PROVIDER: 'enable_provider',
  /** 禁用厂商 */
  DISABLE_PROVIDER: 'disable_provider',
  /** 列出所有隧道日志文件信息 */
  LIST_LOG_FILES: 'list_log_files',
  /** 读取指定隧道的日志文件内容 */
  READ_LOG_FILE: 'read_log_file',
} as const

/**
 * Frp 管理统一调用入口
 *
 * @param action action 名称（见 FRP_ACTIONS）
 * @param params 参数对象（可选）
 */
export async function frpManager<T = unknown>(
  action: string,
  params?: unknown,
): Promise<T> {
  return invoke<T>('frp_manager', { req: { action, params: params ?? null } })
}

// ============================================================
// 便捷封装函数
// ============================================================

/** 列出所有厂商 */
export function listProviders(): Promise<ProviderInfo[]> {
  return frpManager(FRP_ACTIONS.LIST_PROVIDERS)
}

/** 下载/确保 frpc 二进制就绪 */
export function ensureFrpc(): Promise<string> {
  return frpManager<string>(FRP_ACTIONS.ENSURE_FRPC)
}

/** 列出所有隧道（含运行状态） */
export function listTunnels(): Promise<TunnelWithStatus[]> {
  return frpManager(FRP_ACTIONS.LIST_TUNNELS)
}

/** 创建隧道 */
export function createTunnel(params: CreateTunnelParams): Promise<Tunnel> {
  return frpManager(FRP_ACTIONS.CREATE_TUNNEL, params)
}

/** 删除隧道 */
export function deleteTunnel(id: string): Promise<void> {
  return frpManager(FRP_ACTIONS.DELETE_TUNNEL, { id })
}

/** 启动隧道 */
export function startTunnel(id: string): Promise<void> {
  return frpManager(FRP_ACTIONS.START_TUNNEL, { id })
}

/** 停止隧道 */
export function stopTunnel(id: string): Promise<void> {
  return frpManager(FRP_ACTIONS.STOP_TUNNEL, { id })
}

/** 查询隧道状态 */
export function getTunnelStatus(id: string): Promise<TunnelStatus> {
  return frpManager(FRP_ACTIONS.GET_TUNNEL_STATUS, { id })
}

/** 从目录安装厂商（manifest.toml + frpc 二进制） */
export function installProviderFromDir(sourceDir: string): Promise<ProviderInfo> {
  return frpManager<ProviderInfo>(FRP_ACTIONS.INSTALL_PROVIDER_FROM_DIR, { sourceDir })
}

/** 从 ZIP 包安装厂商（sourceDir 复用为 zipPath） */
export function installProviderFromZip(zipPath: string): Promise<ProviderInfo> {
  return frpManager<ProviderInfo>(FRP_ACTIONS.INSTALL_PROVIDER_FROM_ZIP, { sourceDir: zipPath })
}

/** 卸载外部厂商 */
export function uninstallProvider(providerId: string): Promise<void> {
  return frpManager<void>(FRP_ACTIONS.UNINSTALL_PROVIDER, { providerId })
}

/** 启用厂商（内置厂商不可禁用，调用会被后端拒绝） */
export function enableProvider(providerId: string): Promise<void> {
  return frpManager<void>(FRP_ACTIONS.ENABLE_PROVIDER, { providerId })
}

/** 禁用厂商 */
export function disableProvider(providerId: string): Promise<void> {
  return frpManager<void>(FRP_ACTIONS.DISABLE_PROVIDER, { providerId })
}

/** 列出所有隧道日志文件信息 */
export function listLogFiles(): Promise<LogFileInfo[]> {
  return frpManager<LogFileInfo[]>(FRP_ACTIONS.LIST_LOG_FILES)
}

/** 读取指定隧道的日志文件内容 */
export function readLogFile(tunnelId: string, maxLines?: number): Promise<LogFileContent> {
  return frpManager<LogFileContent>(FRP_ACTIONS.READ_LOG_FILE, { tunnelId, maxLines })
}
