//! 工具模块的统一响应类型定义
//!
//! 各子模块的参数 / 响应类型集中声明在此，便于跨模块复用与序列化。
//!
//! 注：原 `ToolsRequest` 已替换为通用的 `utils::dispatcher::ActionRequest`，
//! 与 `meta_manager` 共用同一请求体结构。

use serde::{Deserialize, Serialize};

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

/// 截图列表查询参数
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ScreenshotListParams {
    /// 可选版本 ID
    /// - 传入：按版本隔离配置解析该版本的有效游戏目录，扫 `<effective>/screenshots/`
    /// - 不传：走全局 game_dir/screenshots/
    #[serde(default)]
    pub version_id: Option<String>,
}

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
    /// 可选版本 ID（与 list 时传入的相同，用于解析截图目录做路径校验）
    #[serde(default)]
    pub version_id: Option<String>,
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

/// 资源包列表结果
#[derive(Debug, Serialize, Deserialize)]
pub struct ResourcePackListResult {
    /// 资源包条目
    pub items: Vec<ResourcePackItem>,
}

/// 资源包列表查询参数
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ResourcePackListParams {
    /// 可选版本 ID（同 ScreenshotListParams 语义）
    #[serde(default)]
    pub version_id: Option<String>,
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
    /// 可选版本 ID（同 ResourcePackListParams 语义，按版本隔离配置解析基准目录）
    #[serde(default)]
    pub version_id: Option<String>,
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

/// 存档列表结果
#[derive(Debug, Serialize, Deserialize)]
pub struct ArchiveListResult {
    /// 存档条目（按名称排序）
    pub items: Vec<ArchiveItem>,
    /// 所有存档总字节数
    pub total_size: u64,
}

/// 存档列表查询参数
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ArchiveListParams {
    /// 可选版本 ID（同 ScreenshotListParams 语义）
    #[serde(default)]
    pub version_id: Option<String>,
}

/// 单个存档条目
#[derive(Debug, Serialize, Deserialize)]
pub struct ArchiveItem {
    /// 存档名称（文件夹名）
    pub name: String,
    /// 完整路径
    pub path: String,
    /// 大小（字节，递归）
    pub size: u64,
    /// 最后修改时间（Unix 秒级时间戳）
    pub modified: u64,
    /// 是否包含 level.dat（有效存档标志）
    pub has_level_dat: bool,
}

/// 存档备份请求参数
#[derive(Debug, Serialize, Deserialize)]
pub struct ArchiveBackupParams {
    /// 存档名称（saves/ 下的文件夹名）
    pub world_name: String,
    /// 输出 zip 完整路径
    pub output_path: String,
    /// 是否排除玩家数据（true=导出分享包，false=完整备份）
    pub exclude_player_data: bool,
    /// 可选版本 ID（用于解析版本隔离下的 saves 目录）
    #[serde(default)]
    pub version_id: Option<String>,
}

/// 存档备份结果
#[derive(Debug, Serialize, Deserialize)]
pub struct ArchiveBackupResult {
    pub success: bool,
    pub file_path: String,
    pub file_size: u64,
}

/// 存档恢复请求参数
#[derive(Debug, Serialize, Deserialize)]
pub struct ArchiveRestoreParams {
    /// zip 文件完整路径
    pub zip_path: String,
    /// 恢复后的存档名称（为空则用 zip 文件名）
    pub world_name: String,
    /// 可选版本 ID（用于解析版本隔离下的 saves 目录）
    #[serde(default)]
    pub version_id: Option<String>,
}

/// 存档恢复结果
#[derive(Debug, Serialize, Deserialize)]
pub struct ArchiveRestoreResult {
    pub success: bool,
    pub world_name: String,
    pub message: String,
}

/// 提取存档种子请求参数
///
/// 用于种子地图工具"从存档加载"功能：读取指定存档的 level.dat，
/// 解析其中 WorldGenSettings.seed（1.16+）或 RandomSeed（1.15 及更早），
/// 返回十进制字符串（避免 JS Number 精度丢失，MC 种子是 i64）。
#[derive(Debug, Serialize, Deserialize)]
pub struct ExtractSaveSeedParams {
    /// 存档名称（saves/ 下的文件夹名）
    pub world_name: String,
    /// 可选版本 ID（用于解析版本隔离下的 saves 目录）
    #[serde(default)]
    pub version_id: Option<String>,
}

/// 提取存档种子结果
#[derive(Debug, Serialize, Deserialize)]
pub struct ExtractSaveSeedResult {
    /// 种子（十进制字符串，i64 范围）
    pub seed: String,
    /// 种子来源字段名（WorldGenSettings.seed 或 RandomSeed）
    pub source: String,
}

/// 网络延迟测试请求参数
#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkLatencyTestParams {
    /// 待测 URL 列表
    pub urls: Vec<String>,
}

/// 网络延迟测试结果
#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkLatencyResult {
    pub results: Vec<LatencyItem>,
}

/// 单个 URL 的延迟测试条目
#[derive(Debug, Serialize, Deserialize)]
pub struct LatencyItem {
    pub url: String,
    /// 延迟（毫秒），失败时为 None
    pub latency_ms: Option<u64>,
    /// HTTP 状态码（如 200），失败时为 0
    pub status_code: u16,
    /// 失败原因（成功时为空）
    pub error: String,
}

/// 服务器状态检测请求参数
#[derive(Debug, Serialize, Deserialize)]
pub struct ServerPingParams {
    pub host: String,
    pub port: u16,
}

/// 服务器状态检测结果
#[derive(Debug, Serialize, Deserialize)]
pub struct ServerPingResult {
    /// 服务器 MOTD（纯文本，已从 JSON/section 符号中提取）
    pub motd: String,
    /// 服务器 MOTD 原始文本（保留 § 格式化代码，供前端解析为彩色显示）
    pub motd_raw: String,
    /// 当前在线人数
    pub online: i32,
    /// 最大人数
    pub max: i32,
    /// 服务器版本（如 "1.20.4"）
    pub version: String,
    /// 延迟（毫秒）
    pub latency_ms: u64,
    /// Favicon（base64 data URI），无则为 None
    pub favicon: Option<String>,
    /// 失败原因（成功时为空）
    pub error: String,
}

/// NBT 解析请求参数
#[derive(Debug, Serialize, Deserialize)]
pub struct NbtParseParams {
    /// NBT 文件完整路径
    pub file_path: String,
}

/// NBT 解析结果
#[derive(Debug, Serialize, Deserialize)]
pub struct NbtParseResult {
    /// 根节点
    pub root: NbtNode,
}

/// NBT 树节点
#[derive(Debug, Serialize, Deserialize)]
pub struct NbtNode {
    /// 节点名称
    pub name: String,
    /// 标签类型：compound / list / byte_array / int_array / long_array / string / int / short / long / float / double / byte
    pub tag_type: String,
    /// 值（仅叶子节点有值，compound/list 为 null）
    pub value: Option<serde_json::Value>,
    /// 子节点（仅 compound / list 有）
    pub children: Vec<NbtNode>,
}

// 注：种子地图相关类型已删除——工具迁移至前端 WASM 方案，不再走后端 IPC。
// 前端通过 res:// 协议加载 cubiomes.wasm，在 Worker 中直接调用 cubiomes C 函数。
