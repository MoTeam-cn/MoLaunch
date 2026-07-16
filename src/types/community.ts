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
}

/** 整合包格式 */
export type ModpackFormat = 'curseforge' | 'modrinth'

/** 整合包安装结果 */
export interface InstallModpackResult {
  format: ModpackFormat
  gameVersion: string
  loader: string
  loaderVersion: string
  archivePath: string
  instanceDir: string
}

/** 详情请求 */
export interface DetailRequest {
  platform: Platform
  projectId: string
  resourceType: ResourceType
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
