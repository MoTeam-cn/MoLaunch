/**
 * 管理员提权重启后的加入方房间自动重连
 *
 * 流程：清理服务端旧 participant 记录 → 重新加入同一房间（新 participant_id，
 * 房主 pollParticipants 会自动为新参与者重新生成 Offer）→ 拉取新 Offer →
 * 生成本地 Answer → 提交，完成 WebRTC 重建，期间 roomState 始终保持 guest。
 * 同时承担 P2P 断线自动重连（onlineSession 调用）：close 旧连接、重新注入
 * 加密密钥、虚拟 IP 变化时重启 TUN。
 */

import { onMounted } from 'vue'
import { useOnlineStore } from '@/stores/online'
import type { useWebRTC } from '@/composables/useWebRTC'
import type { useVirtualLan } from '@/composables/useVirtualLan'
import { joinRoom, leaveRoom, submitAnswer } from '@/utils/api/online-manager'
import { importRoomKey } from '@/utils/online/crypto'
import { consumeReconnectPassword } from '@/utils/relaunchSnapshot'
import { toastError, toastSuccess } from '@/utils/toast'

/**
 * 加入方房间重连主体（UAC 提权重启后由 useGuestReconnect 触发；P2P 断线由全局会话触发）
 *
 * @param lan 虚拟网卡实例（非空时在虚拟 IP 重新分配后重启 TUN 接口）
 * @returns 是否重连成功（供自动重连编排决定是否继续退避重试）
 */
export async function reconnectAsGuest(
  guestWebrtc: ReturnType<typeof useWebRTC>,
  password: string,
  lan?: ReturnType<typeof useVirtualLan>,
): Promise<boolean> {
  const store = useOnlineStore()
  const roomCode = store.roomState.roomCode
  if (store.roomState.role !== 'guest' || !roomCode) return false

  try {
    // 清理旧 PC / DataChannel / 密钥（幂等；重连后创建全新连接，避免 failed 态复用）
    guestWebrtc.close()

    // 先移除服务端旧 participant 记录（房间不存在等失败不阻塞，直接走重加入）
    await leaveRoom(roomCode).catch(() => {})

    // 重新加入：新 participant_id 触发房主自动生成新 Offer
    const resp = await joinRoom(roomCode, password)
    if (resp.code !== 1 || !resp.data) throw new Error(resp.msg || '重新加入房间失败')
    const data = resp.data
    store.roomState.participantId = data.participantId
    store.roomState.roomKey = data.roomKey || store.roomState.roomKey
    // 参与者自拉系统 TURN（凭据绑定自身 IP/device，P2P 打洞失败时走中继），
    // 未启用 TURN 时返回云端 ice_servers / stunServers 兜底
    const iceServers = await store.guestPullTurnServers()
    if (iceServers.length > 0) store.roomState.iceServers = iceServers

    // 重新注入加密密钥（新房间密钥）
    const key = await importRoomKey(store.roomState.roomKey)
    guestWebrtc.setRoomKey(key)

    // 服务端按房间递增分配虚拟 IP，重新 join 必然拿到新 IP：重启 TUN 使接口 IP 保持一致
    if (lan && data.playerVirtualIp) {
      store.roomState.selfVirtualIp = data.playerVirtualIp
      await lan.start(data.playerVirtualIp, store.roomState.subnet).catch((e) => {
        toastError(`虚拟网卡重启失败：${e instanceof Error ? e.message : String(e)}`)
      })
    }

    // 拉取房主新 Offer → 生成本地 Answer → 提交，完成 WebRTC 重建
    const { sdp, iceCandidates } = await guestWebrtc.fetchOfferAndAnswer(
      roomCode,
      data.participantId,
      iceServers,
    )
    const answer = await submitAnswer(roomCode, data.participantId, sdp, iceCandidates)
    if (answer.code !== 1) throw new Error(answer.msg || '提交 Answer 失败')

    // 同步房间公开信息（房主 MC 版本/端口等可能已变化）
    await store.refreshRoomInfo().catch(() => {})
    toastSuccess(`已自动重连房间：${roomCode}`)
    return true
  } catch (e) {
    toastError(`房间自动重连失败：${e instanceof Error ? e.message : String(e)}`)
    return false
  }
}

/** 组件挂载时若存在提权重启的待重连密码，自动执行加入方重连 */
export function useGuestReconnect(
  guestWebrtc: ReturnType<typeof useWebRTC>,
  lan?: ReturnType<typeof useVirtualLan>,
): void {
  onMounted(() => {
    const password = consumeReconnectPassword()
    if (password !== null) {
      void reconnectAsGuest(guestWebrtc, password, lan)
    }
  })
}
