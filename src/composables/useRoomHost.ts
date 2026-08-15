/**
 * 房主联机编排 composable（easytier + Scaffolding 收敛版）
 *
 * 新架构去 SDP/轮询：房间登记由 store.hostCreateRoom 完成，此处负责
 * 联机中心一站式拉起（hostStart）与关闭（handleCloseRoom）。
 */
import { computed } from 'vue'
import { useOnlineStore } from '@/stores/online'
import { scaffoldingHostStop } from '@/utils/api/online-manager/easytier'
import { useEasyTier } from './useEasyTier'
import { useScaffolding } from './useScaffolding'

/** 房主联机编排 composable */
export function useRoomHost() {
  const store = useOnlineStore()
  const scaffolding = useScaffolding()
  const easytier = useEasyTier()

  const room = computed(() => store.roomState)

  /** 房主拉起联机中心（房间已登记，roomState.role === 'host'） */
  async function hostStart(): Promise<{ ok: boolean; error?: string }> {
    const state = store.roomState
    if (state.role !== 'host' || !state.roomCode) {
      return { ok: false, error: '尚未创建房间' }
    }
    return scaffolding.hostStart(state.roomCode, state.hostMcPort || undefined)
  }

  /** 房主关闭房间（先停联机中心/easytier，再 room_close） */
  async function handleCloseRoom(): Promise<void> {
    try {
      await scaffolding.hostStop()
    } finally {
      await store.hostCloseRoom()
    }
  }

  return { room, scaffolding, easytier, hostStart, handleCloseRoom }
}
