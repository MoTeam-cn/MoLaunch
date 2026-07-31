/**
 * 工具模块 - 清理游戏垃圾 + 内存优化
 *
 * 对应后端 `tools_manager` 的 cleanup_scan / cleanup_execute / memory_optimize action。
 */

import { TOOLS_ACTIONS, toolsManager } from './core'

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

/** 扫描可清理的游戏垃圾文件 */
export function cleanupScan(): Promise<CleanupScanResult> {
  return toolsManager<CleanupScanResult>(TOOLS_ACTIONS.CLEANUP_SCAN)
}

/** 执行清理 */
export function cleanupExecute(paths: string[]): Promise<CleanupExecuteResult> {
  return toolsManager<CleanupExecuteResult>(TOOLS_ACTIONS.CLEANUP_EXECUTE, { paths })
}

/**
 * 执行内存优化
 * @param mode 优化模式：'light'（轻量，仅清空工作集）或 'strong'（强力，清空 standby list）
 */
export function memoryOptimize(mode: MemoryOptimizeMode = 'light'): Promise<MemoryOptimizeResult> {
  return toolsManager<MemoryOptimizeResult>(TOOLS_ACTIONS.MEMORY_OPTIMIZE, { mode })
}
