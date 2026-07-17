/**
 * 下载进度相关类型定义
 *
 * 统一前后端交互的下载进度数据结构，消除三处重复定义：
 * - `stores/version.ts` 原定义 `DownloadStage` / `DownloadProgress`（前端使用，字段必填）
 * - `composables/useDownloadPolling.ts` 原定义 `RawDownloadStage`（后端原始，部分字段可选）
 * - `utils/api/system.ts` 原内联定义 `getDownloadProgress` 返回类型
 *
 * 设计原则：
 *   - `Raw*` 类型：对应后端 `state/mod.rs::DownloadState` 的 serde 序列化结构，字段可能缺失需 fallback
 *   - 非 `Raw` 类型：前端使用的稳定结构，所有字段必填（已应用 fallback）
 */

/** 阶段状态（与后端 `StageStatus` 对齐） */
export type StageStatus = 'waiting' | 'loading' | 'finished' | 'failed'

/** 单个下载阶段（前端稳定结构，字段必填） */
export interface DownloadStage {
  name: string
  progress: number
  weight: number
  status: StageStatus
  bytes_downloaded: number
  bytes_total: number
  files_downloaded: number
  files_total: number
  /** 所属任务分组（用于前端按"整合包安装"/"MC本体安装"等分组折叠展开），null 表示独立阶段 */
  group: string | null
}

/** 下载进度快照（前端稳定结构，含已计算的 percentage） */
export interface DownloadProgress {
  stages: DownloadStage[]
  current_stage_index: number
  global_speed: number
  global_bytes_downloaded: number
  global_bytes_total: number
  percentage: number
}

/**
 * 后端返回的阶段原始结构（部分字段可能缺失，需做 fallback）
 *
 * 对应后端 `state/mod.rs::DownloadState::get_progress_snapshot()` 的 serde 输出。
 * `files_*` 和 `group` 字段在历史版本中可能不存在，需要默认值。
 */
export interface RawDownloadStage {
  name: string
  progress: number
  weight: number
  status: StageStatus
  bytes_downloaded: number
  bytes_total: number
  files_downloaded?: number
  files_total?: number
  group?: string | null
}

/**
 * 后端返回的下载进度原始结构（不含 percentage，由前端计算）
 *
 * 对应 `commands/system/download.rs::get_download_progress` 命令的返回值。
 */
export interface RawDownloadProgress {
  stages: RawDownloadStage[]
  current_stage_index: number
  global_speed: number
  global_bytes_downloaded: number
  global_bytes_total: number
  is_active: boolean
  is_complete: boolean
  error_code: number
}
