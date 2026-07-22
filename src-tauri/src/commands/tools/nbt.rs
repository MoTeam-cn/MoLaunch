//! NBT 数据查看（解析玩家/方块/物品 NBT）
//!
//! 手动实现 NBT 解析器（big-endian 命名二进制标签格式）。
//! 不使用 simdnbt —— 该 crate 依赖 nightly（`portable_simd`），无法在 stable 编译。
//!
//! 支持的标签类型（与 Minecraft NBT 规范一致）：
//! `TAG_End=0, Byte=1, Short=2, Int=3, Long=4, Float=5, Double=6, Byte_Array=7,
//!  String=8, List=9, Compound=10, Int_Array=11, Long_Array=12`

use std::io::Read;

use crate::error_util::log_err;
use crate::log_info;
use crate::state::AppState;

use super::types::{NbtNode, NbtParseParams, NbtParseResult};

// ===== NBT 标签类型常量 =====
const TAG_END: u8 = 0;
const TAG_BYTE: u8 = 1;
const TAG_SHORT: u8 = 2;
const TAG_INT: u8 = 3;
const TAG_LONG: u8 = 4;
const TAG_FLOAT: u8 = 5;
const TAG_DOUBLE: u8 = 6;
const TAG_BYTE_ARRAY: u8 = 7;
const TAG_STRING: u8 = 8;
const TAG_LIST: u8 = 9;
const TAG_COMPOUND: u8 = 10;
const TAG_INT_ARRAY: u8 = 11;
const TAG_LONG_ARRAY: u8 = 12;

/// 解析 NBT 文件，返回 NbtNode 树
///
/// 读取 `params.file_path` 指定的 NBT 文件（gzip 压缩或原始），
/// 解析后转换为 `NbtNode` 树返回。
pub async fn parse(
    state: &AppState,
    params: NbtParseParams,
) -> Result<serde_json::Value, String> {
    let _ = state; // 当前未使用 state，保留以符合统一命令签名
    let file_path = params.file_path.clone();
    log_info!("[NBT] 解析文件: {}", file_path);

    let root = tokio::task::spawn_blocking(move || -> Result<NbtNode, String> {
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

        // 3. 解析 NBT 二进制格式
        let mut reader = NbtReader::new(&data);
        let root = parse_root(&mut reader)?;
        root.ok_or_else(|| "NBT 文件无有效数据（根为 TAG_End）".to_string())
    })
    .await
    .map_err(log_err("NBT 解析任务失败"))??;

    log_info!(
        "[NBT] 解析完成: 根节点 \"{}\" ({}), {} 个子节点",
        root.name,
        root.tag_type,
        root.children.len()
    );

    let result = NbtParseResult { root };
    serde_json::to_value(&result).map_err(|e| e.to_string())
}

// ===== NBT 解析器 =====

/// 解析根标签：一个带名称的标签（通常为 TAG_Compound），或空（TAG_End）
fn parse_root(r: &mut NbtReader) -> Result<Option<NbtNode>, String> {
    let tag_type = r.read_u8()?;
    if tag_type == TAG_END {
        return Ok(None);
    }
    let name = r.read_string()?;
    let mut node = parse_payload(r, tag_type)?;
    node.name = name;
    Ok(Some(node))
}

/// 解析一个标签的 payload（不含类型字节与名称），返回 name 为空的 NbtNode
fn parse_payload(r: &mut NbtReader, tag_type: u8) -> Result<NbtNode, String> {
    match tag_type {
        TAG_BYTE => {
            let v = r.read_i8()?;
            Ok(leaf("byte", to_value_or_null(v)))
        }
        TAG_SHORT => {
            let v = r.read_i16_be()?;
            Ok(leaf("short", to_value_or_null(v)))
        }
        TAG_INT => {
            let v = r.read_i32_be()?;
            Ok(leaf("int", to_value_or_null(v)))
        }
        TAG_LONG => {
            let v = r.read_i64_be()?;
            Ok(leaf("long", to_value_or_null(v)))
        }
        TAG_FLOAT => {
            let v = r.read_f32_be()?;
            Ok(leaf("float", to_value_or_null(v)))
        }
        TAG_DOUBLE => {
            let v = r.read_f64_be()?;
            Ok(leaf("double", to_value_or_null(v)))
        }
        TAG_BYTE_ARRAY => {
            let len = read_array_len(r)?;
            let bytes = r.read_bytes(len)?.to_vec();
            Ok(leaf("byte_array", to_value_or_null(&bytes)))
        }
        TAG_STRING => {
            let s = r.read_string()?;
            Ok(leaf("string", serde_json::Value::String(s)))
        }
        TAG_LIST => {
            let elem_type = r.read_u8()?;
            let len = read_array_len(r)?;
            let mut children = Vec::with_capacity(len);
            for _ in 0..len {
                children.push(parse_payload(r, elem_type)?);
            }
            Ok(NbtNode {
                name: String::new(),
                tag_type: "list".to_string(),
                value: None,
                children,
            })
        }
        TAG_COMPOUND => {
            let mut children = Vec::new();
            loop {
                let child_type = r.read_u8()?;
                if child_type == TAG_END {
                    break;
                }
                let child_name = r.read_string()?;
                let mut node = parse_payload(r, child_type)?;
                node.name = child_name;
                children.push(node);
            }
            Ok(NbtNode {
                name: String::new(),
                tag_type: "compound".to_string(),
                value: None,
                children,
            })
        }
        TAG_INT_ARRAY => {
            let len = read_array_len(r)?;
            let mut arr = Vec::with_capacity(len);
            for _ in 0..len {
                arr.push(r.read_i32_be()?);
            }
            Ok(leaf("int_array", to_value_or_null(&arr)))
        }
        TAG_LONG_ARRAY => {
            let len = read_array_len(r)?;
            let mut arr = Vec::with_capacity(len);
            for _ in 0..len {
                arr.push(r.read_i64_be()?);
            }
            Ok(leaf("long_array", to_value_or_null(&arr)))
        }
        other => Err(format!("未知 NBT 标签类型: {}", other)),
    }
}

/// 读取数组/列表长度（4 字节大端有符号整数），校验非负
fn read_array_len(r: &mut NbtReader) -> Result<usize, String> {
    let len = r.read_i32_be()?;
    if len < 0 {
        return Err(format!("NBT 长度为负: {}", len));
    }
    Ok(len as usize)
}

/// 构造叶子节点（name 为空，由调用方覆写）
fn leaf(tag_type: &str, value: serde_json::Value) -> NbtNode {
    NbtNode {
        name: String::new(),
        tag_type: tag_type.to_string(),
        value: Some(value),
        children: Vec::new(),
    }
}

/// 序列化为 serde_json::Value，失败（如 NaN/Inf 浮点）时降级为 null
fn to_value_or_null<T: serde::Serialize>(v: T) -> serde_json::Value {
    serde_json::to_value(v).unwrap_or(serde_json::Value::Null)
}

// ===== NBT 字节读取器（大端） =====

struct NbtReader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> NbtReader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    fn read_u8(&mut self) -> Result<u8, String> {
        let b = *self
            .data
            .get(self.pos)
            .ok_or_else(|| "NBT 意外结束（读取 u8）".to_string())?;
        self.pos += 1;
        Ok(b)
    }

    fn read_i8(&mut self) -> Result<i8, String> {
        Ok(self.read_u8()? as i8)
    }

    fn read_i16_be(&mut self) -> Result<i16, String> {
        let b = self.read_fixed(2)?;
        Ok(i16::from_be_bytes([b[0], b[1]]))
    }

    fn read_i32_be(&mut self) -> Result<i32, String> {
        let b = self.read_fixed(4)?;
        Ok(i32::from_be_bytes([b[0], b[1], b[2], b[3]]))
    }

    fn read_i64_be(&mut self) -> Result<i64, String> {
        let b = self.read_fixed(8)?;
        Ok(i64::from_be_bytes([
            b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7],
        ]))
    }

    fn read_f32_be(&mut self) -> Result<f32, String> {
        let b = self.read_fixed(4)?;
        Ok(f32::from_be_bytes([b[0], b[1], b[2], b[3]]))
    }

    fn read_f64_be(&mut self) -> Result<f64, String> {
        let b = self.read_fixed(8)?;
        Ok(f64::from_be_bytes([
            b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7],
        ]))
    }

    /// 读取 NBT 字符串（2 字节大端长度前缀 + UTF-8 字节），用 lossy 转换
    fn read_string(&mut self) -> Result<String, String> {
        let len = self.read_u16_be()? as usize;
        let bytes = self.read_bytes(len)?;
        Ok(String::from_utf8_lossy(bytes).into_owned())
    }

    fn read_u16_be(&mut self) -> Result<u16, String> {
        let b = self.read_fixed(2)?;
        Ok(u16::from_be_bytes([b[0], b[1]]))
    }

    fn read_bytes(&mut self, len: usize) -> Result<&'a [u8], String> {
        let end = self
            .pos
            .checked_add(len)
            .ok_or_else(|| "NBT 长度溢出".to_string())?;
        if end > self.data.len() {
            return Err(format!(
                "NBT 意外结束（读取 {} 字节，剩余 {}）",
                len,
                self.data.len() - self.pos
            ));
        }
        let s = &self.data[self.pos..end];
        self.pos = end;
        Ok(s)
    }

    fn read_fixed(&mut self, n: usize) -> Result<&'a [u8], String> {
        self.read_bytes(n)
    }
}
