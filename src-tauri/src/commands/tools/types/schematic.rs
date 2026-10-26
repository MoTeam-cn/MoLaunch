//! 投影（schematic）相关 IPC 类型。

use serde::{Deserialize, Serialize};

/// 打开投影文件
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchematicOpenParams {
    pub path: String,
}

/// 打开结果：返回会话 ID，方块数据按需分片获取
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SchematicOpenResult {
    /// 会话 ID，后续切片/编辑/导出都用它引用
    pub session_id: String,
    /// sponge_v1 / sponge_v2 / litematic / structure / mcedit_legacy
    pub format: String,
    pub name: String,
    /// [x, y, z]
    pub size: [u32; 3],
    /// 索引 -> 方块 id
    pub palette: Vec<String>,
    pub total: u64,
    pub non_air: u64,
    /// 按数量降序的前若干方块
    pub counts: Vec<SchematicBlockCount>,
}

/// 单个方块的统计条目
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SchematicBlockCount {
    pub id: String,
    pub count: u64,
}

/// 取切片：按轴 + 层 + 视口窗口取一小块
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchematicSliceParams {
    pub session_id: String,
    /// y / x / z
    pub axis: String,
    pub layer: u32,
    /// 视口在切片平面内的起点
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

/// 切片结果
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SchematicSliceResult {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    /// u16 小端数组的 base64
    pub cells_base64: String,
}

/// 编辑：在指定层的矩形区域内填充方块
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchematicEditParams {
    pub session_id: String,
    pub axis: String,
    pub layer: u32,
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
    /// 目标方块在调色板中的索引
    pub block: u16,
    /// fill（无条件填充）/ replace（仅替换指定来源方块）
    pub mode: String,
    /// replace 模式下的来源方块索引
    pub from: Option<u16>,
}

/// 编辑结果
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SchematicEditResult {
    pub changed: u32,
    pub non_air: u64,
    pub can_undo: bool,
    pub can_redo: bool,
}

/// 撤销 / 重做
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchematicHistoryParams {
    pub session_id: String,
    /// undo / redo
    pub action: String,
}

/// 撤销 / 重做结果
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SchematicHistoryResult {
    pub changed: u32,
    pub non_air: u64,
    pub can_undo: bool,
    pub can_redo: bool,
}

/// 变换（旋转 / 镜像）
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchematicTransformParams {
    pub session_id: String,
    /// rotate_cw / rotate_ccw / mirror_x / mirror_z / flip_y
    pub kind: String,
}

/// 变换结果
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SchematicTransformResult {
    pub size: [u32; 3],
    pub non_air: u64,
}

/// 导出投影
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchematicExportParams {
    pub session_id: String,
    pub target_path: String,
    /// sponge_v2 / litematic / structure
    pub format: String,
}

/// 导出结果
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SchematicExportResult {
    pub path: String,
    pub bytes: u64,
}

/// 关闭会话
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchematicCloseParams {
    pub session_id: String,
}
