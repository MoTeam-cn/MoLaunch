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
