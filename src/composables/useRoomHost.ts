/**
 * 房主房间运营 composable（阶段三 mesh 拓扑）
 *
 * 从 RoomHostPanel.vue 抽出，封装房主侧全部业务逻辑，拆分为两个职责切片：
 * - useRoomHostPolling：三路信令轮询（5s 参与者 + 5s Answer + 30s 保活）、
 *   自动 Offer 生成、30s 防刷屏 toast、TURN 广播与定时器启停
 * - useRoomHostActions：确认/拒绝 Answer、踢出（可选封禁）、封禁列表、解封、关闭房间
 *
 * 本文件负责切片组装与生命周期（onMounted 初始轮询 + 事件监听 + onUnmounted 清理），
 * 对外 useRoomHost() 返回结构保持不变，调用方（RoomHostPanel.vue）无需改动。
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

import { watch, onMounted, onUnmounted } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { useOnlineStore } from '@/stores/online'
import type { useWebRTCMesh } from '@/composables/useWebRTCMesh'
import type { useVirtualLan } from '@/composables/useVirtualLan'
import { importRoomKey } from '@/utils/online/crypto'
import { encodeHostMcPort } from '@/utils/online/protocol'
import { toastError } from '@/utils/toast'
import { useRoomHostPolling } from './useRoomHost/useRoomHostPolling'
import { useRoomHostActions } from './useRoomHost/useRoomHostActions'

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

  // 切片组装：轮询切片提供 pendingAnswers/offerGenerating 及轮询函数，
  // 动作切片依赖轮询切片的 pendingAnswers 引用，保持两切片状态同步
  const polling = useRoomHostPolling(store, hostMesh, lan)
  const actions = useRoomHostActions(store, hostMesh, lan, polling.pendingAnswers)
  const {
    pendingAnswers,
    offerGenerating,
    pollParticipants,
    pollAnswers,
    doKeepalive,
    startTimers,
    stopTimers,
    fetchAndBroadcastTurnServers,
  } = polling
  const {
    bannedList,
    banServerTime,
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

  onMounted(() => {
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
      }).catch((e) => console.warn('[Online] 广播 MC 端口失败:', e))
    }).then((unlisten) => {
      mcPortUnlisten = unlisten
    }).catch((e) => console.warn('[Online] 注册 MC 端口检测事件监听失败:', e))
  })

  // 云端连接状态变化时暂停/恢复轮询（避免云端断开后持续失败刷屏）
  watch(() => store.cloudConnected, (connected) => {
    if (connected) {
      startTimers()
    } else {
      stopTimers()
    }
  })

  onUnmounted(() => {
    stopTimers()
    if (mcPortUnlisten) {
      mcPortUnlisten()
      mcPortUnlisten = null
    }
    // lan.stop 由 useVirtualLan 的 onUnmounted 自动处理
  })

  return {
    pendingAnswers,
    offerGenerating,
    bannedList,
    banServerTime,
    handleConfirm,
    handleKick,
    handleUnban,
    refreshBans,
    handleCloseRoom,
  }
}
