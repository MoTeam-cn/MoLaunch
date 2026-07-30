/**
 * Frp 内网穿透相关类型定义
 *
 * 与后端 commands/frp/mod.rs 中的 Rust 类型一一对应。
 */

/** 隧道类型 */
export type TunnelType = 'tcp' | 'udp'

/** 隧道运行状态 */
export type TunnelStatus = 'running' | 'stopped'

/** 隧道配置（持久化） */
export interface Tunnel {
  /** 隧道唯一 ID */
  id: string
  /** 隧道名称 */
  name: string
  /** 所属厂商 ID */
  providerId: string
  /** 隧道类型 */
  tunnelType: TunnelType
  /** 本地 IP（默认 127.0.0.1） */
  localIp: string
  /** 本地端口（如 25565） */
  localPort: number
  /** Frp 服务器地址 */
  serverAddr: string
  /** Frp 服务器端口 */
  serverPort: number
  /** 远程端口 */
  remotePort: number
  /** Frp 服务器鉴权 token（可选） */
  token?: string
  /** 是否启用 TLS */
  useTls: boolean
  /** 创建时间（Unix 毫秒） */
  createdAt: number
}

/** 隧道 + 运行状态（后端返回） */
export interface TunnelWithStatus extends Tunnel {
  /** 当前运行状态 */
  status: TunnelStatus
  /** 运行中的进程 PID（status=running 时有值） */
  pid?: number
}

/** 厂商信息（后端返回，扩展字段） */
export interface ProviderInfo {
  id: string
  name: string
  description: string
  version: string
  author: string
  /** 是否为内置厂商 */
  builtin: boolean
  /** 认证类型：none / oauth2 / device_code / api_key */
  authType: string
  /** frpc 二进制是否就绪 */
  frpcReady: boolean
  /** 是否启用（内置厂商始终 true） */
  enabled: boolean
  /** frpc 分发方式：bundled / url / system */
  distribution: string
  homepage?: string
}

/** 日志文件信息 */
export interface LogFileInfo {
  tunnelId: string
  sizeBytes: number
  modifiedAt: number
}

/** 日志文件内容 */
export interface LogFileContent {
  lines: string[]
  hasMore: boolean
}

/** frpc 日志 event payload */
export interface FrpcLogEvent {
  tunnelId: string
  tunnelName: string
  line: string
  timestamp: number
  level: string
}

/** 隧道状态变更 event payload */
export interface FrpTunnelStatusEvent {
  tunnelId: string
  tunnelName: string
  status: string
  pid?: number
  exitCode?: number
  error?: string
}

/** 安装厂商参数 */
export interface InstallProviderParams {
  sourceDir: string
}

/** 厂商 ID 参数 */
export interface ProviderIdParams {
  providerId: string
}

/** 读取日志参数 */
export interface ReadLogParams {
  tunnelId: string
  maxLines?: number
}

/** 创建隧道参数 */
export interface CreateTunnelParams {
  name: string
  providerId: string
  tunnelType: TunnelType
  localIp?: string
  localPort: number
  serverAddr: string
  serverPort: number
  remotePort: number
  token?: string
  useTls?: boolean
}
