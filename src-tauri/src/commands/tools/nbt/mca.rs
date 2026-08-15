//! mca（Anvil 区块容器）解析与保存
//!
//! 格式：8KiB 头部（4KiB 位置表 1024×4B + 4KiB 时间戳表），数据区从扇区 2 起。
//! 位置条目：3 字节扇区偏移（512B/扇区）+ 1 字节扇区数；全 0 表示区块不存在。
//! 区块数据：4B 长度（含压缩类型字节）+ 1B 压缩类型（1=gzip, 2=zlib, 3=none）+ payload。

use fastnbt::SerOpts;

use crate::error_util::log_err;

use super::super::types::{NbtChunkInfo, NbtNode};
use super::compress::{gunzip_if_needed, zlib_compress, zlib_decompress};
use super::convert::{convert_nbt, node_to_value};

/// 解析 mca 文件，返回所有有效区块的 NBT 树
pub(super) fn parse_mca(raw: &[u8]) -> Result<Vec<NbtChunkInfo>, String> {
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
pub(super) fn save_mca_chunk(
    file_path: &str,
    chunk_index: usize,
    root: &NbtNode,
) -> Result<(), String> {
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
