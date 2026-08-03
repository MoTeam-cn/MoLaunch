/**
 * 房主交互动作切片（useRoomHost 拆分）
 *
 * 确认/拒绝 Answer、踢出（可选封禁）、封禁列表、解封、关闭房间；
 * pendingAnswers 由轮询切片创建，经参数传入保持状态同步。
 */
import type { Ref } from 'vue'
import { ref } from 'vue'
import { useOnlineStore } from '@/stores/online'
import type { useWebRTCMesh } from '@/composables/useWebRTCMesh'
import type { useVirtualLan } from '@/composables/useVirtualLan'
import {
  confirmParticipant,
  kickParticipant,
  listBannedParticipants,
  unbanParticipant,
} from '@/utils/api/online-manager'
import type { PendingAnswer, RoomBan } from '@/types/online'
import { showConfirm } from '@/utils/modal'
import { toastSuccess, toastError } from '@/utils/toast'

export function useRoomHostActions(
  store: ReturnType<typeof useOnlineStore>,
  hostMesh: ReturnType<typeof useWebRTCMesh>,
  lan: ReturnType<typeof useVirtualLan>,
  pendingAnswers: Ref<PendingAnswer[]>,
) {
  /** 封禁列表（仅房主，按需刷新：挂载时 + 踢人带封禁后 + 解封后） */
  const bannedList = ref<RoomBan[]>([])
  /** 服务端当前时间（Unix 秒，由 listBannedParticipants 返回，用于计算剩余封禁时长） */
  const banServerTime = ref(0)

  /** 刷新封禁列表（仅房主，按需调用：挂载时 / 踢人带封禁后 / 解封后） */
  async function refreshBans() {
    if (store.roomState.role !== 'host' || !store.roomState.roomCode) return
    try {
      const result = await listBannedParticipants(store.roomState.roomCode)
      if (result.code === 1 && result.data) {
        bannedList.value = result.data.bans ?? []
        banServerTime.value = result.data.serverTime ?? 0
        toastSuccess('封禁列表已刷新')
      } else {
        console.warn(`[Online] refreshBans 业务失败: code=${result.code}, msg=${result.msg}`)
        toastError(result.msg || '刷新封禁列表失败')
      }
    } catch (e) {
      console.warn('[Online] refreshBans 异常:', e)
      toastError('刷新封禁列表失败')
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

  /** 踢出参与者（可选封禁时长：null=不封禁 / 0=永久 / N=封禁N秒） */
  async function handleKick(participantId: string, _devicePk: string, banDuration: number | null) {
    try {
      const result = await kickParticipant(store.roomState.roomCode, participantId, banDuration)
      if (result.code !== 1) throw new Error(result.msg || '踢出失败')
      // 关闭对应 PC
      hostMesh.closeParticipant(participantId)
      const msg = banDuration === null
        ? '已踢出'
        : banDuration === 0
          ? '已踢出并永久封禁'
          : `已踢出并封禁 ${Math.round(banDuration / 60)} 分钟`
      toastSuccess(msg)
      await store.refreshParticipants()
      // 阶段 6.2：带封禁的踢出刷新封禁列表
      if (banDuration !== null) void refreshBans()
    } catch (e) {
      toastError(`踢出失败：${e instanceof Error ? e.message : String(e)}`)
    }
  }

  /** 解封参与者（仅房主） */
  async function handleUnban(devicePk: string) {
    try {
      const result = await unbanParticipant(store.roomState.roomCode, devicePk)
      if (result.code !== 1) throw new Error(result.msg || '解封失败')
      toastSuccess('已解封')
      await refreshBans()
    } catch (e) {
      toastError(`解封失败：${e instanceof Error ? e.message : String(e)}`)
    }
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

  return {
    bannedList,
    banServerTime,
    refreshBans,
    handleConfirm,
    handleKick,
    handleUnban,
    handleCloseRoom,
  }
}
