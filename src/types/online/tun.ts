/**
 * 联机功能类型定义 - TUN 桥接域（阶段三子任务 5：数据分发打通）
 *
 * 后端 tun_manager 注册 tun_start（建接口 + 读写循环 + emit tun-packet-out）、
 * tun_forward_to（base64 编码写入 TUN）、tun_stop（销毁接口）三个 action。
 */

/** `tun_start` 参数 */
export interface TunStartParams {
  /** 虚拟 IPv4 地址（如 `10.244.1.1`） */
  ipv4: string
  /** 子网前缀长度（如 24，对应 `10.244.1.0/24`） */
  prefixLen: number
}

/** `tun_start` 返回 */
export interface TunStartResponse {
  /** TUN 接口名（如 `tun-molaunch`） */
  interfaceName: string
  /** 虚拟 IP */
  ipv4: string
  /** 子网前缀长度 */
  prefixLen: number
  /** MTU */
  mtu: number
}

/** `tun_forward_to` 返回 */
export interface TunForwardResponse {
  /** 是否为数据包（true=已写入 TUN，false=控制/错误消息，未写入） */
  isData: boolean
  /** 解码出的 IP 包字节数（控制消息为 0） */
  packetLen: number
}

/** 后端 emit 给前端的 TUN 数据包事件名 */
export const EVENT_TUN_PACKET_OUT = 'online://tun-packet-out'

/** TUN 数据包事件 payload（后端 emit 的 `Vec<u8>` 协议帧，Tauri 序列化为 number[]） */
export type TunPacketPayload = number[]

/** `lan_fake_server_start` 参数 */
export interface LanFakeStartParams {
  /** 多人游戏界面显示的服务器名称（MOTD） */
  motd: string
  /** 转发目标 IP（房主虚拟 IP，走 TUN 路由） */
  targetIp: string
  /** 转发目标端口（房主 MC 局域网端口） */
  targetPort: number
}

/** `lan_fake_server_start` 返回 */
export interface LanFakeStartResponse {
  success: boolean
  /** 实际监听的本地端口（MC 客户端将连接本机该端口） */
  port: number
}

/** `lan_port_probe` 参数 */
export interface LanPortProbeParams {
  /** 监听时长（毫秒），默认 6000 */
  timeoutMs?: number
}

/** `lan_port_probe` 返回 */
export interface LanPortProbeResponse {
  success: boolean
  /** 解析出的 MC 局域网端口（0 = 未检测到） */
  port: number
  /** 广播中的 MOTD 文本 */
  motd: string
  /** 失败原因（成功时为空） */
  error: string
}

/** `get_running_mc_port` 返回 */
export interface RunningMcPortResponse {
  success: boolean
  /** 当前游戏进程监听的 MC 局域网候选端口（升序；空=未开放局域网或游戏非本启动器启动） */
  ports: number[]
}
