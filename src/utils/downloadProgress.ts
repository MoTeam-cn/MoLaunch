/**
 * 下载进度伪补丁：后端 ticker 卡在 95% 时，前端继续小数点上涨
 *
 * 后端 ticker 曲线 cap 是 95%（永不到 100% 直到真正完成），
 * 前端基于时间继续上涨到 99.9%（永不到 100%），让用户感受到进度仍在推进。
 *
 * 曲线：对数趋近 PATCH_CEIL，30 秒内趋近上限 63%，永不到达。
 */

/** 补丁阈值：后端 progress >= 0.95 时开始打补丁 */
const PATCH_THRESHOLD = 0.95
/** 补丁上限：永不到 100% */
const PATCH_CEIL = 0.999
/** 补丁时间常数（秒）：30 秒内趋近上限 63% */
const PATCH_TAU = 30

/**
 * 计算补丁后的进度（0-1）
 *
 * @param realProgress 后端真实进度（0-1）
 * @param startTime 进入补丁区间的起始时间戳（ms），null 表示尚未进入
 * @param now 当前时间戳（ms）
 * @returns 补丁后的进度（0-1）
 */
export function applyProgressPatch(
  realProgress: number,
  startTime: number | null,
  now: number,
): number {
  // 已完成（>=1），直接返回
  if (realProgress >= 1) return realProgress
  // 未达阈值，直接返回真实进度
  if (realProgress < PATCH_THRESHOLD) return realProgress
  // 进入补丁区间但 startTime 为 null（首次进入由调用方记录 startTime）
  if (startTime === null) return realProgress

  const elapsed = (now - startTime) / 1000
  const base = Math.max(realProgress, PATCH_THRESHOLD)
  const remaining = PATCH_CEIL - base
  // 对数曲线趋近 PATCH_CEIL，永不到达
  const advance = remaining * (1 - Math.exp(-elapsed / PATCH_TAU))
  return Math.min(PATCH_CEIL, base + advance)
}
