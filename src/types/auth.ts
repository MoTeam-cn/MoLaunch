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

/** 本地认证结果（含 authlib 扩展字段） */
export interface LocalAuthResult extends AuthResult {
  /** authlib 登录的 yggdrasil API 根地址（仅 AuthlibInjector 登录时有值） */
  server_url?: string | null
  /** authlib 登录的服务器显示名（仅 AuthlibInjector 登录时有值，用于 UI 展示） */
  server_name?: string | null
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

/** 已存储的离线账号 */
export interface OfflineAccountInfo {
  username: string
  uuid: string
  skin: string | null
}

/** 已存储的 authlib 外置登录账号（前端列表展示用，不含敏感字段） */
export interface AuthlibAccountInfo {
  /** 登录账号（邮箱或用户名） */
  username: string
  /** 选中的角色 UUID */
  uuid: string
  /** 选中的角色名 */
  player_name: string
  /** yggdrasil API 根地址 */
  server_url: string
  /** 服务器显示名 */
  server_name: string
}

/** yggdrasil 服务器元数据（前端登录页展示用） */
export interface AuthlibServerMeta {
  /** 服务器名（从 meta.serverName 提取） */
  server_name: string
  /** 注册链接（从 meta.links.register 提取） */
  register_url: string | null
  /** 主页链接（从 meta.links.homepage 提取） */
  homepage_url: string | null
}

/** yggdrasil 角色（多角色登录时供用户选择） */
export interface AuthlibProfile {
  /** 角色 UUID */
  id: string
  /** 角色名 */
  name: string
}

/**
 * yggdrasil 角色的皮肤披风信息（后端解析后返回）
 *
 * 由后端 `authlib_get_skin_info` 命令返回，前端据此：
 * - 显示当前皮肤/披风（skin_url / cape_url）
 * - 根据 uploadable_textures 动态启用上传按钮
 *   （"skin" / "cape" / "skin,cape" / 空串 = 不支持上传）
 *
 * 对应后端 `crate::minecraft::auth::authlib::types::SkinCapeInfo`。
 * 后端 `Serialize` derive 默认按字段名输出，故此处采用 snake_case 字段名匹配
 * （与 `AuthlibAccountInfo` / `AuthlibServerMeta` 的现有约定一致）。
 */
export interface AuthlibSkinCapeInfo {
  /** 皮肤 URL（无皮肤时为 null） */
  skin_url: string | null
  /** 皮肤模型（"default" 或 "slim"） */
  skin_model: string
  /** 披风 URL（无披风时为 null） */
  cape_url: string | null
  /** 可上传的材质类型（"skin" / "cape" / "skin,cape"，空串表示不能上传） */
  uploadable_textures: string
}

/** authlib 登录结果（后端按 status 标签返回两种形态） */
export type AuthlibLoginResult =
  | { status: 'success'; user: LocalAuthResult }
  | {
      status: 'need_select'
      access_token: string
      client_token: string
      available_profiles: AuthlibProfile[]
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
