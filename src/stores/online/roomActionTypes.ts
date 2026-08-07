import type { Ref } from 'vue'
import type { IceServerEntry, TurnServersResponse } from '@/types/online'
import type { RoomCreateStep, RoomState } from './types'

export interface RoomActionDeps {
  roomState: Ref<RoomState>
  roomLoading: Ref<boolean>
  roomCreateStep: Ref<RoomCreateStep>
  stunServers: Ref<string[]>
  customTurnServers: Ref<IceServerEntry[]>
  systemTurnServers: Ref<TurnServersResponse | null>
}
