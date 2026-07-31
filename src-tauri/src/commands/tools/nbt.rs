//! NBT 数据查看（解析玩家/方块/物品 NBT）
//! 用 `fastnbt` crate（stable 兼容，serde 设计）解析 NBT 二进制格式，替代早期手动解析器
//! （约 296 行 → 约 130 行），可靠处理嵌套 TAG_List/空 compound/超大数组等边界。
//! gzip 解压由 `flate2` 负责（player .dat / level.dat 通常 gzip 压缩）。

use std::io::Read;

use fastnbt::Value as NbtValue;

use crate::error_util::log_err;
use crate::log_info;
use crate::state::AppState;

use super::types::{NbtNode, NbtParseParams, NbtParseResult};

/// 解析 NBT 文件，返回 NbtNode 树
///
/// 读取 `params.file_path` 指定的 NBT 文件（gzip 压缩或原始），
/// 用 fastnbt 解析后转换为 `NbtNode` 树返回（保持前端 IPC 协议不变）。
pub async fn parse(
    state: &AppState,
    params: NbtParseParams,
) -> Result<serde_json::Value, String> {
    let _ = state; // 当前未使用 state，保留以符合统一命令签名
    let file_path = params.file_path.clone();
    log_info!("[NBT] 解析文件: {}", file_path);

    let (root_name, root_value) = tokio::task::spawn_blocking(move || -> Result<(String, NbtValue), String> {
        // 1. 读取文件字节
        let raw = std::fs::read(&file_path).map_err(log_err("NBT 读取文件失败"))?;
        if raw.is_empty() {
            return Err("NBT 文件为空".to_string());
        }

        // 2. gzip 解压（检测 gzip 魔数 0x1f 0x8b，如 player .dat / level.dat）
        let data: Vec<u8> = if raw.len() >= 2 && raw[0] == 0x1f && raw[1] == 0x8b {
            let mut decoder = flate2::read::GzDecoder::new(&raw[..]);
            let mut out = Vec::new();
            decoder
                .read_to_end(&mut out)
                .map_err(log_err("NBT gzip 解压失败"))?;
            out
        } else {
            raw
        };

        // 3. 根为 TAG_End（空 NBT）→ fastnbt 会报错，提前给出明确提示
        if data.first() == Some(&0u8) {
            return Err("NBT 文件无有效数据（根为 TAG_End）".to_string());
        }

        // 4. 读取根 compound 名称
        // fastnbt::Value 不保留根名称（见 docs.rs/fastnbt Value 文档），
        // 手动从字节流提取：[u8 tag_type][u16 name_len][name bytes][payload...]
        let root_name = read_root_name(&data);

        // 5. fastnbt 解析（serde 风格，处理剩余所有标签，含嵌套 List/Compound）
        let value: NbtValue = fastnbt::from_bytes(&data)
            .map_err(|e| format!("fastnbt 解析失败: {}", e))?;
        Ok((root_name, value))
    })
    .await
    .map_err(log_err("NBT 解析任务失败"))??;

    let root = convert_nbt(&root_name, &root_value);
    log_info!(
        "[NBT] 解析完成: 根节点 \"{}\" ({}), {} 个子节点",
        root.name,
        root.tag_type,
        root.children.len()
    );

    let result = NbtParseResult { root };
    serde_json::to_value(&result).map_err(|e| e.to_string())
}

/// 从 NBT 字节流读取根 compound 名称
///
/// NBT 根格式：`[u8 tag_type][u16 name_len][name bytes][payload...]`
/// 长度不足或解析失败时返回空字符串（不阻塞解析）。
fn read_root_name(data: &[u8]) -> String {
    if data.len() < 3 {
        return String::new();
    }
    let len = u16::from_be_bytes([data[1], data[2]]) as usize;
    if data.len() < 3 + len {
        return String::new();
    }
    String::from_utf8_lossy(&data[3..3 + len]).into_owned()
}

/// 将 fastnbt::Value 递归转换为 NbtNode（保持前端 IPC 协议不变）
///
/// 转换规则：
/// - Compound → tag_type="compound"，children=HashMap 转 Vec<NbtNode>
/// - List     → tag_type="list"，children=Vec<Value> 转 Vec<NbtNode>（list 元素 name 为空）
/// - ByteArray → tag_type="byte_array"，value=Vec<u8>（0-255，与原手动实现一致）
/// - 其他叶子 → tag_type=具体类型，value=serde_json 序列化
fn convert_nbt(name: &str, value: &NbtValue) -> NbtNode {
    match value {
        NbtValue::Compound(map) => {
            let children = map
                .iter()
                .map(|(k, v)| convert_nbt(k, v))
                .collect();
            NbtNode {
                name: name.to_string(),
                tag_type: "compound".to_string(),
                value: None,
                children,
            }
        }
        NbtValue::List(items) => {
            let children = items.iter().map(|v| convert_nbt("", v)).collect();
            NbtNode {
                name: name.to_string(),
                tag_type: "list".to_string(),
                value: None,
                children,
            }
        }
        NbtValue::Byte(v) => leaf(name, "byte", to_value_or_null(v)),
        NbtValue::Short(v) => leaf(name, "short", to_value_or_null(v)),
        NbtValue::Int(v) => leaf(name, "int", to_value_or_null(v)),
        NbtValue::Long(v) => leaf(name, "long", to_value_or_null(v)),
        NbtValue::Float(v) => leaf(name, "float", to_value_or_null(v)),
        NbtValue::Double(v) => leaf(name, "double", to_value_or_null(v)),
        NbtValue::ByteArray(arr) => {
            // 原手动实现读取为 Vec<u8>（0-255），保持 JSON 输出兼容
            let as_u8: Vec<u8> = arr.iter().map(|&b| b as u8).collect();
            leaf(name, "byte_array", to_value_or_null(&as_u8))
        }
        NbtValue::String(s) => leaf(name, "string", serde_json::Value::String(s.clone())),
        NbtValue::IntArray(arr) => {
            let v: Vec<i32> = arr.iter().copied().collect();
            leaf(name, "int_array", to_value_or_null(&v))
        }
        NbtValue::LongArray(arr) => {
            let v: Vec<i64> = arr.iter().copied().collect();
            leaf(name, "long_array", to_value_or_null(&v))
        }
    }
}

/// 构造叶子节点
fn leaf(name: &str, tag_type: &str, value: serde_json::Value) -> NbtNode {
    NbtNode {
        name: name.to_string(),
        tag_type: tag_type.to_string(),
        value: Some(value),
        children: Vec::new(),
    }
}

/// 序列化为 serde_json::Value，失败（如 NaN/Inf 浮点）时降级为 null
fn to_value_or_null<T: serde::Serialize>(v: T) -> serde_json::Value {
    serde_json::to_value(v).unwrap_or(serde_json::Value::Null)
}
