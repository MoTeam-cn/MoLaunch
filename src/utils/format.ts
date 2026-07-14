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

/**
 * 格式化以 MB 为单位的内存数值为可读字符串
 *
 * - >= 1024 MB 显示为 GB（1 位小数，去掉无意义的 .0 后缀）
 * - 否则显示为 MB
 *
 * 供启动设置/版本设置内存分配可视化条复用。
 */
export function formatMemoryMB(mb: number): string {
  if (mb >= 1024) {
    return (mb / 1024).toFixed(1).replace(/\.0$/, '') + ' GB'
  }
  return mb + ' MB'
}
