//! 投影编解码工具：变长整数、位压缩长整型、方块 id 归一化与统计。

use std::collections::HashMap;

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

/// 拆出方块名与属性；属性按 key 升序，保证同一状态字符串稳定可比
pub fn split_block_state(raw: &str) -> (String, Vec<(String, String)>) {
    let trimmed = raw.trim();
    let Some(open) = trimmed.find('[') else {
        return (normalize_block_id(trimmed), Vec::new());
    };
    let base = normalize_block_id(&trimmed[..open]);
    let Some(close) = trimmed.rfind(']') else {
        return (base, Vec::new());
    };
    if close <= open {
        return (base, Vec::new());
    }
    let mut props: Vec<(String, String)> = Vec::new();
    for pair in trimmed[open + 1..close].split(',') {
        let pair = pair.trim();
        if pair.is_empty() {
            continue;
        }
        let Some(eq) = pair.find('=') else {
            continue;
        };
        let key = pair[..eq].trim();
        let value = pair[eq + 1..].trim();
        if key.is_empty() || value.is_empty() {
            continue;
        }
        props.push((key.to_string(), value.to_string()));
    }
    props.sort_by(|a, b| a.0.cmp(&b.0));
    props.dedup_by(|a, b| a.0 == b.0);
    (base, props)
}

/// 拼成 `minecraft:x[k=v,...]`；无属性时只返回方块名
pub fn join_block_state(base: &str, props: &[(String, String)]) -> String {
    if props.is_empty() {
        return base.to_string();
    }
    let body: Vec<String> = props
        .iter()
        .map(|(key, value)| format!("{}={}", key, value))
        .collect();
    format!("{}[{}]", base, body.join(","))
}

/// 规范化完整 blockstate 字符串（补齐命名空间、属性排序）
pub fn normalize_block_state(raw: &str) -> String {
    let (base, props) = split_block_state(raw);
    join_block_state(&base, &props)
}

/// 只取方块名（不含属性），用于贴图、配色与空气判断
pub fn block_base_name(raw: &str) -> &str {
    match raw.find('[') {
        Some(pos) => raw[..pos].trim(),
        None => raw.trim(),
    }
}

/// 是否为空气类方块（不计入非空气统计）
pub fn is_air(id: &str) -> bool {
    matches!(
        block_base_name(id),
        "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air" | "minecraft:structure_void"
    )
}

/// 统计方块数量并按数量降序取前 limit 项
pub fn top_counts(blocks: &[u16], palette: &[String], limit: usize) -> Vec<(String, u64)> {
    let mut counter: HashMap<u16, u64> = HashMap::new();
    for &index in blocks {
        *counter.entry(index).or_insert(0) += 1;
    }
    // 按方块名聚合：上下半砖、不同朝向的同种方块在统计里合并成一行
    let mut merged: HashMap<&str, u64> = HashMap::new();
    for (index, count) in counter {
        let Some(id) = palette.get(index as usize) else {
            continue;
        };
        if is_air(id) {
            continue;
        }
        *merged.entry(block_base_name(id)).or_insert(0) += count;
    }
    let mut items: Vec<(String, u64)> = merged
        .into_iter()
        .map(|(name, count)| (name.to_string(), count))
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
