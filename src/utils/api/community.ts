/**
 * 社区资源 API 封装
 */

import { invoke } from '@tauri-apps/api/core'
import type {
  SearchResult,
  SearchParams,
  CategoryTagInfo,
  ResourceType,
  ResourceProject,
  ResourceVersion,
  Platform,
  DownloadRequest,
  DownloadResult,
} from '@/types/community'

/** 搜索社区资源 */
export async function searchResources(params: SearchParams): Promise<SearchResult> {
  return await invoke<SearchResult>('search_resources', { req: params })
}

/** 获取分类标签列表 */
export async function getCategoryTags(resourceType: ResourceType): Promise<CategoryTagInfo[]> {
  return await invoke<CategoryTagInfo[]>('get_category_tags', { resourceType })
}

/** 获取工程详情 */
export async function getProjectDetail(
  platform: Platform,
  projectId: string,
  resourceType: ResourceType,
): Promise<ResourceProject> {
  return await invoke<ResourceProject>('get_project_detail', {
    req: { platform, projectId, resourceType },
  })
}

/** 获取工程版本列表 */
export async function getProjectVersions(
  platform: Platform,
  projectId: string,
): Promise<ResourceVersion[]> {
  return await invoke<ResourceVersion[]>('get_project_versions', { platform, projectId })
}

/** 下载安装资源 */
export async function downloadResource(req: DownloadRequest): Promise<DownloadResult> {
  return await invoke<DownloadResult>('download_resource', { req })
}

/** 下载资源到自定义路径（流式 + 进度推送） */
export async function downloadResourceToPath(
  url: string,
  fileName: string,
  savePath: string,
): Promise<DownloadResult> {
  return await invoke<DownloadResult>('download_resource_to_path', { url, fileName, savePath })
}

/** 获取资源默认安装路径 */
export async function getResourceInstallPath(
  resourceType: ResourceType,
  versionId?: string,
): Promise<string> {
  return await invoke<string>('get_resource_install_path', { resourceType, versionId })
}

/** CurseForge 加密配置（API Key 走 SDK DES 加密 + 注册表存储） */
export interface CurseForgeConfig {
  enabled: boolean
  apiKey: string
}

/** 读取 CurseForge 配置（从内存缓存读，已解密） */
export async function getCurseForgeConfig(): Promise<CurseForgeConfig> {
  return await invoke<CurseForgeConfig>('get_curseforge_config')
}

/** 保存 CurseForge 配置（API Key 加密后写入注册表 + 更新缓存） */
export async function setCurseForgeConfig(enabled: boolean, apiKey: string): Promise<void> {
  return await invoke<void>('set_curseforge_config', { enabled, apiKey })
}
