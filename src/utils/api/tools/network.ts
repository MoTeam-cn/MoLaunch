/**
 * 工具模块 - 网络工具（延迟测试 / 服务器状态 / TCP 检测 / 端口列表）
 *
 * 对应后端 `tools_manager` 的 network_latency_test / server_ping / tcp_check /
 * list_open_ports action。
 */

import { TOOLS_ACTIONS, toolsManager } from './core'

// ==================== 网络延迟测试 ====================

/** 延迟测试条目 */
export interface LatencyItem {
  url: string
  latency_ms: number | null
  status_code: number
  error: string
}

/** 延迟测试结果 */
export interface NetworkLatencyResult {
  results: LatencyItem[]
}

/** 网络延迟测试 */
export function networkLatencyTest(urls: string[]): Promise<NetworkLatencyResult> {
  return toolsManager<NetworkLatencyResult>(TOOLS_ACTIONS.NETWORK_LATENCY_TEST, { urls })
}

// ==================== 服务器状态检测 ====================

/** 服务器状态检测结果 */
export interface ServerPingResult {
  motd: string
  /** 原始 MOTD（保留 § 格式化代码，供前端解析为彩色显示） */
  motd_raw: string
  online: number
  max: number
  version: string
  latency_ms: number
  favicon: string | null
  error: string
}

/** 服务器状态检测（SLP 协议） */
export function serverPing(host: string, port: number): Promise<ServerPingResult> {
  return toolsManager<ServerPingResult>(TOOLS_ACTIONS.SERVER_PING, { host, port })
}

/** TCP 端口连通性检测结果 */
export interface TcpCheckResult {
  /** 是否可连接 */
  reachable: boolean
  /** TCP 握手耗时（毫秒），失败时为 0 */
  latency_ms: number
  /** 失败原因（成功时为空） */
  error: string
}

/**
 * TCP 端口连通性检测（仅三次握手，3 秒超时）
 *
 * 用于 Frp 等非 Minecraft 协议服务的端口可达性检查，
 * 与 `serverPing`（SLP 协议）的区别：不发送应用层数据，适用于任意 TCP 服务。
 */
export function tcpCheck(host: string, port: number): Promise<TcpCheckResult> {
  return toolsManager<TcpCheckResult>(TOOLS_ACTIONS.TCP_CHECK, { host, port })
}

/** 本机监听端口条目 */
export interface OpenPortInfo {
  local_addr: string
  port: number
  protocol: string
  process_name: string | null
  pid: number | null
}

/** 列出本机监听端口结果 */
export interface ListOpenPortsResult {
  ports: OpenPortInfo[]
}

/** 列出本机所有监听中的 TCP/UDP 端口（供 Frp 内网端口选择） */
export function listOpenPorts(): Promise<ListOpenPortsResult> {
  return toolsManager<ListOpenPortsResult>(TOOLS_ACTIONS.LIST_OPEN_PORTS)
}
