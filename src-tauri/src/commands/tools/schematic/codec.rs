//! 投影编解码工具：变长整数、位压缩长整型、方块 id 归一化与统计。

use std::collections::HashMap;

use base64::Engine as _;

/// 体积上限（方块数）：会话常驻内存，每方块 2 字节，5000 万方块约 100MB
pub const MAX_VOLUME: u64 = 50_000_000;

/// 解码无符号 LEB128 变长整数序列（Sponge BlockData 编码）
pub fn decode_varints(bytes: &[u8]) -> Result<Vec<u32>, String> {
    let mut out = Vec::new();
    let mut value: u32 = 0;
    let mut shift: u32 = 0;
    for byte in bytes {
        let part = u32::from(*byte);
        value |= (part & 0x7f) << shift;
        if part & 0x80 == 0 {
            out.push(value);
            value = 0;
            shift = 0;
        } else {
            shift += 7;
            if shift > 35 {
                return Err("变长整数长度非法".to_string());
            }
        }
    }
    if shift != 0 {
        return Err("变长整数序列被截断".to_string());
    }
    Ok(out)
}

/// 编码为无符号 LEB128（导出 Sponge BlockData 用）
pub fn encode_varints(values: &[u32]) -> Vec<u8> {
    let mut out = Vec::new();
    for &value in values {
        let mut current = value;
        loop {
            let mut byte = (current & 0x7f) as u8;
            current >>= 7;
            if current != 0 {
                byte |= 0x80;
            }
            out.push(byte);
            if current == 0 {
                break;
            }
        }
    }
    out
}

/// 每项占用位数（Litematica 位压缩格式，最少 2 位）
pub fn bits_per_entry(palette_len: usize) -> usize {
    let mut bits = 2;
    while (1usize << bits) < palette_len.max(2) {
        bits += 1;
    }
    bits
}

/// 解码位压缩长整型数组（每项不跨 long 边界）
pub fn decode_packed_longs(longs: &[i64], bits: usize, count: usize) -> Vec<u32> {
    if bits == 0 || bits > 32 {
        return Vec::new();
    }
    let per_long = 64 / bits;
    let mask: u64 = if bits >= 64 {
        u64::MAX
    } else {
        (1u64 << bits) - 1
    };
    let mut out = Vec::with_capacity(count);
    for &raw in longs {
        let word = raw as u64;
        for slot in 0..per_long {
            if out.len() >= count {
                return out;
            }
            let shift = (slot * bits) as u32;
            out.push(((word >> shift) & mask) as u32);
        }
    }
    out.truncate(count);
    out
}

/// 编码位压缩长整型数组（与解码对称）
pub fn encode_packed_longs(values: &[u32], bits: usize) -> Vec<i64> {
    if bits == 0 || bits > 32 {
        return Vec::new();
    }
    let per_long = 64 / bits;
    let mut out = Vec::new();
    let mut index = 0;
    while index < values.len() {
        let mut word: u64 = 0;
        for slot in 0..per_long {
            if index >= values.len() {
                break;
            }
            word |= u64::from(values[index]) << ((slot * bits) as u32);
            index += 1;
        }
        out.push(word as i64);
    }
    out
}

/// 归一化方块 id：去掉属性、补齐命名空间
pub fn normalize_block_id(raw: &str) -> String {
    let trimmed = raw.trim();
    let base = match trimmed.find('[') {
        Some(pos) => &trimmed[..pos],
        None => trimmed,
    };
    let base = base.trim();
    if base.is_empty() {
        return "minecraft:air".to_string();
    }
    if base.contains(':') {
        base.to_string()
    } else {
        format!("minecraft:{}", base)
    }
}

/// 是否为空气类方块（不计入非空气统计）
pub fn is_air(id: &str) -> bool {
    matches!(
        id,
        "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air" | "minecraft:structure_void"
    )
}

/// 统计方块数量并按数量降序取前 limit 项
pub fn top_counts(blocks: &[u16], palette: &[String], limit: usize) -> Vec<(String, u64)> {
    let mut counter: HashMap<u16, u64> = HashMap::new();
    for &index in blocks {
        *counter.entry(index).or_insert(0) += 1;
    }
    let mut items: Vec<(String, u64)> = counter
        .into_iter()
        .filter_map(|(index, count)| palette.get(index as usize).map(|id| (id.clone(), count)))
        .filter(|(id, _)| !is_air(id))
        .collect();
    items.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    items.truncate(limit);
    items
}

/// 非空气方块数
pub fn count_non_air(blocks: &[u16], palette: &[String]) -> u64 {
    blocks
        .iter()
        .filter(|&&index| {
            palette
                .get(index as usize)
                .map(|id| !is_air(id))
                .unwrap_or(false)
        })
        .count() as u64
}

/// u16 数组 → base64（小端）
pub fn encode_blocks_base64(blocks: &[u16]) -> String {
    let mut bytes = Vec::with_capacity(blocks.len() * 2);
    for value in blocks {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    base64::engine::general_purpose::STANDARD.encode(bytes)
}
