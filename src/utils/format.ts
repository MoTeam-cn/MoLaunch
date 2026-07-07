/**
 * 格式化字节数为可读字符串
 */
export function formatBytes(bytes: number, decimals = 2): string {
  // 边界检查：负数 / NaN / Infinity 视为 0，避免 Math.log 产生 NaN 或 sizes 越界
  if (!Number.isFinite(bytes) || bytes <= 0) return '0 B'
  const k = 1024
  const dm = decimals < 0 ? 0 : decimals
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB', 'PB']
  let i = Math.floor(Math.log(bytes) / Math.log(k))
  // 防止 i 超出 sizes 上界（极大数值）
  if (i >= sizes.length) i = sizes.length - 1
  if (i < 0) i = 0
  return parseFloat((bytes / Math.pow(k, i)).toFixed(dm)) + ' ' + sizes[i]
}

/**
 * 格式化速度为可读字符串
 */
export function formatSpeed(bytesPerSecond: number): string {
  return formatBytes(bytesPerSecond) + '/s'
}
