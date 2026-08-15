/**
 * 联机功能类型定义 - NAT 检测域
 * 用于联机前预判 P2P 可行性，NAT 分类参考 RFC 3489 / STUN RFC 5389。
 */

/**
 * NAT 类型枚举（参考 RFC 3489 / STUN RFC 5389）
 * Open 公网无 NAT；FullCone 全锥（联机最佳）；RestrictedCone 限制锥；PortRestrictedCone 端口限制锥（兼容性较差）；
 * Symmetric 无 STUN 中转无法 P2P；Blocked UDP 阻断；Unknown 检测失败。
 */
export type NatType =
  | 'Open'
  | 'FullCone'
  | 'RestrictedCone'
  | 'PortRestrictedCone'
  | 'Symmetric'
  | 'Blocked'
  | 'Unknown'

/** NAT 检测结果 */
export interface NatDetectionResult {
  /** NAT 类型 */
  type: NatType
  /** 检测耗时（毫秒） */
  durationMs: number
  /** 本地出口 IP（如有） */
  localIp?: string
  /** 公网 IP（如有） */
  publicIp?: string
  /** 检测错误信息（失败时） */
  error?: string
}
