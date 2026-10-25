/**
 * 启动耗时展示辅助：阶段中文名、时长格式化、排行百分比。
 */
import type { LaunchTimings } from '@/utils/api/launch'

/** 后端 LaunchStage 枚举名 → 中文 */
const STAGE_LABELS: Record<string, string> = {
  Init: '初始化',
  GetJava: '获取 Java',
  Login: '登录验证',
  ValidateFiles: '文件检查',
  BuildArgs: '构建参数',
  PreLaunch: '启动前命令',
  ExtractNatives: '解压原生库',
  LaunchProcess: '启动进程',
  WaitWindow: '等待窗口',
  Finished: '完成',
  Failed: '失败',
}

/** 阶段中文名（未知阶段原样返回） */
export function stageLabel(stage: string): string {
  return STAGE_LABELS[stage] ?? stage
}

/** 时长格式化：< 1s 显示毫秒，< 60s 显示秒，否则分秒 */
export function formatDuration(ms: number): string {
  if (!Number.isFinite(ms) || ms < 0) return '-'
  if (ms < 1000) return `${Math.round(ms)}ms`
  const seconds = ms / 1000
  if (seconds < 60) return `${seconds.toFixed(seconds < 10 ? 1 : 0)}s`
  const minutes = Math.floor(seconds / 60)
  const rest = Math.round(seconds % 60)
  return `${minutes}m ${String(rest).padStart(2, '0')}s`
}

/** 耗时明细行（按耗时倒序，百分比相对最慢阶段） */
export interface TimingRow {
  stage: string
  label: string
  ms: number
  text: string
  percent: number
}

export function timingRows(timings: LaunchTimings | null | undefined): TimingRow[] {
  if (!timings || timings.stages.length === 0) return []
  const rows = [...timings.stages].sort((a, b) => b.ms - a.ms)
  const max = Math.max(...rows.map((row) => row.ms), 1)
  return rows.map((row) => ({
    stage: row.stage,
    label: stageLabel(row.stage),
    ms: row.ms,
    text: formatDuration(row.ms),
    percent: Math.max(2, Math.round((row.ms / max) * 100)),
  }))
}
