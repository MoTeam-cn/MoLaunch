/**
 * MC 局域网伪装 / 端口探测类型定义（easytier 方案配套 UX）
 */

/** `lan_fake_server_start` 参数 */
export interface LanFakeStartParams {
  /** 多人游戏界面显示的服务器名称（MOTD） */
  motd: string
  /** 转发目标 IP（房主 easytier 虚拟 IP，缺省 10.244.0.1） */
  targetIp?: string
  /** 转发目标端口（房主 MC 局域网端口） */
  targetPort: number
}

/** `lan_fake_server_start` 返回 */
export interface LanFakeStartResult {
  success: boolean
  /** 实际监听的本地端口 */
  port: number
}

/** `lan_port_probe` 返回 */
export interface LanPortProbeResult {
  success: boolean
  /** 解析出的 MC 局域网端口（0 = 未检测到） */
  port: number
  /** 广播中的 MOTD 文本 */
  motd: string
  error: string
}

/** `get_running_mc_port` 返回 */
export interface RunningMcPortResult {
  success: boolean
  /** 当前游戏进程监听的 MC 局域网候选端口（升序） */
  ports: number[]
}
