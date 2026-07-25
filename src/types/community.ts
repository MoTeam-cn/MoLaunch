/**
 * 社区资源类型定义
 * 对应后端 minecraft/community/types.rs
 */

/** 资源类型 */
export type ResourceType = 'Mod' | 'ModPack' | 'ResourcePack' | 'Shader' | 'DataPack'

/** 来源平台 */
export type Platform = 'CurseForge' | 'Modrinth'

/** 发布类型 */
export type ReleaseType = 'Release' | 'Beta' | 'Alpha'

/** 加载器 flags（位枚举） */
export const ModLoaderFlags = {
  None: 0,
  Forge: 1,
  LiteLoader: 2,
  Fabric: 4,
  Quilt: 8,
  NeoForge: 16,
} as const

/** 资源工程 */
export interface ResourceProject {
  platform: Platform
  resource_type: ResourceType
  id: string
  slug: string
  raw_name: string
  translated_name: string
  description: string
  website: string
  last_update: string
  download_count: number
  mod_loaders: number
  tags: string[]
  logo_url: string | null
  game_versions: string[]
}

/** 资源版本/文件 */
export interface ResourceVersion {
  id: string
  display: string
  version: string
  release_date: string
  download_count: number
  mod_loaders: number
  game_versions: string[]
  release_type: ReleaseType
  file_name: string
  download_url: string
  hash: string | null
  size: number
  dependencies: string[]
}

/** 搜索请求参数 */
export interface SearchParams {
  query: string
  resourceType: ResourceType
  gameVersion?: string
  modLoader: number
  source: number
  category?: string
  page: number
}

/** 搜索结果 */
export interface SearchResult {
  projects: ResourceProject[]
  total_count: number
  page: number
  page_size: number
}

/** 分类标签信息 */
export interface CategoryTagInfo {
  combined: string
  label: string
}

/** 下载安装请求 */
export interface DownloadRequest {
  url: string
  fileName: string
  resourceType: ResourceType
  versionId?: string
  hash?: string
  /** 译名（可选，来自 mcmod 数据库，用于按 filenameFormat 拼接新文件名） */
  translatedName?: string
}

/** 下载安装结果 */
export interface DownloadResult {
  path: string
  size: number
}

/** 整合包安装请求 */
export interface InstallModpackRequest {
  platform: Platform
  downloadUrl: string
  fileName: string
  instanceName: string
  /** 是否下载可选 Mod（CF required=false / MR env.client=optional）。undefined 时默认 true */
  includeOptional?: boolean
  /** 外部 Logo 文件本地路径（CF/MR 平台下载时缓存的缩略图，复制到 MoLaunch/Logo.png） */
  logoPath?: string
}

/** 本地整合包安装请求（拖拽安装） */
export interface InstallLocalModpackRequest {
  /** 本地整合包文件绝对路径（.zip / .mrpack） */
  filePath: string
  /** 整合包实例名（用于 versions/{instanceName}/ 目录） */
  instanceName: string
  /** 是否下载可选 Mod（由前端 preview 后弹窗询问用户传入，undefined 时默认 true） */
  includeOptional?: boolean
  /** 外部 Logo 文件本地路径（拖拽安装时通常为空） */
  logoPath?: string
}

/** 整合包格式 */
export type ModpackFormat =
  | 'curseforge'
  | 'modrinth'
  | 'hmcl'
  | 'mmc'
  | 'mcbbs'
  | 'launcherpack'
  | 'compress'

/** 整合包安装结果 */
export interface InstallModpackResult {
  format: ModpackFormat
  gameVersion: string
  loader: string
  loaderVersion: string
  archivePath: string
  instanceDir: string
}

/** 可选 Mod 信息（前端弹窗显示用） */
export interface OptionalModInfo {
  /** 显示名（CF: "CF File #{fileId}"，MR: path 末段） */
  displayName: string
  /** 文件大小（字节，CF 为 0 因为 manifest 不含大小） */
  fileSize: number
  /** CurseForge file_id（仅 CF 格式有值） */
  fileId?: number | null
  /** CurseForge project_id（仅 CF 格式有值） */
  projectId?: number | null
  /** Modrinth 文件路径（仅 MR 格式有值） */
  path?: string | null
}

/** 整合包预览信息（拖拽安装前置步骤，弹窗询问可选 Mod 用） */
export interface ModpackPreview {
  format: ModpackFormat
  gameVersion: string
  loader: string
  loaderVersion: string
  /** 可选 Mod 列表（CF required=false / MR env.client=optional） */
  optionalMods: OptionalModInfo[]
}

/** 来源选项 */
export const SOURCE_OPTIONS = [
  { label: '全部', value: 0 },
  { label: '仅 CurseForge', value: 1 },
  { label: '仅 Modrinth', value: 2 },
] as const

/** 加载器选项 */
export const LOADER_OPTIONS = [
  { label: '全部', value: 0 },
  { label: 'Forge', value: ModLoaderFlags.Forge },
  { label: 'NeoForge', value: ModLoaderFlags.NeoForge },
  { label: 'Fabric', value: ModLoaderFlags.Fabric },
  { label: 'Quilt', value: ModLoaderFlags.Quilt },
  { label: 'LiteLoader', value: ModLoaderFlags.LiteLoader },
] as const
