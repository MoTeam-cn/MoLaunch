/**
 * 红石联机（hongshic 内核隧道）类型定义
 *
 * 内核来自 hongshi.site（Redstone Online2）：优先运行时下载并缓存，
 * 下载不可用时回退应用内置的兜底内核。
 */

/** 中转服务器条目（来自 hongshi.site/api/server/list） */
export interface RedStoneServer {
  host: string
  region: string
}

/** `redstone_get_servers` 返回 */
export interface RedStoneGetServersResult {
  servers: RedStoneServer[]
}

/** `redstone_platform_support` 返回（supported=false 时 reason 为可读原因） */
export interface RedStonePlatformSupportResult {
  supported: boolean
  reason: string | null
}

/** `redstone_start` 参数 */
export interface RedStoneStartParams {
  server: string
  mc_port: number
}

/** `redstone_start` 返回 */
export interface RedStoneStartResult {
  pid: number
}

/** 隧道状态：open=已建立 / closed=已关闭 / unknown=未创建或仍在建链 */
export type RedStoneChannelStatus = 'open' | 'closed' | 'unknown'

/** `redstone_status` 返回（未创建隧道时 running=false, status='unknown'） */
export interface RedStoneStatusResult {
  running: boolean
  status: RedStoneChannelStatus
  server: string | null
  port: number | null
  created: string | null
}

/** `redstone_stop` 返回 */
export interface RedStoneStopResult {}

/** 日志文件条目（`redstone_log_files` 返回项，日志路径 `<temp>/MoLaunch/hongshi/logs/`） */
export interface RedStoneLogFileInfo {
  fileName: string
  sizeBytes: number
  modifiedAt: number
}

/** `redstone_log_files` 返回 */
export interface RedStoneLogFilesResult {
  files: RedStoneLogFileInfo[]
}

/** `redstone_read_log` 返回（读取尾部 maxLines 行） */
export interface RedStoneLogContent {
  lines: string[]
  hasMore: boolean
}

export interface RedStoneReadLogResult {
  content: RedStoneLogContent
}