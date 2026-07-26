/**
 * WebRTC composable
 *
 * 封装 RTCPeerConnection 生命周期管理，提供联机所需的：
 * - SDP Offer / Answer 创建
 * - ICE candidate 收集（非 trickle，一次性上报全部）
 * - 远端 SDP / ICE 设置
 * - 连接状态追踪（new/connecting/connected/disconnected/failed/closed）
 * - NAT 类型检测（创建临时 PeerConnection，复用 utils/online/nat-type）
 *
 * 设计约束：
 * - 房主与加入方使用同一 composable，通过 `role` 区分创建 offer / answer
 * - 不实现 trickle ICE，等待 iceGatheringState === 'complete' 后一次性返回所有 candidate
 *   （后端 `signaling_*` 接口约定 ice_candidates 为数组，非流式）
 * - onUnmounted 自动 close PeerConnection，避免泄漏
 *
 * @example 房主创建 offer
 * const webrtc = useWebRTC('host')
 * const { sdp, iceCandidates } = await webrtc.createOffer(['stun:stun.l.google.com:19302'])
 * // ... 上传到服务端 ...
 * // 收到加入方 answer 后：
 * await webrtc.setRemoteAnswer(answer.sdpAnswer, answer.iceCandidates)
 *
 * @example 加入方创建 answer
 * const webrtc = useWebRTC('guest')
 * const { sdp, iceCandidates } = await webrtc.setRemoteOfferAndCreateAnswer(
 *   stunServers, joinResp.hostSdpOffer, joinResp.hostIceCandidates,
 * )
 * // ... 提交到服务端 ...
 */

import { onUnmounted, ref, shallowRef } from 'vue'
import { detectNatTypeWithStun } from '@/utils/online/nat-type'
import type { NatDetectionResult } from '@/types/online'

/** WebRTC 连接状态 */
export type WebRtcConnectionState =
  | 'new'
  | 'connecting'
  | 'connected'
  | 'disconnected'
  | 'failed'
  | 'closed'

/** 创建 Offer / Answer 的结果 */
export interface SdpResult {
  /** SDP 描述字符串 */
  sdp: string
  /** 收集到的 ICE candidate 字符串数组 */
  iceCandidates: string[]
}

/**
 * WebRTC composable
 *
 * @param role 角色：'host' = 房主（createOffer），'guest' = 加入方（createAnswer）
 */
export function useWebRTC(role: 'host' | 'guest' = 'host') {
  /** 当前 PeerConnection（shallowRef 避免深度响应式开销） */
  const pc = shallowRef<RTCPeerConnection | null>(null)
  /** 连接状态 */
  const connectionState = ref<WebRtcConnectionState>('new')
  /** ICE 收集状态 */
  const iceGatheringState = ref<RTCIceGatheringState>('new')
  /** 是否正在创建 Offer / Answer */
  const negotiating = ref(false)
  /** 数据通道（房主创建，加入方在 ondatachannel 接收） */
  const dataChannel = shallowRef<RTCDataChannel | null>(null)
  /** NAT 检测结果（detectNatType 后填充） */
  const natResult = ref<NatDetectionResult | null>(null)
  /** 是否正在检测 NAT */
  const detectingNat = ref(false)

  /**
   * 创建 PeerConnection
   *
   * @param stunServers STUN 服务器 URL 数组
   */
  function createPeerConnection(stunServers: string[]): RTCPeerConnection {
    if (typeof RTCPeerConnection === 'undefined') {
      throw new Error('当前环境不支持 RTCPeerConnection')
    }
    const config: RTCConfiguration = {
      iceServers: stunServers.map((url) => ({ urls: url })),
      // 仅使用 STUN，不配置 TURN（联机阶段二不实现中转）
      iceTransportPolicy: 'all',
    }
    const newPc = new RTCPeerConnection(config)

    // 连接状态同步
    newPc.onconnectionstatechange = () => {
      connectionState.value = newPc.connectionState as WebRtcConnectionState
    }
    newPc.onicegatheringstatechange = () => {
      iceGatheringState.value = newPc.iceGatheringState
    }

    // 房主创建 DataChannel 用于 P2P 数据传输
    // （阶段三虚拟网卡数据通过该通道传输）
    if (role === 'host') {
      try {
        dataChannel.value = newPc.createDataChannel('molaunch-p2p', {
          ordered: false,
          maxRetransmits: 0,
        })
      } catch {
        /* ignore */
      }
    } else {
      // 加入方监听房主创建的 DataChannel
      newPc.ondatachannel = (event) => {
        dataChannel.value = event.channel
      }
    }

    pc.value = newPc
    return newPc
  }

  /**
   * 收集所有 ICE candidate（等待 iceGatheringState === 'complete'）
   *
   * 非 trickle 模式：必须等待全部收集完成后再返回，否则上报的 candidate 不全
   * 导致 P2P 协商失败。超时 5 秒兜底（极端网络环境下 STUN 慢响应）。
   *
   * 收集完成后会还原 targetPc.onicecandidate / onicegatheringstatechange 回调，
   * 保留外层 createPeerConnection 中设置的状态同步逻辑。
   */
  function collectIceCandidates(targetPc: RTCPeerConnection, timeoutMs = 5000): Promise<string[]> {
    return new Promise((resolve) => {
      const candidates: string[] = []
      let settled = false

      // 保留外层已设置的 gathering 回调（createPeerConnection 中的状态同步）
      const origGatheringHandler = targetPc.onicegatheringstatechange
      const origIceHandler = targetPc.onicecandidate

      const restore = () => {
        targetPc.onicecandidate = origIceHandler
        targetPc.onicegatheringstatechange = origGatheringHandler
      }

      const finish = () => {
        if (settled) return
        settled = true
        restore()
        clearTimeout(timer)
        resolve(candidates)
      }

      targetPc.onicecandidate = (event) => {
        if (event.candidate && event.candidate.candidate) {
          candidates.push(event.candidate.candidate)
        }
      }

      targetPc.onicegatheringstatechange = (event) => {
        if (typeof origGatheringHandler === 'function') {
          origGatheringHandler.call(targetPc, event)
        }
        if (targetPc.iceGatheringState === 'complete') {
          finish()
        }
      }

      // 超时兜底：5 秒内未 complete 也用已有 candidate 返回
      const timer = setTimeout(finish, timeoutMs)
    })
  }

  /**
   * 房主创建 SDP Offer
   *
   * 流程：createOffer → setLocalDescription → 收集 ICE → 返回
   *
   * @param stunServers STUN 服务器列表
   */
  async function createOffer(stunServers: string[]): Promise<SdpResult> {
    negotiating.value = true
    try {
      const targetPc = pc.value ?? createPeerConnection(stunServers)
      const offer = await targetPc.createOffer()
      await targetPc.setLocalDescription(offer)
      const iceCandidates = await collectIceCandidates(targetPc)
      return {
        sdp: targetPc.localDescription?.sdp ?? offer.sdp,
        iceCandidates,
      }
    } finally {
      negotiating.value = false
    }
  }

  /**
   * 加入方设置远端 Offer 后创建 Answer
   *
   * 流程：setRemoteDescription(offer) → createAnswer → setLocalDescription → 收集 ICE → 返回
   *
   * @param stunServers STUN 服务器列表（与房主保持一致）
   * @param remoteSdp 房主的 SDP Offer
   * @param remoteIce 房主的 ICE candidate 数组
   */
  async function setRemoteOfferAndCreateAnswer(
    stunServers: string[],
    remoteSdp: string,
    remoteIce: string[],
  ): Promise<SdpResult> {
    negotiating.value = true
    try {
      const targetPc = pc.value ?? createPeerConnection(stunServers)
      await targetPc.setRemoteDescription({ type: 'offer', sdp: remoteSdp })
      // 添加房主的 ICE candidates
      for (const candidate of remoteIce) {
        try {
          await targetPc.addIceCandidate({ candidate })
        } catch {
          /* 单个 candidate 失败不阻塞整体协商 */
        }
      }
      const answer = await targetPc.createAnswer()
      await targetPc.setLocalDescription(answer)
      const iceCandidates = await collectIceCandidates(targetPc)
      return {
        sdp: targetPc.localDescription?.sdp ?? answer.sdp,
        iceCandidates,
      }
    } finally {
      negotiating.value = false
    }
  }

  /**
   * 房主设置加入方的 Answer
   *
   * 流程：setRemoteDescription(answer) → 添加 ICE → 等待连接建立
   *
   * @param remoteSdp 加入方的 SDP Answer
   * @param remoteIce 加入方的 ICE candidate 数组
   */
  async function setRemoteAnswer(remoteSdp: string, remoteIce: string[]): Promise<void> {
    const targetPc = pc.value
    if (!targetPc) {
      throw new Error('PeerConnection 未初始化，请先调用 createOffer')
    }
    await targetPc.setRemoteDescription({ type: 'answer', sdp: remoteSdp })
    for (const candidate of remoteIce) {
      try {
        await targetPc.addIceCandidate({ candidate })
      } catch {
        /* 单个 candidate 失败不阻塞整体协商 */
      }
    }
  }

  /**
   * 检测 NAT 类型（创建临时 PeerConnection，不污染主连接）
   *
   * @param stunServers STUN 服务器列表（默认 Google STUN）
   */
  async function detectNatType(stunServers?: string[]): Promise<NatDetectionResult> {
    detectingNat.value = true
    try {
      const result = await detectNatTypeWithStun(stunServers)
      natResult.value = result
      return result
    } finally {
      detectingNat.value = false
    }
  }

  /**
   * 关闭 PeerConnection 并释放资源
   *
   * 关闭 DataChannel → close PeerConnection → 清空引用
   */
  function close() {
    try {
      if (dataChannel.value) {
        try {
          dataChannel.value.close()
        } catch {
          /* ignore */
        }
        dataChannel.value = null
      }
      if (pc.value) {
        try {
          pc.value.close()
        } catch {
          /* ignore */
        }
        pc.value = null
      }
      connectionState.value = 'closed'
    } catch {
      /* ignore */
    }
  }

  // 组件卸载时自动关闭，避免 PeerConnection 泄漏
  onUnmounted(() => close())

  return {
    // 状态
    pc,
    connectionState,
    iceGatheringState,
    negotiating,
    dataChannel,
    natResult,
    detectingNat,
    // 方法
    createPeerConnection,
    createOffer,
    setRemoteOfferAndCreateAnswer,
    setRemoteAnswer,
    detectNatType,
    close,
  }
}
