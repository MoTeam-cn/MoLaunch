/**
 * 社区资源 API 封装
 *
 * 重构后配置更新走统一 `applyConfig` 接口（见 system.ts），
 * 本文件仅保留读取命令与业务命令（搜索/详情/安装）。
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
  InstallModpackRequest,
  InstallModpackResult,
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

/**
 * 根据用户设置的 community_filename_format 格式化下载文件名
 *
 * 详情页"下载到任意路径"流程使用：弹保存对话框前调用此命令获取格式化后的文件名作为默认名，
 * 避免 saveFile 直接用原始名导致下载文件名格式设置不生效。
 */
export async function formatDownloadFilename(
  fileName: string,
  translatedName?: string | null,
): Promise<string> {
  return await invoke<string>('format_download_filename', {
    fileName,
    translatedName: translatedName ?? null,
  })
}

/** 获取资源默认安装路径 */
export async function getResourceInstallPath(
  resourceType: ResourceType,
  versionId?: string,
): Promise<string> {
  return await invoke<string>('get_resource_install_path', { resourceType, versionId })
}

/**
 * 安装整合包
 *
 * 完整流程：下载原始包 → 检测格式 → 下载依赖 mods/files → 复制 overrides。
 * 进度通过 download_state 推送（与版本下载共用 DownloadPanel）。
 * 完成后前端需调用 install_merged 安装游戏本体。
 */
export async function installModpack(req: InstallModpackRequest): Promise<InstallModpackResult> {
  return await invoke<InstallModpackResult>('install_modpack', { req })
}

/**
 * 获取资源的 MC 百科详情页直链 URL
 *
 * 参考 PCL2 PageDownloadCompDetail.BtnIntroWiki_Click：
 * 通过 moddata.txt 的 slug → 行号（= class id）查表，拼接 `https://www.mcmod.cn/class/<id>.html`
 * 查不到返回 null，前端可回退到搜索 URL
 */
export async function getMcmodUrl(platform: Platform, slug: string): Promise<string | null> {
  return await invoke<string | null>('get_mcmod_url', { platform, slug })
}

// ==================== CurseForge 配置（读写走 getConfig/applyConfig）====================

/** CurseForge 加密配置（API Key 走 SDK DES 加密 + INI 存储） */
export interface CurseForgeConfig {
  enabled: boolean
  apiKey: string
}

// 读取：getConfig() 返回的 curseforgeEnabled / curseforgeApiKey
// 保存：applyConfig({ curseforgeEnabled, curseforgeApiKey })

// ==================== 社区资源配置（读写走 getConfig/applyConfig）====================

/** 社区资源配置（参考 PCL2 PageSetupSystem "社区资源" 卡片） */
export interface CommunityConfig {
  /** 来源策略：0=尽量镜像 / 1=缓慢时换镜像 / 2=尽量官方 */
  source: number
  /** 文件名格式：0=【译名】原名 / 1=[译名] 原名 / 2=译名-原名 / 3=原名-译名 / 4=仅原名 */
  filenameFormat: number
  /** Mod 管理页显示样式：0=标题译名/详情文件名 / 1=标题文件名/详情译名 */
  modLocalNameStyle: number
  /** 在显示 Mod 加载器时忽略 Quilt */
  ignoreQuilt: boolean
}

// 读取：getConfig() 返回的 communitySource / communityFilenameFormat / communityModLocalNameStyle / communityIgnoreQuilt
// 保存：applyConfig({ communitySource, communityFilenameFormat, communityModLocalNameStyle, communityIgnoreQuilt })
