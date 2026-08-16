/**
 * easytier 虚拟组网 + Scaffolding 联机中心类型定义
 *
 * 房间码格式 `U/NNNN-NNNN-SSSS-SSSS`（Scaffolding）：
 * - N 段：公开标识，大厅展示/搜索用，不泄露密钥
 * - S 段：虚拟网络密钥，仅房主/加入方本地持有
 */

/** 房主 easytier 固定虚拟 IP */
export const EASYTIER_HOST_VIRTUAL_IP = '10.244.0.1'

/** Scaffolding 联机中心默认端口 */
export const SCAFFOLDING_CENTER_PORT = 13448

/** 虚拟网络名前缀（`scaffolding-mc-{N 段}`） */
export const EASYTIER_NETWORK_PREFIX = 'scaffolding-mc-'

/** `easytier_join` 参数 */
export interface EasyTierJoinParams {
  networkName: string
  networkSecret: string
  /** 是否房主（房主固定虚拟 IP，房客 --dhcp） */
  isHost?: boolean
  /** 节点 hostname（房主必须为 `scaffolding-mc-server-{mc_port}`） */
  hostname?: string
  /** 追加 easytier-core CLI 参数 */
  extra?: string[]
}

/** `easytier_join` / `easytier_stop` 返回 */
export interface EasyTierJoinResult {
  success: boolean
  /** rpc-portal 地址（如 `127.0.0.1:12345`） */
  rpcPortal: string
  /** easytier-core 子进程 PID */
  pid?: number
  /** easytier-core 版本号（`--version` 查询失败时为空串） */
  version?: string
}

/** `easytier_status` 查询返回 / `easytier-status` 事件 payload */
export interface EasyTierStatusResult {
  /** 是否已加入虚拟网络 */
  joined: boolean
  /** easytier-core 版本号 */
  version: string
  /** 子进程 PID（未运行时为空） */
  pid?: number
  /** rpc-portal 地址（未运行时为空串） */
  rpcPortal: string
  /** 虚拟网络名（scaffolding-mc-{N 段}，未运行时为空串） */
  networkName: string
  /** 本机虚拟 IP（房主固定 10.244.0.1；房客 DHCP 未回显时为空串） */
  virtualIp: string
}

/** `scaffolding_host_start` 参数 */
export interface ScaffoldingHostStartParams {
  /** 完整房间码 `U/NNNN-NNNN-SSSS-SSSS` */
  roomCode: string
  /** 房主 MC 局域网端口（缺省按游戏进程探测） */
  mcPort?: number
}

/** `scaffolding_host_start` 返回 */
export interface ScaffoldingHostStartResult {
  success: boolean
  /** 联机中心实际监听端口 */
  centerPort: number
  /** 中心 hostname（`scaffolding-mc-server-{mc_port}`） */
  hostname: string
  /** 房主 MC 局域网端口 */
  mcPort: number
  rpcPortal: string
  pid?: number
}

/** `scaffolding_client_probe` 参数 */
export interface ScaffoldingClientProbeParams {
  /** 完整房间码 `U/NNNN-NNNN-SSSS-SSSS` */
  roomCode: string
  /** 联机中心虚拟 IP（缺省取房主固定 10.244.0.1） */
  centerIp?: string
  /** 联机中心 TCP 端口（缺省 13448） */
  centerPort?: number
}

/** `scaffolding_client_probe` 返回 */
export interface ScaffoldingClientProbeResult {
  success: boolean
  /** 房主虚拟 IP（MC 客户端连接目标，配合 lan_fake 转发） */
  mcIp: string
  /** 房主 MC 局域网端口 */
  mcPort: number
}

/** Scaffolding 房间码解析结果 */
export interface ScaffoldingRoomCode {
  /** 完整房间码（U/NNNN-NNNN-SSSS-SSSS） */
  full: string
  /** N 段（公开标识） */
  publicId: string
  /** S 段（组网密钥） */
  secret: string
}

/** 房间码字符集（与后端一致，剔除易混淆的 I / O，保留 L；与 Terracotta 标准一致） */
export const SCAFFOLDING_CODE_CHARSET = '0123456789ABCDEFGHJKLMNPQRSTUVWXYZ'

/** 生成符合后端校验规则的房间码：16 位字符，base-34 小端序整型可被 7 整除 */
export function generateScaffoldingCode(): string {
  let chars = ''
  do {
    chars = ''
    for (let i = 0; i < 16; i++) {
      chars += SCAFFOLDING_CODE_CHARSET[Math.floor(Math.random() * SCAFFOLDING_CODE_CHARSET.length)]
    }
  } while (!validateScaffoldingChecksum(chars))
  const g = (i: number) => chars.slice(i, i + 4)
  return `U/${g(0)}-${g(4)}-${g(8)}-${g(12)}`
}

/** base-34 小端序模 7 校验（与 src-tauri code.rs 一致，生成时使用） */
function validateScaffoldingChecksum(chars: string): boolean {
  if (chars.length !== 16) return false
  const baseMod = SCAFFOLDING_CODE_CHARSET.length % 7
  let pow = 1
  let acc = 0
  for (const c of chars) {
    const v = SCAFFOLDING_CODE_CHARSET.indexOf(c)
    if (v < 0) return false
    acc = (acc + (v % 7) * pow) % 7
    pow = (pow * baseMod) % 7
  }
  return acc === 0
}

/** 解析 Scaffolding 房间码；格式非法时返回 null（仅格式校验，不查校验和，兼容官方字符集） */
export function parseScaffoldingCode(code: string): ScaffoldingRoomCode | null {
  const trimmed = code.trim()
  if (!/^U\//i.test(trimmed)) return null
  const body = trimmed.slice(2).toUpperCase()
  if (body.length !== 19) return null
  if (body[4] !== '-' || body[9] !== '-' || body[14] !== '-') return null
  const parts = body.split('-')
  if (parts.length !== 4) return null
  for (const part of parts) {
    if (part.length !== 4) return null
    for (const c of part) {
      if (!SCAFFOLDING_CODE_CHARSET.includes(c)) return null
    }
  }
  return {
    full: `U/${parts[0]}-${parts[1]}-${parts[2]}-${parts[3]}`,
    publicId: `${parts[0]}-${parts[1]}`,
    secret: `${parts[2]}-${parts[3]}`,
  }
}

/** 由 N 段派生虚拟网络名（scaffolding-mc-{N 段}） */
export function scaffoldingNetworkName(publicId: string): string {
  return `${EASYTIER_NETWORK_PREFIX}${publicId}`
}
