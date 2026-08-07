import type {
  ImportedFrpcConfig,
  LogFileContent,
  LogFileInfo,
  ProviderInfo,
  Tunnel,
  TunnelStatus,
  TunnelWithStatus,
  CreateTunnelParams,
  UpdateTunnelParams,
} from '@/types/frp'
import { FRP_ACTIONS, frpManager } from './core'

export function listTunnels(): Promise<TunnelWithStatus[]> {
  return frpManager(FRP_ACTIONS.LIST_TUNNELS)
}
export function createTunnel(params: CreateTunnelParams): Promise<Tunnel> {
  return frpManager(FRP_ACTIONS.CREATE_TUNNEL, params)
}
export function updateTunnel(params: UpdateTunnelParams): Promise<Tunnel> {
  return frpManager<Tunnel>(FRP_ACTIONS.UPDATE_TUNNEL, params)
}
export function deleteTunnel(id: string): Promise<void> {
  return frpManager(FRP_ACTIONS.DELETE_TUNNEL, { id })
}
export function importFrpcConfig(path: string): Promise<ImportedFrpcConfig> {
  return frpManager(FRP_ACTIONS.IMPORT_FRPC_CONFIG, { path })
}
export function startTunnel(id: string): Promise<void> {
  return frpManager(FRP_ACTIONS.START_TUNNEL, { id })
}
export function stopTunnel(id: string): Promise<void> {
  return frpManager(FRP_ACTIONS.STOP_TUNNEL, { id })
}
export function getTunnelStatus(id: string): Promise<TunnelStatus> {
  return frpManager(FRP_ACTIONS.GET_TUNNEL_STATUS, { id })
}
export function listLogFiles(): Promise<LogFileInfo[]> {
  return frpManager<LogFileInfo[]>(FRP_ACTIONS.LIST_LOG_FILES)
}
export function readLogFile(tunnelId: string, maxLines?: number): Promise<LogFileContent> {
  return frpManager<LogFileContent>(FRP_ACTIONS.READ_LOG_FILE, { tunnelId, maxLines })
}
export function clearLogFile(tunnelId: string): Promise<void> {
  return frpManager<void>(FRP_ACTIONS.CLEAR_LOG_FILE, { tunnelId })
}

export type { ProviderInfo }
