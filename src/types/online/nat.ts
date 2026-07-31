/**
 * 联机功能类型定义 - NAT 检测域
 *
 * 用于联机前预判 P2P 可行性，参考 RFC 3489 / STUN RFC 5389 的 NAT 分类。
 */

/**
 * NAT 类型枚举
 *
 * 参考 RFC 3489 / STUN RFC 5389 的 NAT 分类：
 * - `Open`：公网 IP，无 NAT（罕见）
 * - `FullCone`：全锥 NAT，任意外部主机可访问映射端口（联机最佳）
 * - `RestrictedCone`：限制锥 NAT，仅允许联系过的外部 IP（联机可用）
 * - `PortRestrictedCone`：端口限制锥 NAT，仅允许联系过的外部 IP:Port（联机可用，但兼容性较差）
 * - `Symmetric`：对称 NAT，每个目标分配独立端口（无 STUN 中转无法 P2P）
 * - `Blocked`：UDP 被阻断（无法 P2P）
 * - `Unknown`：检测失败或浏览器不支持
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
