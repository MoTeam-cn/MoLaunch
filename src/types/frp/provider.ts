import type { TunnelType } from './tunnel'

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
  /** 厂商图标绝对路径（后端填充，前端用 convertFileSrc 渲染） */
  icon?: string
}

/** 安装厂商参数 */
export interface InstallProviderParams {
  sourceDir: string
}

/** 厂商 ID 参数 */
export interface ProviderIdParams {
  providerId: string
}

// ============================================================
// 公共 frps 服务器（对接 apiServer `/v1/frp/*`）
// ============================================================

/** 公共 frps 服务器信息（GET /v1/frp/servers 返回数组元素，直接含完整连接信息） */
export interface PublicFrpServer {
  id: string
  name: string
  region: string
  serverAddr: string
  serverPort: number
  /** 公共共享 token（frpc token 字段） */
  publicToken: string
  tlsEnabled: boolean
}

// ============================================================
// 厂商 API 远程隧道（fetch_tunnels 返回）
// ============================================================

/** 厂商 API 返回的远程隧道信息（对应后端 TunnelInfo） */
export interface RemoteTunnelInfo {
  /** 厂商返回的隧道标识 */
  id: string
  /** 真实隧道 id（Lolia 等厂商用于 config 接口查询） */
  name: string
  /** 隧道显示名（厂商返回的 remark 等用户可读名字；为空时用 name） */
  remark: string
  /** 隧道类型（厂商返回的原始字符串，如 tcp/udp/http/https） */
  tunnelType: string
  /** 隧道状态（厂商返回的原始字符串） */
  status: string
  /** Frp 服务器地址 */
  serverHost: string
  /** Frp 服务器端口（字符串，部分厂商返回带前导 0） */
  serverPort: string
  /** Frp 服务器鉴权 token */
  token: string
  /** 本地 IP */
  localHost: string
  /** 本地端口（字符串） */
  localPort: string
  /** 远程端口（字符串） */
  remotePort: string
  /** 自定义域名（http/https 类型隧道） */
  customDomain: string
  /** config 接口返回的完整 frpc 配置，已解码 */
  rawConfig?: string
}

/** 安全导入 frpc 配置的后端结果 */
export interface ImportedFrpcConfig {
  serverAddr?: string
  serverPort?: number
  user?: string
  token?: string
  name?: string
  tunnelType?: TunnelType
  localIp?: string
  localPort?: number
  remotePort?: number
  useTls: boolean
  bandwidthLimit?: string
  bandwidthLimitMode?: string
  proxyUseEncryption?: boolean
  proxyUseCompression?: boolean
  proxyProtocolVersion?: string
}

/** 厂商 API 返回的账号信息（对应后端 AccountInfo） */
export interface RemoteAccountInfo {
  id: string
  username: string
  email: string
  /** 账号级 token（部分厂商用账号 token 而非隧道级 token） */
  token: string
}

/** fetch_tunnels 返回结构 */
export interface FetchTunnelsResult {
  tunnels: RemoteTunnelInfo[]
  account: RemoteAccountInfo
}