/**
 * FRP 厂商展示辅助（厂商列表 / 认证中心共用）
 */
import gofrpIcon from '@/assets/Common/gofrp-icon.png'

/** 系统默认厂商 ID（后端 list_providers 内置，不返回 icon） */
export const SYSTEM_DEFAULT_PROVIDER_ID = 'system-default'

/**
 * 厂商图标地址：系统默认厂商固定使用 gofrp 图标（后端不返回 icon），
 * 外部厂商用 manifest 提供的图标（data URL 或 convertFileSrc 路径）。
 */
export function providerIconSrc(icon: string | undefined, providerId: string): string | undefined {
  if (providerId === SYSTEM_DEFAULT_PROVIDER_ID) return gofrpIcon
  return icon
}