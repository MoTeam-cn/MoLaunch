/**
 * 联机功能类型定义 - 设备认证域
 *
 * 与后端 `minecraft::online` 模块及 `utils::online_manager` 中注册的 action 对应。
 * 字段命名采用 camelCase（后端 `#[serde(rename_all = "camelCase")]` 或显式 `rename`）。
 */

/**
 * 设备认证状态
 *
 * 对应后端 `utils::online_manager::DeviceStatus`。
 * 不发起网络请求，仅读本地凭证 + 配置中的 api_server_url。
 */
export interface DeviceStatus {
  /** 是否已注册（device_pk + 三组密钥齐全） */
  registered: boolean
  /** 是否已登录（device_token 非空） */
  logged_in: boolean
  /** JWT 是否已过期（容差 60 秒） */
  token_expired: boolean
  /** 设备主键（UUID） */
  device_pk: string
  /** 设备友好标识（mcsdk-xxxx-xxxx-xxxx-xxxx） */
  device_id: string
  /** JWT 过期时间（Unix 秒） */
  token_expires_at: number
  /** 最后登录时间（Unix 秒） */
  last_login_at: number
  /** 当前配置的 api-server 地址 */
  api_server_url: string
}

/**
 * 启动静默认证结果
 *
 * 对应后端 `utils::online_manager::AuthInitResult`。
 * 由 `auth_init` action 返回，前端据此设置 `cloudConnected` 全局状态。
 */
export interface AuthInitResult {
  /** 设备认证状态快照 */
  status: DeviceStatus
  /** 错误信息（null 表示成功；非 null 表示云端连接失败，需降级） */
  error: string | null
}

/**
 * 服务器时间信息
 *
 * 对应后端 `utils::online_manager::ServerTimeInfo`。
 * 用于测试 api-server 连通性 + 校准本地时间。
 */
export interface ServerTimeInfo {
  /** 服务器 Unix 时间戳（秒） */
  server_time: number
  /** RFC3339 格式时间字符串 */
  rfc3339: string
  /** 服务器时区名称（如 "Asia/Shanghai"） */
  timezone: string
  /** 时区偏移秒数（如 +28800 表示 UTC+8） */
  offset_seconds: number
}
