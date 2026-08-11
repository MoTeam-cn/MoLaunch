/**
 * WebRTC mesh - PC / DataChannel 生命周期切片
 *
 * 为每个参与者维护独立 PeerConnection + DataChannel（1-N 虚拟局域网分发）。
 * 复用 utils/online/webrtc-helpers.ts；加密密钥 roomKey 由主 composable 注入。
 */
import { reactive, shallowRef, type ShallowRef } from 'vue'
import {
  createPeerConnection,
  collectIceCandidates,
  createDataChannel,
  setupDataChannelHandlers,
  wrapHandlersWithDecrypt,
  type WebRtcConnectionState,
  type DataChannelHandlers,
} from '@/utils/online/webrtc-helpers'
import type { IceServerEntry } from '@/types/online'

/** 创建 Offer / Answer 的结果 */
export interface SdpResult {
  /** SDP 描述字符串 */
  sdp: string
  /** 收集到的 ICE candidate 字符串数组 */
  iceCandidates: string[]
}

/** 单个参与者的连接信息（内部维护，不对外暴露） */
export interface ParticipantConn {
  /** PeerConnection 实例 */
  pc: RTCPeerConnection
  /** 房主创建的 DataChannel（用于下发数据） */
  channel: RTCDataChannel
}

export interface MeshPeerDeps {
  /** DataChannel 加密密钥（null 表示未启用加密） */
  roomKey: ShallowRef<CryptoKey | null>
}

/** 房主 mesh 多 PC 连接生命周期管理 */
export function useMeshPeer(deps: MeshPeerDeps) {
  const { roomKey } = deps
  /** 参与者连接表（shallowRef，内部 Map 不需要深度响应式） */
  const conns = shallowRef<Map<string, ParticipantConn>>(new Map())
  /** 各参与者的连接状态（reactive 让 UI 能监听变化） */
  const connectionStates = reactive<Map<string, WebRtcConnectionState>>(new Map())
  /** 各参与者 DataChannel 的 open 状态（reactive，便于 UI 显示「已联通」人数） */
  const channelOpen = reactive<Map<string, boolean>>(new Map())
  /** 是否正在为某个参与者协商（key=participantId） */
  const negotiating = reactive<Map<string, boolean>>(new Map())

  /**
   * 同步指定参与者的连接状态到 reactive map
   */
  function setConnState(participantId: string, state: WebRtcConnectionState) {
    connectionStates.set(participantId, state)
    if (state === 'closed' || state === 'failed' || state === 'disconnected') {
      channelOpen.set(participantId, false)
    }
  }

  /**
   * 为指定参与者创建 PC + DataChannel + SDP Offer
   *
   * 流程：
   * 1. 若该 participantId 已有 PC，先关闭旧的（避免重复）
   * 2. 创建 PC（绑定状态同步）
   * 3. 创建 DataChannel（绑定 onopen/onmessage/onerror/onclose）
   * 4. createOffer → setLocalDescription → 收集 ICE → 返回
   *
   * @param participantId 参与者 ID
   * @param iceServers ICE 服务器条目数组（含 STUN + TURN 凭据）
   * @param onChannelOpen DataChannel 建立（onOpen）时回调，用于向该参与者下发控制消息
   * @returns SDP Offer + ICE candidates，由调用方上传到后端
   */
  async function createOfferFor(
    participantId: string,
    iceServers: IceServerEntry[],
    onChannelOpen?: (participantId: string) => void,
  ): Promise<SdpResult> {
    // 已存在旧连接 → 先清理（不应发生，防御性处理）
    if (conns.value.has(participantId)) {
      closeParticipant(participantId)
    }

    negotiating.set(participantId, true)
    try {
      const pc = createPeerConnection(iceServers)
      const channel = createDataChannel(pc)

      // 连接状态同步
      pc.onconnectionstatechange = () => {
        setConnState(participantId, pc.connectionState as WebRtcConnectionState)
      }

      // DataChannel 事件（默认空 handler，业务侧可通过 setDataChannelHandlers 重新绑定）
      setupDataChannelHandlers(channel, {
        onOpen: () => {
          channelOpen.set(participantId, true)
          onChannelOpen?.(participantId)
        },
        onClose: () => channelOpen.set(participantId, false),
      })

      conns.value.set(participantId, { pc, channel })
      setConnState(participantId, 'new')

      // 生成 Offer
      const offer = await pc.createOffer()
      await pc.setLocalDescription(offer)
      const iceCandidates = await collectIceCandidates(pc)
      return {
        sdp: pc.localDescription?.sdp ?? offer.sdp ?? '',
        iceCandidates,
      }
    } finally {
      negotiating.set(participantId, false)
    }
  }

  /**
   * 重新绑定指定参与者 DataChannel 的事件处理器
   *
   * 业务侧在 createOfferFor 之后调用，注入实际收包逻辑（如转发到 TUN）。
   * 若 `roomKey` 已注入，传入的 `onMessage` 会被自动包装为「先解密再回调」。
   *
   * @param participantId 参与者 ID
   * @param handlers 处理器集合，未传的字段不绑定
   */
  function setDataChannelHandlers(
    participantId: string,
    handlers: DataChannelHandlers,
  ) {
    const conn = conns.value.get(participantId)
    if (!conn) return
    const wrapped = wrapHandlersWithDecrypt(handlers, roomKey)
    setupDataChannelHandlers(conn.channel, wrapped)
  }

  /**
   * 为指定参与者设置远端 Answer
   *
   * 在收到 `listAnswers` 中该参与者的 Answer 后调用。
   *
   * @param participantId 参与者 ID
   * @param remoteSdp 参与者的 SDP Answer
   * @param remoteIce 参与者的 ICE candidate 数组
   */
  async function setRemoteAnswer(
    participantId: string,
    remoteSdp: string,
    remoteIce: string[],
  ): Promise<void> {
    const conn = conns.value.get(participantId)
    if (!conn) {
      throw new Error(`参与者 ${participantId} 的 PeerConnection 不存在，请先 createOfferFor`)
    }
    await conn.pc.setRemoteDescription({ type: 'answer', sdp: remoteSdp })
    for (const candidate of remoteIce) {
      try {
        await conn.pc.addIceCandidate({ candidate })
      } catch {
        /* 单个 candidate 失败不阻塞整体协商 */
      }
    }
  }

  /**
   * 关闭指定参与者的 PC + DataChannel
   *
   * 用于踢出/参与者离开场景。不抛错（幂等）。
   * 注：不在此清理 `negotiating` 标志 —— 若 createOfferFor 进行中被调用，
   * 其 finally 块会负责清理；其余场景 negotiating 本就为 false。
   */
  function closeParticipant(participantId: string) {
    const conn = conns.value.get(participantId)
    if (!conn) return
    try {
      try {
        conn.channel.close()
      } catch {
        /* ignore */
      }
      try {
        conn.pc.close()
      } catch {
        /* ignore */
      }
    } finally {
      conns.value.delete(participantId)
      channelOpen.set(participantId, false)
      setConnState(participantId, 'closed')
    }
  }

  /**
   * 关闭所有 PeerConnection 并释放资源
   *
   * 房间关闭时调用。幂等。同时清除加密密钥，避免复用 composable 时残留旧密钥。
   */
  function close() {
    for (const participantId of Array.from(conns.value.keys())) {
      closeParticipant(participantId)
    }
    // 阶段三子任务 8：清除加密密钥
    roomKey.value = null
  }

  /** 获取指定参与者的连接状态 */
  function getConnState(participantId: string): WebRtcConnectionState | undefined {
    return connectionStates.get(participantId)
  }

  /** 当前已联通（channel open）的参与者数量 */
  function connectedCount(): number {
    let n = 0
    for (const isOpen of channelOpen.values()) {
      if (isOpen) n++
    }
    return n
  }

  return {
    conns,
    connectionStates,
    channelOpen,
    negotiating,
    createOfferFor,
    setRemoteAnswer,
    setDataChannelHandlers,
    closeParticipant,
    close,
    getConnState,
    connectedCount,
  }
}