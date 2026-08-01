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
