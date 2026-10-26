//! 投影（schematic）相关 IPC 类型。

use serde::{Deserialize, Serialize};

/// 打开投影文件
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchematicOpenParams {
    pub path: String,
}

/// 导出投影文件
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchematicExportParams {
    /// 已打开的源文件路径（导出时重新读取，避免回传整份方块数据）
    pub source_path: String,
    /// 目标文件路径
    pub target_path: String,
    /// 目标格式：sponge_v2 / litematic / structure
    pub format: String,
}

/// 单个方块的统计条目
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SchematicBlockCount {
    pub id: String,
    pub count: u64,
}

/// 打开结果：调色板 + 方块索引（base64 的 u16 小端数组）
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SchematicOpenResult {
    /// sponge_v1 / sponge_v2 / litematic / structure
    pub format: String,
    pub name: String,
    /// [x, y, z]
    pub size: [u32; 3],
    /// 索引 -> 方块 id
    pub palette: Vec<String>,
    /// 长度 = x*y*z 的 u16 数组，按 x 最快、其次 z、最后 y 排列
    pub blocks_base64: String,
    pub total: u64,
    pub non_air: u64,
    /// 按数量降序的前若干方块
    pub counts: Vec<SchematicBlockCount>,
}

/// 导出结果
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SchematicExportResult {
    pub path: String,
    pub bytes: u64,
}
