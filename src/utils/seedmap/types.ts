/**
 * 种子地图 WorkerPool 消息协议与参数类型
 *
 * 主→Worker：init / prepare_seed / generate / find_structures / obsolete / dispose；
 * Worker→主：init_complete / seed_prepared / tile_result / structure_result / error。
 * 参考 minecraftsearch.com（docs/Map/map.md）：单 Worker 内嵌 WASM，主线程以 seedEpoch 协调。
 */

/** MC 版本（与 cubiomes MC_* 常量对应） */
export type McVersion = number

/** 维度：0=主世界, -1=下界, 1=末地 */
export type Dimension = 0 | -1 | 1

// ===== 主线程 → Worker 消息 =====

export interface InitMsg {
  type: 'init'
  /** init 任务 ID（用于错误转发：init 失败时 Worker 通过 error 消息回传 jobId） */
  jobId: string
  /** Emscripten 胶水代码 cubiomes.js 的 res:// URL（无 wasmJsCode 时回退 fetch） */
  wasmJsUrl: string
  /** cubiomes.wasm 的 res:// URL */
  wasmUrl: string
  /** 主线程缓存的胶水 JS 文本（优先使用，避免每个 Worker 重复 fetch） */
  wasmJsCode?: string
  /** 主线程缓存的 WASM 二进制（优先使用） */
  wasmBytes?: ArrayBuffer
  /** 初始 seedEpoch（通常 0） */
  seedEpoch: number
}

export interface PrepareSeedMsg {
  type: 'prepare_seed'
  seed: string
  seedEpoch: number
  mcVersion: McVersion
  dimension: Dimension
  largeBiomes: boolean
}

export interface GenerateTileMsg {
  type: 'generate'
  jobId: string
  seed: string
  mcVersion: McVersion
  dimension: Dimension
  largeBiomes: boolean
  /** tile 左上角方块 X */
  blockX: number
  /** tile 左上角方块 Z */
  blockZ: number
  /** X 方向群系采样数 */
  sx: number
  /** Z 方向群系采样数 */
  sz: number
  /** cubiomes 采样间距（1=精细, 4=粗采样） */
  scale: number
  /** 方块 Y 坐标（用于地下群系查看，默认 64=海平面） */
  y?: number
  /** 是否绘制等高线（默认 false） */
  doContour?: boolean
  /** 最大渲染高度（超过此高度+8 的区域变暗，默认 Infinity=不限） */
  ymax?: number
}

export interface FindStructuresMsg {
  type: 'find_structures'
  jobId: string
  seed: string
  mcVersion: McVersion
  dimension: Dimension
  largeBiomes: boolean
  /** 可视范围最小方块 X */
  minX: number
  /** 可视范围最小方块 Z */
  minZ: number
  /** 可视范围最大方块 X */
  maxX: number
  /** 可视范围最大方块 Z */
  maxZ: number
}

export interface SpecialsMsg {
  type: 'specials'
  jobId: string
  seed: string
  mcVersion: McVersion
  largeBiomes: boolean
}

export interface BiomeAtPointMsg {
  type: 'biome_at_point'
  jobId: string
  seed: string
  mcVersion: McVersion
  dimension: Dimension
  largeBiomes: boolean
  scale: number
  x: number
  y: number
  z: number
}

export interface ObsoleteMsg {
  type: 'obsolete'
  jobId: string
}

export interface DisposeMsg {
  type: 'dispose'
}

export type MainToWorkerMsg =
  | InitMsg
  | PrepareSeedMsg
  | GenerateTileMsg
  | FindStructuresMsg
  | SpecialsMsg
  | BiomeAtPointMsg
  | ObsoleteMsg
  | DisposeMsg

// ===== Worker → 主线程消息 =====

export interface InitCompleteMsg {
  type: 'init_complete'
  /** 对应 InitMsg.jobId */
  jobId: string
  seedEpoch: number
}

export interface SeedPreparedMsg {
  type: 'seed_prepared'
  seed: string
  seedEpoch: number
}

export interface TileResultMsg {
  type: 'tile_result'
  jobId: string
  imageBitmap: ImageBitmap
}

export interface StructureResultMsg {
  type: 'structure_result'
  jobId: string
  structures: WorkerStructure[]
}

export interface ErrorMsg {
  type: 'error'
  jobId: string
  error: string
}

export interface SpecialsResultMsg {
  type: 'specials_result'
  jobId: string
  spawn: { x: number; z: number } | null
  /**
   * 多座要塞坐标数组（cubiomes nextStronghold 迭代，上限 128）。
   * 兼容性：原 firstStronghold 单字段已废弃，前端统一用数组渲染。
   */
  strongholds: { x: number; z: number }[]
}

export interface BiomeAtPointResultMsg {
  type: 'biome_at_point_result'
  jobId: string
  biomeId: number
}

export type WorkerToMainMsg =
  | InitCompleteMsg
  | SeedPreparedMsg
  | TileResultMsg
  | StructureResultMsg
  | SpecialsResultMsg
  | BiomeAtPointResultMsg
  | ErrorMsg

// ===== 公共数据结构 =====

/** Worker 返回的结构标记 */
export interface WorkerStructure {
  /** 结构类型名（与 STRUCTURE_ICONS 的 key 对应） */
  stype: string
  x: number
  z: number
  /** 是否通过群系校验 */
  viable: boolean
}

/** generateTile 入参（不含 jobId/type，由 WorkerPool 注入） */
export interface GenerateTileParams {
  seed: string
  mcVersion: McVersion
  dimension: Dimension
  largeBiomes: boolean
  blockX: number
  blockZ: number
  sx: number
  sz: number
  scale: number
  /** 方块 Y 坐标（用于地下群系查看，默认 64=海平面） */
  y?: number
  /** 是否绘制等高线（默认 false） */
  doContour?: boolean
  /** 最大渲染高度（超过此高度+8 的区域变暗，默认 Infinity=不限） */
  ymax?: number
}

/** findStructures 入参 */
export interface FindStructuresParams {
  seed: string
  mcVersion: McVersion
  dimension: Dimension
  largeBiomes: boolean
  minX: number
  minZ: number
  maxX: number
  maxZ: number
}

/** specials 入参（出生点 + 多座要塞） */
export interface SpecialsParams {
  seed: string
  mcVersion: McVersion
  largeBiomes: boolean
}

/** specials 返回结果 */
export interface SpecialsResult {
  spawn: { x: number; z: number } | null
  /**
   * 多座要塞坐标数组（cubiomes nextStronghold 迭代，上限 128）。
   * 兼容性：原 firstStronghold 单字段已废弃，前端统一用数组渲染。
   */
  strongholds: { x: number; z: number }[]
}

/** biome_at_point 入参（供鼠标悬停显示群系名） */
export interface BiomeAtPointParams {
  seed: string
  mcVersion: McVersion
  dimension: Dimension
  largeBiomes: boolean
  scale: number
  x: number
  y: number
  z: number
}
