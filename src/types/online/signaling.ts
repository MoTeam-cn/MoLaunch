/**
 * 联机功能类型定义 - 信令基础域
 *
 * 与后端 `minecraft::online::signaling` 及 `utils::signaling_handler` 对应。
 * 字段命名 camelCase，与后端 `#[serde(rename_all = "camelCase")]` 一致。
 * 业务响应统一为 `BusinessResult<T>`（含 code/data/msg 字段）。
 */

/** 统一业务响应（解密后） */
export interface BusinessResult<T = unknown> {
  /** 业务码：1=成功，0=系统错误，1001+=业务错误 */
  code: number
  /** 业务数据，失败时为 null */
  data: T | null
  /** 提示消息，成功默认 "ok" */
  msg: string
  /** 服务端时间（ISO 8601 UTC） */
  time: string
  /** 请求 ID */
  req_id: string
}

/** STUN 服务器列表响应 */
export interface StunServersResponse {
  servers: string[]
}

/**
 * ICE 服务器条目（统一 STUN + TURN）
 *
 * 阶段三子任务 7 新增。对应后端 `IceServerEntry`：
 * - `urls` 长度 ≥ 1，首项通常为 `stun:` / `turn:` / `turns:` 协议 URL
 * - `username` / `credential` 仅 TURN 需要填充（STUN 缺省）
 *
 * 浏览器侧 `RTCIceServer` 接口对齐：
 * ```ts
 * { urls: ['stun:stun.example.com:3478'] }
 * { urls: ['turn:turn.example.com:3478?transport=udp'], username: 'foo', credential: 'bar' }
 * ```
 */
export interface IceServerEntry {
  urls: string[]
  /** TURN 用户名（STUN 时缺省） */
  username?: string
  /** TURN 凭据（STUN 时缺省） */
  credential?: string
}

/**
 * TURN 服务器列表响应（房主独占接口）
 *
 * 阶段三子任务 7 新增。对应后端 `TurnServersResponse`：
 * - `servers`：经服务端负载过滤后的可下发 TURN 条目（可能为空数组）
 * - `enabled`：全局 TURN 开关（false 时始终不下发）
 * - `currentTotalLoad`：当前集群已用负载（用于客户端展示/调试）
 * - `loadThreshold`：集群负载阈值（≥ 时本次不下发新 TURN）
 *
 * 房主拉取后通过 DataChannel 控制消息 0x05 广播给房间内所有参与者。
 */
export interface TurnServersResponse {
  /** 过滤后的 TURN 服务器列表（可能为空） */
  servers: IceServerEntry[]
  /** TURN 全局开关 */
  enabled: boolean
  /** 当前集群总负载 */
  currentTotalLoad: number
  /** 集群负载阈值 */
  loadThreshold: number
}
