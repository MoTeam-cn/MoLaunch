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

// ==================== Mod 依赖检测 ====================

/** 缺失的依赖项 */
export interface MissingDep {
  /** 依赖此 mod 的文件名 */
  required_by: string
  /** 缺失的 mod_id */
  mod_id: string
}

/** 冲突依赖项（未来扩展用） */
export interface ConflictDep {
  mod_id: string
  reason: string
}

/** Mod 依赖检测结果 */
export interface ModDependencyResult {
  /** 依赖的 mod_id 不在已安装列表中 */
  missing: MissingDep[]
  /** 冲突依赖（暂时留空，未来扩展） */
  conflicts: ConflictDep[]
}

/** Mod 依赖检测 */
export function modDependencyCheck(versionId: string): Promise<ModDependencyResult> {
  return toolsManager<ModDependencyResult>('mod_dependency_check', { version_id: versionId })
}

// ==================== Mod 去重扫描 ====================

/** 重复 Mod 的单个版本条目 */
export interface DuplicateVersion {
  version: string
  file_name: string
  file_size: number
}

/** 重复的 Mod（同一 mod_id 有多个版本） */
export interface DuplicateMod {
  mod_id: string
  versions: DuplicateVersion[]
}

/** Mod 去重扫描结果 */
export interface ModDedupResult {
  duplicates: DuplicateMod[]
}

/** Mod 去重扫描 */
export function modDedupScan(versionId: string): Promise<ModDedupResult> {
  return toolsManager<ModDedupResult>('mod_dedup_scan', { version_id: versionId })
}

// ==================== 启动器数据导出 ====================

/** 启动器数据导出请求参数 */
export interface ExportLauncherDataParams {
  /** 导出 zip 的完整路径 */
  output_path: string
  include_config: boolean
  include_versions: boolean
  include_accounts: boolean
}

/** 启动器数据导出结果 */
export interface ExportResult {
  success: boolean
  file_path: string
  file_size: number
  /** 导出的数据类型（"config" / "versions" / "accounts"） */
  exported_items: string[]
}

/** 启动器数据导出 */
export function exportLauncherData(params: ExportLauncherDataParams): Promise<ExportResult> {
  return toolsManager<ExportResult>('export_launcher_data', params)
}

// ==================== 崩溃日志分析 ====================

/** 单个崩溃分析条目 */
export interface CrashAnalysisItem {
  /** 分类：java_version / missing_mod / memory / driver / mod_conflict / other */
  category: string
  /** 严重级别：error / warning / info */
  severity: string
  /** 标题 */
  title: string
  /** 匹配到的相关行片段 */
  detail: string
  /** 中文修复建议 */
  suggestion: string
}

/** 崩溃日志分析结果 */
export interface CrashAnalyzeResult {
  analyses: CrashAnalysisItem[]
}

/** 崩溃日志分析 */
export function crashAnalyze(logText: string): Promise<CrashAnalyzeResult> {
  return toolsManager<CrashAnalyzeResult>('crash_analyze', { log_text: logText })
}

// ==================== 截图批量管理 ====================

/** 单个截图条目 */
export interface ScreenshotItem {
  path: string
  name: string
  size: number
  /** 修改时间（Unix 秒级时间戳） */
  modified: number
}

/** 截图列表结果 */
export interface ScreenshotListResult {
  items: ScreenshotItem[]
  total_size: number
}

/** 截图删除失败项 */
export interface ScreenshotFailedItem {
  path: string
  error: string
}

/** 截图删除结果 */
export interface ScreenshotDeleteResult {
  deleted_count: number
  freed_bytes: number
  failed: ScreenshotFailedItem[]
}

/** 列举游戏截图 */
export function screenshotList(): Promise<ScreenshotListResult> {
  return toolsManager<ScreenshotListResult>('screenshot_list')
}

/** 批量删除截图 */
export function screenshotDelete(paths: string[]): Promise<ScreenshotDeleteResult> {
  return toolsManager<ScreenshotDeleteResult>('screenshot_delete', { paths })
}

// ==================== 资源包转换 ====================

/** 单个资源包条目 */
export interface ResourcePackItem {
  name: string
  path: string
  /** 格式：zip / folder */
  format: string
  size: number
}

/** 资源包列表结果 */
export interface ResourcePackListResult {
  items: ResourcePackItem[]
}

/** 资源包转换结果 */
export interface ResourcePackConvertResult {
  success: boolean
  output_path: string
  message: string
}

/** 列举资源包 */
export function resourcepackList(): Promise<ResourcePackListResult> {
  return toolsManager<ResourcePackListResult>('resourcepack_list')
}

/** 转换资源包格式（zip ↔ folder） */
export function resourcepackConvert(path: string, targetFormat: 'zip' | 'folder'): Promise<ResourcePackConvertResult> {
  return toolsManager<ResourcePackConvertResult>('resourcepack_convert', { path, target_format: targetFormat })
}

// ==================== 版本 JSON 编辑 ====================

/** 版本 JSON 读取结果 */
export interface VersionJsonReadResult {
  content: string
  path: string
}

/** 版本 JSON 保存结果 */
export interface VersionJsonSaveResult {
  success: boolean
}

/** 读取版本 JSON */
export function versionJsonRead(versionId: string): Promise<VersionJsonReadResult> {
  return toolsManager<VersionJsonReadResult>('version_json_read', { version_id: versionId })
}

/** 保存版本 JSON（后端会先校验 JSON 合法性） */
export function versionJsonSave(versionId: string, content: string): Promise<VersionJsonSaveResult> {
  return toolsManager<VersionJsonSaveResult>('version_json_save', { version_id: versionId, content })
}
