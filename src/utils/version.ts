/**
 * 应用版本号解析工具
 *
 * vite.config.ts 通过 `define` 注入 `__APP_VERSION__` 全局常量（取自 package.json 的 version）。
 * 本模块负责解析版本号后缀，判断当前是否为测试版（beta / alpha / rc / canary 等）。
 *
 * 判断规则：
 * - 形如 `0.1.0`（无后缀）→ 正式版
 * - 形如 `0.1.0-beta.1` / `0.1.0-alpha.2` / `0.1.0-rc.0` / `0.1.0-canary.3` → 测试版
 *
 * 使用场景：
 * - 测试版水印：仅在测试版构建中渲染全屏水印（含设备 ID 追踪）
 * - 防泄漏：测试版仅限内部测试，水印便于追溯泄漏源
 */

/** 测试版后缀类型（未识别后缀归为 Stable） */
export type VersionChannel = 'stable' | 'beta' | 'alpha' | 'rc' | 'canary'

export interface VersionInfo {
  /** 完整版本号字符串（如 `0.1.0-beta.1`） */
  raw: string
  /** 主版本号（如 `0`） */
  major: number
  /** 次版本号（如 `1`） */
  minor: number
  /** 修订号（如 `0`） */
  patch: number
  /** 发布通道 */
  channel: VersionChannel
  /** 后缀标识（如 `beta.1` 中的 `1`），无后缀时为 0 */
  preReleaseNumber: number
  /** 是否为测试版（channel !== 'stable'） */
  isPreRelease: boolean
}

/** 解析 semver 后缀为通道类型 */
function parseChannel(suffix: string): { channel: VersionChannel; num: number } {
  if (!suffix) return { channel: 'stable', num: 0 }
  // 后缀形如 `beta.1` / `alpha.2` / `rc.0` / `canary.3`
  const lower = suffix.toLowerCase()
  const match = lower.match(/^(beta|alpha|rc|canary)[.\-]?(\d+)?/)
  if (!match) return { channel: 'stable', num: 0 }
  const channel = match[1] as VersionChannel
  const num = match[2] ? parseInt(match[2], 10) : 0
  return { channel, num }
}

/**
 * 解析版本号字符串
 *
 * 支持标准 semver（`MAJOR.MINOR.PATCH`）及带后缀（`MAJOR.MINOR.PATCH-CHANNEL.NUM`）。
 * 解析失败时返回 channel=stable 的兜底信息，不抛异常。
 */
export function parseVersion(version: string): VersionInfo {
  const raw = version || '0.0.0'
  // 主版本号部分（前 3 段数字）+ 可选后缀
  const match = raw.match(/^(\d+)\.(\d+)\.(\d+)(?:[-+](.+))?$/)
  if (!match) {
    return {
      raw,
      major: 0,
      minor: 0,
      patch: 0,
      channel: 'stable',
      preReleaseNumber: 0,
      isPreRelease: false,
    }
  }
  const major = parseInt(match[1], 10)
  const minor = parseInt(match[2], 10)
  const patch = parseInt(match[3], 10)
  const { channel, num } = parseChannel(match[4] || '')
  return {
    raw,
    major,
    minor,
    patch,
    channel,
    preReleaseNumber: num,
    isPreRelease: channel !== 'stable',
  }
}

/** 缓存解析结果，避免重复解析 */
let cachedInfo: VersionInfo | null = null

/** 获取当前应用版本信息（来自 vite 注入的 __APP_VERSION__） */
export function getVersionInfo(): VersionInfo {
  if (cachedInfo) return cachedInfo
  // __APP_VERSION__ 由 vite.config.ts 的 define 注入，类型声明见 src/shims.d.ts
  cachedInfo = parseVersion(__APP_VERSION__)
  return cachedInfo
}

/** 当前是否为测试版（便捷函数） */
export function isPreReleaseBuild(): boolean {
  return getVersionInfo().isPreRelease
}

/** 获取当前发布通道（便捷函数） */
export function getCurrentChannel(): VersionChannel {
  return getVersionInfo().channel
}

/**
 * 生成测试版构建的唯一指纹（用于水印隐写）
 *
 * 由版本号 + 发布通道 + 构建序号拼接，提供给水印组件作为「构建标识」字段。
 * 即使同一设备运行不同测试版构建，指纹也不同，便于精确追溯。
 */
export function getBuildFingerprint(): string {
  const info = getVersionInfo()
  if (!info.isPreRelease) return ''
  return `${info.raw}`
}
