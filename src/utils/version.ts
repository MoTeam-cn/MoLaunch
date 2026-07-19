/**
 * 版本号比较工具（参考 PCL2 CompareVersion）
 *
 * 从 `useVersionGroups.ts` 抽出的共享工具，供 ModUpdateDialog 等其他场景复用。
 */

/**
 * 比较两个版本号字符串
 *
 * 按 `.` 分段，逐段按数字比较。非数字段视为 0。
 *
 * @returns 负数 a < b，0 相等，正数 a > b
 *
 * @example
 * compareVersion('1.2.0', '1.2.1') // -1
 * compareVersion('1.10', '1.9')    // 1
 * compareVersion('1.0', '1.0.0')   // 0
 */
export function compareVersion(a: string, b: string): number {
  const parseVer = (s: string) => s.split('.').map(n => parseInt(n) || 0)
  const pa = parseVer(a)
  const pb = parseVer(b)
  const len = Math.max(pa.length, pb.length)
  for (let i = 0; i < len; i++) {
    const va = pa[i] || 0
    const vb = pb[i] || 0
    if (va !== vb) return va - vb
  }
  return 0
}

/**
 * 版本变化类型
 */
export type VersionChangeType = 'upgrade' | 'downgrade' | 'same' | 'unknown'

/**
 * 判断从 current 到 target 的版本变化类型
 *
 * 当任一版本号为空时返回 'unknown'。
 */
export function versionChangeType(current: string, target: string): VersionChangeType {
  if (!current || !target) return 'unknown'
  const cmp = compareVersion(target, current)
  if (cmp > 0) return 'upgrade'
  if (cmp < 0) return 'downgrade'
  return 'same'
}
