/**
 * 系统操作、目录选择、下载进度查询 API
 *
 * 全局配置读写（getConfig/applyConfig/refreshConfig 等）已拆分到 `./config.ts`。
 */

import { invoke } from '@tauri-apps/api/core'
import type { RawDownloadProgress } from '@/types/download'

// ==================== 系统操作 ====================

/** 打开游戏目录 */
export async function openGameDir(): Promise<void> {
  return await invoke<void>('open_game_dir')
}

/** 打开任意路径（文件夹或文件） */
export async function openPath(path: string): Promise<void> {
  return await invoke<void>('open_path', { path })
}

/** 在资源管理器中打开并选中指定文件（Windows: explorer /select, macOS: open -R） */
export async function revealInExplorer(path: string): Promise<void> {
  return await invoke<void>('reveal_in_explorer', { path })
}

/** 获取游戏目录 */
export async function getGameDir(): Promise<string> {
  return await invoke<string>('get_game_dir')
}

/**
 * 将文本内容写入指定路径的文件
 *
 * 文件 / 文件夹选择对话框请使用 `@/utils/fileDialog`（基于 @tauri-apps/plugin-dialog）。
 * 此命令仅负责写入文本，会自动创建父目录。
 */
export async function writeTextFile(path: string, content: string): Promise<void> {
  return await invoke<void>('write_text_file', { path, content })
}

/**
 * 更新游戏目录
 *
 * 注意：此命令保留独立，因为它在版本切换流程中被内部调用。
 * 用户在设置页改 game_dir 时应走 `applyConfig({ gameDir })`。
 */
export async function setGameDir(gameDir: string): Promise<void> {
  return await invoke<void>('set_game_dir', { gameDir })
}

/** 获取系统内存信息 */
export async function getSystemMemory(): Promise<{ total: number; used: number; available: number; usage_percent: number }> {
  return await invoke('get_system_memory')
}

/** 获取配置文件路径 */
export async function getConfigPath(): Promise<string> {
  return await invoke<string>('get_config_path')
}

/** 手动保存配置到文件 */
export async function saveConfigToFile(): Promise<void> {
  return await invoke<void>('save_config_to_file')
}

// ==================== 下载进度查询 ====================

/** 获取下载进度快照（返回后端原始结构 RawDownloadProgress，由调用方做 fallback） */
export async function getDownloadProgress(): Promise<RawDownloadProgress> {
  return await invoke('get_download_progress')
}

/** 检查是否正在下载 */
export async function isDownloading(): Promise<boolean> {
  return await invoke('is_downloading')
}

/** 重置下载进度 */
export async function resetDownloadProgress(): Promise<void> {
  return await invoke('reset_download_progress')
}

/** 取消下载（设置 cancel_flag，正在进行的下载会尽快中止） */
export async function cancelDownload(): Promise<void> {
  return await invoke('cancel_download')
}

/** 暂停下载（设置 pause_flag，新任务不再开始，已进行的任务完成当前文件后等待） */
export async function pauseDownload(): Promise<void> {
  return await invoke('pause_download')
}

/** 恢复下载（清除 pause_flag） */
export async function resumeDownload(): Promise<void> {
  return await invoke('resume_download')
}
