/**
 * 系统操作、目录选择、下载进度查询 API
 * 全局配置读写已拆分至 `./config.ts`；9 个原 Tauri 命令聚合为 `system_manager` 单一 IPC 入口（经 `action` 分发）。
 */

import type { RawDownloadProgress } from '@/types/download'
import { SYSTEM_ACTIONS, systemManager } from './system-manager'
import { VERSION_PROGRESS_ACTIONS, versionProgressManager } from './version-progress-manager'

// ==================== 系统操作 ====================

/** 打开游戏目录 */
export async function openGameDir(): Promise<void> {
  return systemManager<void>(SYSTEM_ACTIONS.OPEN_GAME_DIR)
}

/** 打开任意路径（文件夹或文件） */
export async function openPath(path: string): Promise<void> {
  return systemManager<void>(SYSTEM_ACTIONS.OPEN_PATH, { path })
}

/** 在资源管理器中打开并选中指定文件（Windows: explorer /select, macOS: open -R） */
export async function revealInExplorer(path: string): Promise<void> {
  return systemManager<void>(SYSTEM_ACTIONS.REVEAL_IN_EXPLORER, { path })
}

/** 获取游戏目录 */
export async function getGameDir(): Promise<string> {
  return systemManager<string>(SYSTEM_ACTIONS.GET_GAME_DIR)
}

/**
 * 将文本内容写入指定路径的文件
 *
 * 文件 / 文件夹选择对话框请使用 `@/utils/fileDialog`（基于 @tauri-apps/plugin-dialog）。
 * 此命令仅负责写入文本，会自动创建父目录。
 */
export async function writeTextFile(path: string, content: string): Promise<void> {
  return systemManager<void>(SYSTEM_ACTIONS.WRITE_TEXT_FILE, { path, content })
}

/**
 * 更新游戏目录
 *
 * 通过 `system_manager` 的 `set_game_dir` action 调用。
 * 用户在设置页改 game_dir 时应走 `applyConfig({ gameDir })`；
 * 此命令用于版本切换等内部流程需要直接修改 game_dir 的场景。
 */
export async function setGameDir(gameDir: string): Promise<void> {
  return systemManager<void>(SYSTEM_ACTIONS.SET_GAME_DIR, { gameDir })
}

/** 获取系统内存信息 */
export async function getSystemMemory(): Promise<{ total: number; used: number; available: number; usage_percent: number }> {
  return systemManager(SYSTEM_ACTIONS.GET_SYSTEM_MEMORY)
}

/** 获取配置文件路径 */
export async function getConfigPath(): Promise<string> {
  return systemManager<string>(SYSTEM_ACTIONS.GET_CONFIG_PATH)
}

/** 手动保存配置到文件 */
export async function saveConfigToFile(): Promise<void> {
  return systemManager<void>(SYSTEM_ACTIONS.SAVE_CONFIG_TO_FILE)
}

// ==================== 下载进度查询 ====================
// 注：底层已聚合为 `version_progress_manager` 单一 IPC 入口，通过 `action` 字段分发。

/** 获取下载进度快照（返回后端原始结构 RawDownloadProgress，由调用方做 fallback） */
export async function getDownloadProgress(): Promise<RawDownloadProgress> {
  return versionProgressManager<RawDownloadProgress>(VERSION_PROGRESS_ACTIONS.GET_DOWNLOAD_PROGRESS)
}

/** 检查是否正在下载 */
export async function isDownloading(): Promise<boolean> {
  return versionProgressManager<boolean>(VERSION_PROGRESS_ACTIONS.IS_DOWNLOADING)
}

/** 重置下载进度 */
export async function resetDownloadProgress(): Promise<void> {
  return versionProgressManager<void>(VERSION_PROGRESS_ACTIONS.RESET_DOWNLOAD_PROGRESS)
}

/** 取消下载（设置 cancel_flag，正在进行的下载会尽快中止） */
export async function cancelDownload(): Promise<void> {
  return versionProgressManager<void>(VERSION_PROGRESS_ACTIONS.CANCEL_DOWNLOAD)
}

/** 暂停下载（设置 pause_flag，新任务不再开始，已进行的任务完成当前文件后等待） */
export async function pauseDownload(): Promise<void> {
  return versionProgressManager<void>(VERSION_PROGRESS_ACTIONS.PAUSE_DOWNLOAD)
}

/** 恢复下载（清除 pause_flag） */
export async function resumeDownload(): Promise<void> {
  return versionProgressManager<void>(VERSION_PROGRESS_ACTIONS.RESUME_DOWNLOAD)
}
