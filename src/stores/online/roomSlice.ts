/**
 * 联机 store 房间切片（聚合入口）
 *
 * 组合 roomState.ts（状态）+ roomActions.ts（动作），对外保持原 useOnlineRoomSlice
 * 签名不变，供 stores/online.ts 组合白名单/认证等切片。
 */
import { useRoomStateSlice } from './roomState'
import { useRoomActionsSlice } from './roomActions'

/** 创建联机 store 房间切片 */
export function useOnlineRoomSlice() {
  const state = useRoomStateSlice()
  const actions = useRoomActionsSlice(state)

  return {
    ...state,
    ...actions,
  }
}