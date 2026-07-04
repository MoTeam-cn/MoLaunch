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

/** 已安装版本 */
export interface InstalledVersion {
  id: string
  version_type: VersionType
  installed_at: string
  has_forge: boolean
  has_fabric: boolean
  has_neoforge: boolean
}

/** 加载器类型 */
export type LoaderType = 'vanilla' | 'forge' | 'fabric' | 'neoforge' | 'optifine' | 'liteloader'
