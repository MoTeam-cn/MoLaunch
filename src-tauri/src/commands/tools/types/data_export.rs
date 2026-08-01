use serde::{Deserialize, Serialize};

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
