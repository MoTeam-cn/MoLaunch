use serde::{Deserialize, Serialize};

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
