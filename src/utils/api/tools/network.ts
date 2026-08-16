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

// ==================== 地址延迟测试 ====================

/** 地址延迟测试目标 */
export interface AddressTarget {
  /** 显示名（如「南京」），缺省用 host */
  name?: string
  host: string
  port: number
  /** 测延迟协议：tcp（默认，TCP 握手）/ udp（UDP 探针）/ ping（ICMP，系统 ping） */
  protocol?: 'tcp' | 'udp' | 'ping'
}

/** 地址延迟测试单条结果 */
export interface AddressLatencyItem {
  name: string | null
  host: string
  port: number
  protocol: string
  /** 是否可达 */
  reachable: boolean
  /** 延迟（毫秒），失败时为 0 */
  latency_ms: number
  /** 失败原因（成功时为空） */
  error: string
}

/** 地址延迟测试结果 */
export interface AddressLatencyResult {
  results: AddressLatencyItem[]
  /** 持续测试任务 id（persistent=true 时返回，供停止） */
  task_id: string | null
}

/** 地址延迟持续测试 emit 事件名（payload = AddressLatencyResult） */
export const LATENCY_UPDATE_EVENT = 'tools-latency-update'

/**
 * 地址延迟测试（tcp 握手 / udp 探针 / 系统 ping）
 * @param persistent true 时后端按 intervalMs 周期测试并经 `tools-latency-update` 事件推送，需调用 addressLatencyStop 停止
 */
export function addressLatencyTest(
  targets: AddressTarget[],
  opts?: { persistent?: boolean; intervalMs?: number },
): Promise<AddressLatencyResult> {
  return toolsManager<AddressLatencyResult>(TOOLS_ACTIONS.ADDRESS_LATENCY_TEST, {
    targets,
    persistent: opts?.persistent ?? false,
    interval_ms: opts?.intervalMs ?? 3000,
  })
}

/** 停止持续地址延迟测试 */
export function addressLatencyStop(): Promise<Record<string, never>> {
  return toolsManager<Record<string, never>>(TOOLS_ACTIONS.ADDRESS_LATENCY_STOP)
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

// ==================== 正版玩家皮肤 ====================

/** 正版玩家皮肤获取结果 */
export interface SkinFetchResult {
  /** 玩家名（正版 API 返回的规范化名称） */
  name: string
  /** 玩家 UUID（32 位十六进制，无连字符） */
  uuid: string
  /** 皮肤模型："slim"（Alex 细手臂）| "classic"（Steve 粗手臂） */
  skin_model: string
  /** 皮肤图片地址 */
  skin_url: string
  /** 皮肤 PNG（base64 data URI，供直接预览） */
  skin_image: string
  /** 披风地址（无披风时为 null） */
  cape_url: string | null
  /** 披风 PNG（base64 data URI，无披风时为 null） */
  cape_image: string | null
  /** 失败原因（成功时为空） */
  error: string
}

/** 获取正版玩家皮肤（输入玩家名，返回 UUID / 模型 / 皮肤与披风图片） */
export function skinFetch(name: string): Promise<SkinFetchResult> {
  return toolsManager<SkinFetchResult>(TOOLS_ACTIONS.SKIN_FETCH, { name })
}

/**
 * 保存皮肤图片到本地路径（base64 → PNG 文件）
 * @param savePath 保存路径（含文件名，如 D:/skin/Steve.png）
 * @param imageBase64 图片 base64（不含 data URI 前缀）
 */
export function skinSaveImage(savePath: string, imageBase64: string): Promise<{ success: boolean }> {
  return toolsManager<{ success: boolean }>(TOOLS_ACTIONS.SKIN_SAVE_IMAGE, {
    save_path: savePath,
    image_base64: imageBase64,
  })
}
