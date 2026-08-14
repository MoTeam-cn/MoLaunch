/**
 * 房主房间运营 composable（阶段三 mesh 拓扑）：切片组装 + 生命周期
 *
 * - useRoomHostPolling：三路信令轮询 / 自动 Offer / ICE restart / 系统 TURN 按需就位 / 定时器启停
 * - useRoomHostActions：确认/拒绝 Answer / 踢出封禁 / 解封 / 关闭房间
 * 只负责业务逻辑不渲染 UI；默认 onMounted 启动轮询与事件监听，onUnmounted 清理；
 * 全局联机会话传 `autoLifecycle: false`，由会话显式调用 `start` / `stop`。
 */

import { watch, onMounted, onUnmounted } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { useOnlineStore } from '@/stores/online'
import type { useWebRTCMesh } from '@/composables/useWebRTCMesh'
import type { useVirtualLan } from '@/composables/useVirtualLan'
import { importRoomKey } from '@/utils/online/crypto'
import { encodeHostMcPort } from '@/utils/online/protocol'
import { getRunningMcPort } from '@/utils/api/online-manager'
import { toastError } from '@/utils/toast'
import { useRoomHostPolling } from './useRoomHost/useRoomHostPolling'
import { useRoomHostActions } from './useRoomHost/useRoomHostActions'

/**
 * 房主房间运营 composable
 *
 * @param options.hostMesh 房主多 PC 管理器（由 RoomManager.vue 通过 provide/inject 注入）
 * @param options.lan 虚拟网卡桥接实例（由 RoomHostPanel.vue 创建并传入）
 * @param options.autoLifecycle 是否自动挂载/卸载生命周期（默认 true；全局会话传 false）
 * @param options.onRoomClosed 房间被服务端关闭时回调（全局会话用于统一清理）
 */
export function useRoomHost(options: {
  hostMesh: ReturnType<typeof useWebRTCMesh>
  lan: ReturnType<typeof useVirtualLan>
  autoLifecycle?: boolean
  onRoomClosed?: (msg: string) => void
}) {
  const { hostMesh, lan, autoLifecycle = true, onRoomClosed } = options
  const store = useOnlineStore()

  // 切片组装：轮询切片提供 pendingAnswers/offerGenerating 及轮询函数，
  // 动作切片依赖轮询切片的 pendingAnswers 引用，保持两切片状态同步
  const polling = useRoomHostPolling(store, hostMesh, lan, {
    onRoomClosed: (msg) => {
      // 服务端已关闭/销毁房间（keepalive 返回 1001）：
      // 组件侧仅负责清理连接与 TUN（hostMesh/lan 为组件持有，store 无法释放）；
      // store 层全局保活定时器已负责 resetRoomState + toast（见 stores/online.ts）
      stopTimers()
      void lan.stop()
      hostMesh.close()
      hostMesh.setRoomKey(null)
      onRoomClosed?.(msg)
    },
  })
  const actions = useRoomHostActions(store, hostMesh, lan)
  const {
    pendingAnswers,
    offerGenerating,
    participantNatTypes,
    pollParticipants,
    pollAnswers,
    doKeepalive,
    startTimers,
    stopTimers,
  } = polling
  const {
    bannedList,
    banServerTime,
    confirming,
    handleConfirm,
    handleKick,
    handleUnban,
    refreshBans,
    handleCloseRoom,
  } = actions

  // 定时器句柄（由轮询切片管理），此处仅维护 MC 端口事件监听器
  /** MC 端口检测事件监听器卸载函数 */
  let mcPortUnlisten: UnlistenFn | null = null
  /** HostMcPort 控制消息的本地 seq 计数器（与 TUN 数据包 seq 独立，避免混淆） */
  let mcPortSeq = 0

  /**
   * 应用自动捕获的 MC 端口（watcher 事件 / 进房回查共用）
   *
   * 手动指定端口为最高可信度，自动捕获结果不再覆盖；端口与当前一致时跳过，
   * 避免进房回查与事件驱动对同一端口重复广播。
   */
  function applyDetectedPort(port: number) {
    if (!port || port <= 0) return
    if (store.roomState.hostMcPortManual) return
    if (store.roomState.hostMcPort === port) return
    store.roomState.hostMcPort = port
    // 阶段三子任务 8：broadcastPacket 异步加密后发送，sent 计数仅用于日志
    void hostMesh.broadcastPacket(encodeHostMcPort(mcPortSeq++, port)).then((sent) => {
      console.info(
        `[Online] 房主 MC 局域网端口已捕获: ${port}，已广播给 ${sent} 个参与者`,
      )
    }).catch((e) => console.warn('[Online] 广播 MC 端口失败:', e))
  }

  /**
   * 启动房主运营（轮询 + 密钥注入 + TUN + TURN 广播 + MC 端口监听）
   *
   * 全局会话在进入房间（role=host）时调用；组件默认在 onMounted 调用。
   */
  function start() {
    void pollParticipants()
    void pollAnswers()
    void doKeepalive()
    // 阶段 6.2：加载初始封禁列表
    void refreshBans()
    startTimers()

    // 阶段三子任务 8：注入 DataChannel 加密密钥（空字符串表示未启用加密，importRoomKey 返回 null）
    // 在 lan.start 之前注入，确保首个 TUN 包就能正确加密
    void importRoomKey(store.roomState.roomKey)
      .then((key) => hostMesh.setRoomKey(key))
      .catch((e) => console.warn('[Online] 加密密钥导入失败:', e))

    // 启动 TUN 桥接：房主进入面板即创建 TUN 接口，开始读包 → broadcastPacket
    // 失败仅 toast（如 wintun.dll 缺失 / 无管理员权限），不阻塞信令流程
    void lan.start(store.roomState.selfVirtualIp, store.roomState.subnet).catch((e) => {
      toastError(`虚拟网卡启动失败：${e instanceof Error ? e.message : String(e)}`)
    })

    // 建房瞬间不主动拉取系统 TURN（尚无参与者，避免白费 /turn 请求与 PoW 计算）；
    // 首个参与者加入生成 Offer 时按需拉取、一个房间一次（见 useRoomHostPolling.ensureSystemTurnServers）

    // 监听后端 GameWatcher 的 MC 局域网端口检测事件
    // 房主在 MC 中「Open to LAN」后，watcher 捕获 stdout/监听端口 → emit 此事件
    void listen<number>('online://mc-port-detected', (event) => {
      applyDetectedPort(event.payload)
    }).then((unlisten) => {
      mcPortUnlisten = unlisten
    }).catch((e) => console.warn('[Online] 注册 MC 端口检测事件监听失败:', e))

    // 进房回查：先启动 MC（已开放局域网）再进房时，端口事件在监听注册前发出
    // 已被丢弃且 watcher 按端口去重不会重发，主动回查当前游戏进程补上
    void getRunningMcPort().then((res) => {
      if (!res.success || res.ports.length === 0) return
      // 与 watcher 事件「后者覆盖」的生效顺序一致，取候选端口最后一项
      applyDetectedPort(res.ports[res.ports.length - 1])
    }).catch((e) => console.warn('[Online] 回查 MC 局域网端口失败:', e))
  }

  /** 停止房主运营（停轮询 + 移除 MC 端口监听），幂等 */
  function stop() {
    stopTimers()
    if (mcPortUnlisten) {
      mcPortUnlisten()
      mcPortUnlisten = null
    }
  }

  /**
   * 房主手动指定 MC 端口（最高可信度）
   *
   * 自动捕获（日志/监听端口）失败或不可靠时由用户直接指定；
   * 设置后自动捕获结果不再覆盖，并立即广播给所有已联通参与者。
   */
  function setManualMcPort(port: number) {
    if (store.roomState.role !== 'host') return
    if (!Number.isInteger(port) || port <= 0 || port > 65535) return
    store.roomState.hostMcPort = port
    store.roomState.hostMcPortManual = true
    void hostMesh.broadcastPacket(encodeHostMcPort(mcPortSeq++, port)).then((sent) => {
      console.info(`[Online] 房主手动设置 MC 端口: ${port}，已广播给 ${sent} 个参与者`)
    }).catch((e) => console.warn('[Online] 广播手动 MC 端口失败:', e))
  }

  /** 清除手动端口标记，恢复自动捕获更新 */
  function clearManualMcPort() {
    if (store.roomState.role !== 'host') return
    store.roomState.hostMcPortManual = false
  }

  if (autoLifecycle) {
    onMounted(start)
    onUnmounted(stop)
  }

  // 云端连接状态变化时暂停/恢复轮询（避免云端断开后持续失败刷屏）
  watch(() => store.cloudConnected, (connected) => {
    if (connected) {
      startTimers()
      // 断连恢复自动补发：网络恢复后立即上报一次保活/参与者/Answer，
      // 避免在 keepalive_timeout(120s) 窗口内因漏报被服务端判定失联关房
      void doKeepalive()
      void pollParticipants()
      void pollAnswers()
    } else {
      stopTimers()
    }
  })

  return {
    pendingAnswers,
    offerGenerating,
    participantNatTypes,
    bannedList,
    banServerTime,
    confirming,
    handleConfirm,
    handleKick,
    handleUnban,
    refreshBans,
    handleCloseRoom,
    setManualMcPort,
    clearManualMcPort,
    start,
    stop,
  }
}
