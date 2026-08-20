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

/**
 * 格式化下载量（中文万/亿单位）
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

export interface FormatDateTimeOptions {
  /** 是否包含年份，默认包含 */
  withYear?: boolean
  /** 日期无效时返回的内容，默认返回 '-' */
  invalidValue?: string
}

/**
 * 格式化日期时间为 YYYY-MM-DD HH:mm 或 MM-DD HH:mm（本地时区）
 *
 * 支持 Date、日期字符串和毫秒时间戳。无效日期返回 invalidValue。
 */
export function formatDateTime(
  value: Date | string | number,
  options?: FormatDateTimeOptions,
): string {
  const d = value instanceof Date ? value : new Date(value)
  if (Number.isNaN(d.getTime())) return options?.invalidValue ?? '-'
  const pad = (n: number) => String(n).padStart(2, '0')
  const date = `${pad(d.getMonth() + 1)}-${pad(d.getDate())}`
  const time = `${pad(d.getHours())}:${pad(d.getMinutes())}`
  return options?.withYear === false ? `${date} ${time}` : `${d.getFullYear()}-${date} ${time}`
}

/**
 * 校验文件名是否安全（拒绝路径穿越、绝对路径与非法字符）
 *
 * 用于远程资源返回的 file_name 拼接本地路径前的防御性校验，
 * 防止 `../` 穿越或绝对路径逃逸到目标目录之外。
 */
export function isSafeFileName(name: string): boolean {
  if (!name || name.length > 255) return false
  // 拒绝路径分隔符（/ \）、穿越（..）与绝对路径（盘符前缀）
  if (/[\\/]/.test(name)) return false
  if (name === '.' || name === '..') return false
  if (/^[a-zA-Z]:/.test(name)) return false
  // 拒绝 Windows 非法字符
  if (/[<>:"|?*]/.test(name)) return false
  // 拒绝控制字符（\x00-\x1f，正则字面量含控制字符会触发 no-control-regex，改用码点判断）
  for (let i = 0; i < name.length; i++) {
    if (name.charCodeAt(i) < 0x20) return false
  }
  return true
}

/**
 * 格式化 Unix 时间戳（秒）为 YYYY-MM-DD HH:mm[:ss]（本地时区）
 *
 * 用于展示登录时间、JWT 过期时间等场景。无效或非正数返回 '-'。
 * options.withSeconds === false 时省略秒（HH:mm），默认含秒，不影响既有调用方。
 */
export function formatTimestamp(unixSeconds: number, options?: { withSeconds?: boolean }): string {
  if (!Number.isFinite(unixSeconds) || unixSeconds <= 0) return '-'
  const d = new Date(unixSeconds * 1000)
  const pad = (n: number) => String(n).padStart(2, '0')
  const time = options?.withSeconds === false
    ? `${pad(d.getHours())}:${pad(d.getMinutes())}`
    : `${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ${time}`
}

/**
 * 格式化时间为 MM-DD HH:mm（本地时区，不含年份）
 *
 * 支持 Date、日期字符串与毫秒时间戳，秒级时间戳（< 1e12）自动转毫秒；
 * 无效值（0 / 空串 / 非法日期）返回 invalidValue（默认 '-'）。
 */
export function formatTime(
  value: Date | string | number,
  options?: { invalidValue?: string },
): string {
  if (!value) return options?.invalidValue ?? '-'
  const ms = typeof value === 'number' && value < 1e12 ? value * 1000 : value
  return formatDateTime(ms, { withYear: false, invalidValue: options?.invalidValue ?? '-' })
}

/**
 * 设备 ID 打码展示（保留前 4 后 4，中间用 **** 遮盖）
 */
export function maskDeviceId(id: string): string {
  if (id.length <= 8) return id
  return `${id.slice(0, 4)}****${id.slice(-4)}`
}
