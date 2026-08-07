/** 版本类型 */
export type VersionType = 'release' | 'snapshot' | 'old_beta' | 'old_alpha' | 'fool'

/** 版本信息 */
export interface VersionInfo {
  id: string
  version_type: VersionType
  release_time: number
  url?: string
  description?: string
}

/** 版本列表 */
export interface VersionList {
  versions: VersionInfo[]
  latest_release: string
  latest_snapshot: string
}

/** 加载器类型 */
export type LoaderType = 'vanilla' | 'forge' | 'fabric' | 'neoforge' | 'optifine' | 'liteloader'

/** 崩溃类别（与后端 CrashCategory 枚举对应） */
export type CrashCategory =
  | 'Java'
  | 'Mod'
  | 'Graphics'
  | 'Memory'
  | 'Forge'
  | 'Fabric'
  | 'OptiFine'
  | 'ResourcePack'
  | 'Shader'
  | 'Unknown'

/** 崩溃详情（与后端 CrashInfo 结构对应） */
export interface CrashInfo {
  reason: string
  category: CrashCategory
  log_lines: string[]
  suggestion: string
  problematic_mod: string | null
  crash_report_path?: string
  log_tail: string[]
}
