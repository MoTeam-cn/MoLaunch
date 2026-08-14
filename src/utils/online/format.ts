import type { NatType } from '@/types/online'

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