/**
 * WebRTC 共享工具（阶段三子任务 5 抽取）
 *
 * 房主侧 `useWebRTCMesh.ts` 与加入方侧 `useWebRTC.ts` 共用的底层函数：
 * - `createPeerConnection`：构造 RTCPeerConnection（仅 STUN 配置，不含 DataChannel）
 * - `collectIceCandidates`：等待 ICE 收集完成（非 trickle 模式，一次性返回全部 candidate）
 * - `createDataChannel`：在指定 PC 上创建 DataChannel
 * - `setupDataChannelHandlers`：统一绑定 onopen/onmessage/onerror/onclose
 *
 * 设计约束：
 * - 不持有任何响应式状态，纯函数便于复用与单测
 * - 不引入业务概念（participantId 等），由上层 composable 维护映射
 */

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
 * 创建 RTCPeerConnection
 *
 * @param stunServers STUN 服务器 URL 数组（如 `stun:stun.l.google.com:19302`）
 * @throws 当前环境不支持 RTCPeerConnection 时抛错
 */
export function createPeerConnection(stunServers: string[]): RTCPeerConnection {
  if (typeof RTCPeerConnection === 'undefined') {
    throw new Error('当前环境不支持 RTCPeerConnection')
  }
  const config: RTCConfiguration = {
    iceServers: stunServers.map((url) => ({ urls: url })),
    // 仅 STUN，不配置 TURN（阶段二不实现中转）
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
