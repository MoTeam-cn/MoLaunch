use serde::{Deserialize, Serialize};

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
