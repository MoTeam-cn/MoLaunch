//! fastnbt::Value ↔ NbtNode 树转换（parse 的逆操作供保存使用）

use std::collections::HashMap;

use fastnbt::Value as NbtValue;

use super::super::types::NbtNode;

/// 将 fastnbt::Value 递归转换为 NbtNode（保持前端 IPC 协议）
///
/// 转换规则：
/// - Compound → tag_type="compound"，children=HashMap 转 Vec<NbtNode>
/// - List     → tag_type="list"，children=Vec<Value> 转 Vec<NbtNode>（list 元素 name 为空）
/// - ByteArray → tag_type="byte_array"，value=Vec<u8>（0-255）
/// - 其他叶子 → tag_type=具体类型，value=serde_json 序列化
pub(super) fn convert_nbt(name: &str, value: &NbtValue) -> NbtNode {
    match value {
        NbtValue::Compound(map) => {
            let children = map.iter().map(|(k, v)| convert_nbt(k, v)).collect();
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

/// 将 NbtNode 树转换回 fastnbt::Value（parse 的逆操作，供保存使用）
pub(super) fn node_to_value(node: &NbtNode) -> Result<NbtValue, String> {
    match node.tag_type.as_str() {
        "compound" => {
            let mut map = HashMap::new();
            for child in &node.children {
                map.insert(child.name.clone(), node_to_value(child)?);
            }
            Ok(NbtValue::Compound(map))
        }
        "list" => {
            let mut items = Vec::new();
            for child in &node.children {
                items.push(node_to_value(child)?);
            }
            Ok(NbtValue::List(items))
        }
        "byte" => Ok(NbtValue::Byte(as_i64(node, "byte")? as i8)),
        "short" => Ok(NbtValue::Short(as_i64(node, "short")? as i16)),
        "int" => Ok(NbtValue::Int(as_i64(node, "int")? as i32)),
        "long" => Ok(NbtValue::Long(as_i64(node, "long")?)),
        "float" => Ok(NbtValue::Float(as_f64(node, "float")? as f32)),
        "double" => Ok(NbtValue::Double(as_f64(node, "double")?)),
        "string" => Ok(NbtValue::String(
            node.value
                .as_ref()
                .and_then(|v| v.as_str())
                .ok_or("string 值无效")?
                .to_string(),
        )),
        "byte_array" => {
            let arr = node
                .value
                .as_ref()
                .and_then(|v| v.as_array())
                .ok_or("byte_array 值无效")?;
            let items = arr.iter().map(|v| v.as_i64().unwrap_or(0) as i8).collect();
            Ok(NbtValue::ByteArray(fastnbt::ByteArray::new(items)))
        }
        "int_array" => {
            let arr = node
                .value
                .as_ref()
                .and_then(|v| v.as_array())
                .ok_or("int_array 值无效")?;
            let items = arr.iter().map(|v| v.as_i64().unwrap_or(0) as i32).collect();
            Ok(NbtValue::IntArray(fastnbt::IntArray::new(items)))
        }
        "long_array" => {
            let arr = node
                .value
                .as_ref()
                .and_then(|v| v.as_array())
                .ok_or("long_array 值无效")?;
            let items = arr.iter().map(|v| v.as_i64().unwrap_or(0)).collect();
            Ok(NbtValue::LongArray(fastnbt::LongArray::new(items)))
        }
        other => Err(format!("未知标签类型: {}", other)),
    }
}

fn as_i64(node: &NbtNode, ty: &str) -> Result<i64, String> {
    node.value
        .as_ref()
        .and_then(|v| v.as_i64())
        .ok_or_else(|| format!("{} 值无效", ty))
}

fn as_f64(node: &NbtNode, ty: &str) -> Result<f64, String> {
    node.value
        .as_ref()
        .and_then(|v| v.as_f64())
        .ok_or_else(|| format!("{} 值无效", ty))
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
