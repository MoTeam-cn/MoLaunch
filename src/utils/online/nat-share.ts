/**
 * NAT 分享算法：序列化/解析 NAT 检测结果 + 双方 NAT 联机可能性判断
 *
 * 分享格式（单行，便于聊天中复制粘贴）：
 * `MoLaunchNATv1|<类型>|<本地IP>|<公网IP>|<STUN列表>|<ICE列表>`
 * - STUN 列表：逗号分隔
 * - ICE 列表：分号分隔，每项 `kind:address:port`（address 支持 IPv6）
 */
import type { NatDetectionResult, NatType, IceCandidateInfo } from '@/types/online'
import { NAT_TYPE_META } from './nat'

/** 分享文本头部标识（解析时校验） */
export const NAT_SHARE_HEADER = 'MoLaunchNATv1'

/** 解析后的分享数据 */
export interface NatShareData {
  type: NatType
  localIp?: string
  publicIp?: string
  stunServers: string[]
  ice: IceCandidateInfo[]
}

const ICE_KINDS: IceCandidateInfo['kind'][] = ['host', 'srflx', 'prflx', 'relay']

/** 序列化 NAT 检测结果为可分享文本 */
export function serializeNatShare(result: NatDetectionResult): string {
  const stun = (result.stunServers ?? []).join(',')
  const ice = (result.ice ?? [])
    .map((c) => `${c.kind}:${c.address}:${c.port}`)
    .join(';')
  return [NAT_SHARE_HEADER, result.type, result.localIp ?? '', result.publicIp ?? '', stun, ice].join('|')
}

/** 解析分享文本，格式非法返回 null */
export function parseNatShare(text: string): NatShareData | null {
  const parts = text.trim().split('|')
  if (parts.length < 2 || parts[0] !== NAT_SHARE_HEADER) return null
  const type = parts[1] as NatType
  if (!NAT_TYPE_META[type]) return null

  const stunServers = (parts[4] ?? '').split(',').filter(Boolean)
  const ice: IceCandidateInfo[] = []
  for (const item of (parts[5] ?? '').split(';')) {
    if (!item) continue
    // kind:address:port，address 可能含冒号（IPv6），用正则从尾部提取端口
    const m = /^(\w+):(.+):(\d+)$/.exec(item)
    if (!m) continue
    const [, kind, address, portStr] = m
    if (!(ICE_KINDS as string[]).includes(kind)) continue
    ice.push({ kind: kind as IceCandidateInfo['kind'], address, port: Number(portStr) })
  }

  return {
    type,
    localIp: parts[2] || undefined,
    publicIp: parts[3] || undefined,
    stunServers,
    ice,
  }
}

/** 联机可能性等级 */
export type P2PLevel = 'high' | 'medium' | 'low' | 'none' | 'unknown'

/** 双方 NAT 联机可能性判断结果 */
export interface P2PVerdict {
  level: P2PLevel
  label: string
  detail: string
}

/**
 * 判断双方 NAT 类型组合的 P2P 联机可能性
 *
 * 规则（参考 RFC 3489 / 5389 与业界 NAT 穿透实践）：
 * - Blocked / Unknown：无法或无法判断
 * - Open / FullCone：可直连任意对端
 * - 双方 Symmetric：无法 P2P，需 TURN 中转
 * - Symmetric + 锥 NAT：仅对称方主动发起时可能成功，成功率低
 * - 锥 NAT 组合：按限制程度递减（RestrictedCone×RestrictedCone 最佳）
 */
export function judgeP2PFeasibility(a: NatType, b: NatType): P2PVerdict {
  if (a === 'Blocked' || b === 'Blocked') {
    return {
      level: 'none',
      label: '无法 P2P',
      detail: '一方 UDP 出站被阻断，无法建立 P2P 连接，需依赖 TURN 中转或更换网络。',
    }
  }
  if (a === 'Unknown' || b === 'Unknown') {
    return {
      level: 'unknown',
      label: '无法判断',
      detail: '一方 NAT 检测失败，无法判断联机可能性，请重新检测后再试。',
    }
  }
  if (a === 'Open' || b === 'Open') {
    return {
      level: 'high',
      label: '可直连',
      detail: '一方为公网直连（无 NAT），可直接建立 P2P 连接，无需任何中转。',
    }
  }
  if (a === 'FullCone' || b === 'FullCone') {
    return {
      level: 'high',
      label: '可直连',
      detail: '一方为全锥 NAT，任意外部主机均可访问其映射端口，可直接建立 P2P 连接。',
    }
  }
  if (a === 'Symmetric' && b === 'Symmetric') {
    return {
      level: 'none',
      label: '无法 P2P',
      detail: '双方均为对称 NAT，映射端口不固定，无法建立 P2P 连接，需依赖 TURN 中转。',
    }
  }
  if (a === 'Symmetric' || b === 'Symmetric') {
    return {
      level: 'low',
      label: '较难 P2P',
      detail: '一方为对称 NAT，仅当对称方主动发起连接且对方为锥 NAT 时可能建立 P2P，成功率较低。',
    }
  }
  if (a === 'RestrictedCone' && b === 'RestrictedCone') {
    return {
      level: 'high',
      label: '可 P2P',
      detail: '双方均为限制锥 NAT，通过 STUN 交换反射地址即可建立 P2P 连接。',
    }
  }
  if (a === 'PortRestrictedCone' && b === 'PortRestrictedCone') {
    return {
      level: 'medium',
      label: '可尝试 P2P',
      detail: '双方均为端口限制锥 NAT，需双方上报 ICE candidate 后建立连接，成功率一般。',
    }
  }
  return {
    level: 'medium',
    label: '可尝试 P2P',
    detail: '限制锥与端口限制锥组合，通过 STUN 交换反射地址可建立 P2P，端口限制方需先发起连接。',
  }
}