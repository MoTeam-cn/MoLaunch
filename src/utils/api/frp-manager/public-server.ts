import type {
  AllocatePublicServerParams,
  AllocateResponse,
  PublicFrpServer,
} from '@/types/frp'
import { FRP_ACTIONS, frpManager } from './core'

export function listPublicServers(): Promise<PublicFrpServer[]> {
  return frpManager<PublicFrpServer[]>(FRP_ACTIONS.LIST_PUBLIC_SERVERS)
}
export function allocatePublicServer(params: AllocatePublicServerParams): Promise<AllocateResponse> {
  return frpManager<AllocateResponse>(FRP_ACTIONS.ALLOCATE_PUBLIC_SERVER, params)
}
export function releasePublicServer(allocationId: string): Promise<void> {
  return frpManager<void>(FRP_ACTIONS.RELEASE_PUBLIC_SERVER, { allocationId })
}
export function keepalivePublicServer(allocationId: string): Promise<unknown> {
  return frpManager(FRP_ACTIONS.KEEPALIVE_PUBLIC_SERVER, { allocationId })
}
