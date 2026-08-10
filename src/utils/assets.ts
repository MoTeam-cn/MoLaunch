/**
 * 通用保底图（Common 目录）
 * default.png 为大图，default-min.png 为小图。
 */

import defaultLogo from '@/assets/Common/default.png'
import defaultMinLogo from '@/assets/Common/default-min.png'

/** 获取通用保底图 URL；min 为 true 时返回小尺寸图 */
export function defaultAsset(min = false): string {
  return min ? defaultMinLogo : defaultLogo
}
