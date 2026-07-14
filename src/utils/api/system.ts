/**
 * 系统操作、目录选择、全局配置（下载/内存/线程/隔离/代理/进度）API
 */

import { invoke } from '@tauri-apps/api/core'

// ==================== 系统操作 ====================

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
 * 在资源管理器中打开并选中指定文件（Windows: explorer /select, macOS: open -R）
 */
export async function revealInExplorer(path: string): Promise<void> {
  return await invoke<void>('reveal_in_explorer', { path })
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

// ==================== 下载源与镜像 ====================

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

// ==================== 内存配置 ====================

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

// ==================== 下载线程与分片 ====================

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

// ==================== 版本隔离 ====================

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

// ==================== 配置通用读写 ====================

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

// ==================== 代理配置 ====================

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

// ==================== 下载进度查询 ====================

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
