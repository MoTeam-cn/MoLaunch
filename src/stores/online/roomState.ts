/**
 * 联机 store 房间状态切片（状态 + 加载标记 + 创建步骤）
 */
import { ref } from 'vue'
import type { RoomCreateStep, RoomState } from './types'
import { emptyRoom } from './types'

/** 房间状态切片 */
export function useRoomStateSlice() {
  const roomState = ref<RoomState>(emptyRoom())
  const roomLoading = ref(false)
  const roomCreateStep = ref<RoomCreateStep>(null)

  /** 重置房间状态（不调用后端，仅清空本地） */
  function resetRoomState(): void {
    roomState.value = emptyRoom()
    roomCreateStep.value = null
  }

  return { roomState, roomLoading, roomCreateStep, resetRoomState }
}
