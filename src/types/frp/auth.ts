// ============================================================
// 认证体系（阶段三）
// ============================================================

/** 认证状态（get_auth_status 返回） */
export interface AuthStatus {
  providerId: string
  /** 是否已认证（有有效 token） */
  authenticated: boolean
  /** 认证类型：none / oauth2 / device_code / api_key */
  authType: string
  /** token 过期时间（Unix 秒），已过期时仍返回供前端展示 */
  expiresAt?: number
  /** 权限范围 */
  scopes?: string[]
  /** 续期中：token 已过期但存在 refresh_token，正在/刚尝试静默续期（失败时 authenticated=false 且 refreshing=true） */
  refreshing?: boolean
}

/** OAuth2 流程结果（start_oauth2 返回） */
export interface OAuth2Result {
  /** token 过期时间（Unix 秒） */
  expiresAt?: number
  /** 权限范围 */
  scopes?: string[]
}

/** Device Code 流程启动结果（start_device_code 返回） */
export interface DeviceCodeResult {
  /** 用户码（前端显示给用户输入） */
  userCode: string
  /** 验证链接（用户访问此 URL 输入用户码） */
  verificationUri: string
  /** 过期时间（秒） */
  expiresIn: number
  /** 轮询间隔（秒） */
  interval: number
}

/** Device Code 轮询状态 */
export type DeviceCodePollStatus = 'pending' | 'success' | 'expired' | 'declined' | 'slow_down'

/** Device Code 轮询结果（poll_device_code 返回） */
export interface DeviceCodePollResult {
  /** 状态：pending / success / expired / declined / slow_down */
  status: DeviceCodePollStatus
  /** token 过期时间（仅 status=success 时有值） */
  expiresAt?: number
  /** 权限范围（仅 status=success 时有值） */
  scopes?: string[]
}

/** 保存 API Key 参数 */
export interface SaveApiKeyParams {
  providerId: string
  apiKey: string
}