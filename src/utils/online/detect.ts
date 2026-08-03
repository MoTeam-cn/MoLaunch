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
 * 算法（参考 PCL2 与业界同类实现）：
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
        resolve({
          type,
          durationMs: Math.round(performance.now() - start),
          error,
        })
      } else {
        // 复用 detectNatType 的解析逻辑
        const result = detectNatType(candidates)
        resolve(result)
      }
    }

    // 超时兜底
    const timer = setTimeout(() => {
      if (candidates.length === 0) {
        finish('Unknown', `ICE 收集超时（${timeoutMs}ms 内无 candidate）`)
      } else {
        // 用已有 candidate 推断
        const result = detectNatType(candidates)
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
      }
    }

    pc.onicegatheringstatechange = () => {
      if (pc.iceGatheringState === 'complete') {
        clearTimeout(timer)
        if (candidates.length === 0) {
          finish('Unknown', 'ICE 收集完成但未获取任何 candidate')
        } else {
          const result = detectNatType(candidates)
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
      .then((offer) => pc.setLocalDescription(offer))
      .catch((e) => {
        clearTimeout(timer)
        finish('Unknown', `createOffer/setLocalDescription 失败: ${String(e)}`)
      })
  })
}