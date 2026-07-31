/**
 * 联机 API - mesh 拓扑：参与者级 SDP Offer（阶段三子任务 5）
 *
 * 房主为每个新加入的参与者单独创建 PeerConnection + Offer 后上传；
 * 参与者轮询拉取自己的 Offer，ready=false 表示房主尚未生成。
 */

import type { BusinessResult, ParticipantOfferResponse } from '@/types/online'
import { ONLINE_ACTIONS, onlineManager } from './core'

/** 房主为指定参与者上传 SDP Offer（mesh 拓扑） */
export function uploadParticipantOffer(
  roomCode: string,
  participantId: string,
  sdpOffer: string,
  iceCandidates: string[],
): Promise<BusinessResult<unknown>> {
  return onlineManager(ONLINE_ACTIONS.ROOM_UPLOAD_PARTICIPANT_OFFER, {
    roomCode,
    participantId,
    sdpOffer,
    iceCandidates,
  })
}

/** 参与者拉取房主为自己生成的 SDP Offer（mesh 拓扑，轮询用） */
export function fetchParticipantOffer(
  roomCode: string,
  participantId: string,
): Promise<BusinessResult<ParticipantOfferResponse>> {
  return onlineManager(ONLINE_ACTIONS.ROOM_FETCH_PARTICIPANT_OFFER, {
    roomCode,
    participantId,
  })
}
