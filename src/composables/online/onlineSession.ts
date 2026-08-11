/**
 * 联机会话全局单例（脱离页面生命周期）
 *
 * WebRTC / TUN / 信令轮询原本绑定在 Online.vue 与房间面板组件上，
 * 离开联机页（路由切走）触发 onUnmounted → 关闭 P2P、销毁虚拟网卡、停止轮询。
 * 此模块在 App.vue 初始化，会话常驻整个应用生命周期，切页不断连。
 *
 * 职责：
 * - hostMesh / guestWebrtc / lan 全局实例（autoCleanup=false）
 * - watch roomState.role 驱动会话启停（进房自动启动，退房自动清理）
 * - 加入方 30s 房间状态监控：房主关闭/房间过期/被服务端清理时自动感知并退出
 */
import { ref, watch, type Ref } from 'vue'
import { useOnlineStore } from '@/stores/online'
import { useWebRTC } from '@/composables/useWebRTC'
import { useWebRTCMesh } from '@/composables/useWebRTCMesh'
import { useVirtualLan } from '@/composables/useVirtualLan'
import { useRoomHost } from '@/composables/useRoomHost'
import { reconnectAsGuest } from '@/composables/useRoomReconnect'
import {
  fetchParticipantOffer,
  getRoomInfo,
  lanFakeServerStart,
  lanFakeServerStop,
  submitAnswer,
} from '@/utils/api/online-manager'
import { mergeIceServerEntries, stunUrlsToIceServers } from '@/utils/online/webrtc-helpers'
import { resolveTunParticipantId } from '@/utils/online/tunRouting'
import { importRoomKey } from '@/utils/online/crypto'
import { peekJoinPassword } from '@/utils/relaunchSnapshot'
import {
  decode,
  CONTROL_SUBTYPE,
  parseHostMcPortPayload,
  decodeTurnServersPayload,
  parseHostVirtualIpPayload,
} from '@/utils/online/protocol'
import { toastError } from '@/utils/toast'

/** 加入方房间状态监控间隔（ms） */
const GUEST_ROOM_MONITOR_INTERVAL = 30_000
/** 服务端房间不存在/已关闭/已过期错误码（RoomNotFound → not_found → 1002） */
const ROOM_NOT_FOUND_CODE = 1002

export interface OnlineSession {
  hostMesh: ReturnType<typeof useWebRTCMesh>
  guestWebrtc: ReturnType<typeof useWebRTC>
  lan: ReturnType<typeof useVirtualLan>
  pendingAnswers: ReturnType<typeof useRoomHost>['pendingAnswers']
  offerGenerating: ReturnType<typeof useRoomHost>['offerGenerating']
  bannedList: ReturnType<typeof useRoomHost>['bannedList']
  banServerTime: ReturnType<typeof useRoomHost>['banServerTime']
  handleConfirm: ReturnType<typeof useRoomHost>['handleConfirm']
  handleKick: ReturnType<typeof useRoomHost>['handleKick']
  handleUnban: ReturnType<typeof useRoomHost>['handleUnban']
  refreshBans: ReturnType<typeof useRoomHost>['refreshBans']
  handleCloseRoom: ReturnType<typeof useRoomHost>['handleCloseRoom']
  /** 房主手动指定 MC 端口（最高可信度，自动捕获不再覆盖） */
  setManualMcPort: ReturnType<typeof useRoomHost>['setManualMcPort']
  /** 清除手动端口标记，恢复自动捕获更新 */
  clearManualMcPort: ReturnType<typeof useRoomHost>['clearManualMcPort']
  /** 加入方退出房间（停 TUN + 关 P2P + 云端退出） */
  guestLeaveAndCleanup: () => Promise<void>
  /** 局域网伪装是否启用（加入方本地伪装 LAN 服务器） */
  lanFakeActive: Ref<boolean>
  /** 局域网伪装本地监听端口（0 表示未启用） */
  lanFakePort: Ref<number>
}

let session: OnlineSession | null = null
let initialized = false

/** App 级初始化（幂等）：必须在应用启动时调用一次 */
export function initOnlineSession(): OnlineSession {
  if (initialized) return session!
  initialized = true
  session = createSession()
  return session!
}

/** 获取全局联机会话（惰性创建，组件随时可调） */
export function getOnlineSession(): OnlineSession {
  if (!session) session = createSession()
  return session
}

function createSession(): OnlineSession {
  const store = useOnlineStore()

  // 全局实例：关闭/停止由会话显式管理，不随组件卸载
  const hostMesh = useWebRTCMesh({ autoClose: false })
  const guestWebrtc = useWebRTC({ autoClose: false })
  const lan = useVirtualLan({
    autoStop: false,
    // TUN 读到 IP 包 → 按当前角色路由到对应发送通道
    onTunPacket: (raw) => {
      if (store.roomState.role === 'host') {
        // 优先按目标虚拟 IP 定向单播（消除广播冗余），未命中目标回退广播
        const targetId = resolveTunParticipantId(raw, store.roomState.participants)
        if (targetId) {
          void hostMesh.sendToParticipant(targetId, raw).then((ok) => {
            if (!ok) void hostMesh.broadcastPacket(raw)
          })
        } else {
          void hostMesh.broadcastPacket(raw)
        }
      } else {
        void guestWebrtc.sendPacket(raw)
      }
    },
  })

  // 房主运营（轮询/自动 Offer/TURN 广播/MC 端口监听），生命周期由会话控制
  const hostOps = useRoomHost({
    hostMesh,
    lan,
    autoLifecycle: false,
    onRoomClosed: (msg) => handleRoomClosed(msg),
  })

  // 加入方房间状态监控定时器：感知房主关闭/房间过期/被服务端清理
  let guestMonitorTimer: ReturnType<typeof setInterval> | null = null
  function stopGuestMonitor() {
    if (guestMonitorTimer) {
      clearInterval(guestMonitorTimer)
      guestMonitorTimer = null
    }
  }
  function startGuestMonitor() {
    stopGuestMonitor()
    guestMonitorTimer = setInterval(() => {
      void checkGuestRoomAlive()
    }, GUEST_ROOM_MONITOR_INTERVAL)
  }
  async function checkGuestRoomAlive() {
    const st = store.roomState
    if (st.role !== 'guest' || !st.roomCode) return
    try {
      const result = await getRoomInfo(st.roomCode)
      if (result.code === 1) return
      // 房间已关闭/不存在/过期（服务端把 closed 视为不存在，返回 1002）
      if (result.code === ROOM_NOT_FOUND_CODE) {
        handleRoomClosed(`房间已关闭或已不存在（${st.roomCode}）`)
      }
      // 其他 code（认证/网络类）静默，下轮重试
    } catch {
      // 网络异常静默，下轮重试
    }
  }

  // 加入方 P2P 断线自动重连
  // 网络抖动先让 ICE 自行恢复（disconnected → connected）；disconnected 超时未恢复
  // 或直接 failed 时走服务端信令重建（leaveRoom → joinRoom，房主自动生成新 Offer），
  // 保留 ICE 固有恢复能力的同时提供确定性兜底。
  const GUEST_RECONNECT_ATTEMPTS = 3
  const RECONNECT_BACKOFF_MS = [3_000, 6_000, 12_000]
  const DISCONNECT_RECOVERY_DELAY_MS = 5_000
  /** 轻量重启（ICE restart）超时：超时未恢复回退全量重建 */
  const LIGHT_RESTART_TIMEOUT_MS = 30_000
  /** Offer 监控慢速间隔（连接正常时轮询，感知房主主动 restart） */
  const RESTART_MONITOR_SLOW_MS = 15_000
  /** Offer 监控快速间隔（断线恢复期间高频轮询） */
  const RESTART_MONITOR_FAST_MS = 2_500
  let reconnectAttempts = 0
  let reconnecting = false
  let reconnectTimer: ReturnType<typeof setTimeout> | null = null
  let disconnectedRecoveryTimer: ReturnType<typeof setTimeout> | null = null
  /** 房主新 Offer 监控定时器（ICE restart 检测） */
  let restartMonitorTimer: ReturnType<typeof setTimeout> | null = null
  /** 监控是否处于快速模式（断线恢复期间为 true） */
  let restartMonitorFast = false
  /** 轻量重启超时回退定时器（超时未恢复 → 全量重建） */
  let lightRestartFallbackTimer: ReturnType<typeof setTimeout> | null = null

  function clearRestartMonitor() {
    if (restartMonitorTimer) {
      clearTimeout(restartMonitorTimer)
      restartMonitorTimer = null
    }
    if (lightRestartFallbackTimer) {
      clearTimeout(lightRestartFallbackTimer)
      lightRestartFallbackTimer = null
    }
    restartMonitorFast = false
  }

  function scheduleRestartMonitor() {
    if (store.roomState.role !== 'guest' || !store.roomState.roomCode) return
    if (restartMonitorTimer) clearTimeout(restartMonitorTimer)
    restartMonitorTimer = setTimeout(
      () => void restartMonitorTick(),
      restartMonitorFast ? RESTART_MONITOR_FAST_MS : RESTART_MONITOR_SLOW_MS,
    )
  }

  /**
   * 轮询房主为本参与者上传的 Offer，发现新 Offer（ice-ufrag 变化）即重新 Answer 并提交。
   *
   * 房主在参与者 P2P 断线时执行 ICE restart 并上传新 Offer，加入方据此轻量恢复
   * （无需 leaveRoom/joinRoom 全量重建）；连接正常时慢速轮询，兼容「房主侧感知断线、
   * 加入方侧仍 connected」的不对称故障。
   */
  async function restartMonitorTick() {
    if (store.roomState.role !== 'guest' || !store.roomState.roomCode) return
    // 全量重建 / 初次协商进行中跳过，避免竞争（negotiating 由 useWebRTC 协商期间置 true）
    if (reconnecting || guestWebrtc.negotiating.value) return
    try {
      const pid = store.roomState.participantId
      if (!pid) return
      const result = await fetchParticipantOffer(store.roomState.roomCode, pid)
      if (result.code !== 1 || !result.data) return
      if (!result.data.ready || !result.data.sdpOffer) return
      if (result.data.sdpOffer === guestWebrtc.lastOfferSdp.value) return
      const iceServers = store.roomState.iceServers.length > 0
        ? store.roomState.iceServers
        : stunUrlsToIceServers(store.roomState.stunServers)
      const { sdp, iceCandidates } = await guestWebrtc.setRemoteOfferAndCreateAnswer(
        iceServers,
        result.data.sdpOffer,
        result.data.iceCandidates ?? [],
      )
      const resp = await submitAnswer(store.roomState.roomCode, pid, sdp, iceCandidates)
      if (resp.code === 1) {
        console.info(`[Online] 已响应房主 ICE restart，重新提交 Answer（${pid}）`)
      } else {
        console.warn(`[Online] 重新提交 Answer 失败: ${resp.msg}`)
      }
    } catch (e) {
      console.warn('[Online] 轮询房主新 Offer 异常:', e)
    } finally {
      scheduleRestartMonitor()
    }
  }

  /** 启动轻量重启：快速轮询捕获房主新 Offer，超时未恢复回退全量重建 */
  function startLightRestart() {
    restartMonitorFast = true
    scheduleRestartMonitor()
    if (lightRestartFallbackTimer) clearTimeout(lightRestartFallbackTimer)
    lightRestartFallbackTimer = setTimeout(() => {
      lightRestartFallbackTimer = null
      restartMonitorFast = false
      const cur = guestWebrtc.connectionState.value
      if (cur === 'failed' || cur === 'disconnected' || cur === 'closed') {
        console.info('[Online] 轻量重启超时未恢复，回退全量重建')
        void attemptGuestReconnect()
      }
    }, LIGHT_RESTART_TIMEOUT_MS)
  }

  function clearReconnectTimers() {
    if (reconnectTimer) {
      clearTimeout(reconnectTimer)
      reconnectTimer = null
    }
    if (disconnectedRecoveryTimer) {
      clearTimeout(disconnectedRecoveryTimer)
      disconnectedRecoveryTimer = null
    }
  }

  function scheduleGuestReconnect(delay: number) {
    if (reconnectTimer) return
    reconnectTimer = setTimeout(() => {
      reconnectTimer = null
      void attemptGuestReconnect()
    }, delay)
  }

  async function attemptGuestReconnect() {
    if (reconnecting) return
    const st = store.roomState
    if (st.role !== 'guest' || !st.roomCode) return
    if (reconnectAttempts >= GUEST_RECONNECT_ATTEMPTS) {
      toastError(`P2P 连接自动重连失败，已停止重试（房间 ${st.roomCode}）`)
      return
    }
    reconnectAttempts++
    reconnecting = true
    try {
      const ok = await reconnectAsGuest(guestWebrtc, peekJoinPassword(), lan)
      if (ok) reconnectAttempts = 0
      else {
        const backoff =
          RECONNECT_BACKOFF_MS[Math.min(reconnectAttempts - 1, RECONNECT_BACKOFF_MS.length - 1)]
        scheduleGuestReconnect(backoff)
      }
    } finally {
      reconnecting = false
    }
  }

  watch(() => guestWebrtc.connectionState.value, (state) => {
    if (store.roomState.role !== 'guest' || !store.roomState.roomCode) return
    if (state === 'connected') {
      reconnectAttempts = 0
      clearReconnectTimers()
      // 恢复后切回慢速轮询（房主主动 restart 时仍能感知新 Offer）
      restartMonitorFast = false
      if (lightRestartFallbackTimer) {
        clearTimeout(lightRestartFallbackTimer)
        lightRestartFallbackTimer = null
      }
      return
    }
    if (state === 'failed') {
      if (disconnectedRecoveryTimer) {
        clearTimeout(disconnectedRecoveryTimer)
        disconnectedRecoveryTimer = null
      }
      // 优先轻量重启：快速轮询捕获房主 ICE restart 的新 Offer 后重答；超时回退全量重建
      if (guestWebrtc.pc.value) {
        startLightRestart()
      } else {
        void attemptGuestReconnect()
      }
      return
    }
    if (state === 'disconnected') {
      if (disconnectedRecoveryTimer) return
      disconnectedRecoveryTimer = setTimeout(() => {
        disconnectedRecoveryTimer = null
        const cur = guestWebrtc.connectionState.value
        if (cur === 'failed' || cur === 'disconnected') startLightRestart()
      }, DISCONNECT_RECOVERY_DELAY_MS)
    }
  })

  // 加入方 DataChannel 全局绑定：控制消息（MC 端口/TURN）+ 数据包转发 TUN
  watch(
    () => guestWebrtc.dataChannel.value,
    (channel) => {
      if (!channel) return
      guestWebrtc.setDataChannelHandlers({
        onMessage: (raw) => {
          const msg = decode(raw)
          if (!msg) return
          if (msg.kind === 'control' && msg.subtype === CONTROL_SUBTYPE.HOST_MC_PORT) {
            const port = parseHostMcPortPayload(msg.payload)
            if (port !== null && port > 0) store.roomState.hostMcPort = port
            return
          }
          if (msg.kind === 'control' && msg.subtype === CONTROL_SUBTYPE.TURN_SERVERS) {
            const turnServers = decodeTurnServersPayload(msg.payload)
            if (!turnServers || turnServers.length === 0) return
            // 房主广播的 TURN 凭据绑定房主 IP+device，对参与者无效；
            // 保留参与者自拉的系统 TURN（regionCode 为标记），仅合并广播中的 STUN/自定义 TURN
            const ownTurn = store.roomState.iceServers.filter((e) => e.regionCode)
            const usable = turnServers.filter((e) => !e.regionCode)
            const merged = mergeIceServerEntries(ownTurn, usable)
            store.roomState.iceServers = merged
            const currentPc = guestWebrtc.pc.value
            if (currentPc) {
              try {
                currentPc.setConfiguration({
                  iceServers: merged.map((entry) => {
                    const server: RTCIceServer = { urls: entry.urls }
                    if (entry.username) server.username = entry.username
                    if (entry.credential) server.credential = entry.credential
                    return server
                  }),
                  iceTransportPolicy: 'all',
                })
              } catch (e) {
                console.warn('[Online] 加入方更新 PC 配置失败:', e)
              }
            }
            return
          }
          if (msg.kind === 'control' && msg.subtype === CONTROL_SUBTYPE.HOST_VIRTUAL_IP) {
            const ip = parseHostVirtualIpPayload(msg.payload)
            if (ip) store.roomState.hostVirtualIp = ip
            return
          }
          if (msg.kind === 'data') void lan.forwardToTun(raw)
        },
      })
    },
    { immediate: true },
  )

  // 加入方 MC 局域网伪装：本地伪装 LAN 服务器，本机 MC 多人游戏界面直接发现房主房间
  const lanFakePort = ref(0)
  const lanFakeActive = ref(false)
  let fakeSeq = 0
  let fakeKey = ''

  async function stopLanFake() {
    if (!lanFakeActive.value) return
    lanFakeActive.value = false
    lanFakePort.value = 0
    try {
      await lanFakeServerStop()
    } catch (e) {
      console.warn('[Online] 局域网伪装停止失败:', e)
    }
  }

  async function syncLanFake() {
    const st = store.roomState
    const active =
      st.role === 'guest' && lan.running.value && st.hostMcPort > 0 && !!st.hostVirtualIp
    if (!active) {
      await stopLanFake()
      return
    }
    const key = `${st.hostVirtualIp}:${st.hostMcPort}`
    if (lanFakeActive.value && fakeKey === key) return
    await stopLanFake()
    const seq = ++fakeSeq
    try {
      const res = await lanFakeServerStart({
        motd: st.roomCode ? `MoLaunch 联机 ${st.roomCode}` : 'MoLaunch 联机',
        targetIp: st.hostVirtualIp!,
        targetPort: st.hostMcPort,
      })
      // 期间被再次触发（停止/重启）则丢弃本次结果
      if (seq !== fakeSeq) {
        void lanFakeServerStop().catch(() => {})
        return
      }
      lanFakeActive.value = true
      fakeKey = key
      lanFakePort.value = res.port
    } catch (e) {
      console.warn('[Online] 局域网伪装启动失败:', e)
    }
  }

  watch(
    () =>
      [
        store.roomState.role,
        store.roomState.hostMcPort,
        store.roomState.hostVirtualIp,
        lan.running.value,
      ] as const,
    () => {
      void syncLanFake()
    },
  )

  /** 房间失效统一清理（房主 keepalive 关房 / 加入方监控发现关闭） */
  function handleRoomClosed(msg: string) {
    stopGuestMonitor()
    reconnectAttempts = 0
    clearReconnectTimers()
    clearRestartMonitor()
    hostOps.stop()
    void stopLanFake()
    void lan.stop()
    guestWebrtc.close()
    hostMesh.close()
    // store 全局保活也可能触发 resetRoomState，用 role 判断避免重复 toast
    if (store.roomState.role) {
      store.resetRoomState()
      toastError(msg)
    }
  }

  /** 加入方退出房间（退出按钮） */
  async function guestLeaveAndCleanup() {
    reconnectAttempts = 0
    clearReconnectTimers()
    clearRestartMonitor()
    await stopLanFake()
    await lan.stop()
    guestWebrtc.close()
    await store.guestLeaveRoom()
  }

  /** 加入方会话：密钥注入 + TUN 启动 + 房间状态监控 */
  function startGuestSession() {
    void importRoomKey(store.roomState.roomKey)
      .then((key) => guestWebrtc.setRoomKey(key))
      .catch((e) => console.warn('[Online] 加入方加密密钥导入失败:', e))
    void lan.start(store.roomState.selfVirtualIp, store.roomState.subnet).catch((e) => {
      toastError(`虚拟网卡启动失败：${e instanceof Error ? e.message : String(e)}`)
    })
    startGuestMonitor()
    // 启动房主新 Offer 监控（慢速模式；断线时 startLightRestart 切快速）
    scheduleRestartMonitor()
  }

  /** 按当前角色同步会话：进房启动、退房清理 */
  function syncSessionWithRole(role: string) {
    if (role === 'host') {
      hostOps.start()
    } else if (role === 'guest') {
      startGuestSession()
    } else {
      stopGuestMonitor()
      reconnectAttempts = 0
      clearReconnectTimers()
      clearRestartMonitor()
      hostOps.stop()
      void lan.stop()
      guestWebrtc.close()
      hostMesh.close()
    }
  }

  watch(() => store.roomState.role, (role) => syncSessionWithRole(role ?? ''), { immediate: true })

  return {
    hostMesh,
    guestWebrtc,
    lan,
    pendingAnswers: hostOps.pendingAnswers,
    offerGenerating: hostOps.offerGenerating,
    bannedList: hostOps.bannedList,
    banServerTime: hostOps.banServerTime,
    handleConfirm: hostOps.handleConfirm,
    handleKick: hostOps.handleKick,
    handleUnban: hostOps.handleUnban,
    refreshBans: hostOps.refreshBans,
    handleCloseRoom: hostOps.handleCloseRoom,
    setManualMcPort: hostOps.setManualMcPort,
    clearManualMcPort: hostOps.clearManualMcPort,
    guestLeaveAndCleanup,
    lanFakeActive,
    lanFakePort,
  }
}
