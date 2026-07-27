/**
 * WebRTC composable（加入方专用，阶段三子任务 5 mesh 拓扑）
 *
 * 房主侧多 PC 管理见 `useWebRTCMesh.ts`。本 composable 仅负责加入方单 PC：
 * - 轮询房主为自己生成的 SDP Offer（mesh 拓扑，房主 per-participant Offer）
 * - 设置远端 Offer → 创建 Answer → 收集 ICE → 提交 Answer
 * - 连接状态追踪 + DataChannel 接收（房主创建，加入方 ondatachannel 接收）
 * - NAT 类型检测（创建临时 PeerConnection，复用 utils/online/nat-type）
 *
 * 设计约束：
 * - 不实现 trickle ICE，等待 iceGatheringState === 'complete' 后一次性返回所有 candidate
 *   （后端 `signaling_*` 接口约定 ice_candidates 为数组，非流式）
 * - 复用 `utils/online/webrtc-helpers.ts` 的底层函数，避免与 useWebRTCMesh 重复实现
 * - onUnmounted 自动 close PeerConnection，避免泄漏
 *
 * @example 加入房间
 * const webrtc = useWebRTC()
 * const { sdp, iceCandidates } = await webrtc.fetchOfferAndAnswer(
 *   roomCode, participantId, stunServers,
 * )
 * await submitAnswer(roomCode, participantId, sdp, iceCandidates)
 */

import { onUnmounted, ref, shallowRef } from 'vue'
import {
  createPeerConnection,
  collectIceCandidates,
  setupDataChannelHandlers,
  type WebRtcConnectionState,
} from '@/utils/online/webrtc-helpers'
import { detectNatTypeWithStun } from '@/utils/online/nat-type'
import { fetchParticipantOffer } from '@/utils/api/online-manager'
import type { NatDetectionResult } from '@/types/online'

/** 创建 Offer / Answer 的结果 */
export interface SdpResult {
  /** SDP 描述字符串 */
  sdp: string
  /** 收集到的 ICE candidate 字符串数组 */
  iceCandidates: string[]
}

/** fetchOfferAndAnswer 默认轮询参数 */
const DEFAULT_POLL_INTERVAL_MS = 2000
const DEFAULT_POLL_TIMEOUT_MS = 30_000

/**
 * 加入方 WebRTC composable
 *
 * 内部维护单一 PeerConnection，生命周期由 composable 管理。
 * 调用 `fetchOfferAndAnswer` 完成 SDP 协商；调用 `close` 主动释放。
 */
export function useWebRTC() {
  /** 当前 PeerConnection（shallowRef 避免深度响应式开销） */
  const pc = shallowRef<RTCPeerConnection | null>(null)
  /** 连接状态 */
  const connectionState = ref<WebRtcConnectionState>('new')
  /** ICE 收集状态 */
  const iceGatheringState = ref<RTCIceGatheringState>('new')
  /** 是否正在协商（fetchOfferAndAnswer / setRemoteOfferAndCreateAnswer 期间为 true） */
  const negotiating = ref(false)
  /** 数据通道（房主创建，加入方在 ondatachannel 接收） */
  const dataChannel = shallowRef<RTCDataChannel | null>(null)
  /** NAT 检测结果（detectNatType 后填充） */
  const natResult = ref<NatDetectionResult | null>(null)
  /** 是否正在检测 NAT */
  const detectingNat = ref(false)

  /**
   * 创建 PeerConnection 并绑定状态同步 + DataChannel 接收
   *
   * @param stunServers STUN 服务器 URL 数组
   */
  function ensurePeerConnection(stunServers: string[]): RTCPeerConnection {
    if (pc.value) return pc.value
    const newPc = createPeerConnection(stunServers)

    // 连接状态同步
    newPc.onconnectionstatechange = () => {
      connectionState.value = newPc.connectionState as WebRtcConnectionState
    }
    newPc.onicegatheringstatechange = () => {
      iceGatheringState.value = newPc.iceGatheringState
    }

    // 加入方监听房主创建的 DataChannel
    newPc.ondatachannel = (event) => {
      dataChannel.value = event.channel
      // 默认绑定空 handler，业务侧可通过 setDataChannelHandlers 重新绑定
      setupDataChannelHandlers(event.channel, {})
    }

    pc.value = newPc
    return newPc
  }

  /**
   * 重新绑定 DataChannel 事件处理器
   *
   * 用于在 ondatachannel 触发后或协商完成后，由业务侧注入收包逻辑。
   *
   * @param handlers 处理器集合，未传的字段不绑定
   */
  function setDataChannelHandlers(handlers: Parameters<typeof setupDataChannelHandlers>[1]) {
    if (dataChannel.value) {
      setupDataChannelHandlers(dataChannel.value, handlers)
    }
  }

  /**
   * 设置远端 Offer 后创建 Answer
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
      const targetPc = ensurePeerConnection(stunServers)
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
        sdp: targetPc.localDescription?.sdp ?? answer.sdp ?? '',
        iceCandidates,
      }
    } finally {
      negotiating.value = false
    }
  }

  /**
   * 轮询房主为自己生成的 SDP Offer，收到后立即创建 Answer 并返回
   *
   * mesh 拓扑核心流程：
   * 1. 调用 `fetchParticipantOffer` 拉取房主为本参与者生成的 Offer
   * 2. `ready=false` 时按 `pollIntervalMs` 间隔重试，直到 `timeoutMs` 超时
   * 3. `ready=true` 时调用 `setRemoteOfferAndCreateAnswer` 生成本地 Answer
   * 4. 返回 `{ sdp, iceCandidates }`，由调用方提交给后端 `submitAnswer`
   *
   * @param roomCode 房间码
   * @param participantId 本参与者的 ID（来自 joinRoom 响应）
   * @param stunServers STUN 服务器列表
   * @param pollIntervalMs 轮询间隔，默认 2000ms
   * @param timeoutMs 总超时，默认 30000ms（超时抛错）
   */
  async function fetchOfferAndAnswer(
    roomCode: string,
    participantId: string,
    stunServers: string[],
    pollIntervalMs: number = DEFAULT_POLL_INTERVAL_MS,
    timeoutMs: number = DEFAULT_POLL_TIMEOUT_MS,
  ): Promise<SdpResult> {
    const deadline = Date.now() + timeoutMs
    negotiating.value = true
    try {
      // 轮询房主生成的 Offer，超时抛错
      // eslint-disable-next-line no-constant-condition
      while (true) {
        if (Date.now() >= deadline) {
          throw new Error(`等待房主生成 SDP Offer 超时（${timeoutMs / 1000}s）`)
        }
        const result = await fetchParticipantOffer(roomCode, participantId)
        if (result.code !== 1 || !result.data) {
          throw new Error(result.msg || '拉取房主 SDP Offer 失败')
        }
        if (result.data.ready && result.data.sdpOffer) {
          return await setRemoteOfferAndCreateAnswer(
            stunServers,
            result.data.sdpOffer,
            result.data.iceCandidates ?? [],
          )
        }
        // 未就绪，等待下一轮
        await new Promise((resolve) => setTimeout(resolve, pollIntervalMs))
      }
    } finally {
      negotiating.value = false
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
    ensurePeerConnection,
    setRemoteOfferAndCreateAnswer,
    fetchOfferAndAnswer,
    setDataChannelHandlers,
    detectNatType,
    close,
  }
}
