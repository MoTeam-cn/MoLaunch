import type {
  AuthStatus,
  DeviceCodePollResult,
  DeviceCodeResult,
  OAuth2Result,
  SaveApiKeyParams,
} from '@/types/frp'
import { FRP_ACTIONS, frpManager } from './core'

export function getAuthStatus(providerId: string): Promise<AuthStatus> {
  return frpManager<AuthStatus>(FRP_ACTIONS.GET_AUTH_STATUS, { providerId })
}
export function startOAuth2(providerId: string): Promise<OAuth2Result> {
  return frpManager<OAuth2Result>(FRP_ACTIONS.START_OAUTH2, { providerId })
}
export function startDeviceCode(providerId: string): Promise<DeviceCodeResult> {
  return frpManager<DeviceCodeResult>(FRP_ACTIONS.START_DEVICE_CODE, { providerId })
}
export function pollDeviceCode(providerId: string): Promise<DeviceCodePollResult> {
  return frpManager<DeviceCodePollResult>(FRP_ACTIONS.POLL_DEVICE_CODE, { providerId })
}
export function refreshToken(providerId: string): Promise<void> {
  return frpManager<void>(FRP_ACTIONS.REFRESH_TOKEN, { providerId })
}
export function revokeAuth(providerId: string): Promise<void> {
  return frpManager<void>(FRP_ACTIONS.REVOKE_AUTH, { providerId })
}
export function saveApiKey(params: SaveApiKeyParams): Promise<void> {
  return frpManager<void>(FRP_ACTIONS.SAVE_API_KEY, params)
}
