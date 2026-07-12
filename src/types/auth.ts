/** 认证类型 */
export enum AuthType {
  Microsoft = 0,
  Offline = 1,
  External = 2,
}

/** 认证结果 */
export interface AuthResult {
  name: string
  uuid: string
  access_token: string
  client_token: string
  login_type: string
  profile_json?: string
}

/** 登录状态 */
export type LoginStatus = 'idle' | 'loading' | 'success' | 'error'

/** SDK 状态 */
export interface SdkStatus {
  loaded: boolean
  version?: string
  platform: string
  library_path: string
}

// ============================================================
// 微软登录相关类型
// ============================================================

/** 设备码信息 */
export interface DeviceCodeInfo {
  user_code: string
  verification_uri: string
  device_code: string
  expires_in: number
  interval: number
  message: string
}

/** 轮询结果 */
export type PollResult =
  | { status: 'Pending' }
  | {
      status: 'Success'
      name: string
      uuid: string
      access_token: string
      client_token: string
      login_type: string
      profile_json: string | null
    }

/** 已存储的微软账号 */
export interface MsAccountInfo {
  username: string
  uuid: string
  expires_at: number
  is_expired: boolean
}
