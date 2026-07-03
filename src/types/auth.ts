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
