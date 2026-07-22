//! 工具模块的统一请求 / 响应类型定义
//!
//! - `ToolsRequest` 作为 `tools_manager` IPC 入口的统一请求体
//! - 各子模块的参数 / 响应类型集中声明在此，便于跨模块复用与序列化

use serde::{Deserialize, Serialize};

/// 统一请求体
///
/// `action` 决定分发到哪个子模块函数，`params` 由对应子模块自行反序列化。
#[derive(Debug, Serialize, Deserialize)]
pub struct ToolsRequest {
    pub action: String,
    #[serde(default)]
    pub params: serde_json::Value,
}

// ===== 外部下载相关 =====

/// 外部下载请求参数
#[derive(Debug, Serialize, Deserialize)]
pub struct DownloadFileParams {
    pub url: String,
    pub file_name: String,
}

/// 删除已下载文件请求参数
#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteDownloadParams {
    pub file_name: String,
}

/// 从 URL 获取文件名请求参数
#[derive(Debug, Serialize, Deserialize)]
pub struct FetchFilenameParams {
    pub url: String,
}

/// 外部下载结果
#[derive(Debug, Serialize, Deserialize)]
pub struct ExternalDownloadResult {
    pub path: String,
    pub size: u64,
    pub file_name: String,
}

/// 已下载文件条目
#[derive(Debug, Serialize, Deserialize)]
pub struct ExternalDownloadEntry {
    pub name: String,
    pub size: u64,
    /// Unix 时间戳（秒）
    pub modified: u64,
}

/// 从 URL 获取文件名结果
#[derive(Debug, Serialize, Deserialize)]
pub struct FetchFilenameResult {
    pub filename: String,
    pub file_size: u64,
}

// ===== 清理相关 =====

/// 单个可清理项
#[derive(Debug, Serialize, Deserialize)]
pub struct CleanupItem {
    pub path: String,
    pub display_name: String,
    pub category: String,
    pub size: u64,
    pub file_count: u64,
}

/// 清理扫描结果
#[derive(Debug, Serialize, Deserialize)]
pub struct CleanupScanResult {
    pub items: Vec<CleanupItem>,
    pub total_size: u64,
    pub total_files: u64,
}

/// 清理执行请求参数
#[derive(Debug, Serialize, Deserialize)]
pub struct CleanupExecuteParams {
    pub paths: Vec<String>,
}

/// 清理执行失败项
#[derive(Debug, Serialize, Deserialize)]
pub struct CleanupFailedItem {
    pub path: String,
    pub error: String,
}

/// 清理执行结果
#[derive(Debug, Serialize, Deserialize)]
pub struct CleanupExecuteResult {
    pub cleaned_size: u64,
    pub cleaned_files: u64,
    pub failed: Vec<CleanupFailedItem>,
}

// ===== 内存优化相关 =====

/// 内存优化请求参数
#[derive(Debug, Serialize, Deserialize)]
pub struct MemoryOptimizeParams {
    /// 优化模式："light"（轻量）或 "strong"（强力）
    pub mode: String,
}

/// 内存优化结果（所有字段单位：字节）
#[derive(Debug, Serialize, Deserialize)]
pub struct MemoryOptimizeResult {
    pub freed_bytes: u64,
    pub before_bytes: u64,
    pub after_bytes: u64,
    /// 本次优化使用的模式："light" / "strong"
    pub mode: String,
}

// ===== Mod 依赖检测相关 =====

/// Mod 依赖检测请求参数
#[derive(Debug, Serialize, Deserialize)]
pub struct ModDependencyCheckParams {
    pub version_id: String,
}

/// Mod 依赖检测结果
#[derive(Debug, Serialize, Deserialize)]
pub struct ModDependencyResult {
    /// 依赖的 mod_id 不在已安装列表中
    pub missing: Vec<MissingDep>,
    /// 冲突依赖（暂时留空，未来扩展）
    pub conflicts: Vec<ConflictDep>,
}

/// 缺失的依赖项
#[derive(Debug, Serialize, Deserialize)]
pub struct MissingDep {
    /// 依赖此 mod 的文件名
    pub required_by: String,
    /// 缺失的 mod_id
    pub mod_id: String,
}

/// 冲突依赖项（未来扩展用）
#[derive(Debug, Serialize, Deserialize)]
pub struct ConflictDep {
    pub mod_id: String,
    pub reason: String,
}

// ===== Mod 去重扫描相关 =====

/// Mod 去重扫描请求参数
#[derive(Debug, Serialize, Deserialize)]
pub struct ModDedupScanParams {
    pub version_id: String,
}

/// Mod 去重扫描结果
#[derive(Debug, Serialize, Deserialize)]
pub struct ModDedupResult {
    pub duplicates: Vec<DuplicateMod>,
}

/// 重复的 Mod（同一 mod_id 有多个版本）
#[derive(Debug, Serialize, Deserialize)]
pub struct DuplicateMod {
    pub mod_id: String,
    pub versions: Vec<DuplicateVersion>,
}

/// 重复 Mod 的单个版本条目
#[derive(Debug, Serialize, Deserialize)]
pub struct DuplicateVersion {
    pub version: String,
    pub file_name: String,
    pub file_size: u64,
}

// ===== 启动器数据导出相关 =====

/// 启动器数据导出请求参数
#[derive(Debug, Serialize, Deserialize)]
pub struct ExportLauncherDataParams {
    /// 导出 zip 的完整路径
    pub output_path: String,
    pub include_config: bool,
    pub include_versions: bool,
    pub include_accounts: bool,
}

/// 启动器数据导出结果
#[derive(Debug, Serialize, Deserialize)]
pub struct ExportResult {
    pub success: bool,
    pub file_path: String,
    pub file_size: u64,
    /// 导出的数据类型（"config" / "versions" / "accounts"）
    pub exported_items: Vec<String>,
}

// ===== 崩溃日志分析相关 =====

/// 崩溃日志分析请求参数
#[derive(Debug, Serialize, Deserialize)]
pub struct CrashAnalyzeParams {
    /// 崩溃日志原文
    pub log_text: String,
}

/// 崩溃日志分析结果
#[derive(Debug, Serialize, Deserialize)]
pub struct CrashAnalyzeResult {
    /// 识别出的崩溃分析条目
    pub analyses: Vec<CrashAnalysisItem>,
}

/// 单个崩溃分析条目
#[derive(Debug, Serialize, Deserialize)]
pub struct CrashAnalysisItem {
    /// 分类：java_version / missing_mod / memory / driver / mod_conflict / other
    pub category: String,
    /// 严重级别：error / warning / info
    pub severity: String,
    /// 标题
    pub title: String,
    /// 匹配到的相关行片段
    pub detail: String,
    /// 中文修复建议
    pub suggestion: String,
}

// ===== 截图管理相关 =====

/// 截图列表结果
#[derive(Debug, Serialize, Deserialize)]
pub struct ScreenshotListResult {
    /// 截图条目（按修改时间降序）
    pub items: Vec<ScreenshotItem>,
    /// 所有截图总字节数
    pub total_size: u64,
}

/// 单个截图条目
#[derive(Debug, Serialize, Deserialize)]
pub struct ScreenshotItem {
    /// 截图完整路径
    pub path: String,
    /// 文件名
    pub name: String,
    /// 文件大小（字节）
    pub size: u64,
    /// 修改时间（Unix 秒级时间戳）
    pub modified: u64,
}

/// 截图删除请求参数
#[derive(Debug, Serialize, Deserialize)]
pub struct ScreenshotDeleteParams {
    /// 待删除的截图路径列表
    pub paths: Vec<String>,
}

/// 截图删除失败项
#[derive(Debug, Serialize, Deserialize)]
pub struct ScreenshotFailedItem {
    /// 失败的路径
    pub path: String,
    /// 失败原因
    pub error: String,
}

/// 截图删除结果
#[derive(Debug, Serialize, Deserialize)]
pub struct ScreenshotDeleteResult {
    /// 成功删除数量
    pub deleted_count: u64,
    /// 释放的字节数
    pub freed_bytes: u64,
    /// 删除失败的项
    pub failed: Vec<ScreenshotFailedItem>,
}

// ===== 资源包管理相关 =====

/// 资源包列表结果
#[derive(Debug, Serialize, Deserialize)]
pub struct ResourcePackListResult {
    /// 资源包条目
    pub items: Vec<ResourcePackItem>,
}

/// 单个资源包条目
#[derive(Debug, Serialize, Deserialize)]
pub struct ResourcePackItem {
    /// 名称（文件名或目录名）
    pub name: String,
    /// 完整路径
    pub path: String,
    /// 格式：zip / folder
    pub format: String,
    /// 大小（字节，folder 为递归总字节）
    pub size: u64,
}

/// 资源包转换请求参数
#[derive(Debug, Serialize, Deserialize)]
pub struct ResourcePackConvertParams {
    /// 源路径
    pub path: String,
    /// 目标格式：zip / folder
    pub target_format: String,
}

/// 资源包转换结果
#[derive(Debug, Serialize, Deserialize)]
pub struct ResourcePackConvertResult {
    /// 是否成功
    pub success: bool,
    /// 输出路径
    pub output_path: String,
    /// 提示信息
    pub message: String,
}

// ===== 版本 JSON 读写相关 =====

/// 版本 JSON 读取请求参数
#[derive(Debug, Serialize, Deserialize)]
pub struct VersionJsonReadParams {
    /// 版本 ID
    pub version_id: String,
}

/// 版本 JSON 读取结果
#[derive(Debug, Serialize, Deserialize)]
pub struct VersionJsonReadResult {
    /// 文件内容
    pub content: String,
    /// 文件路径
    pub path: String,
}

/// 版本 JSON 保存请求参数
#[derive(Debug, Serialize, Deserialize)]
pub struct VersionJsonSaveParams {
    /// 版本 ID
    pub version_id: String,
    /// JSON 内容
    pub content: String,
}

/// 版本 JSON 保存结果
#[derive(Debug, Serialize, Deserialize)]
pub struct VersionJsonSaveResult {
    /// 是否成功
    pub success: bool,
}
