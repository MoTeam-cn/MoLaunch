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

/** 已存储的微软账号 */
export interface MsAccountInfo {
  username: string
  uuid: string
  expires_at: number
  is_expired: boolean
}

/** 设备码信息（后端返回给前端显示） */
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
  | { status: 'Success'; auth: AuthResult }
  | { status: 'Declined' }
  | { status: 'Expired' }
  | { status: 'Error'; message: string }

/** 登录流程配置 */
export interface LoginConfig {
  /** "web" = Web Auth Code Flow, "device_code" = Device Code Flow */
  flow: string
}
