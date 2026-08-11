import type { PublicFrpServer } from '@/types/frp'
import { FRP_ACTIONS, frpManager } from './core'

export function listPublicServers(): Promise<PublicFrpServer[]> {
  return frpManager<PublicFrpServer[]>(FRP_ACTIONS.LIST_PUBLIC_SERVERS)
}
