/**
 * 管理员提权重启后的加入方房间自动重连
 *
 * 流程：清理服务端旧 participant 记录 → 重新加入同一房间（新 participant_id，
 * 房主 pollParticipants 会自动为新参与者重新生成 Offer）→ 拉取新 Offer →
 * 生成本地 Answer → 提交，完成 WebRTC 重建，期间 roomState 始终保持 guest。
 */

import { onMounted } from 'vue'
import { useOnlineStore } from '@/stores/online'
import type { useWebRTC } from '@/composables/useWebRTC'
import { joinRoom, leaveRoom, submitAnswer } from '@/utils/api/online-manager'
import { resolveIceServers } from '@/utils/online/webrtc-helpers'
import { consumeReconnectPassword } from '@/utils/relaunchSnapshot'
import { toastError, toastSuccess } from '@/utils/toast'

/** 加入方房间重连主体（RoomGuestPanel 挂载后由 useGuestReconnect 触发） */
export async function reconnectAsGuest(
  guestWebrtc: ReturnType<typeof useWebRTC>,
  password: string,
): Promise<void> {
  const store = useOnlineStore()
  const roomCode = store.roomState.roomCode
  if (store.roomState.role !== 'guest' || !roomCode) return

  try {
    // 先移除服务端旧 participant 记录（房间不存在等失败不阻塞，直接走重加入）
    await leaveRoom(roomCode).catch(() => {})

    // 重新加入：新 participant_id 触发房主自动生成新 Offer
    const resp = await joinRoom(roomCode, password)
    if (resp.code !== 1 || !resp.data) throw new Error(resp.msg || '重新加入房间失败')
    const data = resp.data
    store.roomState.participantId = data.participantId
    store.roomState.roomKey = data.roomKey || store.roomState.roomKey
    const iceServers = resolveIceServers(data.iceServers, data.stunServers)
    if (iceServers.length > 0) store.roomState.iceServers = iceServers

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
  } catch (e) {
    toastError(`房间自动重连失败：${e instanceof Error ? e.message : String(e)}`)
  }
}

/** 组件挂载时若存在提权重启的待重连密码，自动执行加入方重连 */
export function useGuestReconnect(guestWebrtc: ReturnType<typeof useWebRTC>): void {
  onMounted(() => {
    const password = consumeReconnectPassword()
    if (password !== null) {
      void reconnectAsGuest(guestWebrtc, password)
    }
  })
}
