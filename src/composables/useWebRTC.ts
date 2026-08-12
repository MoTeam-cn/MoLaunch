/**
 * WebRTC composable（加入方专用）
 *
 * 单 PC 流程：轮询房主 Offer → 设远端 → 建 Answer → 收集 ICE → 提交；
 * NAT 检测复用 utils/online/nat-type。无 trickle ICE（candidate 一次性返回）；
 * setRoomKey 注入 AES-GCM 后 onMessage 自动先解密再回调，null 时透传。
 */

import { onUnmounted, ref, shallowRef } from 'vue'
import {
  createPeerConnection,
  collectIceCandidates,
  setupDataChannelHandlers,
  toRtcIceServers,
  wrapHandlersWithDecrypt,
  type WebRtcConnectionState,
  type DataChannelHandlers,
} from '@/utils/online/webrtc-helpers'
import { encryptFrame } from '@/utils/online/crypto'
import { detectNatTypeWithStun } from '@/utils/online/nat-type'
import { fetchParticipantOffer } from '@/utils/api/online-manager'
import type { IceServerEntry, NatDetectionResult } from '@/types/online'

/** 创建 Offer / Answer 的结果 */
export interface SdpResult {
  /** SDP 描述字符串 */
  sdp: string
  /** 收集到的 ICE candidate 字符串数组 */
  iceCandidates: string[]
}

/** fetchOfferAndAnswer 默认轮询参数 */
const DEFAULT_POLL_INTERVAL_MS = 2000
const DEFAULT_POLL_MAX_INTERVAL_MS = 10_000
/** 等待房主接受申请并生成 Offer 的超时：授权前置后加入方需等房主确认，不宜过短 */
const DEFAULT_POLL_TIMEOUT_MS = 180_000

/**
 * 加入方 WebRTC composable
 *
 * 内部维护单一 PeerConnection，生命周期由 composable 管理。
 * 调用 `fetchOfferAndAnswer` 完成 SDP 协商；调用 `close` 主动释放。
 * 全局联机会话传 `autoClose: false`（常驻应用生命周期，不随组件卸载关闭）。
 */
export function useWebRTC(options?: { autoClose?: boolean }) {
  const autoClose = options?.autoClose ?? true
  /** 当前 PeerConnection（shallowRef 避免深度响应式开销） */
  const pc = shallowRef<RTCPeerConnection | null>(null)
  /** 连接状态 */
  const connectionState = ref<WebRtcConnectionState>('new')
  /** ICE 收集状态 */
  const iceGatheringState = ref<RTCIceGatheringState>('new')
  /** 是否正在协商（fetchOfferAndAnswer / setRemoteOfferAndCreateAnswer 期间为 true） */
  const negotiating = ref(false)
  /** 最近一次应用的房主 Offer SDP（用于 ICE restart 检测：房主重新上传 Offer 后对比变化） */
  const lastOfferSdp = ref('')
  /** 数据通道（房主创建，加入方在 ondatachannel 接收） */
  const dataChannel = shallowRef<RTCDataChannel | null>(null)
  /** NAT 检测结果（detectNatType 后填充） */
  const natResult = ref<NatDetectionResult | null>(null)
  /** 是否正在检测 NAT */
  const detectingNat = ref(false)
  /**
   * DataChannel 加密密钥（阶段三子任务 8）
   *
   * null 表示未启用加密（兼容旧服务器）；非 null 时 `sendPacket` 自动加密，
   * `setDataChannelHandlers` 绑定的 `onMessage` 自动先解密再回调。
   */
  const roomKey = shallowRef<CryptoKey | null>(null)

  /**
   * 注入 / 清除 DataChannel 加密密钥
   *
   * 加入方加入房间后调用 `importRoomKey(store.roomState.roomKey)` 导入密钥，
   * 再调用此方法注入。退出房间时调用 `setRoomKey(null)` 清除。
   *
   * @param key AES-GCM 密钥；null 表示禁用加密（透传原始帧）
   */
  function setRoomKey(key: CryptoKey | null): void {
    roomKey.value = key
  }

  /**
   * 创建 PeerConnection 并绑定状态同步 + DataChannel 接收
   *
   * @param iceServers ICE 服务器条目数组（含 STUN + TURN 凭据）
   */
  function ensurePeerConnection(iceServers: IceServerEntry[]): RTCPeerConnection {
    if (pc.value) return pc.value
    const newPc = createPeerConnection(iceServers)
    // 新建连接时重置状态，避免复用实例残留上一次会话的 closed
    connectionState.value = 'new'

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
   * 刷新现有 PC 的 ICE 服务器配置（P2P 失败懒加载系统 TURN 后调用，重协商生效）
   *
   * PC 不存在时静默跳过；仅在下次 ICE restart / 重答时应用新服务器。
   */
  function applyIceServers(iceServers: IceServerEntry[]): void {
    const targetPc = pc.value
    if (!targetPc) return
    try {
      targetPc.setConfiguration({ iceServers: toRtcIceServers(iceServers) })
    } catch {
      /* 配置注入失败不阻塞重协商 */
    }
  }

  /**
   * 重新绑定 DataChannel 事件处理器
   *
   * 用于在 ondatachannel 触发后或协商完成后，由业务侧注入收包逻辑。
   *
   * 阶段三子任务 8：若 `roomKey` 已注入，传入的 `onMessage` 会被自动包装为
   * 「先解密再回调」，业务层收到的 `raw` 是已解密的原始协议帧。
   *
   * @param handlers 处理器集合，未传的字段不绑定
   */
  function setDataChannelHandlers(handlers: DataChannelHandlers) {
    if (!dataChannel.value) return
    const wrapped = wrapHandlersWithDecrypt(handlers, roomKey)
    setupDataChannelHandlers(dataChannel.value, wrapped)
  }

  /**
   * 向房主发送二进制包（TUN 上行）
   *
   * 阶段三子任务 8：若 `roomKey` 已注入，先加密 `raw` 再发送；否则透传原始帧。
   * DataChannel 未就绪或发送异常时返回 false，调用方应静默跳过（TUN 包丢失不影响后续）。
   *
   * @param raw 原始协议帧
   * @returns 是否发送成功
   */
  async function sendPacket(raw: ArrayBuffer): Promise<boolean> {
    const channel = dataChannel.value
    if (!channel || channel.readyState !== 'open') return false
    const key = roomKey.value
    const payload = key ? await encryptFrame(raw, key) : raw
    try {
      channel.send(payload)
      return true
    } catch {
      return false
    }
  }

  /**
   * 设置远端 Offer 后创建 Answer
   *
   * 流程：setRemoteDescription(offer) → createAnswer → setLocalDescription → 收集 ICE → 返回
   *
   * @param iceServers ICE 服务器条目数组（与房主保持一致，含 STUN + TURN 凭据）
   * @param remoteSdp 房主的 SDP Offer
   * @param remoteIce 房主的 ICE candidate 数组
   */
  async function setRemoteOfferAndCreateAnswer(
    iceServers: IceServerEntry[],
    remoteSdp: string,
    remoteIce: string[],
  ): Promise<SdpResult> {
    negotiating.value = true
    try {
      const targetPc = ensurePeerConnection(iceServers)
      // ensurePeerConnection 仅首次创建时应用配置；已存在的 PC（ICE restart 重答）
      // 需显式 setConfiguration，使 P2P 失败后懒加载的系统 TURN 参与本次协商
      try {
        targetPc.setConfiguration({ iceServers: toRtcIceServers(iceServers) })
      } catch {
        /* 配置注入失败不阻塞协商 */
      }
      await targetPc.setRemoteDescription({ type: 'offer', sdp: remoteSdp })
      // 记录本次应用的 Offer，供 ICE restart 检测对比
      lastOfferSdp.value = remoteSdp
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
   * @param iceServers ICE 服务器条目数组（含 STUN + TURN 凭据）
   * @param pollIntervalMs 轮询间隔，默认 2000ms
   * @param timeoutMs 总超时，默认 180000ms（等待房主接受申请；超时抛错）
   */
  async function fetchOfferAndAnswer(
    roomCode: string,
    participantId: string,
    iceServers: IceServerEntry[],
    pollIntervalMs: number = DEFAULT_POLL_INTERVAL_MS,
    timeoutMs: number = DEFAULT_POLL_TIMEOUT_MS,
  ): Promise<SdpResult> {
    const deadline = Date.now() + timeoutMs
    negotiating.value = true
    let waitMs = pollIntervalMs
    try {
      // 轮询房主生成的 Offer，超时抛错
      // eslint-disable-next-line no-constant-condition
      while (true) {
        if (Date.now() >= deadline) {
          throw new Error(`等待房主接受申请超时（${timeoutMs / 1000}s）`)
        }
        const result = await fetchParticipantOffer(roomCode, participantId)
        if (result.code !== 1 || !result.data) {
          throw new Error(result.msg || '拉取房主 SDP Offer 失败')
        }
        if (result.data.ready && result.data.sdpOffer) {
          return await setRemoteOfferAndCreateAnswer(
            iceServers,
            result.data.sdpOffer,
            result.data.iceCandidates ?? [],
          )
        }
        // 未就绪，指数退避等待下一轮（上限 10s，避免长期等待时对云端高频轮询）
        await new Promise((resolve) => setTimeout(resolve, waitMs))
        waitMs = Math.min(Math.floor(waitMs * 2), DEFAULT_POLL_MAX_INTERVAL_MS)
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
   * 关闭 DataChannel → close PeerConnection → 清空引用 → 清除加密密钥
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
        // 仅在确实存在连接时置 closed，从未连接的实例保持 'new'
        connectionState.value = 'closed'
      }
      // 阶段三子任务 8：清除加密密钥，避免复用 composable 时残留旧密钥
      roomKey.value = null
    } catch {
      /* ignore */
    }
  }

  // 组件卸载时自动关闭，避免 PeerConnection 泄漏
  if (autoClose) {
    onUnmounted(() => close())
  }

  return {
    // 状态
    pc,
    connectionState,
    iceGatheringState,
    negotiating,
    dataChannel,
    lastOfferSdp,
    natResult,
    detectingNat,
    // 方法
    ensurePeerConnection,
    setRemoteOfferAndCreateAnswer,
    fetchOfferAndAnswer,
    setDataChannelHandlers,
    applyIceServers,
    sendPacket,
    setRoomKey,
    detectNatType,
    close,
  }
}
