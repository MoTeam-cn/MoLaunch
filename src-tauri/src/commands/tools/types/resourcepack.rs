use serde::{Deserialize, Serialize};

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
