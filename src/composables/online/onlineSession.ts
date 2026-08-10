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
import { watch } from 'vue'
import { useOnlineStore } from '@/stores/online'
import { useWebRTC } from '@/composables/useWebRTC'
import { useWebRTCMesh } from '@/composables/useWebRTCMesh'
import { useVirtualLan } from '@/composables/useVirtualLan'
import { useRoomHost } from '@/composables/useRoomHost'
import { reconnectAsGuest } from '@/composables/useRoomReconnect'
import { getRoomInfo } from '@/utils/api/online-manager'
import { importRoomKey } from '@/utils/online/crypto'
import { peekJoinPassword } from '@/utils/relaunchSnapshot'
import {
  decode,
  CONTROL_SUBTYPE,
  parseHostMcPortPayload,
  decodeTurnServersPayload,
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
  /** 加入方退出房间（停 TUN + 关 P2P + 云端退出） */
  guestLeaveAndCleanup: () => Promise<void>
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
        void hostMesh.broadcastPacket(raw)
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
  let reconnectAttempts = 0
  let reconnecting = false
  let reconnectTimer: ReturnType<typeof setTimeout> | null = null
  let disconnectedRecoveryTimer: ReturnType<typeof setTimeout> | null = null

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
      return
    }
    if (state === 'failed') {
      if (disconnectedRecoveryTimer) {
        clearTimeout(disconnectedRecoveryTimer)
        disconnectedRecoveryTimer = null
      }
      void attemptGuestReconnect()
      return
    }
    if (state === 'disconnected') {
      if (disconnectedRecoveryTimer) return
      disconnectedRecoveryTimer = setTimeout(() => {
        disconnectedRecoveryTimer = null
        const cur = guestWebrtc.connectionState.value
        if (cur === 'failed' || cur === 'disconnected') void attemptGuestReconnect()
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
            store.roomState.iceServers = turnServers
            const currentPc = guestWebrtc.pc.value
            if (currentPc) {
              try {
                currentPc.setConfiguration({
                  iceServers: turnServers.map((entry) => {
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
          if (msg.kind === 'data') void lan.forwardToTun(raw)
        },
      })
    },
    { immediate: true },
  )

  /** 房间失效统一清理（房主 keepalive 关房 / 加入方监控发现关闭） */
  function handleRoomClosed(msg: string) {
    stopGuestMonitor()
    reconnectAttempts = 0
    clearReconnectTimers()
    hostOps.stop()
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
    guestLeaveAndCleanup,
  }
}
