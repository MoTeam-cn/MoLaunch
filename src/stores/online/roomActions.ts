/**
 * 联机 store 房间动作聚合入口。
 * 房间 CRUD、信息/保活/TURN、参与者职责分别由相邻模块实现。
 */
import { useRoomCrudActions } from './roomCrudActions'
import { useRoomParticipantsActions } from './roomParticipantsActions'
import { useRoomRefreshActions } from './roomRefreshActions'
import type { RoomActionDeps } from './roomActionTypes'

export type { RoomActionDeps } from './roomActionTypes'
export { RoomClosedError } from './roomRefreshActions'

export function useRoomActionsSlice(deps: RoomActionDeps) {
  const refreshActions = useRoomRefreshActions({
    roomState: deps.roomState,
    systemTurnServers: deps.systemTurnServers,
  })
  const crudActions = useRoomCrudActions(deps, refreshActions.refreshRoomInfo)
  const participantsActions = useRoomParticipantsActions(deps.roomState)

  return {
    ...crudActions,
    ...refreshActions,
    ...participantsActions,
  }
}
