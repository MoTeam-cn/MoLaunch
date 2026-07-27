/**
 * WebRTC 共享工具（阶段三子任务 5 抽取，子任务 7 扩展 ICE/TURN）
 *
 * 房主侧 `useWebRTCMesh.ts` 与加入方侧 `useWebRTC.ts` 共用的底层函数：
 * - `createPeerConnection`：构造 RTCPeerConnection（接受 `IceServerEntry[]`，含 STUN + TURN 凭据）
 * - `collectIceCandidates`：等待 ICE 收集完成（非 trickle 模式，一次性返回全部 candidate）
 * - `createDataChannel`：在指定 PC 上创建 DataChannel
 * - `setupDataChannelHandlers`：统一绑定 onopen/onmessage/onerror/onclose
 * - `stunUrlsToIceServers` / `resolveIceServers` / `buildIceServers`：ICE 服务器列表构造与回退
 *
 * 设计约束：
 * - 不持有任何响应式状态，纯函数便于复用与单测
 * - 不引入业务概念（participantId 等），由上层 composable 维护映射
 */

import type { IceServerEntry } from '@/types/online'

/** WebRTC 连接状态（与 RTCPeerConnection.connectionState 对齐） */
export type WebRtcConnectionState =
  | 'new'
  | 'connecting'
  | 'connected'
  | 'disconnected'
  | 'failed'
  | 'closed'

/** ICE 收集结果 */
export interface IceCollectionResult {
  /** SDP 描述字符串（setLocalDescription 后的 localDescription.sdp） */
  sdp: string
  /** 收集到的 ICE candidate 字符串数组 */
  iceCandidates: string[]
}

/** DataChannel 事件处理器（按需传入，未传的字段不绑定） */
export interface DataChannelHandlers {
  /** 通道打开 */
  onOpen?: () => void
  /** 收到二进制消息（ArrayBuffer） */
  onMessage?: (data: ArrayBuffer) => void
  /** 通道错误 */
  onError?: (event: Event) => void
  /** 通道关闭 */
  onClose?: () => void
}

/** DataChannel 通道名（房主创建、加入方接收，名称必须一致） */
export const P2P_DATA_CHANNEL_LABEL = 'molaunch-p2p'

/** DataChannel 配置：UDP 语义（ordered=false + maxRetransmits=0） */
export const P2P_DATA_CHANNEL_OPTIONS: RTCDataChannelInit = {
  ordered: false,
  maxRetransmits: 0,
}

/**
 * 将 STUN URL 字符串数组转换为 IceServerEntry 数组
 *
 * 用于向后兼容：旧客户端仅传 `stunServers: string[]`，新代码统一使用 `IceServerEntry`。
 * STUN 不需要用户名/凭据，仅 `urls` 字段填充。
 *
 * @param urls STUN 服务器 URL 数组（如 `['stun:stun.l.google.com:19302']`）
 */
export function stunUrlsToIceServers(urls: string[]): IceServerEntry[] {
  if (!urls.length) return []
  return [{ urls: [...urls] }]
}

/**
 * 从房间响应中解析最终使用的 ICE 服务器列表
 *
 * 优先级：`iceServers`（非空）→ `stunServers`（回退）→ 空数组
 *
 * 用于解析后端 `RoomInfoResponse` / `JoinRoomResponse`，新房间 `iceServers` 非空，
 * 旧房间可能仅有 `stunServers`。
 *
 * @param iceServers 新格式 ICE 服务器列表（含 TURN 凭据）
 * @param stunServers 旧格式 STUN URL 字符串数组
 */
export function resolveIceServers(
  iceServers: IceServerEntry[] | undefined,
  stunServers: string[] | undefined,
): IceServerEntry[] {
  if (iceServers && iceServers.length > 0) return iceServers
  if (stunServers && stunServers.length > 0) return stunUrlsToIceServers(stunServers)
  return []
}

/** `buildIceServers` 输入参数 */
export interface BuildIceServersOptions {
  /** STUN 服务器 URL 数组（来自 `room_get_stun`） */
  stunServers?: string[]
  /** 用户自定义 TURN 服务器（来自 SettingsOnline 配置，备用） */
  customTurnServers?: IceServerEntry[]
  /** 系统提供的 TURN 服务器（来自 `room_get_turn`，房主独占） */
  systemTurnServers?: IceServerEntry[]
}

/**
 * 合并 STUN + 用户自定义 TURN + 系统 TURN 为统一 ICE 服务器列表
 *
 * 阶段三子任务 7 抽取。调用场景：
 * - 房主创建房间时：`buildIceServers({ stunServers, customTurnServers })` → 上报后端
 * - 房主运行期间广播：`buildIceServers({ stunServers, customTurnServers, systemTurnServers })` → DataChannel 下发
 *
 * 顺序约定：STUN 在前（优先 P2P 直连），用户自定义 TURN 居中（备用），系统 TURN 在后（兜底）。
 * 调用方负责去重（一般 STUN/TURN 来源不同，无需去重）。
 */
export function buildIceServers(options: BuildIceServersOptions): IceServerEntry[] {
  const { stunServers, customTurnServers, systemTurnServers } = options
  const result: IceServerEntry[] = []
  if (stunServers && stunServers.length > 0) {
    result.push(...stunUrlsToIceServers(stunServers))
  }
  if (customTurnServers && customTurnServers.length > 0) {
    result.push(...customTurnServers)
  }
  if (systemTurnServers && systemTurnServers.length > 0) {
    result.push(...systemTurnServers)
  }
  return result
}

/**
 * 创建 RTCPeerConnection
 *
 * @param iceServers ICE 服务器条目数组（可含 STUN + TURN 凭据）
 * @throws 当前环境不支持 RTCPeerConnection 时抛错
 */
export function createPeerConnection(iceServers: IceServerEntry[]): RTCPeerConnection {
  if (typeof RTCPeerConnection === 'undefined') {
    throw new Error('当前环境不支持 RTCPeerConnection')
  }
  const config: RTCConfiguration = {
    iceServers: iceServers.map((entry) => {
      const server: RTCIceServer = { urls: entry.urls }
      if (entry.username) server.username = entry.username
      if (entry.credential) server.credential = entry.credential
      return server
    }),
    // iceTransportPolicy='all' 同时收集 host/srflx/relay candidate
    // 若未来需要强制走 TURN 中转可改为 'relay'，当前默认允许 P2P 直连优先
    iceTransportPolicy: 'all',
  }
  return new RTCPeerConnection(config)
}

/**
 * 收集所有 ICE candidate（等待 iceGatheringState === 'complete'）
 *
 * 非 trickle 模式：必须等待全部收集完成后再返回，否则上报的 candidate 不全
 * 导致 P2P 协商失败。超时 5 秒兜底（极端网络环境下 STUN 慢响应）。
 *
 * 调用方应在 `setLocalDescription` 之后调用此函数。
 * 函数内部不修改 `onicecandidate` / `onicegatheringstatechange` 之外的字段。
 *
 * @param targetPc 已执行 setLocalDescription 的 PeerConnection
 * @param timeoutMs 超时毫秒，默认 5000
 */
export function collectIceCandidates(
  targetPc: RTCPeerConnection,
  timeoutMs = 5000,
): Promise<string[]> {
  return new Promise((resolve) => {
    const candidates: string[] = []
    let settled = false

    const finish = () => {
      if (settled) return
      settled = true
      targetPc.onicecandidate = null
      targetPc.onicegatheringstatechange = null
      clearTimeout(timer)
      resolve(candidates)
    }

    targetPc.onicecandidate = (event) => {
      if (event.candidate && event.candidate.candidate) {
        candidates.push(event.candidate.candidate)
      }
    }
    targetPc.onicegatheringstatechange = () => {
      if (targetPc.iceGatheringState === 'complete') {
        finish()
      }
    }

    // 超时兜底：5 秒内未 complete 也用已有 candidate 返回
    const timer = setTimeout(finish, timeoutMs)
  })
}

/**
 * 房主在指定 PC 上创建 DataChannel
 *
 * 通道名与配置由本模块常量统一管理，确保房主与加入方协商一致。
 */
export function createDataChannel(pc: RTCPeerConnection): RTCDataChannel {
  return pc.createDataChannel(P2P_DATA_CHANNEL_LABEL, P2P_DATA_CHANNEL_OPTIONS)
}

/**
 * 绑定 DataChannel 事件处理器
 *
 * 加入方在 `pc.ondatachannel` 接收通道后调用此函数；
 * 房主在 `createDataChannel` 之后也可调用（统一收包路径）。
 *
 * @param channel DataChannel 实例
 * @param handlers 处理器集合，未传的字段不绑定
 */
export function setupDataChannelHandlers(
  channel: RTCDataChannel,
  handlers: DataChannelHandlers,
): void {
  // 强制 binaryType = 'arraybuffer'，便于直接传给后端 Tauri IPC
  channel.binaryType = 'arraybuffer'
  if (handlers.onOpen) channel.onopen = handlers.onOpen
  if (handlers.onMessage) channel.onmessage = (event) => {
    if (event.data instanceof ArrayBuffer) {
      handlers.onMessage!(event.data)
    } else if (typeof event.data === 'string') {
      // 兼容字符串消息（控制信令备用），转为 ArrayBuffer
      handlers.onMessage!(new TextEncoder().encode(event.data).buffer)
    }
  }
  if (handlers.onError) channel.onerror = handlers.onError
  if (handlers.onClose) channel.onclose = handlers.onClose
}
