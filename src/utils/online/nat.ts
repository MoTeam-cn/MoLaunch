/**
 * NAT 类型检测工具 + Tooltip 提示文案
 *
 * 基于 WebRTC ICE candidates 的 srflx/host 类型组合判断 NAT 类型，
 * 展示文案/颜色映射见文件内 NAT_TYPE_META。
 */
import type { NatDetectionResult, NatType } from '@/types/online'

/**
 * 解析 ICE candidate 字符串中的 candidate 类型
 *
 * ICE candidate 格式（RFC 8445）：
 * `candidate:<foundation> <component> <protocol> <priority> <addr> <port> typ <type> [raddr <raddr> rport <rport>]`
 *
 * typ 字段取值：
 * - `host`：本地接口地址（如 192.168.x.x）
 * - `srflx`：STUN 反射地址（公网 IP）
 * - `prflx`：对端反射地址（少见）
 * - `relay`：TURN 中继地址
 */
export function parseCandidateType(candidate: string): 'host' | 'srflx' | 'prflx' | 'relay' | null {
  const match = /typ\s+(\w+)/.exec(candidate)
  if (!match) return null
  const t = match[1]
  if (t === 'host' || t === 'srflx' || t === 'prflx' || t === 'relay') return t
  return null
}

/** 从 candidate 字符串中解析地址（host/srflx 的连接地址） */
export function parseCandidateAddress(candidate: string): string | null {
  // 跳过 "candidate:" 前缀，按空格切分
  // 字段顺序：foundation component protocol priority address port typ ...
  const parts = candidate.replace(/^candidate:/i, '').split(/\s+/)
  if (parts.length < 5) return null
  return parts[4] ?? null
}

/**
 * 根据 ICE candidate 列表推断 NAT 类型
 *
 * 算法：
 * 1. 收集所有 host / srflx candidate
 * 2. 无 host 且无 srflx → Unknown
 * 3. 有 host 但无 srflx → Blocked（UDP 出站被拦截）
 * 4. 1 个 srflx（仅一个公网反射地址）→ FullCone / RestrictedCone / PortRestrictedCone
 *    纯前端无法精确区分，默认归类为 RestrictedCone（最常见场景）
 * 5. 多个 srflx 且端口不同 → Symmetric（不同 STUN 服务器返回不同端口）
 *
 * @param candidates ICE candidate 字符串数组（来自 RTCPeerConnection.onicecandidate）
 */
export function detectNatType(candidates: string[]): NatDetectionResult {
  const start = performance.now()

  const hostCandidates: string[] = []
  const srflxCandidates: { address: string; port: number }[] = []

  console.log(`[NAT] 收集到 ${candidates.length} 个 candidate，开始推断 NAT 类型`)
  for (const c of candidates) {
    const type = parseCandidateType(c)
    if (!type) continue
    const addr = parseCandidateAddress(c)
    if (!addr) continue

    if (type === 'host') {
      hostCandidates.push(addr)
    } else if (type === 'srflx') {
      // 从 candidate 中提取端口（address 后紧跟的字段）
      const parts = c.replace(/^candidate:/i, '').split(/\s+/)
      const port = Number(parts[5])
      if (!Number.isNaN(port)) {
        srflxCandidates.push({ address: addr, port })
      }
    }
  }
  console.log(
    `[NAT] host=${hostCandidates.length} 个, srflx=${srflxCandidates.length} 个 (${srflxCandidates.map((c) => `${c.address}:${c.port}`).join(', ') || '无'})`,
  )

  let type: NatType
  let publicIp: string | undefined
  let localIp: string | undefined

  if (hostCandidates.length === 0 && srflxCandidates.length === 0) {
    type = 'Unknown'
  } else if (srflxCandidates.length === 0) {
    type = 'Blocked'
    localIp = hostCandidates[0]
  } else {
    // 有 srflx，取第一个作为公网 IP
    publicIp = srflxCandidates[0].address
    localIp = hostCandidates[0]

    if (srflxCandidates.length > 1) {
      // 多个 srflx：检查端口是否相同
      const ports = new Set(srflxCandidates.map((c) => c.port))
      if (ports.size > 1) {
        type = 'Symmetric'
      } else {
        // 端口相同 → Cone NAT（默认 RestrictedCone）
        type = 'RestrictedCone'
      }
    } else {
      // 单个 srflx → Cone NAT，默认 RestrictedCone
      type = 'RestrictedCone'
    }
  }

  return {
    type,
    durationMs: Math.round(performance.now() - start),
    localIp,
    publicIp,
  }
}

/**
 * 同步检测 NAT 类型（创建临时 PeerConnection 收集 candidate）
 *
 * 实现思路：
 * 1. 创建 RTCPeerConnection，配置 Google 公共 STUN 服务器
 * 2. createDataChannel + createOffer + setLocalDescription 触发 ICE 收集
 * 3. 监听 onicecandidate 收集所有 candidate（iceGatheringState === 'complete' 后停止）
 * 4. 调用 detectNatType 推断类型
 *
 * @param stunServers STUN 服务器列表（默认 Google STUN）
 * @param timeoutMs 超时时间（默认 8 秒，足够收集 candidate）
 */
export function detectNatTypeWithStun(
  stunServers: string[] = ['stun:stun.l.google.com:19302', 'stun:stun1.l.google.com:19302'],
  timeoutMs = 8000,
): Promise<NatDetectionResult> {
  return new Promise((resolve) => {
    if (typeof RTCPeerConnection === 'undefined') {
      resolve({
        type: 'Unknown',
        durationMs: 0,
        error: '当前环境不支持 RTCPeerConnection',
      })
      return
    }

    const start = performance.now()
    const candidates: string[] = []
    let settled = false

    console.log(`[NAT] 开始检测：使用 ${stunServers.length} 个 STUN 服务器: ${stunServers.join(', ')}，超时 ${timeoutMs}ms`)

    const pc = new RTCPeerConnection({
      iceServers: stunServers.map((url) => ({ urls: url })),
    })

    const finish = (type: NatType, error?: string) => {
      if (settled) return
      settled = true
      try {
        pc.close()
      } catch {
        /* ignore */
      }
      if (error) {
        console.log(`[NAT] 检测结束（异常）: type=${type}, error=${error}`)
        resolve({
          type,
          durationMs: Math.round(performance.now() - start),
          error,
        })
      } else {
        // 复用 detectNatType 的解析逻辑
        const result = detectNatType(candidates)
        console.log(`[NAT] 检测完成: type=${result.type}, 耗时 ${result.durationMs}ms`)
        resolve(result)
      }
    }

    // 超时兜底
    const timer = setTimeout(() => {
      console.log(`[NAT] 超时触发（${timeoutMs}ms），已收集 ${candidates.length} 个 candidate`)
      if (candidates.length === 0) {
        finish('Unknown', `ICE 收集超时（${timeoutMs}ms 内无 candidate）`)
      } else {
        // 用已有 candidate 推断
        const result = detectNatType(candidates)
        console.log(`[NAT] 超时后按已有 candidate 推断: type=${result.type}`)
        settled = true
        try {
          pc.close()
        } catch {
          /* ignore */
        }
        resolve(result)
      }
    }, timeoutMs)

    // 创建 DataChannel 触发 ICE（无 MediaStream 时必需）
    try {
      pc.createDataChannel('nat-detect')
    } catch {
      /* ignore */
    }

    pc.onicecandidate = (event) => {
      if (event.candidate && event.candidate.candidate) {
        candidates.push(event.candidate.candidate)
        console.log(`[NAT] ICE candidate #${candidates.length}: ${event.candidate.candidate}`)
      }
    }

    pc.onicegatheringstatechange = () => {
      console.log(`[NAT] ICE gathering 状态: ${pc.iceGatheringState}（已收集 ${candidates.length} 个 candidate）`)
      if (pc.iceGatheringState === 'complete') {
        clearTimeout(timer)
        if (candidates.length === 0) {
          finish('Unknown', 'ICE 收集完成但未获取任何 candidate')
        } else {
          const result = detectNatType(candidates)
          console.log(`[NAT] 收集完成，推断结果: type=${result.type}`)
          settled = true
          try {
            pc.close()
          } catch {
            /* ignore */
          }
          resolve(result)
        }
      }
    }

    pc.createOffer()
      .then((offer) => {
        console.log('[NAT] createOffer 成功，调用 setLocalDescription 触发 ICE 收集')
        return pc.setLocalDescription(offer)
      })
      .catch((e) => {
        clearTimeout(timer)
        finish('Unknown', `createOffer/setLocalDescription 失败: ${String(e)}`)
      })
  })
}

/** NAT 类型元数据（展示文案 + Tooltip 解释 + 联机可行性） */
export interface NatTypeMeta {
  /** 中文显示名 */
  label: string
  /** 颜色主题（用于徽章） */
  color: 'green' | 'blue' | 'yellow' | 'red' | 'gray'
  /** Tooltip 详细说明 */
  tooltip: string
  /** 联机可行性：high / medium / low / none */
  feasibility: 'high' | 'medium' | 'low' | 'none'
}

/** 各 NAT 类型的元数据 */
export const NAT_TYPE_META: Record<NatType, NatTypeMeta> = {
  Open: {
    label: '公网直连',
    color: 'green',
    feasibility: 'high',
    tooltip:
      '当前设备直接暴露在公网，无 NAT。\n联机兼容性：极佳，可作为房主或加入方，无需任何中转。\n适用场景：服务器、云主机、DMZ 主机。',
  },
  FullCone: {
    label: '全锥 NAT',
    color: 'green',
    feasibility: 'high',
    tooltip:
      '任意外部主机可通过映射端口访问本机。\n联机兼容性：极佳，房主与加入方均可建立 P2P 连接，无需 TURN 中转。\n典型场景：家用路由器「端口映射」「DMZ 主机」开启后。',
  },
  RestrictedCone: {
    label: '限制锥 NAT',
    color: 'blue',
    feasibility: 'high',
    tooltip:
      '仅允许本机主动联系过的外部 IP 访问映射端口。\n联机兼容性：良好，房主与加入方通过 STUN 即可建立 P2P，无需 TURN 中转。\n典型场景：家用路由器默认模式（多数家庭网络）。',
  },
  PortRestrictedCone: {
    label: '端口限制锥 NAT',
    color: 'blue',
    feasibility: 'medium',
    tooltip:
      '仅允许本机主动联系过的外部 IP:Port 访问映射端口。\n联机兼容性：可用，但需房主与加入方均上报 ICE candidate 后才能建立连接。\n典型场景：企业网络、校园网、部分家用路由器。',
  },
  Symmetric: {
    label: '对称 NAT',
    color: 'yellow',
    feasibility: 'low',
    tooltip:
      '每个目标地址分配独立映射端口，STUN 反射地址不固定。\n联机兼容性：较差，仅当房主为锥 NAT 时可作为加入方接入；若双方均为对称 NAT 则无法建立 P2P，需依赖 TURN 中转。\n典型场景：4G/5G 移动网络、运营商 CGNAT、部分企业 NAT。',
  },
  Blocked: {
    label: 'UDP 阻断',
    color: 'red',
    feasibility: 'none',
    tooltip:
      '检测到本地候选地址但无法获取任何 STUN 反射地址，UDP 出站被阻断。\n联机兼容性：无法建立 P2P，需依赖 TURN 中转或更换网络。\n典型场景：防火墙严格的企业网络、部分校园网。',
  },
  Unknown: {
    label: '检测失败',
    color: 'gray',
    feasibility: 'none',
    tooltip:
      'WebRTC 未收集到任何 ICE candidate，或浏览器不支持 RTCPeerConnection。\n联机兼容性：未知，请检查浏览器或网络环境。',
  },
}

/** 解析 NAT 类型元数据（未知/空值回退 null，兼容协议上报的原始字符串） */
export function resolveNatMeta(type: string | null | undefined): NatTypeMeta | null {
  if (!type) return null
  return NAT_TYPE_META[type as NatType] ?? null
}

/** 根据可行性等级返回徽章颜色 class（Tailwind） */
export function getNatFeasibilityColorClass(feasibility: NatTypeMeta['feasibility']): string {
  switch (feasibility) {
    case 'high':
      return 'bg-green-50 text-green-700'
    case 'medium':
      return 'bg-blue-50 text-blue-700'
    case 'low':
      return 'bg-yellow-50 text-yellow-700'
    case 'none':
      return 'bg-red-50 text-red-700'
    default:
      return 'bg-gray-100 text-gray-600'
  }
}
