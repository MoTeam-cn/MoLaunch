/**
 * 联机 store 房间切片组合（状态 + 动作）
 */
import { useRoomActionsSlice } from './roomActions'
import { useRoomStateSlice } from './roomState'

/** 创建房间切片组合 */
export function useOnlineRoomSlice() {
  const state = useRoomStateSlice()
  const actions = useRoomActionsSlice(state)
  return { ...state, ...actions }
}
