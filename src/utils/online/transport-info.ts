/**
 * WebRTC 实际传输方式检测（P2P 直连 / TURN 中继）
 *
 * 通过 getStats 检查已选 candidate-pair：localCandidate 为 relay 时判定走
 * TURN 中继，并用 ICE entry 的 URL host 匹配 regionCode/name 用于国旗展示。
 * 纯函数，不持有响应式状态。
 */

import type { IceServerEntry } from '@/types/online'

/** 实际传输方式信息 */
export interface TransportInfo {
  /** direct=P2P 直连；relay=TURN 中继 */
  mode: 'direct' | 'relay'
  /** 中继节点区域国旗代码（仅 relay，可能缺省） */
  regionCode?: string
  /** 中继节点名称（仅 relay，可能缺省） */
  name?: string
}

/** 从 stun/turn URL 中提取 host（忽略协议/端口/query） */
function hostFromUrl(url: string): string {
  return url
    .replace(/^(turn|turns|stun|stuns):\/\//i, '')
    .split('?')[0]
    .split(':')[0]
    .toLowerCase()
}

/** candidate stats 最小结构（lib.dom 未提供 RTCIceCandidateStats 具名类型） */
interface CandidateStatsLike {
  candidateType?: string
  url?: string
}

/**
 * 检测单个 PeerConnection 的实际传输方式
 *
 * 仅对 `connectionState === 'connected'` 的连接判定（其他状态返回 null）；
 * 遍历 stats 找到选中（succeeded/in-progress）的 candidate-pair，
 * 其 localCandidate 为 relay 时表示正在走 TURN 中继。
 *
 * @param pc 目标 PeerConnection（可能为 null）
 * @param iceServers ICE 服务器条目（用于把中继 URL 匹配到 regionCode/name）
 * @returns null=未连接/无有效 pair；否则返回实际传输方式
 */
export async function detectTransportInfo(
  pc: RTCPeerConnection | null,
  iceServers: IceServerEntry[],
): Promise<TransportInfo | null> {
  if (!pc || pc.connectionState !== 'connected') return null
  let stats: RTCStatsReport
  try {
    stats = await pc.getStats()
  } catch {
    return null
  }
  for (const stat of stats.values()) {
    if (stat.type !== 'candidate-pair') continue
    const pair = stat as RTCIceCandidatePairStats
    if (pair.state !== 'succeeded' && pair.state !== 'in-progress') continue
    const local = stats.get(pair.localCandidateId) as CandidateStatsLike | undefined
    if (!local || local.candidateType !== 'relay') {
      return { mode: 'direct' }
    }
    const url = local.url ?? ''
    const host = hostFromUrl(url)
    for (const entry of iceServers) {
      if (entry.urls.some((u) => hostFromUrl(u) === host)) {
        return { mode: 'relay', regionCode: entry.regionCode, name: entry.name }
      }
    }
    return { mode: 'relay' }
  }
  return null
}
