/**
 * MC 局域网伪装 / 端口探测类型定义（easytier 方案配套 UX）
 */

/** `lan_fake_server_start` 参数 */
export interface LanFakeStartParams {
  /** 多人游戏界面显示的服务器名称（MOTD） */
  motd: string
  /** 进服端口（本地 port-forward 端口，MC 客户端连接 127.0.0.1:port） */
  port: number
}

/** `lan_fake_server_start` 返回 */
export interface LanFakeStartResult {
  success: boolean
  /** 广播进服端口 */
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
