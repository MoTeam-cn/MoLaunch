/**
 * Frp 隧道自检逻辑
 *
 * 对每条隧道执行 4 项检查：配置完整性 / Frp 服务器可达性 / 本地端口监听 / frpc 就绪。
 *
 * 复用现有能力：
 * - `tcpCheck`（@/utils/api/tools）：TCP 三次握手连通性检测，3 秒超时由后端控制
 * - `listOpenPorts`（@/utils/api/tools）：一次性枚举本机所有监听端口，避免每条隧道重复调用
 * - `TunnelWithStatus` / `ProviderInfo`（@/types/frp）：与后端类型一一对应
 *
 * 设计要点：
 * - `listOpenPorts()` 仅调用一次，结果在所有隧道间共享
 * - 各隧道的 `tcpCheck` 通过 `Promise.allSettled` 并发执行，避免串行等待
 * - 单条隧道检查内部已捕获 `tcpCheck` 异常，`allSettled` 兜底防止意外 rejection
 */
import { tcpCheck, listOpenPorts, type OpenPortInfo } from '@/utils/api/tools'
import type { TunnelWithStatus, ProviderInfo } from '@/types/frp'

/** 单项检查结果 */
export interface CheckEntry {
  ok: boolean
  message: string
  /** 可选的附加信息（如延迟毫秒） */
  detail?: string
}

/** 单条隧道的自检结果 */
export interface TunnelCheckResult {
  tunnelId: string
  tunnelName: string
  /** 配置完整性：名称/地址/端口是否有效 */
  config: CheckEntry
  /** Frp 服务器可达性：TCP 连通性检测 */
  serverReachable: CheckEntry
  /** 本地端口是否在监听（MC 服务器是否已启动） */
  localPortListening: CheckEntry
  /** frpc 二进制是否就绪 */
  frpcReady: CheckEntry
  /** 整体是否通过（4 项全 ok） */
  overall: boolean
}

/**
 * 批量自检所有隧道
 *
 * @param tunnels 隧道列表（含运行状态）
 * @param providers 厂商列表（用于检查 frpcReady）
 * @returns 每条隧道的检查结果数组（顺序与入参一致）
 */
export async function checkTunnels(
  tunnels: TunnelWithStatus[],
  providers: ProviderInfo[],
): Promise<TunnelCheckResult[]> {
  // 一次性获取本机所有监听端口（避免每条隧道重复调用）
  let openPorts: OpenPortInfo[] = []
  try {
    const result = await listOpenPorts()
    openPorts = result.ports
  } catch {
    // 获取失败时按空列表处理，所有 localPortListening 检查都会失败
    openPorts = []
  }

  /** 单条隧道检查 */
  async function checkOne(tunnel: TunnelWithStatus): Promise<TunnelCheckResult> {
    // 1. 配置完整性：name 非空、serverAddr 非空、localPort>0、serverPort>0、remotePort>0
    const configErrors: string[] = []
    if (!tunnel.name) configErrors.push('名称为空')
    if (!tunnel.serverAddr) configErrors.push('服务器地址为空')
    if (!(tunnel.localPort > 0)) configErrors.push('本地端口无效')
    if (!(tunnel.serverPort > 0)) configErrors.push('服务器端口无效')
    if (!(tunnel.remotePort > 0)) configErrors.push('远程端口无效')
    const config: CheckEntry =
      configErrors.length === 0
        ? { ok: true, message: '配置完整' }
        : { ok: false, message: configErrors.join('、') }

    // 2. Frp 服务器可达性：TCP 连通性检测（3 秒超时由后端控制）
    let serverReachable: CheckEntry
    try {
      const r = await tcpCheck(tunnel.serverAddr, tunnel.serverPort)
      if (r.reachable) {
        serverReachable = {
          ok: true,
          message: '连接成功',
          detail: r.latency_ms > 0 ? `${r.latency_ms}ms` : undefined,
        }
      } else {
        serverReachable = { ok: false, message: r.error || '连接失败' }
      }
    } catch (e) {
      serverReachable = {
        ok: false,
        message: e instanceof Error ? e.message : '检测异常',
      }
    }

    // 3. 本地端口是否在监听（MC 服务器是否已启动）
    const listening = openPorts.some(p => p.port === tunnel.localPort)
    const localPortListening: CheckEntry = listening
      ? { ok: true, message: `端口 ${tunnel.localPort} 已监听` }
      : { ok: false, message: `端口 ${tunnel.localPort} 未监听` }

    // 4. frpc 二进制是否就绪：builtin 厂商始终 ok，外部厂商检查 frpcReady 字段
    const provider = providers.find(p => p.id === tunnel.providerId)
    let frpcReady: CheckEntry
    if (!provider) {
      frpcReady = { ok: false, message: '未找到对应厂商' }
    } else if (provider.builtin) {
      frpcReady = { ok: true, message: '内置厂商已就绪' }
    } else if (provider.frpcReady) {
      frpcReady = { ok: true, message: 'frpc 已就绪' }
    } else {
      frpcReady = { ok: false, message: 'frpc 未就绪' }
    }

    return {
      tunnelId: tunnel.id,
      tunnelName: tunnel.name,
      config,
      serverReachable,
      localPortListening,
      frpcReady,
      overall:
        config.ok &&
        serverReachable.ok &&
        localPortListening.ok &&
        frpcReady.ok,
    }
  }

  // 并发检查所有隧道（allSettled 保证顺序与入参一致，单条失败不影响其他）
  const settled = await Promise.allSettled(tunnels.map(t => checkOne(t)))
  return settled.map((s, i) => {
    if (s.status === 'fulfilled') return s.value
    // checkOne 内部已捕获 tcpCheck 异常，此处兜底防止意外 rejection
    const tunnel = tunnels[i]
    const fail: CheckEntry = { ok: false, message: '检测异常' }
    return {
      tunnelId: tunnel.id,
      tunnelName: tunnel.name,
      config: fail,
      serverReachable: fail,
      localPortListening: fail,
      frpcReady: fail,
      overall: false,
    }
  })
}
