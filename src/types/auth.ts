/** 认证类型 */
export enum AuthType {
  Microsoft = 0,
  Offline = 1,
  External = 2,
}

/** 认证结果 */
export interface AuthResult {
  auth_type: AuthType
  access_token: string
  refresh_token?: string
  uuid: string
  username: string
  expires_at: number
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
