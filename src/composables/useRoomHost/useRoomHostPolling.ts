/**
 * 房主轮询切片（useRoomHost 拆分）
 *
 * 三路信令轮询（参与者/Answer 2s、保活 30s）、自动 Offer 生成、ICE restart
 * 断线恢复（系统 TURN 在参与者开始协商时按需就位）、30s 防刷屏 toast 与
 * 定时器启停；生命周期由主文件 useRoomHost.ts 负责。
 */
import { ref, watch } from 'vue'
import { useOnlineStore } from '@/stores/online'
import { RoomClosedError } from '@/stores/online/roomActions'
import type { useWebRTCMesh } from '@/composables/useWebRTCMesh'
import type { useVirtualLan } from '@/composables/useVirtualLan'
import {
  confirmParticipant,
  listAnswers,
  uploadParticipantOffer,
} from '@/utils/api/online-manager'
import { buildIceServers, stunUrlsToIceServers } from '@/utils/online/webrtc-helpers'
import type { IceServerEntry, PendingAnswer } from '@/types/online'
import { toastError } from '@/utils/toast'
import { encodeHostMcPort, encodeHostVirtualIp } from '@/utils/online/protocol'

/** 防刷屏 toast 间隔：30s 内同类型错误不重复弹 */
const POLL_ERROR_TOAST_INTERVAL = 30_000
/** 轮询活跃间隔：存在待处理 Offer/Answer 时（ms） */
const POLL_ACTIVE_INTERVAL_MS = 2_000
/** 轮询空闲退避间隔：无待处理项时降低云端压力（ms） */
const POLL_IDLE_INTERVAL_MS = 10_000
/** Answer 轮询全连接慢速档：所有已确认参与者均建立连接后，仅需低频感知 ICE restart 重答（ms） */
const POLL_ANSWERS_CONNECTED_MS = 30_000
/** 单个参与者 ICE restart 最大次数（超限关闭连接，交由加入方全量重建） */
const MAX_ICE_RESTART_ATTEMPTS = 2
/** disconnected 状态触发 ICE restart 的宽限期（ICE 可自行恢复，需给足时间） */
const DISCONNECT_RESTART_DELAY_MS = 8_000
/** 两次 ICE restart 的最小间隔（等待加入方响应新 Offer 重答，避免耗尽次数） */
const RESTART_COOLDOWN_MS = 15_000

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
  /**
   * 已生成本地 Offer 但服务端尚未置 hostOfferReady 的参与者（key=participantId）。
   *
   * 服务端上传 Offer 后置 hostOfferReady 存在 <2s 轮询周期延迟：若不记录，
   * 下一轮 pollParticipants 刷新仍返回 hostOfferReady=false，此时 offerGenerating 已移除
   * 会触发重复生成，而 createOfferFor 会先关闭旧 PC，破坏进行中的协商。此处作为本地
   * 「Offer 已生成待确认」标记，抑制重复生成；待 refresh 到 hostOfferReady=true 后清除。
   */
  const offerReadyLocal = new Set<string>()
  /** 正在执行 ICE restart 的参与者（防并发，key=participantId） */
  const restarting = new Set<string>()
  /** 重启中且重启前已确认的参与者（key=participantId，value=wasConfirmed，用于自动放行重答） */
  const restartInFlight = new Map<string, boolean>()
  /** 各参与者 ICE restart 尝试次数 */
  const restartAttempts = new Map<string, number>()
  /** 各参与者 disconnected 状态起始时间（用于断连宽限期判定） */
  const disconnectedAt = new Map<string, number>()
  /** 各参与者 ICE restart 冷却截止时间（避免在加入方重答前重复重启） */
  const restartCooldownUntil = new Map<string, number>()
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
      // 参与者加入、开始协商前确保系统 TURN 已就位（同房间缓存一次）：
      // 首次 Offer 即带 relay candidate，P2P 打洞失败时无需等待 ICE restart
      await ensureSystemTurnServers()
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
      // 服务端可能尚未将 hostOfferReady 置 true（> 轮询周期），本地标记已生成，
      // 抑制同 participantId 在刷新滞后窗口内的重复生成（否则 createOfferFor 会先关旧 PC）
      offerReadyLocal.add(participantId)
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
   * 对指定参与者执行 ICE restart（P2P 断线恢复）
   *
   * 复用现有 PC（restartIce → 新 Offer），新 Offer 上传后加入方轮询到
   * ice-ufrag 变化会自动重答，此间通过 restartInFlight 标记自动放行。
   */
  async function restartIceForParticipant(participantId: string, wasConfirmed: boolean) {
    if (restarting.has(participantId)) return
    restarting.add(participantId)
    try {
      // ensure 幂等：首轮协商已拉取成功则零开销；首轮拉取失败时此处自动重试，
      // restart 前注入该参与者 PC 配置，重新收集 relay candidate
      await ensureSystemTurnServers()
      const iceServers: IceServerEntry[] = store.roomState.iceServers.length > 0
        ? store.roomState.iceServers
        : stunUrlsToIceServers(store.roomState.stunServers)
      const result = await hostMesh.restartIceFor(participantId, iceServers)
      if (!result) {
        hostMesh.closeParticipant(participantId)
        return
      }
      const upload = await uploadParticipantOffer(
        store.roomState.roomCode,
        participantId,
        result.sdp,
        result.iceCandidates,
      )
      if (upload.code !== 1) throw new Error(upload.msg || '上传新 Offer 失败')
      // 重启前已确认的参与者，重答后自动放行（不再弹确认框）
      restartInFlight.set(participantId, wasConfirmed)
      restartAttempts.set(participantId, (restartAttempts.get(participantId) ?? 0) + 1)
      restartCooldownUntil.set(participantId, Date.now() + RESTART_COOLDOWN_MS)
      // 新 Offer 已上传，切回 Answer 快档（2s），避免停在 30s 慢速档延迟感知加入方重答
      scheduleAnswersNext()
      console.info(`[Online] 已对参与者 ${participantId} 执行 ICE restart`)
    } catch (e) {
      console.warn(`[Online] 参与者 ${participantId} ICE restart 失败:`, e)
      maybeToastOfferError(
        `ICE restart 失败：${e instanceof Error ? e.message : String(e)}`,
      )
    } finally {
      restarting.delete(participantId)
    }
  }

  /**
   * 自动放行 ICE restart 重答（重启前已确认的参与者，不再弹确认框）
   */
  async function autoAcceptRestartAnswer(answer: PendingAnswer) {
    restartInFlight.delete(answer.participantId)
    try {
      const result = await confirmParticipant(
        store.roomState.roomCode,
        answer.participantId,
        true,
      )
      if (result.code !== 1) throw new Error(result.msg || '自动确认失败')
      const ok = await hostMesh.setRemoteAnswer(
        answer.participantId,
        answer.sdpAnswer,
        answer.iceCandidates ?? [],
      )
      if (ok) {
        console.info(`[Online] 已自动确认参与者 ${answer.participantId} 的 ICE restart 重答`)
      }
    } catch (e) {
      console.warn(`[Online] 自动确认参与者 ${answer.participantId} 重答失败:`, e)
    }
  }

  /**
   * 自动放行已确认参与者的 Answer（授权前置：房主在 Offer 生成前已确认）
   *
   * 直接 setRemoteAnswer 建立 P2P 连接，无需二次确认；幂等跳过（已放行/PC 未就绪）
   * 不得关闭已建立的连接，仅真正协商失败时关闭残留 PC。
   */
  async function autoAcceptConfirmedAnswer(answer: PendingAnswer) {
    try {
      const ok = await hostMesh.setRemoteAnswer(
        answer.participantId,
        answer.sdpAnswer,
        answer.iceCandidates ?? [],
      )
      if (ok) {
        console.info(`[Online] 已自动放行参与者 ${answer.participantId} 的 Answer`)
      }
    } catch (e) {
      console.warn(`[Online] 自动放行参与者 ${answer.participantId} 的 Answer 失败:`, e)
      hostMesh.closeParticipant(answer.participantId)
    }
  }

  /**
   * 扫描参与者连接状态，对 failed / 长时间 disconnected 的连接执行 ICE restart
   *
   * 由 pollParticipants 与 connectionStates 变化 watch 共同触发；
   * restarting / restartAttempts 双重防并发与限次。
   */
  function scanRestartCandidates() {
    const roomCode = store.roomState.roomCode
    if (!roomCode || store.roomState.role !== 'host') return
    const activeIds = new Set(
      store.roomState.participants
        .filter((p) => p.status === 'joined' || p.status === 'answered' || p.status === 'confirmed')
        .map((p) => p.participantId),
    )
    const now = Date.now()
    for (const [id, state] of hostMesh.connectionStates.entries()) {
      if (!activeIds.has(id)) continue
      if (offerGenerating.value.has(id) || restarting.has(id)) continue
      const attempts = restartAttempts.get(id) ?? 0
      const inCooldown = (restartCooldownUntil.get(id) ?? 0) > now
      const giveUp = () => {
        hostMesh.closeParticipant(id)
        restartAttempts.delete(id)
        restartInFlight.delete(id)
        restartCooldownUntil.delete(id)
        disconnectedAt.delete(id)
      }
      if (state === 'connected') {
        // 恢复后清理重启状态（重答已由 autoAcceptRestartAnswer 处理或旧路径自愈）
        restartAttempts.delete(id)
        restartCooldownUntil.delete(id)
        restartInFlight.delete(id)
        disconnectedAt.delete(id)
        continue
      }
      if (state === 'failed') {
        if (inCooldown) continue
        if (attempts >= MAX_ICE_RESTART_ATTEMPTS) {
          console.warn(`[Online] 参与者 ${id} 多次 ICE restart 仍失败，关闭连接等待全量重建`)
          giveUp()
        } else {
          const p = store.roomState.participants.find((x) => x.participantId === id)
          void restartIceForParticipant(id, p?.status === 'confirmed')
        }
        continue
      }
      if (state === 'disconnected') {
        const since = disconnectedAt.get(id) ?? now
        disconnectedAt.set(id, since)
        if (now - since >= DISCONNECT_RESTART_DELAY_MS && !inCooldown) {
          if (attempts >= MAX_ICE_RESTART_ATTEMPTS) {
            giveUp()
          } else {
            const p = store.roomState.participants.find((x) => x.participantId === id)
            void restartIceForParticipant(id, p?.status === 'confirmed')
          }
        }
      }
    }
  }

  /**
   * 扫描参与者列表，为 status='confirmed' && !hostOfferReady 的参与者生成 Offer
   *
   * 由 pollParticipants 调用，每次刷新参与者列表后触发。
   * 授权前置：只有房主在「加入申请」中确认接受（status='confirmed'）后才生成
   * Offer，加入方在授权前只会看到 ready=false，保持「等待房主接受」而不启动连接。
   */
  async function scanAndGenerateOffers() {
    const roomCode = store.roomState.roomCode
    if (!roomCode || store.roomState.role !== 'host') return
    const needOffer = store.roomState.participants.filter(
      (p) =>
        p.status === 'confirmed' &&
        !p.hostOfferReady &&
        // 已生成但服务端尚未确认的跳过，避免刷新滞后窗口内重复生成关闭旧 PC
        !offerReadyLocal.has(p.participantId),
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
      // 已确认参与者：服务端 hostOfferReady=true 后清除本地生成标记，恢复正常生成状态
      for (const p of store.roomState.participants) {
        if (p.status !== 'confirmed' || p.hostOfferReady) {
          offerReadyLocal.delete(p.participantId)
        }
      }
      for (const id of Array.from(hostMesh.connectionStates.keys())) {
        if (!activeIds.has(id)) {
          void hostMesh.closeParticipant(id)
          // 清理已离开参与者的连接状态键，避免 connectionStates 残留 'closed' 条目无界累积
          hostMesh.removeConnState(id)
          offerReadyLocal.delete(id)
          restartAttempts.delete(id)
          restartInFlight.delete(id)
          restartCooldownUntil.delete(id)
          disconnectedAt.delete(id)
        }
      }
      await scanAndGenerateOffers()
      // P2P 断线自动 ICE restart（failed / 长时间 disconnected）
      scanRestartCandidates()
      // 发现新申请或已确认参与者时联动刷新 Answer（申请随 join/confirm 提交，及时呈现）
      if (
        store.roomState.participants.some(
          (p) => p.status === 'joined' || p.status === 'confirmed',
        )
      ) {
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
    const reqRoomCode = store.roomState.roomCode
    answering.value = true
    try {
      const result = await listAnswers(reqRoomCode)
      // 离开/关闭房间（roomCode 已变更）后不再处理本次结果，避免对陈旧 roomCode 发起无效请求
      if (store.roomState.role !== 'host' || store.roomState.roomCode !== reqRoomCode) return
      if (result.code === 1 && result.data) {
        const answers = result.data.answers ?? []
        // 已确认参与者的 Answer 自动放行：授权前置（房主在 Offer 生成前已确认），
        // Answer 到达即 setRemoteAnswer 建立连接，不再二次确认。
        // ICE restart 重答（restartInFlight 标记）同样自动放行。
        const manual: PendingAnswer[] = []
        for (const a of answers) {
          if (restartInFlight.get(a.participantId) === true) {
            void autoAcceptRestartAnswer(a)
            continue
          }
          const p = store.roomState.participants.find(
            (x) => x.participantId === a.participantId,
          )
          if (p && p.status === 'confirmed') {
            void autoAcceptConfirmedAnswer(a)
          } else {
            manual.push(a)
          }
        }
        pendingAnswers.value = manual
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
   * 确保系统 TURN 已合并进 roomState.iceServers（参与者开始协商时调用）
   *
   * 建房瞬间不拉取（此时尚无参与者）；首个参与者加入生成 Offer 时首次拉取
   * （同房间缓存一次），首轮协商即带 relay candidate——P2P 直连不受影响
   * （ICE 优先 host/srflx），打洞失败时中继立即可用，无需等 failed 后 restart。
   * 失败仅 warn 不阻塞协商（降级 STUN + 自定义 TURN），restart 路径自动重试。
   */
  let systemTurnLoaded = false
  let systemTurnLoading: Promise<void> | null = null
  async function ensureSystemTurnServers() {
    if (store.roomState.role !== 'host' || !store.roomState.roomCode) return
    if (systemTurnLoaded) return
    if (systemTurnLoading) return systemTurnLoading
    systemTurnLoading = (async () => {
      try {
        const turnResp = await store.fetchTurnServers()
        const systemTurn: IceServerEntry[] = turnResp?.servers ?? []
        if (systemTurn.length === 0) return
        const merged = buildIceServers({
          stunServers: store.roomState.stunServers,
          customTurnServers: store.customTurnServers,
          systemTurnServers: systemTurn,
        })
        if (merged.length === 0) return
        store.roomState.iceServers = merged
        systemTurnLoaded = true
        console.info(
          `[Online] 房主已就位系统 TURN：${systemTurn.length} 个，用于首轮/后续协商`,
        )
      } catch (e) {
        console.warn('[Online] 拉取系统 TURN 失败，继续按 STUN/自定义 TURN 协商:', e)
      } finally {
        systemTurnLoading = null
      }
    })()
    return systemTurnLoading
  }

  // 定时器句柄（setTimeout 链式调度：稳态退避，活跃保持高频）
  let answerTimer: ReturnType<typeof setTimeout> | null = null
  let participantsTimer: ReturnType<typeof setTimeout> | null = null
  /** 调度开关：stopTimers 置 false，防止进行中的请求完成后重新拉起定时器 */
  let timersActive = false
  /** HostVirtualIp 控制消息的本地 seq 计数器 */
  let hostIpSeq = 0
  /** HostMcPort 控制消息的本地 seq 计数器 */
  let mcPortSeq = 0

  /** 存在待授权申请或待生成 Offer 的参与者时保持活跃间隔，否则退避到空闲间隔 */
  function scheduleParticipantsNext() {
    if (!timersActive) return
    if (participantsTimer) clearTimeout(participantsTimer)
    const hasPendingOffer = store.roomState.participants.some(
      (p) =>
        p.status === 'joined' ||
        (p.status === 'confirmed' && !p.hostOfferReady),
    )
    participantsTimer = setTimeout(
      () => void pollParticipants(),
      hasPendingOffer ? POLL_ACTIVE_INTERVAL_MS : POLL_IDLE_INTERVAL_MS,
    )
  }

  /**
   * 存在待确认申请、待授权申请或未建立连接的已确认参与者时保持活跃间隔，
   * 否则（全部已连接）退避到慢速档，仅低频感知 ICE restart 重答。
   */
  function scheduleAnswersNext() {
    if (!timersActive) return
    if (answerTimer) clearTimeout(answerTimer)
    const hasPendingAnswers =
      pendingAnswers.value.length > 0 ||
      store.roomState.participants.some(
        (p) => p.status === 'joined' || p.status === 'answered',
      ) ||
      store.roomState.participants.some(
        (p) => p.status === 'confirmed' && !hostMesh.channelOpen.get(p.participantId),
      ) ||
      // ICE restart 进行中（已上传新 Offer、等待加入方重答）保持快档，尽快捕获重答
      Array.from(restartInFlight.keys()).some(
        (id) => !hostMesh.channelOpen.get(id),
      )
    answerTimer = setTimeout(
      () => void pollAnswers(),
      hasPendingAnswers ? POLL_ACTIVE_INTERVAL_MS : POLL_ANSWERS_CONNECTED_MS,
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

  // 连接状态变化即时触发 ICE restart 扫描（无需等待下一轮参与者轮询）
  watch(
    () => Array.from(hostMesh.connectionStates.entries()),
    () => {
      if (timersActive) scanRestartCandidates()
    },
  )

  // 房间切换时重置系统 TURN 就位标记（新房间重新拉取）
  watch(
    () => store.roomState.roomCode,
    () => {
      systemTurnLoaded = false
    },
  )

  return {
    pendingAnswers,
    offerGenerating,
    pollParticipants,
    pollAnswers,
    doKeepalive,
    startTimers,
    stopTimers,
  }
}
