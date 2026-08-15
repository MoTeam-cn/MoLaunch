use serde::{Deserialize, Serialize};

/// NBT 解析请求参数
#[derive(Debug, Serialize, Deserialize)]
pub struct NbtParseParams {
    /// NBT 文件完整路径
    pub file_path: String,
}

/// NBT 解析结果
#[derive(Debug, Serialize, Deserialize)]
pub struct NbtParseResult {
    /// 根节点（普通 NBT 文件为文件根；mca 文件为空 compound）
    pub root: NbtNode,
    /// 文件类型：nbt（普通 NBT）/ mca（Anvil 区块容器）
    pub file_type: String,
    /// mca 文件的区块列表（普通 NBT 文件为空）
    pub chunks: Vec<NbtChunkInfo>,
}

/// mca 文件中的单个区块
#[derive(Debug, Serialize, Deserialize)]
pub struct NbtChunkInfo {
    /// 区块索引（0-1023，x + z * 32）
    pub index: usize,
    /// 区块在区域内的 x（0-31）
    pub x: i32,
    /// 区块在区域内的 z（0-31）
    pub z: i32,
    /// 区块 NBT 树
    pub root: NbtNode,
}

/// NBT 保存请求参数
#[derive(Debug, Serialize, Deserialize)]
pub struct NbtSaveParams {
    /// 目标文件完整路径
    pub file_path: String,
    /// 编辑后的树（普通 NBT 文件为文件根；mca 为区块树）
    pub root: NbtNode,
    /// mca 文件专用：待写回的区块索引
    #[serde(default)]
    pub chunk_index: Option<usize>,
}

/// NBT 保存结果
#[derive(Debug, Serialize, Deserialize)]
pub struct NbtSaveResult {
    pub success: bool,
}

/// 存档内 NBT 文件列表请求参数
#[derive(Debug, Serialize, Deserialize)]
pub struct NbtListSaveFilesParams {
    /// 存档名称（saves/ 下的文件夹名）
    pub world_name: String,
    /// 可选版本 ID（版本隔离目录）
    #[serde(default)]
    pub version_id: Option<String>,
}

/// 存档内 NBT 文件条目
#[derive(Debug, Serialize, Deserialize)]
pub struct NbtSaveFileItem {
    /// 相对存档目录的路径（如 level.dat / playerdata/xxx.dat / region/r.0.0.mca）
    pub rel_path: String,
    /// 文件名
    pub name: String,
    /// 文件大小（字节）
    pub size: u64,
    /// 类别：level / player / region / other
    pub kind: String,
    /// 完整绝对路径
    pub path: String,
}

/// 存档内 NBT 文件列表结果
#[derive(Debug, Serialize, Deserialize)]
pub struct NbtListSaveFilesResult {
    pub items: Vec<NbtSaveFileItem>,
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
