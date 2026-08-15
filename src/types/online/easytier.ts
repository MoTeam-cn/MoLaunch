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

/** 生成 Scaffolding 格式房间码（U/NNNN-NNNN-SSSS-SSSS） */
export function generateScaffoldingCode(): string {
  const segment = () => {
    let s = ''
    for (let i = 0; i < 4; i++) s += Math.floor(Math.random() * 10).toString()
    return s
  }
  const n1 = segment()
  const n2 = segment()
  const s1 = segment()
  const s2 = segment()
  return `U/${n1}-${n2}-${s1}-${s2}`
}

/** 解析 Scaffolding 房间码；格式非法时返回 null */
export function parseScaffoldingCode(code: string): ScaffoldingRoomCode | null {
  const m = /^U\/(\d{4})-(\d{4})-(\d{4})-(\d{4})$/.exec(code.trim().toUpperCase())
  if (!m) return null
  return {
    full: `U/${m[1]}-${m[2]}-${m[3]}-${m[4]}`,
    publicId: `${m[1]}-${m[2]}`,
    secret: `${m[3]}-${m[4]}`,
  }
}
