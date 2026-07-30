/**
 * MoTeam 官方公共 Frp 服务器 API（apiServer 端）
 *
 * apiServer 尚未实现 /v1/frp/* 路由，此文件为预留封装。
 * 实际请求会返回 404，等 apiServer 实现后联调。
 * 设计文档：docs/FRP_PUBLIC_SERVER_API_DESIGN.md
 */

// 注：阶段二仅占位，函数体直接抛错。apiServer 实现后改为实际 HTTP 请求，
// 届时再引入 invoke / fetch 等依赖，避免当前出现未使用 import 警告。

/** 公共 Frp 服务器信息 */
export interface PublicFrpServer {
  id: string
  name: string
  region: string
  serverAddr: string
  serverPort: number
  onlineUsers: number
  maxUsers: number
  loadPercent: number
  allocatable: boolean
  tlsEnabled: boolean
}

/** 分配端口请求 */
export interface AllocateRequest {
  serverId: string
  tunnelType: 'tcp' | 'udp'
}

/** 分配端口响应 */
export interface AllocateResponse {
  serverId: string
  serverAddr: string
  serverPort: number
  remotePort: number
  token: string
  tlsEnabled: boolean
  /** 分配的隧道有效期（秒） */
  expiresAt: number
}

/**
 * 获取公共 Frp 服务器列表
 *
 * 调用 apiServer 的 GET /v1/frp/servers 路由。
 * 注意：apiServer 尚未实现此路由，调用会失败。
 */
export async function listPublicFrpServers(): Promise<PublicFrpServer[]> {
  // TODO: apiServer 实现后改为实际的 HTTP 请求
  // 当前通过后端代理调用（避免前端直接暴露 apiServer 地址）
  // 后端 frp_manager 可新增 action 转发，或前端直接 fetch
  throw new Error('apiServer 公共 Frp 服务器 API 尚未实现')
}

/**
 * 分配公共 Frp 服务器端口
 */
export async function allocatePublicFrpServer(req: AllocateRequest): Promise<AllocateResponse> {
  // TODO: apiServer 实现后改为实际的 HTTP 请求（使用 req 参数）
  void req
  throw new Error('apiServer 公共 Frp 服务器 API 尚未实现')
}

/**
 * 释放公共 Frp 服务器端口
 */
export async function releasePublicFrpServer(serverId: string, remotePort: number): Promise<void> {
  // TODO: apiServer 实现后改为实际的 HTTP 请求（使用 serverId / remotePort 参数）
  void serverId
  void remotePort
  throw new Error('apiServer 公共 Frp 服务器 API 尚未实现')
}
