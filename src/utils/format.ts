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
 * 紧凑速度格式（用于下载进度条等空间受限场景）
 *
 * - >= 1 MB/s：显示 "X.X MB/s"
 * - >= 1 KB/s：显示 "X KB/s"
 * - 否则：显示 "X B/s"
 */
export function formatSpeedCompact(bytesPerSec: number): string {
  if (bytesPerSec >= 1_048_576) return (bytesPerSec / 1_048_576).toFixed(1) + ' MB/s'
  if (bytesPerSec >= 1024) return (bytesPerSec / 1024).toFixed(0) + ' KB/s'
  return bytesPerSec + ' B/s'
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

/**
 * 格式化下载量（参考 PCL2，中文万/亿单位）
 *
 * - >= 1 亿：显示 "X.XX 亿"
 * - >= 1 万：显示 "X.X 万"（去掉无意义的 .0 后缀）
 * - 否则：显示原始数字
 */
export function formatDownloads(n: number): string {
  if (n >= 100_000_000) return (n / 100_000_000).toFixed(2) + ' 亿'
  if (n >= 10_000) return (n / 10_000).toFixed(1).replace(/\.0$/, '') + ' 万'
  return String(n)
}

/**
 * 格式化 ISO 日期字符串为 YYYY-MM-DD
 *
 * 用于展示文件发布日期等场景。非 ISO 格式原样返回。
 */
export function formatDate(dateStr: string): string {
  if (!dateStr) return ''
  return dateStr.includes('T') ? dateStr.split('T')[0] : dateStr
}
