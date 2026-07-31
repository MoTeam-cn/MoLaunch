/**
 * 工具模块 - 外部下载 + 文件名获取
 *
 * 对应后端 `tools_manager` 的 download_file / get_download_dir / list_downloads /
 * delete_download / fetch_filename action。
 */

import { TOOLS_ACTIONS, toolsManager } from './core'

export interface ExternalDownloadResult {
  path: string
  size: number
  file_name: string
}

export interface ExternalDownloadEntry {
  name: string
  size: number
  modified: number
}

export interface FetchFilenameResult {
  filename: string
  file_size: number
}

/** 新建外部下载任务 */
export function downloadFile(url: string, fileName: string): Promise<ExternalDownloadResult> {
  return toolsManager<ExternalDownloadResult>(TOOLS_ACTIONS.DOWNLOAD_FILE, { url, file_name: fileName })
}

/** 获取下载目录（自定义或默认 .Molaunch/Download/） */
export function getDownloadDir(): Promise<string> {
  return toolsManager<string>(TOOLS_ACTIONS.GET_DOWNLOAD_DIR)
}

/** 列举已下载文件 */
export function listDownloads(): Promise<ExternalDownloadEntry[]> {
  return toolsManager<ExternalDownloadEntry[]>(TOOLS_ACTIONS.LIST_DOWNLOADS)
}

/** 删除已下载文件 */
export function deleteDownload(fileName: string): Promise<void> {
  return toolsManager<void>(TOOLS_ACTIONS.DELETE_DOWNLOAD, { file_name: fileName })
}

/** 从 URL 响应头获取文件名（Content-Disposition / URL 路径推断） */
export function fetchFilename(url: string): Promise<FetchFilenameResult> {
  return toolsManager<FetchFilenameResult>(TOOLS_ACTIONS.FETCH_FILENAME, { url })
}
