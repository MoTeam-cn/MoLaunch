/**
 * 红石联机（hongshi 内核隧道）类型定义
 *
 * 与后端 redstone_manager action 契约一一对应（见 docs/REDSTONE_ONLINE_DESIGN.md）。
 */

/** 中转服务器条目（来自 hongshi.site/newserver.json） */
export interface RedStoneServer {
  host: string
  region: string
}

/** `redstone_get_servers` 返回 */
export interface RedStoneGetServersResult {
  servers: RedStoneServer[]
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

/** 隧道状态：open=已建立 / closed=已关闭 / unknown=未创建 */
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