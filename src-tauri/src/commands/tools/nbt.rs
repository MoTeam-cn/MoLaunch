//! NBT 数据查看与编辑（level.dat / playerdata / region .mca 等）
//! 用 `fastnbt` crate（serde 设计）解析/序列化 NBT 二进制格式。
//! gzip 解压由 `flate2` 负责（player .dat / level.dat 通常 gzip 压缩）；
//! .mca 为 Anvil 区块容器：8KiB 头部（4KiB 位置表 + 4KiB 时间戳表）+ 各区块 zlib/gzip NBT。

use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::Path;

use fastnbt::SerOpts;
use fastnbt::Value as NbtValue;

use crate::error_util::log_err;
use crate::log_info;
use crate::state::AppState;

use super::types::{
    NbtChunkInfo, NbtListSaveFilesParams, NbtListSaveFilesResult, NbtNode, NbtParseParams,
    NbtParseResult, NbtSaveFileItem, NbtSaveParams, NbtSaveResult,
};

/// 解析 NBT / mca 文件，返回 NbtNode 树（或 mca 的区块列表）
///
/// 读取 `params.file_path` 指定的文件：普通 NBT（gzip 或原始）解析为树；
/// .mca 按 Anvil 容器解析，返回全部有效区块的 NBT 树。
pub async fn parse(state: &AppState, params: NbtParseParams) -> Result<serde_json::Value, String> {
    let _ = state; // 当前未使用 state，保留以符合统一命令签名
    let file_path = params.file_path.clone();
    log_info!("[NBT] 解析文件: {}", file_path);

    let (root_name, root_value, file_type, chunks) = tokio::task::spawn_blocking(
        move || -> Result<(String, Option<NbtValue>, String, Vec<NbtChunkInfo>), String> {
            let raw = std::fs::read(&file_path).map_err(log_err("NBT 读取文件失败"))?;
            if raw.is_empty() {
                return Err("NBT 文件为空".to_string());
            }
            // mca 容器：解析全部区块
            if file_path.to_lowercase().ends_with(".mca") {
                let chunks = parse_mca(&raw)?;
                return Ok((String::new(), None, "mca".to_string(), chunks));
            }
            // 普通 NBT：gzip 解压 + fastnbt 解析
            let data = gunzip_if_needed(&raw)?;
            if data.first() == Some(&0u8) {
                return Err("NBT 文件无有效数据（根为 TAG_End）".to_string());
            }
            let root_name = read_root_name(&data);
            let value: NbtValue =
                fastnbt::from_bytes(&data).map_err(|e| format!("fastnbt 解析失败: {}", e))?;
            Ok((root_name, Some(value), "nbt".to_string(), Vec::new()))
        },
    )
    .await
    .map_err(log_err("NBT 解析任务失败"))??;

    let root = match root_value {
        Some(v) => convert_nbt(&root_name, &v),
        None => NbtNode {
            name: String::new(),
            tag_type: "compound".to_string(),
            value: None,
            children: Vec::new(),
        },
    };

    log_info!(
        "[NBT] 解析完成: 类型 {}, {} 个节点 / {} 个区块",
        file_type,
        root.children.len(),
        chunks.len()
    );

    let result = NbtParseResult {
        root,
        file_type,
        chunks,
    };
    serde_json::to_value(&result).map_err(|e| e.to_string())
}

/// 保存 NBT / mca 文件
///
/// 普通 NBT 文件：NbtNode 树序列化后写回（原 gzip 则保持 gzip）。
/// mca 文件：整体重打包（保留其他区块原字节与时间戳表），写回指定区块。
pub async fn save(state: &AppState, params: NbtSaveParams) -> Result<serde_json::Value, String> {
    let _ = state;
    let file_path = params.file_path;
    let fp = file_path.clone();
    let root = params.root;
    let chunk_index = params.chunk_index;
    log_info!("[NBT] 保存文件: {} (chunk: {:?})", file_path, chunk_index);

    tokio::task::spawn_blocking(move || -> Result<(), String> {
        if fp.to_lowercase().ends_with(".mca") {
            let idx = chunk_index.ok_or("mca 文件保存必须指定区块索引")?;
            save_mca_chunk(&fp, idx, &root)
        } else {
            save_nbt_file(&fp, &root)
        }
    })
    .await
    .map_err(log_err("NBT 保存任务失败"))??;

    log_info!("[NBT] 保存成功: {}", file_path);
    let result = NbtSaveResult { success: true };
    serde_json::to_value(&result).map_err(|e| e.to_string())
}

/// 列出存档目录内的 NBT 文件（level.dat / playerdata / region .mca 等）
pub async fn list_save_files(
    state: &AppState,
    params: NbtListSaveFilesParams,
) -> Result<serde_json::Value, String> {
    if params.world_name.is_empty()
        || !crate::utils::path::is_safe_relative_path(&params.world_name)
        || params.world_name.contains('/')
        || params.world_name.contains('\\')
    {
        return Err("非法存档名称".to_string());
    }

    let saves_dir = super::archive::resolve_saves_dir(state, params.version_id.as_deref()).await;
    let world_dir = saves_dir.join(&params.world_name);
    if !world_dir.is_dir() {
        return Err(format!("存档目录不存在: {}", world_dir.display()));
    }

    log_info!("[NBT] 扫描存档目录: {}", world_dir.display());

    let world_dir_clone = world_dir.clone();
    let items = tokio::task::spawn_blocking(move || -> Vec<NbtSaveFileItem> {
        let mut items = Vec::new();
        collect_save_files(&world_dir_clone, "", &mut items);
        items.sort_by(|a, b| a.rel_path.cmp(&b.rel_path));
        items
    })
    .await
    .map_err(log_err("NBT 存档文件列表任务失败"))?;

    log_info!(
        "[NBT] 存档 {} 内 NBT 文件 {} 个",
        params.world_name,
        items.len()
    );
    let result = NbtListSaveFilesResult { items };
    serde_json::to_value(&result).map_err(|e| e.to_string())
}

// ==================== 普通 NBT 保存 ====================

/// 保存普通 NBT 文件（保持原 gzip 状态，原子写）
fn save_nbt_file(file_path: &str, root: &NbtNode) -> Result<(), String> {
    if root.tag_type != "compound" {
        return Err("NBT 根节点必须为 compound".to_string());
    }
    let raw = std::fs::read(file_path).map_err(log_err("NBT 读取文件失败"))?;
    let gzipped = raw.len() >= 2 && raw[0] == 0x1f && raw[1] == 0x8b;

    let value = node_to_value(root)?;
    let bytes = fastnbt::to_bytes_with_opts(&value, SerOpts::new().root_name(&root.name))
        .map_err(|e| format!("NBT 序列化失败: {}", e))?;
    let out: Vec<u8> = if gzipped {
        gzip_compress(&bytes)?
    } else {
        bytes
    };
    atomic_write(Path::new(file_path), &out)
}

// ==================== mca（Anvil 容器） ====================

/// 解析 mca 文件，返回所有有效区块的 NBT 树
///
/// 格式：8KiB 头部（4KiB 位置表 1024×4B + 4KiB 时间戳表），数据区从扇区 2 起。
/// 位置条目：3 字节扇区偏移（512B/扇区）+ 1 字节扇区数；全 0 表示区块不存在。
/// 区块数据：4B 长度（含压缩类型字节）+ 1B 压缩类型（1=gzip, 2=zlib, 3=none）+ payload。
fn parse_mca(raw: &[u8]) -> Result<Vec<NbtChunkInfo>, String> {
    if raw.len() < 8192 {
        return Err("mca 文件过小（不足 8KiB 头部）".to_string());
    }
    let mut chunks = Vec::new();
    for index in 0..1024 {
        let entry = index * 4;
        let sector_offset = ((raw[entry] as usize) << 16
            | (raw[entry + 1] as usize) << 8
            | raw[entry + 2] as usize)
            * 512;
        if sector_offset == 0 || sector_offset + 5 > raw.len() {
            continue;
        }
        let length = u32::from_be_bytes([
            raw[sector_offset],
            raw[sector_offset + 1],
            raw[sector_offset + 2],
            raw[sector_offset + 3],
        ]) as usize;
        if length < 1 || sector_offset + 4 + length > raw.len() {
            continue;
        }
        let compression = raw[sector_offset + 4];
        let payload = &raw[sector_offset + 5..sector_offset + 4 + length];
        let data = match compression {
            1 => gunzip_if_needed(payload)?,
            2 => zlib_decompress(payload)?,
            3 => payload.to_vec(),
            _ => continue,
        };
        let value = match fastnbt::from_bytes(&data) {
            Ok(v) => v,
            Err(_) => continue,
        };
        chunks.push(NbtChunkInfo {
            index,
            x: (index % 32) as i32,
            z: (index / 32) as i32,
            root: convert_nbt("", &value),
        });
    }
    if chunks.is_empty() {
        return Err("mca 文件中没有可解析的区块".to_string());
    }
    Ok(chunks)
}

/// 保存 mca 文件中的单个区块（整体重打包，保留其他区块原字节与时间戳表）
fn save_mca_chunk(file_path: &str, chunk_index: usize, root: &NbtNode) -> Result<(), String> {
    if chunk_index >= 1024 {
        return Err("区块索引超出范围（0-1023）".to_string());
    }
    let raw = std::fs::read(file_path).map_err(log_err("mca 读取文件失败"))?;
    if raw.len() < 8192 {
        return Err("mca 文件过小（不足 8KiB 头部）".to_string());
    }

    // 新区块数据：zlib 压缩
    let value = node_to_value(root)?;
    let nbt_bytes = fastnbt::to_bytes_with_opts(&value, SerOpts::new().root_name(&root.name))
        .map_err(|e| format!("NBT 序列化失败: {}", e))?;
    let compressed = zlib_compress(&nbt_bytes)?;
    let chunk_len = compressed.len() + 1;
    if chunk_len + 4 > 255 * 512 {
        return Err("区块数据过大，无法写入 mca（超过 255 扇区）".to_string());
    }
    let mut chunk_data = Vec::with_capacity(chunk_len + 4);
    chunk_data.extend_from_slice(&(chunk_len as u32).to_be_bytes());
    chunk_data.push(2); // zlib
    chunk_data.extend_from_slice(&compressed);

    // 收集全部已有区块数据（编辑的区块用新数据替换）
    let mut chunks: Vec<(usize, Vec<u8>)> = Vec::new();
    let mut replaced = false;
    for index in 0..1024 {
        let entry = index * 4;
        let sector_offset = ((raw[entry] as usize) << 16
            | (raw[entry + 1] as usize) << 8
            | raw[entry + 2] as usize)
            * 512;
        if sector_offset == 0 || sector_offset + 4 > raw.len() {
            continue;
        }
        if index == chunk_index {
            chunks.push((index, chunk_data.clone()));
            replaced = true;
            continue;
        }
        let length = u32::from_be_bytes([
            raw[sector_offset],
            raw[sector_offset + 1],
            raw[sector_offset + 2],
            raw[sector_offset + 3],
        ]) as usize;
        if sector_offset + 4 + length > raw.len() {
            continue;
        }
        chunks.push((
            index,
            raw[sector_offset..sector_offset + 4 + length].to_vec(),
        ));
    }
    if !replaced {
        chunks.push((chunk_index, chunk_data));
    }

    // 重建：新位置表 + 保留时间戳表 + 按索引顺序写区块（扇区对齐）
    let mut out = vec![0u8; 8192];
    out[4096..8192].copy_from_slice(&raw[4096..8192]);
    let mut cursor = 2usize; // 数据区从扇区 2（8KiB 之后）开始
    chunks.sort_by_key(|(i, _)| *i);
    for (index, data) in chunks {
        let sector_count = data.len().div_ceil(512);
        let entry = index * 4;
        out[entry] = (cursor >> 16) as u8;
        out[entry + 1] = (cursor >> 8) as u8;
        out[entry + 2] = cursor as u8;
        out[entry + 3] = sector_count as u8;
        out.extend_from_slice(&data);
        out.extend(std::iter::repeat_n(0u8, sector_count * 512 - data.len()));
        cursor += sector_count;
    }
    std::fs::write(file_path, &out).map_err(log_err("mca 写入文件失败"))?;
    Ok(())
}

// ==================== 存档文件扫描 ====================

/// 递归收集存档内 NBT 文件（仅递归 playerdata / region 目录，避免噪音）
fn collect_save_files(dir: &Path, rel: &str, out: &mut Vec<NbtSaveFileItem>) {
    let read = match std::fs::read_dir(dir) {
        Ok(r) => r,
        Err(_) => return,
    };
    for entry in read.flatten() {
        let path = entry.path();
        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };
        let rel_path = if rel.is_empty() {
            name.clone()
        } else {
            format!("{}/{}", rel, name)
        };
        if path.is_file() {
            let lower = name.to_lowercase();
            let parent_name = path
                .parent()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .unwrap_or("");
            let kind = if lower == "level.dat" {
                "level"
            } else if parent_name == "playerdata" && lower.ends_with(".dat") {
                "player"
            } else if lower.ends_with(".mca") {
                "region"
            } else if lower.ends_with(".dat") || lower.ends_with(".nbt") {
                "other"
            } else {
                continue;
            };
            let size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
            out.push(NbtSaveFileItem {
                rel_path,
                name,
                size,
                kind: kind.to_string(),
                path: path.to_str().unwrap_or("").to_string(),
            });
        } else if path.is_dir() && (name == "playerdata" || name == "region") {
            collect_save_files(&path, &rel_path, out);
        }
    }
}

// ==================== 树 ↔ Value 转换 ====================

/// 将 fastnbt::Value 递归转换为 NbtNode（保持前端 IPC 协议）
///
/// 转换规则：
/// - Compound → tag_type="compound"，children=HashMap 转 Vec<NbtNode>
/// - List     → tag_type="list"，children=Vec<Value> 转 Vec<NbtNode>（list 元素 name 为空）
/// - ByteArray → tag_type="byte_array"，value=Vec<u8>（0-255）
/// - 其他叶子 → tag_type=具体类型，value=serde_json 序列化
fn convert_nbt(name: &str, value: &NbtValue) -> NbtNode {
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
fn node_to_value(node: &NbtNode) -> Result<NbtValue, String> {
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

// ==================== 通用辅助 ====================

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

/// gzip 解压（检测 gzip 魔数 0x1f 0x8b，如 player .dat / level.dat）
fn gunzip_if_needed(raw: &[u8]) -> Result<Vec<u8>, String> {
    if raw.len() >= 2 && raw[0] == 0x1f && raw[1] == 0x8b {
        let mut decoder = flate2::read::GzDecoder::new(raw);
        let mut out = Vec::new();
        decoder
            .read_to_end(&mut out)
            .map_err(log_err("NBT gzip 解压失败"))?;
        Ok(out)
    } else {
        Ok(raw.to_vec())
    }
}

fn gzip_compress(data: &[u8]) -> Result<Vec<u8>, String> {
    let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    encoder
        .write_all(data)
        .map_err(log_err("NBT gzip 压缩失败"))?;
    encoder.finish().map_err(log_err("NBT gzip 压缩失败"))
}

fn zlib_compress(data: &[u8]) -> Result<Vec<u8>, String> {
    let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
    encoder
        .write_all(data)
        .map_err(log_err("NBT zlib 压缩失败"))?;
    encoder.finish().map_err(log_err("NBT zlib 压缩失败"))
}

fn zlib_decompress(data: &[u8]) -> Result<Vec<u8>, String> {
    let mut decoder = flate2::read::ZlibDecoder::new(data);
    let mut out = Vec::new();
    decoder
        .read_to_end(&mut out)
        .map_err(log_err("NBT zlib 解压失败"))?;
    Ok(out)
}

/// 原子写文件：先写同目录临时文件再替换目标（避免写一半损坏原文件）
fn atomic_write(path: &Path, data: &[u8]) -> Result<(), String> {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("bin");
    let tmp = path.with_extension(format!("{}.tmp", ext));
    std::fs::write(&tmp, data).map_err(log_err("NBT 写入临时文件失败"))?;
    if path.exists() {
        std::fs::remove_file(path).map_err(log_err("NBT 删除原文件失败"))?;
    }
    std::fs::rename(&tmp, path).map_err(log_err("NBT 替换文件失败"))?;
    Ok(())
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

#[cfg(test)]
#[path = "nbt_test.rs"]
mod nbt_test;
