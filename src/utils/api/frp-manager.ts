/**
 * Frp 管理 IPC API
 *
 * 所有调用走 `invoke('frp_manager', { req: { action, params } })` 单一入口，
 * 通过 `action` 字段分发到后端对应处理函数。
 * 与 online-manager.ts / plugins-manager.ts 模式一致。
 */

import { invoke } from '@tauri-apps/api/core'
import type {
  AllocatePublicServerParams,
  AllocateResponse,
  AuthStatus,
  CreateTunnelParams,
  DeviceCodePollResult,
  DeviceCodeResult,
  FetchTunnelsResult,
  OAuth2Result,
  UpdateTunnelParams,
  ImportedFrpcConfig,
  LogFileContent,
  LogFileInfo,
  ProviderInfo,
  PublicFrpServer,
  SaveApiKeyParams,
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
  /** 更新隧道配置 */
  UPDATE_TUNNEL: 'update_tunnel',
  /** 删除隧道 */
  DELETE_TUNNEL: 'delete_tunnel',
  /** 安全导入 frpc 配置文件 */
  IMPORT_FRPC_CONFIG: 'import_frpc_config',
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
  /** 从 URL 下载并安装厂商 */
  INSTALL_PROVIDER_FROM_URL: 'install_provider_from_url',
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
  /** 清空指定隧道日志文件内容（保留文件） */
  CLEAR_LOG_FILE: 'clear_log_file',
  /** 列出可用的公共 frps 服务器（GET /v1/frp/servers） */
  LIST_PUBLIC_SERVERS: 'list_public_servers',
  /** 分配端口 + per-user token（POST /v1/frp/allocate） */
  ALLOCATE_PUBLIC_SERVER: 'allocate_public_server',
  /** 释放分配（POST /v1/frp/release） */
  RELEASE_PUBLIC_SERVER: 'release_public_server',
  /** 续期分配（POST /v1/frp/keepalive） */
  KEEPALIVE_PUBLIC_SERVER: 'keepalive_public_server',
  /** 查询认证状态 */
  GET_AUTH_STATUS: 'get_auth_status',
  /** 启动 OAuth2 授权流程 */
  START_OAUTH2: 'start_oauth2',
  /** 启动 Device Code 流程 */
  START_DEVICE_CODE: 'start_device_code',
  /** 轮询 Device Code token */
  POLL_DEVICE_CODE: 'poll_device_code',
  /** 刷新 token */
  REFRESH_TOKEN: 'refresh_token',
  /** 撤销认证 */
  REVOKE_AUTH: 'revoke_auth',
  /** 保存 API Key */
  SAVE_API_KEY: 'save_api_key',
  /** 从厂商 API 拉取隧道列表（需先认证） */
  FETCH_TUNNELS: 'fetch_tunnels',
  /** 检测拖拽包类型（frp 厂商包 / 整合包 / 未知） */
  DETECT_PACKAGE_TYPE: 'detect_package_type',
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

/** 更新隧道配置（编辑隧道） */
export function updateTunnel(params: UpdateTunnelParams): Promise<Tunnel> {
  return frpManager<Tunnel>(FRP_ACTIONS.UPDATE_TUNNEL, params)
}

/** 删除隧道 */
export function deleteTunnel(id: string): Promise<void> {
  return frpManager(FRP_ACTIONS.DELETE_TUNNEL, { id })
}

export function importFrpcConfig(path: string): Promise<ImportedFrpcConfig> {
  return frpManager(FRP_ACTIONS.IMPORT_FRPC_CONFIG, { path })
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

/** 从 URL 下载并安装厂商（仅支持 HTTPS） */
export function installProviderFromUrl(url: string): Promise<ProviderInfo> {
  return frpManager<ProviderInfo>(FRP_ACTIONS.INSTALL_PROVIDER_FROM_URL, { url })
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

/** 清空指定隧道的日志文件内容（tunnelId 为空时清空全部；保留日志文件本身） */
export function clearLogFile(tunnelId: string): Promise<void> {
  return frpManager<void>(FRP_ACTIONS.CLEAR_LOG_FILE, { tunnelId })
}

// ============================================================
// 公共 frps 服务器（对接 apiServer `/v1/frp/*`）
// ============================================================

/** 列出可用的公共 frps 服务器 */
export function listPublicServers(): Promise<PublicFrpServer[]> {
  return frpManager<PublicFrpServer[]>(FRP_ACTIONS.LIST_PUBLIC_SERVERS)
}

/** 分配端口 + per-user token */
export function allocatePublicServer(params: AllocatePublicServerParams): Promise<AllocateResponse> {
  return frpManager<AllocateResponse>(FRP_ACTIONS.ALLOCATE_PUBLIC_SERVER, params)
}

/** 释放分配（停止隧道时调用，便于端口回收） */
export function releasePublicServer(allocationId: string): Promise<void> {
  return frpManager<void>(FRP_ACTIONS.RELEASE_PUBLIC_SERVER, { allocationId })
}

/** 续期分配（frpc 运行期间定时调用，延长过期时间） */
export function keepalivePublicServer(allocationId: string): Promise<unknown> {
  return frpManager(FRP_ACTIONS.KEEPALIVE_PUBLIC_SERVER, { allocationId })
}

// ============================================================
// 认证体系（阶段三：OAuth2 / Device Code / API Key）
// ============================================================

/** 查询指定厂商的认证状态 */
export function getAuthStatus(providerId: string): Promise<AuthStatus> {
  return frpManager<AuthStatus>(FRP_ACTIONS.GET_AUTH_STATUS, { providerId })
}

/** 启动 OAuth2 授权流程（打开浏览器，等待回调，换取 token） */
export function startOAuth2(providerId: string): Promise<OAuth2Result> {
  return frpManager<OAuth2Result>(FRP_ACTIONS.START_OAUTH2, { providerId })
}

/** 启动 Device Code 流程（获取用户码 + 验证链接） */
export function startDeviceCode(providerId: string): Promise<DeviceCodeResult> {
  return frpManager<DeviceCodeResult>(FRP_ACTIONS.START_DEVICE_CODE, { providerId })
}

/** 轮询 Device Code token（前端按 interval 调用） */
export function pollDeviceCode(providerId: string): Promise<DeviceCodePollResult> {
  return frpManager<DeviceCodePollResult>(FRP_ACTIONS.POLL_DEVICE_CODE, { providerId })
}

/** 刷新 token（手动触发或自动刷新） */
export function refreshToken(providerId: string): Promise<void> {
  return frpManager<void>(FRP_ACTIONS.REFRESH_TOKEN, { providerId })
}

/** 撤销认证（删除所有存储的 token） */
export function revokeAuth(providerId: string): Promise<void> {
  return frpManager<void>(FRP_ACTIONS.REVOKE_AUTH, { providerId })
}

/** 保存 API Key（auth_type=api_key 的厂商） */
export function saveApiKey(params: SaveApiKeyParams): Promise<void> {
  return frpManager<void>(FRP_ACTIONS.SAVE_API_KEY, params)
}

/**
 * 从厂商 API 拉取隧道列表
 *
 * 调用前必须先检查厂商授权状态（getAuthStatus），未授权时引导用户去认证中心。
 * 返回的隧道列表由厂商 endpoints.json 配置的 envelope/itemsField/fields 映射而来。
 */
export function fetchTunnels(providerId: string): Promise<FetchTunnelsResult> {
  return frpManager<FetchTunnelsResult>(FRP_ACTIONS.FETCH_TUNNELS, { providerId })
}

// ============================================================
// 拖拽包类型检测（frp 厂商包 / 整合包 / 未知）
// ============================================================

/** 拖拽包类型 */
export type PackageType = 'frp_provider' | 'modpack' | 'unknown'

/** 包类型检测结果（来自后端 detect_package_type） */
export interface DetectPackageResult {
  type: PackageType
  /** frp 厂商包：manifest 中的 id */
  providerId?: string
  /** frp 厂商包：manifest 中的 name */
  providerName?: string
}

/**
 * 检测拖拽包类型
 *
 * 后端解析 zip 内容特征：
 * - frp 厂商包：manifest.json 含 id + binary/api
 * - 整合包：addons（MCBBS）或 files+minecraft（CurseForge）等
 * - 其余：unknown
 */
export function detectPackageType(path: string): Promise<DetectPackageResult> {
  return frpManager<DetectPackageResult>(FRP_ACTIONS.DETECT_PACKAGE_TYPE, { path })
}
