/**
 * 联机 API - TUN 桥接管理（阶段三子任务 5：数据分发打通）
 *
 * 3 个 action 与后端 `utils::tun_manager` 注册一一对应。
 * 数据流：
 * - 后端 TUN 读包 → emit `online://tun-packet-out` 事件 → 前端 listen → DataChannel.send
 * - 前端 DataChannel.onmessage → ArrayBuffer → base64 → invoke `tun_forward_to` → 写入 TUN
 */

import type {
  LanFakeStartParams,
  LanFakeStartResponse,
  LanPortProbeParams,
  LanPortProbeResponse,
  RunningMcPortResponse,
  TunForwardResponse,
  TunStartParams,
  TunStartResponse,
} from '@/types/online'
import { ONLINE_ACTIONS, onlineManager } from './core'

/**
 * 启动 TUN 桥接（房主与加入方通用）
 *
 * 调用后后端会：
 * 1. 若已有 bridge，先停止（防止泄漏）
 * 2. 创建 TUN 接口（绑定 ipv4/prefix_len）
 * 3. 启动 select! 单读写循环 task
 * 4. emit `online://tun-packet-out` 事件给前端（每读到 IP 包就 emit）
 *
 * @param params 含虚拟 IP 与子网前缀长度
 * @returns 接口信息（接口名 / IP / 前缀长度 / MTU）
 */
export function tunStart(params: TunStartParams): Promise<TunStartResponse> {
  return onlineManager<TunStartResponse>(ONLINE_ACTIONS.TUN_START, params)
}

/**
 * 将 DataChannel 收到的消息转发到后端 TUN
 *
 * 前端从 `DataChannel.onmessage` 拿到 `ArrayBuffer` 后，转 base64 调用此函数。
 * 后端 base64 解码 → 协议帧 decode → 写入 TUN 接口。
 *
 * @param dataChannelMessage DataChannel 收到的二进制消息（协议帧编码后的字节）
 * @returns 是否为数据包 + IP 包字节数
 */
export function tunForwardTo(
  dataChannelMessage: ArrayBuffer | Uint8Array,
): Promise<TunForwardResponse> {
  // ArrayBuffer / Uint8Array → base64 字符串
  const bytes = dataChannelMessage instanceof Uint8Array
    ? dataChannelMessage
    : new Uint8Array(dataChannelMessage)
  // 分块处理避免 apply 参数上限（DataChannel 消息一般 < 64KB，但稳妥起见分块）
  let binary = ''
  const chunkSize = 0x8000
  for (let i = 0; i < bytes.length; i += chunkSize) {
    binary += String.fromCharCode.apply(null, bytes.subarray(i, i + chunkSize) as unknown as number[])
  }
  const messageBase64 = btoa(binary)
  return onlineManager<TunForwardResponse>(ONLINE_ACTIONS.TUN_FORWARD_TO, {
    messageBase64,
  })
}

/** 停止 TUN 桥接，销毁 TUN 接口（幂等） */
export function tunStop(): Promise<{ success: boolean }> {
  return onlineManager<{ success: boolean }>(ONLINE_ACTIONS.TUN_STOP)
}

/**
 * 启动 MC 局域网服务器伪装（加入方调用）
 *
 * 加入方本地起 TCP 转发代理 + 周期 UDP 广播，本机 MC 客户端在多人游戏界面
 * 即可直接发现房主房间，点击进入时经代理转发到房主虚拟 IP:MC 端口（走 TUN）。
 *
 * @param params 伪装名称与转发目标（房主虚拟 IP:房主 MC 端口）
 * @returns 实际监听的本地端口
 */
export function lanFakeServerStart(
  params: LanFakeStartParams,
): Promise<LanFakeStartResponse> {
  return onlineManager<LanFakeStartResponse>(ONLINE_ACTIONS.LAN_FAKE_SERVER_START, params)
}

/** 停止 MC 局域网服务器伪装（幂等） */
export function lanFakeServerStop(): Promise<{ success: boolean }> {
  return onlineManager<{ success: boolean }>(ONLINE_ACTIONS.LAN_FAKE_SERVER_STOP)
}

/**
 * 监听 MC 局域网发现广播并解析端口（与多人游戏发现房间同源）
 *
 * 后端绑定 UDP 4445 并加入多播组 224.0.2.60，等待 MC 服务器周期广播的
 * `[AD]port[/AD]`。房主可探测本机服务器实际端口；加入方可探测本地伪装代理端口。
 *
 * @param params 监听时长（毫秒），默认 6000
 * @returns 检测到的端口与广播 MOTD；超时未检测到时 success=false
 */
export function lanPortProbe(params: LanPortProbeParams): Promise<LanPortProbeResponse> {
  return onlineManager<LanPortProbeResponse>(ONLINE_ACTIONS.LAN_PORT_PROBE, params)
}

/**
 * 按当前游戏进程 PID 扫描监听端口，回查 MC 局域网候选端口（进房时调用）
 *
 * 先启动 MC（已开放局域网）再开房间时，watcher 的端口事件在监听注册前发出
 * 已被丢弃且不会重发，进房后主动回查补上；取 `ports` 最后一项作为当前端口。
 *
 * @returns 当前游戏进程监听的候选端口列表（升序）；空表示未开放局域网或游戏非本启动器启动
 */
export function getRunningMcPort(): Promise<RunningMcPortResponse> {
  return onlineManager<RunningMcPortResponse>(ONLINE_ACTIONS.GET_RUNNING_MC_PORT)
}

/**
 * 以管理员权限重启启动器
 *
 * `tun_start` 返回 `TUN_PERMISSION_DENIED:` 前缀错误时，前端经用户确认后调用。
 * - release 模式：后端通过 `ShellExecuteW("runas")` 触发 UAC 提权并延迟退出当前进程
 * - dev 模式：后端返回 `dev_mode: true` 不重启（避免丢失 Vite dev server 连接），
 *   前端应提示用户用管理员权限终端运行 `npm run tauri dev`
 */
export interface RestartAsAdminResult {
  success: boolean
  /** dev 模式下为 true，表示未自动重启，需提示用户手动以管理员权限启动 */
  dev_mode?: boolean
  /** dev 模式下的提示文案 */
  message?: string
}

export function restartAsAdmin(): Promise<RestartAsAdminResult> {
  return onlineManager<RestartAsAdminResult>(ONLINE_ACTIONS.RESTART_AS_ADMIN)
}
