/**
 * 社区资源 API 封装
 *
 * 重构后配置更新走统一 `applyConfig` 接口（见 system.ts），
 * 本文件仅保留读取命令与业务命令（搜索/详情/安装）。
 *
 * 注：底层已聚合为 `community_manager` 单一 IPC 入口，通过 `action` 字段分发。
 */

import { COMMUNITY_ACTIONS, communityManager } from './community-manager'
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
  InstallLocalModpackRequest,
  InstallModpackResult,
  ModpackPreview,
} from '@/types/community'

/** 搜索社区资源 */
export async function searchResources(params: SearchParams): Promise<SearchResult> {
  return communityManager<SearchResult>(COMMUNITY_ACTIONS.SEARCH_RESOURCES, params)
}

/** 获取分类标签列表 */
export async function getCategoryTags(resourceType: ResourceType): Promise<CategoryTagInfo[]> {
  return communityManager<CategoryTagInfo[]>(COMMUNITY_ACTIONS.GET_CATEGORY_TAGS, { resourceType })
}

/** 获取工程详情 */
export async function getProjectDetail(
  platform: Platform,
  projectId: string,
  resourceType: ResourceType,
): Promise<ResourceProject> {
  return communityManager<ResourceProject>(COMMUNITY_ACTIONS.GET_PROJECT_DETAIL, {
    platform,
    projectId,
    resourceType,
  })
}

/** 获取工程版本列表 */
export async function getProjectVersions(
  platform: Platform,
  projectId: string,
  resourceType: ResourceType,
): Promise<ResourceVersion[]> {
  return communityManager<ResourceVersion[]>(COMMUNITY_ACTIONS.GET_PROJECT_VERSIONS, {
    platform,
    projectId,
    resourceType,
  })
}

/** 下载安装资源 */
export async function downloadResource(req: DownloadRequest): Promise<DownloadResult> {
  return communityManager<DownloadResult>(COMMUNITY_ACTIONS.DOWNLOAD_RESOURCE, req)
}

/** 下载资源到自定义路径（流式 + 进度推送） */
export async function downloadResourceToPath(
  url: string,
  fileName: string,
  savePath: string,
): Promise<DownloadResult> {
  return communityManager<DownloadResult>(COMMUNITY_ACTIONS.DOWNLOAD_RESOURCE_TO_PATH, {
    url,
    fileName,
    savePath,
  })
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
  return communityManager<string>(COMMUNITY_ACTIONS.FORMAT_DOWNLOAD_FILENAME, {
    fileName,
    translatedName: translatedName ?? null,
  })
}

/** 获取资源默认安装路径 */
export async function getResourceInstallPath(
  resourceType: ResourceType,
  versionId?: string,
): Promise<string> {
  return communityManager<string>(COMMUNITY_ACTIONS.GET_RESOURCE_INSTALL_PATH, {
    resourceType,
    versionId,
  })
}

/**
 * 安装整合包
 *
 * 完整流程：下载原始包 → 检测格式 → 下载依赖 mods/files → 复制 overrides。
 * 进度通过 download_state 推送（与版本下载共用 DownloadPanel）。
 * 完成后前端需调用 install_merged 安装游戏本体。
 */
export async function installModpack(req: InstallModpackRequest): Promise<InstallModpackResult> {
  return communityManager<InstallModpackResult>(COMMUNITY_ACTIONS.INSTALL_MODPACK, req)
}

/**
 * 安装本地整合包（拖拽安装）
 *
 * 与 `installModpack` 的差异：直接使用本地文件路径，跳过下载阶段。
 * 共享解析 → 下载依赖 mods → 复制 overrides 流程。
 * 完成后前端需调用 install_merged 安装游戏本体。
 */
export async function installLocalModpack(
  req: InstallLocalModpackRequest,
): Promise<InstallModpackResult> {
  return communityManager<InstallModpackResult>(COMMUNITY_ACTIONS.INSTALL_LOCAL_MODPACK, req)
}

/**
 * 预览本地整合包（拖拽安装前置步骤）
 *
 * 仅打开 zip + 检测格式 + 解析 manifest/index，不下载、不复制 overrides。
 * 返回整合包基本信息 + 可选 Mod 列表，前端据弹窗询问用户是否下载可选 Mod。
 * 用户选择后调用 `installLocalModpack` 传入 `includeOptional` 参数完成安装。
 */
export async function previewLocalModpack(filePath: string): Promise<ModpackPreview> {
  return communityManager<ModpackPreview>(COMMUNITY_ACTIONS.PREVIEW_LOCAL_MODPACK, { filePath })
}

/**
 * 获取资源的 MC 百科详情页直链 URL
 *
 * 通过 moddata.txt 的 slug → 行号（= class id）查表，拼接 `https://www.mcmod.cn/class/<id>.html`
 * 查不到返回 null，前端可回退到搜索 URL
 */
export async function getMcmodUrl(platform: Platform, slug: string): Promise<string | null> {
  return communityManager<string | null>(COMMUNITY_ACTIONS.GET_MCMOD_URL, { platform, slug })
}

// ==================== CurseForge 配置（读写走 getConfig/applyConfig）====================
//
// 类型已内嵌于 `ConfigSnapshot`/`ConfigPatch`（参见 src/utils/api/system.ts）：
//   - curseforgeEnabled: boolean
//   - curseforgeApiKey: string   // 后端 SDK DES 加密后存 INI
//
// 读取：getConfig() 返回的 curseforgeEnabled / curseforgeApiKey
// 保存：applyConfig({ curseforgeEnabled, curseforgeApiKey })

// ==================== 社区资源配置（读写走 getConfig/applyConfig）====================
//
// 类型已内嵌于 `ConfigSnapshot`/`ConfigPatch`（参见 src/utils/api/system.ts）：
//   - communitySource: number           // 0=尽量镜像 / 1=缓慢时换镜像 / 2=尽量官方
//   - communityFilenameFormat: number  // 0=【译名】原名 / 1=[译名] 原名 / 2=译名-原名 / 3=原名-译名 / 4=仅原名
//   - communityModLocalNameStyle: number // 0=标题译名/详情文件名 / 1=标题文件名/详情译名
//   - communityIgnoreQuilt: boolean
//
// 读取：getConfig() 返回的 communitySource / communityFilenameFormat / communityModLocalNameStyle / communityIgnoreQuilt
// 保存：applyConfig({ communitySource, communityFilenameFormat, communityModLocalNameStyle, communityIgnoreQuilt })
