/**
 * 工具模块统一 API
 *
 * 后端 `tools_manager` IPC 命令通过 `action` 字段分发到不同子模块。
 * 本文件提供类型安全的封装，避免业务代码直接拼 invoke 参数。
 */

import { invoke } from '@tauri-apps/api/core'

// ==================== 类型定义 ====================

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

export interface CleanupItem {
  path: string
  display_name: string
  category: string
  size: number
  file_count: number
}

export interface CleanupScanResult {
  items: CleanupItem[]
  total_size: number
  total_files: number
}

export interface CleanupFailedItem {
  path: string
  error: string
}

export interface CleanupExecuteResult {
  cleaned_size: number
  cleaned_files: number
  failed: CleanupFailedItem[]
}

export type MemoryOptimizeMode = 'light' | 'strong'

export interface MemoryOptimizeResult {
  /** 释放的内存量（字节） */
  freed_bytes: number
  /** 优化前可用内存（字节） */
  before_bytes: number
  /** 优化后可用内存（字节） */
  after_bytes: number
  /** 本次优化使用的模式："light" / "strong" */
  mode: MemoryOptimizeMode
}

// ==================== 统一调用入口 ====================

/**
 * 调用 tools_manager IPC
 * @param action 操作名称
 * @param params 参数对象（可选）
 */
export async function toolsManager<T = unknown>(
  action: string,
  params?: unknown,
): Promise<T> {
  return invoke<T>('tools_manager', { req: { action, params: params ?? null } })
}

// ==================== 外部下载 ====================

/** 新建外部下载任务 */
export function downloadFile(url: string, fileName: string): Promise<ExternalDownloadResult> {
  return toolsManager<ExternalDownloadResult>('download_file', { url, file_name: fileName })
}

/** 获取下载目录（自定义或默认 .Molaunch/Download/） */
export function getDownloadDir(): Promise<string> {
  return toolsManager<string>('get_download_dir')
}

/** 列举已下载文件 */
export function listDownloads(): Promise<ExternalDownloadEntry[]> {
  return toolsManager<ExternalDownloadEntry[]>('list_downloads')
}

/** 删除已下载文件 */
export function deleteDownload(fileName: string): Promise<void> {
  return toolsManager<void>('delete_download', { file_name: fileName })
}

// ==================== 文件名获取 ====================

/** 从 URL 响应头获取文件名（Content-Disposition / URL 路径推断） */
export function fetchFilename(url: string): Promise<FetchFilenameResult> {
  return toolsManager<FetchFilenameResult>('fetch_filename', { url })
}

// ==================== 清理游戏垃圾 ====================

/** 扫描可清理的游戏垃圾文件 */
export function cleanupScan(): Promise<CleanupScanResult> {
  return toolsManager<CleanupScanResult>('cleanup_scan')
}

/** 执行清理 */
export function cleanupExecute(paths: string[]): Promise<CleanupExecuteResult> {
  return toolsManager<CleanupExecuteResult>('cleanup_execute', { paths })
}

// ==================== 内存优化 ====================

/**
 * 执行内存优化
 * @param mode 优化模式：'light'（轻量，仅清空工作集）或 'strong'（强力，清空 standby list）
 */
export function memoryOptimize(mode: MemoryOptimizeMode = 'light'): Promise<MemoryOptimizeResult> {
  return toolsManager<MemoryOptimizeResult>('memory_optimize', { mode })
}
