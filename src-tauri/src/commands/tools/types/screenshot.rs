use serde::{Deserialize, Serialize};

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
