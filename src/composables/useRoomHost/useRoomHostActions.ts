/**
 * 房主交互动作切片（useRoomHost 拆分）
 *
 * 接受/拒绝加入申请（授权前置）、踢出（可选封禁）、封禁列表、解封、关闭房间；
 * 参与者列表由轮询切片维护，此处仅消费 store + hostMesh + lan。
 */
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
import type { ParticipantInfo, RoomBan } from '@/types/online'
import { showConfirm } from '@/utils/modal'
import { toastSuccess, toastError } from '@/utils/toast'

export function useRoomHostActions(
  store: ReturnType<typeof useOnlineStore>,
  hostMesh: ReturnType<typeof useWebRTCMesh>,
  lan: ReturnType<typeof useVirtualLan>,
) {
  /** 封禁列表（仅房主，按需刷新：挂载时 + 踢人带封禁后 + 解封后） */
  const bannedList = ref<RoomBan[]>([])
  /** 服务端当前时间（Unix 秒，由 listBannedParticipants 返回，用于计算剩余封禁时长） */
  const banServerTime = ref(0)
  /** 正在处理确认/拒绝的参与者（key=participantId，防连点重复提交） */
  const confirming = new Set<string>()

  /** 刷新封禁列表（仅房主，按需调用：挂载时 / 踢人带封禁后 / 解封后） */
  async function refreshBans() {
    if (store.roomState.role !== 'host' || !store.roomState.roomCode) return
    try {
      const result = await listBannedParticipants(store.roomState.roomCode)
      if (result.code === 1 && result.data) {
        bannedList.value = result.data.bans ?? []
        banServerTime.value = result.data.serverTime ?? 0
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
   * 确认/拒绝加入申请（授权前置：接受后才生成 Offer，加入方授权前只等待不连接）
   *
   * - 接受：confirmParticipant(true) → 参与者状态变 confirmed → 下一轮 pollParticipants
   *   自动为其生成 SDP Offer，加入方拿到 Offer 后提交 Answer，pollAnswers 自动放行建连
   * - 拒绝：confirmParticipant(false) → 清理残留 PC，参与者状态变 rejected
   */
  async function handleConfirm(participant: ParticipantInfo, accepted: boolean) {
    // 防连点：同一条申请在请求返回前禁止再次提交
    if (confirming.has(participant.participantId)) return
    confirming.add(participant.participantId)
    try {
      const result = await confirmParticipant(
        store.roomState.roomCode,
        participant.participantId,
        accepted,
      )
      if (result.code !== 1) throw new Error(result.msg || '确认操作失败')
      if (!accepted) {
        // 拒绝申请：关闭可能存在的残留 PC（Offer 尚未生成时 PC 不存在）
        hostMesh.closeParticipant(participant.participantId)
      }
      toastSuccess(accepted ? '已接受加入申请' : '已拒绝加入申请')
      await store.refreshParticipants()
    } catch (e) {
      toastError(`确认失败：${e instanceof Error ? e.message : String(e)}`)
    } finally {
      confirming.delete(participant.participantId)
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
    confirming,
    refreshBans,
    handleConfirm,
    handleKick,
    handleUnban,
    handleCloseRoom,
  }
}
