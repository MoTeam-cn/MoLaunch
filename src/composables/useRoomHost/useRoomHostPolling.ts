/**
 * 房主轮询切片（useRoomHost 拆分）
 *
 * 三路信令轮询（参与者/Answer 2s、保活 30s）、自动 Offer 生成、TURN 广播、
 * 30s 防刷屏 toast 与定时器启停；生命周期由主文件 useRoomHost.ts 负责。
 */
import { ref } from 'vue'
import { useOnlineStore } from '@/stores/online'
import { RoomClosedError } from '@/stores/online/roomActions'
import type { useWebRTCMesh } from '@/composables/useWebRTCMesh'
import type { useVirtualLan } from '@/composables/useVirtualLan'
import { listAnswers, uploadParticipantOffer } from '@/utils/api/online-manager'
import { buildIceServers, stunUrlsToIceServers } from '@/utils/online/webrtc-helpers'
import type { IceServerEntry, PendingAnswer } from '@/types/online'
import { toastError } from '@/utils/toast'
import { encodeHostMcPort, encodeHostVirtualIp, encodeTurnServers } from '@/utils/online/protocol'

/** 防刷屏 toast 间隔：30s 内同类型错误不重复弹 */
const POLL_ERROR_TOAST_INTERVAL = 30_000
/** 轮询活跃间隔：存在待处理 Offer/Answer 时（ms） */
const POLL_ACTIVE_INTERVAL_MS = 2_000
/** 轮询空闲退避间隔：无待处理项时降低云端压力（ms） */
const POLL_IDLE_INTERVAL_MS = 10_000

export interface RoomHostPollingOptions {
  /** 房间被服务端关闭（keepalive 返回 1001）时回调，由主文件清理连接并退出房间 */
  onRoomClosed?: (msg: string) => void
}

export function useRoomHostPolling(
  store: ReturnType<typeof useOnlineStore>,
  hostMesh: ReturnType<typeof useWebRTCMesh>,
  lan: ReturnType<typeof useVirtualLan>,
  options: RoomHostPollingOptions = {},
) {
  /** 待确认 Answer 列表（pollAnswers 5s 刷新） */
  const pendingAnswers = ref<PendingAnswer[]>([])
  /** 正在轮询参与者（防重入） */
  const polling = ref(false)
  /** 正在轮询 Answer（防重入） */
  const answering = ref(false)
  /** 正在为参与者生成 Offer 的集合，防止重复生成（key=participantId） */
  const offerGenerating = ref<Set<string>>(new Set())
  /** 轮询失败 toast 防刷屏：记录上次 toast 时间 */
  const lastAnswerErrorToastAt = ref(0)
  const lastOfferErrorToastAt = ref(0)

  /** 30s 防刷屏 toast：避免轮询连续失败时刷屏 */
  function maybeToastAnswerError(msg: string) {
    const now = Date.now()
    if (now - lastAnswerErrorToastAt.value < POLL_ERROR_TOAST_INTERVAL) return
    lastAnswerErrorToastAt.value = now
    toastError(msg)
  }
  function maybeToastOfferError(msg: string) {
    const now = Date.now()
    if (now - lastOfferErrorToastAt.value < POLL_ERROR_TOAST_INTERVAL) return
    lastOfferErrorToastAt.value = now
    toastError(msg)
  }

  /**
   * 为单个参与者生成 SDP Offer 并上传到后端
   *
   * 流程：hostMesh.createOfferFor → 绑定 onMessage → uploadParticipantOffer
   * 失败时 toast 提示但不阻塞其他参与者。
   */
  async function generateOfferForParticipant(participantId: string) {
    if (offerGenerating.value.has(participantId)) return
    offerGenerating.value.add(participantId)
    try {
      // 阶段三子任务 7：优先使用 iceServers（含 STUN + 用户自定义 TURN + 系统 TURN）
      // 旧房间 iceServers 为空时回退到 stunServers 并转为 IceServerEntry[]
      const iceServers: IceServerEntry[] = store.roomState.iceServers.length > 0
        ? store.roomState.iceServers
        : stunUrlsToIceServers(store.roomState.stunServers)
      // 通道建立后向该参与者广播房主虚拟 IP 与当前 MC 端口（加入方连接界面显示用；
      // 端口在参与者连上后才捕获时也能立即同步，避免首轮广播错过的时序问题）
      const hostIp = store.roomState.selfVirtualIp
      const { sdp, iceCandidates } = await hostMesh.createOfferFor(
        participantId,
        iceServers,
        hostIp
          ? () => {
              void hostMesh.sendToParticipant(
                participantId,
                encodeHostVirtualIp(hostIpSeq++, hostIp),
              )
              const hostPort = store.roomState.hostMcPort
              if (hostPort > 0) {
                void hostMesh.sendToParticipant(
                  participantId,
                  encodeHostMcPort(mcPortSeq++, hostPort),
                )
              }
            }
          : undefined,
      )

      // 绑定 DataChannel.onMessage：参与者发来的包 → 转发到后端 TUN
      // setupDataChannelHandlers 仅更新传入字段，不影响 createOfferFor 默认绑定的 onOpen/onClose
      hostMesh.setDataChannelHandlers(participantId, {
        onMessage: (raw) => {
          void lan.forwardToTun(raw)
        },
      })

      const result = await uploadParticipantOffer(
        store.roomState.roomCode,
        participantId,
        sdp,
        iceCandidates,
      )
      if (result.code !== 1) {
        throw new Error(result.msg || '上传 SDP Offer 失败')
      }
    } catch (e) {
      console.warn(`[Online] 为参与者 ${participantId} 生成 Offer 失败:`, e)
      maybeToastOfferError(
        `生成 SDP Offer 失败：${e instanceof Error ? e.message : String(e)}`,
      )
      // 生成失败时清理 PC，避免下次轮询跳过
      hostMesh.closeParticipant(participantId)
    } finally {
      offerGenerating.value.delete(participantId)
    }
  }

  /**
   * 扫描参与者列表，为 status='joined' && !hostOfferReady 的参与者生成 Offer
   *
   * 由 pollParticipants 调用，每次刷新参与者列表后触发。
   */
  async function scanAndGenerateOffers() {
    const roomCode = store.roomState.roomCode
    if (!roomCode || store.roomState.role !== 'host') return
    const needOffer = store.roomState.participants.filter(
      (p) => p.status === 'joined' && !p.hostOfferReady,
    )
    // 并发生成（每个参与者独立 PC，互不干扰）
    await Promise.all(needOffer.map((p) => generateOfferForParticipant(p.participantId)))
  }

  /** 轮询参与者列表 + 触发 Offer 生成 */
  async function pollParticipants() {
    if (store.roomState.role !== 'host' || !store.roomState.roomCode || polling.value) return
    polling.value = true
    try {
      await store.refreshParticipants()
      // 清理已离开/被拒绝参与者的残留 PC（加入方断线自动重连重新 join 后旧 participant_id 也在此列）
      const activeIds = new Set(
        store.roomState.participants
          .filter((p) => p.status === 'joined' || p.status === 'answered' || p.status === 'confirmed')
          .map((p) => p.participantId),
      )
      for (const id of Array.from(hostMesh.connectionStates.keys())) {
        if (!activeIds.has(id)) void hostMesh.closeParticipant(id)
      }
      await scanAndGenerateOffers()
      // 发现新参与者时联动刷新 Answer（新申请随 join 提交，及时呈现给房主）
      if (store.roomState.participants.some((p) => p.status === 'joined')) {
        void pollAnswers()
      }
    } catch (e) {
      console.warn('[Online] pollParticipants 异常:', e)
    } finally {
      polling.value = false
      scheduleParticipantsNext()
    }
  }

  /** 轮询待确认 Answer */
  async function pollAnswers() {
    if (store.roomState.role !== 'host' || !store.roomState.roomCode || answering.value) return
    answering.value = true
    try {
      const result = await listAnswers(store.roomState.roomCode)
      if (result.code === 1 && result.data) {
        pendingAnswers.value = result.data.answers ?? []
        lastAnswerErrorToastAt.value = 0
      } else {
        console.warn(
          `[Online] pollAnswers 业务失败: code=${result.code}, msg=${result.msg}, req_id=${result.req_id}`,
        )
        maybeToastAnswerError(`获取待确认 Answer 失败：${result.msg}`)
      }
    } catch (e) {
      console.warn('[Online] pollAnswers 异常:', e)
      maybeToastAnswerError(
        `获取待确认 Answer 异常：${e instanceof Error ? e.message : String(e)}`,
      )
    } finally {
      answering.value = false
      scheduleAnswersNext()
    }
  }

  /** 房主保活 */
  async function doKeepalive() {
    try {
      await store.keepalive()
    } catch (e) {
      // 房间已被服务端关闭/销毁（code=1001）：停止轮询并通知主文件清理退出，
      // 避免无意义地持续上报并让用户感知房间已失效
      if (e instanceof RoomClosedError) {
        stopTimers()
        options.onRoomClosed?.(e.message)
        return
      }
      console.warn('[Online] keepalive 失败:', e)
    }
  }

  /**
   * 拉取系统 TURN 服务器并广播给已联通参与者（阶段三子任务 7 阶段 F）
   *
   * 流程：
   * 1. 调 `store.fetchTurnServers`（房主独占接口）拉取经服务端负载过滤的可用 TURN
   * 2. 与 STUN + 用户自定义 TURN 合并为统一 iceServers
   * 3. 更新本地 `store.roomState.iceServers`（影响后续 `generateOfferForParticipant`）
   * 4. 通过 `encodeTurnServers` 编码 + `hostMesh.broadcastPacket` 下发
   *
   * 失败仅 warn，不阻塞主流程（系统 TURN 不可用时降级为 STUN + 用户自定义 TURN）。
   * 房间刚创建时参与者尚未联通，broadcastPacket 返回 0 属正常；后续参与者 PC 建立后
   * 由房主手动重新触发或下次轮询时通过其他机制获取（当前实现仅 onMounted 触发一次）。
   */
  async function fetchAndBroadcastTurnServers() {
    if (store.roomState.role !== 'host' || !store.roomState.roomCode) return
    try {
      const turnResp = await store.fetchTurnServers()
      const systemTurn: IceServerEntry[] = turnResp?.servers ?? []
      const merged = buildIceServers({
        stunServers: store.roomState.stunServers,
        customTurnServers: store.customTurnServers,
        systemTurnServers: systemTurn,
      })
      if (merged.length === 0) {
        console.info('[Online] 房主无可用 ICE 服务器，跳过 TURN 广播')
        return
      }
      // 更新本地 iceServers，影响后续 generateOfferForParticipant
      store.roomState.iceServers = merged
      // 阶段三子任务 8：broadcastPacket 异步加密后发送，sent 计数仅用于日志
      void hostMesh.broadcastPacket(encodeTurnServers(turnSeq++, merged)).then((sent) => {
        console.info(
          `[Online] 房主已广播 ICE 服务器列表：${systemTurn.length} 系统 TURN + ${store.customTurnServers.length} 自定义 TURN，已发送给 ${sent} 个参与者`,
        )
      }).catch((e) => console.warn('[Online] 广播 ICE 服务器列表失败:', e))
    } catch (e) {
      console.warn('[Online] 拉取/广播 TURN 服务器失败:', e)
    }
  }

  // 定时器句柄（setTimeout 链式调度：稳态退避，活跃保持高频）
  let answerTimer: ReturnType<typeof setTimeout> | null = null
  let participantsTimer: ReturnType<typeof setTimeout> | null = null
  /** 调度开关：stopTimers 置 false，防止进行中的请求完成后重新拉起定时器 */
  let timersActive = false
  /** TurnServers 控制消息的本地 seq 计数器（与 HostMcPort/TUN 数据包 seq 独立） */
  let turnSeq = 0
  /** HostVirtualIp 控制消息的本地 seq 计数器 */
  let hostIpSeq = 0
  /** HostMcPort 控制消息的本地 seq 计数器 */
  let mcPortSeq = 0

  /** 存在待生成 Offer 的参与者时保持活跃间隔，否则退避到空闲间隔 */
  function scheduleParticipantsNext() {
    if (!timersActive) return
    if (participantsTimer) clearTimeout(participantsTimer)
    const hasPendingOffer = store.roomState.participants.some(
      (p) => p.status === 'joined' && !p.hostOfferReady,
    )
    participantsTimer = setTimeout(
      () => void pollParticipants(),
      hasPendingOffer ? POLL_ACTIVE_INTERVAL_MS : POLL_IDLE_INTERVAL_MS,
    )
  }

  /** 存在待确认申请时保持活跃间隔，否则退避到空闲间隔 */
  function scheduleAnswersNext() {
    if (!timersActive) return
    if (answerTimer) clearTimeout(answerTimer)
    answerTimer = setTimeout(
      () => void pollAnswers(),
      pendingAnswers.value.length > 0 ? POLL_ACTIVE_INTERVAL_MS : POLL_IDLE_INTERVAL_MS,
    )
  }

  /**
   * 启动两路信令轮询（参与者/Answer 活跃 2s、空闲退避 10s）
   *
   * 注：保活(30s)已由 store 层全局定时器承担（src/stores/online.ts GLOBAL_KEEPALIVE_INTERVAL），
   * 切页不停止；此处 doKeepalive 仅保留给「断连恢复补发」使用，避免重复上报。
   */
  function startTimers() {
    timersActive = true
    scheduleParticipantsNext()
    scheduleAnswersNext()
  }

  /** 停止所有轮询定时器（云端断开或组件卸载时调用，避免持续失败刷屏） */
  function stopTimers() {
    timersActive = false
    if (participantsTimer) { clearTimeout(participantsTimer); participantsTimer = null }
    if (answerTimer) { clearTimeout(answerTimer); answerTimer = null }
  }

  return {
    pendingAnswers,
    offerGenerating,
    pollParticipants,
    pollAnswers,
    doKeepalive,
    startTimers,
    stopTimers,
    fetchAndBroadcastTurnServers,
  }
}
