import type {
  DownloadProgress,
  DownloadStage,
  RawDownloadProgress,
  RawDownloadStage,
} from '@/types/download'

/**
 * 下载进度伪补丁：后端 ticker 卡在 95% 时，前端按时间对数续涨至 99.9%（永不到 100%），
 * 让用户感知进度推进。曲线 30 秒内趋近上限 63%。
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

/** Raw 阶段 → 前端稳定结构（字段缺失时补默认值） */
export function mapStage(raw: RawDownloadStage): DownloadStage {
  return {
    name: raw.name,
    progress: raw.progress,
    weight: raw.weight,
    status: raw.status,
    bytes_downloaded: raw.bytes_downloaded,
    bytes_total: raw.bytes_total,
    files_downloaded: raw.files_downloaded || 0,
    files_total: raw.files_total || 0,
    group: raw.group ?? null,
  }
}

/**
 * Raw 快照 → 前端稳定结构（含两个百分比）
 *
 * `percentage` 按阶段权重折算，是「整体干了多少」的真实值；但解析整合包、复制配置文件、
 * 批量查询这类阶段不产生字节，权重又小（如 1/56），它的进度只能把总体推高百分之一点几，
 * 观感就是卡在 0%。所以另算一个 `stage_percentage` 供界面对「当前阶段」单独展示。
 */
export function toProgressSnapshot(raw: RawDownloadProgress): DownloadProgress {
  const stages = (raw.stages || []).map(mapStage)
  let weighted = 0
  let totalWeight = 0
  for (const stage of stages) {
    totalWeight += stage.weight
    weighted += stage.progress * stage.weight
  }
  const percentage = totalWeight > 0
    ? Math.min(100, parseFloat(((weighted / totalWeight) * 100).toFixed(1)))
    : 0
  const index = raw.current_stage_index ?? 0
  const current = stages[index] ?? stages[stages.length - 1]
  const stagePercentage = current
    ? Math.min(100, parseFloat(((current.progress || 0) * 100).toFixed(1)))
    : 0
  return {
    stages,
    current_stage_index: index,
    global_speed: raw.global_speed ?? 0,
    global_bytes_downloaded: raw.global_bytes_downloaded ?? 0,
    global_bytes_total: raw.global_bytes_total ?? 0,
    percentage,
    stage_percentage: stagePercentage,
    isPaused: (raw.stages || []).some((stage) => stage.is_paused === true),
  }
}

/** 展示用百分比的输入 */
export interface DisplayPercentageInput {
  /** 按权重折算的总体百分比（0-100） */
  weighted: number
  /** 已确知的总字节数：为 0 说明还在不产生字节的阶段 */
  globalBytesTotal: number
  /** 当前阶段自身百分比（0-100） */
  stage: number
}

/**
 * 决定界面上那个大号百分比显示什么
 *
 * 还没有任何字节时（解析整合包 / 复制配置文件 / 批量查询）总体百分比注定贴着 0，
 * 与其让用户以为卡死，不如直接显示当前阶段的进度；一旦开始有字节，就回到总体值——
 * 那是真正剩下的工作量。上限 99.9，避免在任何阶段提前宣称 100%。
 */
export function resolveDisplayPercentage(input: DisplayPercentageInput): number {
  const clamp = (value: number) => Math.min(100, Math.max(0, value))
  if (input.globalBytesTotal > 0) return clamp(input.weighted)
  return Math.min(99.9, clamp(input.stage))
}
