import type { Ref } from 'vue'
import type { ListParticipantsResponse, ParticipantInfo } from '@/types/online'
import { listParticipants } from '@/utils/api/online-manager'
import type { RoomState } from './types'

export function useRoomParticipantsActions(roomState: Ref<RoomState>) {
  async function refreshParticipants(): Promise<ParticipantInfo[]> {
    if (roomState.value.role !== 'host' || !roomState.value.roomCode) return []
    const result = await listParticipants(roomState.value.roomCode)
    if (result.code !== 1 || !result.data) return []
    const list = (result.data as ListParticipantsResponse).participants ?? []
    roomState.value.participants = list
    return list
  }

  return { refreshParticipants }
}
