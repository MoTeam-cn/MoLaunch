/**
 * Tauri API 封装工具
 */

import { invoke } from '@tauri-apps/api/core'
import type { AuthResult, SdkStatus, MsAccountInfo, OfflineAccountInfo, DeviceCodeInfo, PollResult, LoginConfig } from '@/types/auth'
import type { VersionList } from '@/types/version'
import type { JavaRuntime, JavaRequirements, JavaCompatResult } from '@/types/java'

/**
 * 获取平台信息
 */
export async function getPlatformInfo(): Promise<SdkStatus> {
  return await invoke<SdkStatus>('get_platform_info')
}

/**
 * 获取 SDK 版本
 */
export async function getSdkVersion(): Promise<string | null> {
  return await invoke<string | null>('get_sdk_version')
}

/**
 * 检查 SDK 是否已初始化
 */
export async function isSdkInitialized(): Promise<boolean> {
  return await invoke<boolean>('is_sdk_initialized')
}

/**
 * 离线登录
 */
export async function loginOffline(username: string): Promise<AuthResult> {
  return await invoke<AuthResult>('login_offline', { username })
}

/**
 * 获取登录状态
 */
export async function getLoginStatus(): Promise<AuthResult | null> {
  return await invoke<AuthResult | null>('get_login_status')
}

/**
 * 登出
 */
export async function logout(): Promise<void> {
  return await invoke<void>('logout')
}

// ============================================================
// 微软登录相关（支持 Web Auth Code Flow 和 Device Code Flow）
// ============================================================

/** 获取登录流程配置 */
export async function msLoginGetConfig(): Promise<LoginConfig> {
  return await invoke<LoginConfig>('ms_login_get_config')
}

/** Web Auth Code Flow：打开 Webview 窗口 */
export async function msLoginWebStart(): Promise<void> {
  return await invoke<void>('ms_login_web_start')
}

/** Web Auth Code Flow：用授权码完成登录 */
export async function msLoginWebExchange(code: string): Promise<PollResult> {
  return await invoke<PollResult>('ms_login_web_exchange', { code })
}

/** Device Code Flow：请求设备码 */
export async function msLoginRequestDeviceCode(): Promise<DeviceCodeInfo> {
  return await invoke<DeviceCodeInfo>('ms_login_request_device_code')
}

/** Device Code Flow：轮询授权状态 */
export async function msLoginPoll(deviceCode: string): Promise<PollResult> {
  return await invoke<PollResult>('ms_login_poll', { deviceCode })
}

/**
 * 微软登录：使用 Refresh Token 静默刷新
 */
export async function msLoginRefresh(): Promise<AuthResult> {
  return await invoke<AuthResult>('ms_login_refresh')
}

/**
 * 获取已存储的微软账号列表
 */
export async function getMsAccounts(): Promise<MsAccountInfo[]> {
  return await invoke<MsAccountInfo[]>('get_ms_accounts')
}

/**
 * 删除已存储的微软账号
 */
export async function removeMsAccount(uuid: string): Promise<void> {
  return await invoke<void>('remove_ms_account', { uuid })
}

/**
 * 切换到已存储的微软账号
 */
export async function switchMsAccount(uuid: string): Promise<AuthResult> {
  return await invoke<AuthResult>('switch_ms_account', { uuid })
}

/**
 * 获取已存储的离线账号列表
 */
export async function getOfflineAccounts(): Promise<OfflineAccountInfo[]> {
  return await invoke<OfflineAccountInfo[]>('get_offline_accounts')
}

/**
 * 删除已存储的离线账号
 */
export async function removeOfflineAccount(uuid: string): Promise<void> {
  return await invoke<void>('remove_offline_account', { uuid })
}

/**
 * 切换到已存储的离线账号
 */
export async function switchOfflineAccount(uuid: string): Promise<AuthResult> {
  return await invoke<AuthResult>('switch_offline_account', { uuid })
}

/**
 * 设置离线账号的皮肤选择
 */
export async function setOfflineSkin(uuid: string, skin: string | null): Promise<void> {
  return await invoke<void>('set_offline_skin', { uuid, skin })
}

/**
 * 获取版本列表
 */
export async function listVersions(): Promise<VersionList> {
  return await invoke<VersionList>('list_versions')
}

/**
 * 下载版本
 */
export async function downloadVersion(versionId: string): Promise<void> {
  return await invoke<void>('download_version', { versionId })
}

/**
 * 获取已安装版本列表
 */
export async function listInstalledVersions(): Promise<string[]> {
  return await invoke<string[]>('list_installed_versions')
}

export interface InstalledVersionInfo {
  id: string
  version_type: string
  /** 自定义图标文件名（空=自动判断） */
  logo: string
}

/**
 * 获取已安装版本列表（包含类型信息）
 */
export async function listInstalledVersionsWithType(): Promise<InstalledVersionInfo[]> {
  return await invoke<InstalledVersionInfo[]>('list_installed_versions_with_type')
}

/**
 * Minecraft 文件夹项
 */
export interface McFolder {
  name: string
  path: string
}

/** 列出所有 Minecraft 文件夹 */
export async function listMcFolders(): Promise<McFolder[]> {
  return await invoke<McFolder[]>('list_mc_folders')
}

/** 添加 Minecraft 文件夹（自动去重） */
export async function addMcFolder(name: string, path: string): Promise<McFolder[]> {
  return await invoke<McFolder[]>('add_mc_folder', { name, path })
}

/** 移除 Minecraft 文件夹 */
export async function removeMcFolder(path: string): Promise<McFolder[]> {
  return await invoke<McFolder[]>('remove_mc_folder', { path })
}

/** 切换当前 Minecraft 文件夹 */
export async function switchMcFolder(path: string): Promise<string> {
  return await invoke<string>('switch_mc_folder', { path })
}

/** 重命名 Minecraft 文件夹 */
export async function renameMcFolder(path: string, newName: string): Promise<McFolder[]> {
  return await invoke<McFolder[]>('rename_mc_folder', { path, newName })
}

/**
 * 卸载版本
 */
export async function uninstallVersion(versionId: string): Promise<void> {
  return await invoke<void>('uninstall_version', { versionId })
}

/**
 * 获取版本的有效游戏目录（考虑版本隔离）
 * 隔离时返回 `{game_dir}/versions/{version_id}/`，非隔离时返回 `{game_dir}/`
 */
export async function getVersionEffectiveDir(versionId: string): Promise<string> {
  return await invoke<string>('get_version_effective_dir', { versionId })
}

/** 版本个性化信息 */
export interface VersionPersonalization {
  logo: string
  custom_info: string
  display_type: number
  is_star: boolean
  indie_type: number
  version_type: string
  original_version: string
  window_title: string
  server_enter: string
  advance_jvm_args: string
  advance_game_args: string
  advance_run_cmd: string
  java_path: string
  /** Java 选择模式：空/auto=自动选择, "auto_version"=自动选择指定版本范围, "folder"=使用版本文件夹中的 Java, "custom"=使用指定的 Java */
  java_mode: string
  /** 自动选择时的最小 Java 主版本（仅 auto_version 模式生效，0=不限） */
  java_version_min: number
  /** 自动选择时的最大 Java 主版本（仅 auto_version 模式生效，0=不限） */
  java_version_max: number
  /** 内存模式（空=跟随全局, "auto"=自动, "custom"=自定义） */
  memory_mode: string
  /** 版本独立最小内存（MB，仅 custom 模式生效，0 表示未设置） */
  min_memory: number
  /** 版本独立最大内存（MB，仅 custom 模式生效，0 表示未设置） */
  max_memory: number
}

/** 版本个性化字段更新（undefined 的字段不会被修改） */
export interface PersonalizationUpdate {
  logo?: string
  customInfo?: string
  displayType?: number
  isStar?: boolean
  indieType?: number
  windowTitle?: string
  serverEnter?: string
  advanceJvmArgs?: string
  advanceGameArgs?: string
  advanceRunCmd?: string
  javaPath?: string
  /** Java 选择模式：空/auto=自动选择, "auto_version"=自动选择指定版本范围, "folder"=使用版本文件夹中的 Java, "custom"=使用指定的 Java */
  javaMode?: string
  /** 自动选择时的最小 Java 主版本（仅 auto_version 模式生效，0=不限） */
  javaVersionMin?: number
  /** 自动选择时的最大 Java 主版本（仅 auto_version 模式生效，0=不限） */
  javaVersionMax?: number
  /** 内存模式：传空字符串=跟随全局, "auto"=自动, "custom"=自定义 */
  memoryMode?: string
  /** 版本独立最小内存（MB） */
  minMemory?: number
  /** 版本独立最大内存（MB） */
  maxMemory?: number
}

/**
 * 获取版本个性化设置
 */
export async function getVersionPersonalization(versionId: string): Promise<VersionPersonalization> {
  return await invoke<VersionPersonalization>('get_version_personalization', { versionId })
}

/**
 * 更新版本个性化字段（传 undefined 表示不修改该字段）
 */
export async function updateVersionPersonalization(
  versionId: string,
  update: PersonalizationUpdate,
): Promise<void> {
  return await invoke<void>('update_version_personalization', { versionId, update })
}

/**
 * 导出启动脚本（.bat，使用绝对路径 Java + 版权信息）
 *
 * @param javaPath 用户指定的 Java 路径（可选，为空时后端按 MC 版本自动检测）
 */
export async function exportLaunchScript(
  versionId: string,
  username: string,
  uuid: string,
  accessToken: string,
  loginType: string,
  javaPath: string | null,
  savePath: string,
): Promise<void> {
  return await invoke<void>('export_launch_script', {
    versionId,
    username,
    uuid,
    accessToken,
    loginType,
    javaPath,
    savePath,
  })
}

/**
 * 补全版本文件（校验并下载缺失的 libraries/assets）
 */
export async function fixVersionFiles(versionId: string): Promise<void> {
  return await invoke<void>('fix_version_files', { versionId })
}

// ==================== Mod 管理 ====================

/**
 * 单个 Mod 信息
 */
export interface ModInfo {
  /** 文件名（含扩展名） */
  file_name: string
  /** 启用时的文件名（去除 .disabled / .old 后缀） */
  enabled_name: string
  /** 是否启用 */
  is_enabled: boolean
  /** 文件大小（字节） */
  size: number
  /** 加载器类型（forge/fabric/neoforge/liteloader/unknown） */
  loader_type: string
}

/**
 * 判断版本是否可安装 Mod（含 Forge/Fabric/NeoForge/LiteLoader 或个性化分类为"可安装Mod"）
 */
export async function isVersionModable(versionId: string): Promise<boolean> {
  return await invoke<boolean>('is_version_modable', { versionId })
}

/**
 * 列出版本的 Mod
 */
export async function listMods(versionId: string): Promise<ModInfo[]> {
  return await invoke<ModInfo[]>('list_mods', { versionId })
}

/**
 * 启用/禁用 Mod
 */
export async function toggleMod(
  versionId: string,
  fileName: string,
  enable: boolean,
): Promise<void> {
  return await invoke<void>('toggle_mod', { versionId, fileName, enable })
}

/**
 * 删除 Mod
 */
export async function deleteMod(versionId: string, fileName: string): Promise<void> {
  return await invoke<void>('delete_mod', { versionId, fileName })
}

/**
 * 从外部文件安装 Mod（复制到 mods 目录）
 */
export async function installMod(versionId: string, sourcePath: string): Promise<void> {
  return await invoke<void>('install_mod', { versionId, sourcePath })
}

/**
 * 打开版本的 mods 目录（自动创建）
 */
export async function openModsDir(versionId: string): Promise<void> {
  return await invoke<void>('open_mods_dir', { versionId })
}

/**
 * 重命名版本
 */
export async function renameVersion(versionId: string, newName: string): Promise<void> {
  return await invoke<void>('rename_version', { versionId, newName })
}

/**
 * 获取上次选中的版本（持久化）
 */
export async function getSelectedVersion(): Promise<string | null> {
  return await invoke<string | null>('get_selected_version')
}

/**
 * 保存当前选中的版本（持久化到 config.ini）
 */
export async function setSelectedVersion(versionId: string | null): Promise<void> {
  return await invoke<void>('set_selected_version', { versionId })
}

/**
 * 获取设备 ID
 */
export async function getDeviceId(): Promise<string> {
  return await invoke<string>('get_device_id')
}

/**
 * 检测 Java
 */
export async function detectJava(): Promise<JavaRuntime> {
  return await invoke<JavaRuntime>('detect_java')
}

/**
 * 列出所有 Java
 */
export async function listJava(): Promise<JavaRuntime[]> {
  return await invoke<JavaRuntime[]>('list_java')
}

/**
 * 获取 MC 版本的 Java 需求（支持加载器约束）
 */
export async function getJavaRequirements(
  mcVersion: string,
  loader?: string | null,
): Promise<JavaRequirements> {
  return await invoke<JavaRequirements>('get_java_requirements', {
    mcVersion,
    loader: loader ?? null,
  })
}

/**
 * 检查指定 Java 是否兼容 MC 版本需求
 */
export async function checkJavaCompatible(
  javaPath: string,
  mcVersion: string,
  loader?: string | null,
): Promise<JavaCompatResult> {
  return await invoke<JavaCompatResult>('check_java_compatible', {
    javaPath,
    mcVersion,
    loader: loader ?? null,
  })
}

/**
 * Java 下载进度事件名
 */
export const JAVA_DOWNLOAD_PROGRESS_EVENT = 'java-download-progress'

// 重新导出 Java 下载进度类型（便于 store/组件通过 tauri 命名空间访问）
export type { JavaDownloadProgress } from '@/types/java'

/**
 * 下载 Java Runtime（从 Mojang 官方 Java Runtime 索引）
 *
 * @param targetMajor 目标 Java 大版本号（如 21、17、8）
 * @returns 下载的 java.exe 完整路径
 *
 * 进度通过 `java-download-progress` 事件推送，监听 `JavaDownloadProgress` payload
 */
export async function downloadJava(targetMajor: number): Promise<string> {
  return await invoke<string>('download_java', { targetMajor })
}

/**
 * 打开游戏目录
 */
export async function openGameDir(): Promise<void> {
  return await invoke<void>('open_game_dir')
}

/**
 * 打开任意路径（文件夹或文件）
 */
export async function openPath(path: string): Promise<void> {
  return await invoke<void>('open_path', { path })
}

/**
 * 获取游戏目录
 */
export async function getGameDir(): Promise<string> {
  return await invoke<string>('get_game_dir')
}

/**
 * 选择文件夹（打开系统对话框）
 */
export async function selectFolder(): Promise<string | null> {
  return await invoke<string | null>('select_folder')
}

/**
 * 选择文件（打开系统文件选择对话框）
 */
export async function selectFile(title?: string, filters?: { name: string; extensions: string[] }[]): Promise<string | null> {
  return await invoke<string | null>('select_file', { title, filters })
}

/**
 * 保存文件对话框（让用户选择保存位置）
 */
export async function saveFile(
  title?: string,
  defaultName?: string,
  filters?: { name: string; extensions: string[] }[],
): Promise<string | null> {
  return await invoke<string | null>('save_file', { title, defaultName, filters })
}

/**
 * 更新游戏目录
 */
export async function setGameDir(gameDir: string): Promise<void> {
  return await invoke<void>('set_game_dir', { gameDir })
}

/**
 * 获取镜像源
 */
export async function getMirrorUrl(): Promise<string | null> {
  return await invoke<string | null>('get_mirror_url')
}

/**
 * 设置镜像源
 */
export async function setMirrorUrl(mirrorUrl: string | null, skipReinit = false): Promise<void> {
  return await invoke<void>('set_mirror_url', { mirrorUrl, skipReinit })
}

/**
 * 获取下载源模式
 */
export async function getDownloadSource(): Promise<string> {
  return await invoke<string>('get_download_source')
}

/**
 * 设置下载源模式
 */
export async function setDownloadSource(source: string, skipReinit = false): Promise<void> {
  return await invoke<void>('set_download_source', { source, skipReinit })
}

/**
 * 获取最大下载速度
 */
export async function getMaxDownloadSpeed(): Promise<number> {
  return await invoke<number>('get_max_download_speed')
}

/**
 * 设置最大下载速度
 */
export async function setMaxDownloadSpeed(speed: number, skipReinit = false): Promise<void> {
  return await invoke<void>('set_max_download_speed', { speed, skipReinit })
}

/**
 * 获取系统内存信息
 */
export async function getSystemMemory(): Promise<{ total: number; used: number; available: number; usage_percent: number }> {
  return await invoke('get_system_memory')
}

/**
 * 获取配置文件路径
 */
export async function getConfigPath(): Promise<string> {
  return await invoke<string>('get_config_path')
}

/**
 * 手动保存配置到文件
 */
export async function saveConfigToFile(): Promise<void> {
  return await invoke<void>('save_config_to_file')
}

/**
 * 获取下载进度快照
 */
export async function getDownloadProgress(): Promise<{
  stages: { name: string; progress: number; weight: number; status: string; bytes_downloaded: number; bytes_total: number }[]
  current_stage_index: number
  global_speed: number
  global_bytes_downloaded: number
  global_bytes_total: number
  is_active: boolean
  is_complete: boolean
  error_code: number
}> {
  return await invoke('get_download_progress')
}

/**
 * 检查是否正在下载
 */
export async function isDownloading(): Promise<boolean> {
  return await invoke('is_downloading')
}

/**
 * 重置下载进度
 */
export async function resetDownloadProgress(): Promise<void> {
  return await invoke('reset_download_progress')
}

/**
 * 查询 Forge 版本列表
 */
export async function listForgeVersions(mcVersion: string): Promise<{ version: string; is_recommended: boolean; release_time: string }[]> {
  const json = await invoke<string>('list_forge_versions', { mcVersion })
  try { return JSON.parse(json) } catch { return [] }
}

/**
 * 查询 NeoForge 版本列表
 */
export async function listNeoforgeVersions(mcVersion: string): Promise<{ version: string; recommended: boolean }[]> {
  const json = await invoke<string>('list_neoforge_versions', { mcVersion })
  try { return JSON.parse(json) } catch { return [] }
}

/**
 * 查询 Fabric 版本列表
 */
export async function listFabricVersions(): Promise<{ version: string; stable: boolean }[]> {
  const json = await invoke<string>('list_fabric_versions')
  try { return JSON.parse(json) } catch { return [] }
}

/**
 * 查询 OptiFine 版本列表
 */
export async function listOptifineVersions(): Promise<{ display_name: string; is_preview: boolean }[]> {
  const json = await invoke<string>('list_optifine_versions')
  try { return JSON.parse(json) } catch { return [] }
}

/**
 * 查询 LiteLoader 版本列表
 */
export async function listLiteloaderVersions(mcVersion: string): Promise<string[]> {
  const json = await invoke<string>('list_liteloader_versions', { mcVersion })
  try { return JSON.parse(json) } catch { return [] }
}

/**
 * 校验加载器兼容性
 */
export async function validateLoaders(mcVersion: string, forge?: string, neoforge?: string, fabric?: string, optifine?: string): Promise<boolean> {
  return await invoke<boolean>('validate_loaders', { mcVersion, forgeVersion: forge, neoforgeVersion: neoforge, fabricVersion: fabric, optifineVersion: optifine })
}

/**
 * 合并安装（MC + 加载器）
 */
export async function installMerged(mcVersion: string, forge?: string, neoforge?: string, fabric?: string, optifine?: string, liteloader?: string, instanceName?: string): Promise<void> {
  return await invoke('install_merged', { mcVersion, forgeVersion: forge, neoforgeVersion: neoforge, fabricVersion: fabric, optifineVersion: optifine, liteloaderVersion: liteloader, instanceName })
}

/**
 * 设置最小内存
 */
export async function setMinMemory(memory: number): Promise<void> {
  return await invoke<void>('set_min_memory', { memory })
}

/**
 * 设置最大内存
 */
export async function setMaxMemory(memory: number): Promise<void> {
  return await invoke<void>('set_max_memory', { memory })
}

/**
 * 获取内存配置
 */
export async function getMemoryConfig(): Promise<[number, number]> {
  return await invoke<[number, number]>('get_memory_config')
}

/**
 * 获取内存模式
 */
export async function getMemoryMode(): Promise<string> {
  return await invoke<string>('get_memory_mode')
}

/**
 * 设置内存模式
 */
export async function setMemoryMode(mode: string): Promise<void> {
  return await invoke<void>('set_memory_mode', { mode })
}

/**
 * 设置下载线程数
 */
export async function setMaxDownloadThreads(threads: number): Promise<void> {
  return await invoke<void>('set_max_download_threads', { threads })
}

/**
 * 获取下载线程数
 */
export async function getMaxDownloadThreads(): Promise<number> {
  return await invoke<number>('get_max_download_threads')
}

/**
 * 设置分片数量
 */
export async function setChunkCount(count: number): Promise<void> {
  return await invoke<void>('set_chunk_count', { count })
}

/**
 * 获取分片数量
 */
export async function getChunkCount(): Promise<number> {
  return await invoke<number>('get_chunk_count')
}

/**
 * 设置版本隔离模式
 */
export async function setIsolationMode(mode: number): Promise<void> {
  return await invoke<void>('set_isolation_mode', { mode })
}

/**
 * 获取版本隔离模式
 */
export async function getIsolationMode(): Promise<number> {
  return await invoke<number>('get_isolation_mode')
}

/**
 * 获取配置值
 */
export async function getConfigValue(section: string, key: string): Promise<string | null> {
  return await invoke<string | null>('get_config_value', { section, key })
}

/**
 * 设置配置值
 */
export async function setConfigValue(section: string, key: string, value: string): Promise<void> {
  return await invoke<void>('set_config_value', { section, key, value })
}

/**
 * 获取代理模式
 */
export async function getProxyMode(): Promise<string> {
  return await invoke<string>('get_proxy_mode')
}

/**
 * 设置代理模式
 */
export async function setProxyMode(mode: string): Promise<void> {
  return await invoke<void>('set_proxy_mode', { mode })
}

/**
 * 获取代理类型
 */
export async function getProxyType(): Promise<string> {
  return await invoke<string>('get_proxy_type')
}

/**
 * 设置代理类型
 */
export async function setProxyType(proxyType: string): Promise<void> {
  return await invoke<void>('set_proxy_type', { proxyType })
}

/**
 * 获取代理地址
 */
export async function getProxyUrl(): Promise<string> {
  return await invoke<string>('get_proxy_url')
}

/**
 * 设置代理地址
 */
export async function setProxyUrl(url: string): Promise<void> {
  return await invoke<void>('set_proxy_url', { url })
}

/**
 * 启动游戏
 */
export async function launchGame(params: {
  versionId: string
  javaPath?: string
  username: string
  uuid: string
  accessToken: string
  loginType?: string
  windowWidth?: number
  windowHeight?: number
  serverAddress?: string
  serverPort?: number
}): Promise<number> {
  return await invoke<number>('launch_game', {
    versionId: params.versionId,
    javaPath: params.javaPath ?? null,
    username: params.username,
    uuid: params.uuid,
    accessToken: params.accessToken,
    windowWidth: params.windowWidth ?? null,
    windowHeight: params.windowHeight ?? null,
    serverAddress: params.serverAddress ?? null,
    serverPort: params.serverPort ?? null,
    loginType: params.loginType ?? null,
  })
}

export interface LaunchProgress {
  stage: string
  stage_progress: number
  overall_progress: number
  message: string
}

/**
 * 获取启动进度
 */
export async function getLaunchProgress(): Promise<LaunchProgress | null> {
  return await invoke<LaunchProgress | null>('get_launch_progress')
}

/**
 * 取消启动
 */
export async function cancelLaunch(): Promise<void> {
  return await invoke<void>('cancel_launch')
}

/**
 * 停止游戏
 */
export async function stopGame(): Promise<void> {
  return await invoke<void>('stop_game')
}

/**
 * 获取当前运行的游戏PID
 */
export async function getRunningGame(): Promise<number | null> {
  return await invoke<number | null>('get_running_game')
}

// ============================================================
// 皮肤与披风管理
// ============================================================

export interface SkinInfo {
  id: string
  state: string
  url: string
  variant: string
  alias: string | null
}

export interface CapeInfo {
  id: string
  state: string
  alias: string
  display_name: string
  url: string | null
}

export interface SkinCapeInfo {
  skins: SkinInfo[]
  capes: CapeInfo[]
}

/**
 * 获取当前账号的皮肤/披风信息
 */
export async function getSkinCapeInfo(): Promise<SkinCapeInfo> {
  return await invoke<SkinCapeInfo>('get_skin_cape_info')
}

/**
 * 获取皮肤 PNG 下载 URL
 */
export async function getSkinUrl(): Promise<string | null> {
  return await invoke<string | null>('get_skin_url')
}

/**
 * 下载皮肤 PNG，返回 data:image/png;base64,... 格式
 *
 * 前端收到后用 canvas 裁剪 (8,8,8,8) 区域作为头像（PCL2 的方式）
 */
export async function downloadSkinPng(uuid?: string): Promise<string> {
  return await invoke<string>('download_skin_png', { uuid: uuid ?? null })
}

/**
 * 下载当前已装备披风的 PNG，返回 data:image/png;base64,... 格式
 *
 * 无披风时返回 null
 */
export async function downloadCapePng(): Promise<string | null> {
  return await invoke<string | null>('download_cape_png')
}

/**
 * 将 data URL（如 data:image/png;base64,xxxx）保存到本地文件
 *
 * 用于"下载当前皮肤到本地"：前端已有 dataURL，用户选择保存位置后调用此命令写入
 */
export async function saveDataUrlToFile(dataUrl: string, path: string): Promise<void> {
  return await invoke<void>('save_data_url_to_file', { dataUrl, path })
}

/**
 * 上传/修改皮肤
 * @param filePath PNG 文件本地路径
 * @param variant 'classic' (Steve) 或 'slim' (Alex)
 */
export async function uploadSkin(filePath: string, variant: 'classic' | 'slim'): Promise<void> {
  return await invoke<void>('upload_skin', { filePath, variant })
}

/**
 * 装备披风
 */
export async function equipCape(capeId: string): Promise<void> {
  return await invoke<void>('equip_cape', { capeId })
}

/**
 * 取消披风
 */
export async function unequipCape(): Promise<void> {
  return await invoke<void>('unequip_cape')
}
