/**
 * Tauri API 封装工具
 */

import { invoke } from '@tauri-apps/api/core'
import type { AuthResult, SdkStatus, DeviceCodeInfo, PollResult, MsAccountInfo } from '@/types/auth'
import type { VersionList } from '@/types/version'
import type { JavaRuntime } from '@/types/java'

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
// 微软登录相关
// ============================================================

/**
 * 微软登录步骤 1：申请设备码
 */
export async function msLoginStart(): Promise<DeviceCodeInfo> {
  return await invoke<DeviceCodeInfo>('ms_login_start')
}

/**
 * 微软登录步骤 2：轮询设备码授权结果
 */
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
}

/**
 * 获取已安装版本列表（包含类型信息）
 */
export async function listInstalledVersionsWithType(): Promise<InstalledVersionInfo[]> {
  return await invoke<InstalledVersionInfo[]>('list_installed_versions_with_type')
}

/**
 * 卸载版本
 */
export async function uninstallVersion(versionId: string): Promise<void> {
  return await invoke<void>('uninstall_version', { versionId })
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
 * 打开游戏目录
 */
export async function openGameDir(): Promise<void> {
  return await invoke<void>('open_game_dir')
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
