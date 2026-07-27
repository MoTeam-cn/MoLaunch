/**
 * 房主房间运营 composable（阶段三 mesh 拓扑）
 *
 * 从 RoomHostPanel.vue 抽出，封装房主侧全部业务逻辑：
 * - 信令轮询（mesh 三路并行）：5s 参与者轮询 + 5s 待确认 Answer 轮询 + 5min 保活
 * - 自动为 status='joined' && !hostOfferReady 的参与者生成 per-participant Offer 并上传
 * - 30s 防刷屏 toast：轮询连续失败时仅每 30s 弹一次
 * - 交互处理：确认/拒绝 Answer、踢出参与者、关闭房间
 *
 * # 职责边界
 *
 * - 本 composable 只负责业务逻辑，不渲染 UI
 * - 调用方（RoomHostPanel.vue）负责注入 hostMesh 与 lan 实例，并通过 computed 暴露 UI 状态
 * - onMounted 自动启动 timer + lan.start，onUnmounted 自动清理 timer（lan.stop 由 useVirtualLan 自身处理）
 *
 * @example
 * const hostMesh = inject('hostMesh') as ReturnType<typeof useWebRTCMesh>
 * const lan = useVirtualLan({ onTunPacket: (raw) => hostMesh.broadcastPacket(raw) })
 * const { pendingAnswers, handleConfirm, handleKick, handleCloseRoom } = useRoomHost({ hostMesh, lan })
 */

import { ref, onMounted, onUnmounted } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { useOnlineStore } from '@/stores/online'
import type { useWebRTCMesh } from '@/composables/useWebRTCMesh'
import type { useVirtualLan } from '@/composables/useVirtualLan'
import {
  listAnswers,
  confirmParticipant,
  kickParticipant,
  uploadParticipantOffer,
} from '@/utils/api/online-manager'
import { buildIceServers, stunUrlsToIceServers } from '@/utils/online/webrtc-helpers'
import type { IceServerEntry, PendingAnswer } from '@/types/online'
import { showConfirm } from '@/utils/modal'
import { toastSuccess, toastError } from '@/utils/toast'
import { encodeHostMcPort, encodeTurnServers } from '@/utils/online/protocol'
import { importRoomKey } from '@/utils/online/crypto'

/** 防刷屏 toast 间隔：30s 内同类型错误不重复弹 */
const POLL_ERROR_TOAST_INTERVAL = 30_000

/**
 * 房主房间运营 composable
 *
 * @param options.hostMesh 房主多 PC 管理器（由 RoomManager.vue 通过 provide/inject 注入）
 * @param options.lan 虚拟网卡桥接实例（由 RoomHostPanel.vue 创建并传入）
 */
export function useRoomHost(options: {
  hostMesh: ReturnType<typeof useWebRTCMesh>
  lan: ReturnType<typeof useVirtualLan>
}) {
  const { hostMesh, lan } = options
  const store = useOnlineStore()

  /** 待确认 Answer 列表（pollAnswers 5s 刷新） */
  const pendingAnswers = ref<PendingAnswer[]>([])
  /** 正在轮询参与者（防重入） */
  const polling = ref(false)
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
      const { sdp, iceCandidates } = await hostMesh.createOfferFor(participantId, iceServers)

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
      await scanAndGenerateOffers()
    } catch (e) {
      console.warn('[Online] pollParticipants 异常:', e)
    } finally {
      polling.value = false
    }
  }

  /** 轮询待确认 Answer */
  async function pollAnswers() {
    if (store.roomState.role !== 'host' || !store.roomState.roomCode) return
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
    }
  }

  /** 房主保活 */
  async function doKeepalive() {
    try {
      await store.keepalive()
    } catch (e) {
      console.warn('[Online] keepalive 失败:', e)
    }
  }

  /**
   * 确认/拒绝参与者连接
   *
   * - 接受：confirmParticipant(true) → hostMesh.setRemoteAnswer(participantId, ...)
   * - 拒绝：confirmParticipant(false) → hostMesh.closeParticipant(participantId)
   */
  async function handleConfirm(answer: PendingAnswer, accepted: boolean) {
    try {
      const result = await confirmParticipant(
        store.roomState.roomCode,
        answer.participantId,
        accepted,
      )
      if (result.code !== 1) throw new Error(result.msg || '确认操作失败')
      if (accepted) {
        await hostMesh.setRemoteAnswer(
          answer.participantId,
          answer.sdpAnswer,
          answer.iceCandidates ?? [],
        )
      } else {
        // 拒绝连接：关闭对应 PC 释放资源
        hostMesh.closeParticipant(answer.participantId)
      }
      pendingAnswers.value = pendingAnswers.value.filter(
        (a) => a.participantId !== answer.participantId,
      )
      toastSuccess(accepted ? '已接受连接' : '已拒绝连接')
      await store.refreshParticipants()
    } catch (e) {
      toastError(`确认失败：${e instanceof Error ? e.message : String(e)}`)
    }
  }

  /** 踢出参与者（不封禁） */
  function handleKick(participantId: string, devicePk: string) {
    showConfirm('踢出参与者', `确定踢出 ${devicePk.slice(0, 8)}...？`, async () => {
      try {
        const result = await kickParticipant(store.roomState.roomCode, participantId, null)
        if (result.code !== 1) throw new Error(result.msg || '踢出失败')
        // 关闭对应 PC
        hostMesh.closeParticipant(participantId)
        toastSuccess('已踢出')
        await store.refreshParticipants()
      } catch (e) {
        toastError(`踢出失败：${e instanceof Error ? e.message : String(e)}`)
      }
    })
  }

  /** 关闭房间 */
  function handleCloseRoom() {
    showConfirm('关闭房间', '关闭后所有加入方将被断开连接，且无法恢复。确定关闭？', async () => {
      try {
        // 先停止 TUN 桥接，再关闭 mesh，最后调后端关闭房间
        await lan.stop()
        hostMesh.close()
        await store.hostCloseRoom()
      } catch (e) {
        toastError(`关闭失败：${e instanceof Error ? e.message : String(e)}`)
      }
    })
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
      })
    } catch (e) {
      console.warn('[Online] 拉取/广播 TURN 服务器失败:', e)
    }
  }

  // 定时器句柄
  let answerTimer: ReturnType<typeof setInterval> | null = null
  let keepaliveTimer: ReturnType<typeof setInterval> | null = null
  let participantsTimer: ReturnType<typeof setInterval> | null = null
  /** MC 端口检测事件监听器卸载函数 */
  let mcPortUnlisten: UnlistenFn | null = null
  /** HostMcPort 控制消息的本地 seq 计数器（与 TUN 数据包 seq 独立，避免混淆） */
  let mcPortSeq = 0
  /** TurnServers 控制消息的本地 seq 计数器（与 HostMcPort/TUN 数据包 seq 独立） */
  let turnSeq = 0

  onMounted(() => {
    void pollParticipants()
    void pollAnswers()
    void doKeepalive()
    // 参与者轮询 5s（同时触发 Offer 生成）
    participantsTimer = setInterval(() => void pollParticipants(), 5000)
    // Answer 轮询 5s
    answerTimer = setInterval(() => void pollAnswers(), 5000)
    // 保活 5min
    keepaliveTimer = setInterval(() => void doKeepalive(), 5 * 60 * 1000)

    // 阶段三子任务 8：注入 DataChannel 加密密钥（空字符串表示未启用加密，importRoomKey 返回 null）
    // 在 lan.start 之前注入，确保首个 TUN 包就能正确加密
    void importRoomKey(store.roomState.roomKey).then((key) => {
      hostMesh.setRoomKey(key)
    })

    // 启动 TUN 桥接：房主进入面板即创建 TUN 接口，开始读包 → broadcastPacket
    // 失败仅 toast（如 wintun.dll 缺失 / 无管理员权限），不阻塞信令流程
    void lan.start(store.roomState.selfVirtualIp, store.roomState.subnet).catch((e) => {
      toastError(`虚拟网卡启动失败：${e instanceof Error ? e.message : String(e)}`)
    })

    // 房主进入面板后拉取系统 TURN 服务器并广播给已联通参与者（阶段三子任务 7 阶段 F）
    // 失败仅 warn，不阻塞主流程；房间刚创建时参与者尚未联通，broadcastPacket 返回 0 属正常
    void fetchAndBroadcastTurnServers()

    // 监听后端 GameWatcher 的 MC 局域网端口检测事件
    // 房主在 MC 中「Open to LAN」后，watcher 捕获 stdout 端口 → emit 此事件
    // 收到后：1) 更新本地 store.roomState.hostMcPort  2) 通过 DataChannel 广播给所有已联通参与者
    void listen<number>('online://mc-port-detected', (event) => {
      const port = event.payload
      if (!port || port <= 0) return
      store.roomState.hostMcPort = port
      // 阶段三子任务 8：broadcastPacket 异步加密后发送，sent 计数仅用于日志
      void hostMesh.broadcastPacket(encodeHostMcPort(mcPortSeq++, port)).then((sent) => {
        console.info(
          `[Online] 房主 MC 局域网端口已捕获: ${port}，已广播给 ${sent} 个参与者`,
        )
      })
    }).then((unlisten) => {
      mcPortUnlisten = unlisten
    })
  })

  onUnmounted(() => {
    if (answerTimer) clearInterval(answerTimer)
    if (keepaliveTimer) clearInterval(keepaliveTimer)
    if (participantsTimer) clearInterval(participantsTimer)
    if (mcPortUnlisten) {
      mcPortUnlisten()
      mcPortUnlisten = null
    }
    // lan.stop 由 useVirtualLan 的 onUnmounted 自动处理
  })

  return {
    pendingAnswers,
    offerGenerating,
    handleConfirm,
    handleKick,
    handleCloseRoom,
  }
}
