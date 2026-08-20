//! class 常量池解析：Utf8 条目与 String 引用提取

use std::collections::HashSet;

/// 解析 class 常量池，返回全部 Utf8 条目 (文本, 起始偏移, 结束偏移)
pub fn parse_class_constant_pool(bytes: &[u8]) -> Result<Vec<(String, usize, usize)>, String> {
    Ok(parse_pool(bytes)?
        .0
        .into_iter()
        .map(|(_, text, start, end)| (text, start, end))
        .collect())
}

/// 被 String 常量引用的 Utf8 文本（运行时可见字符串）
pub fn class_string_constants(bytes: &[u8]) -> Vec<String> {
    let Ok((entries, string_indices)) = parse_pool(bytes) else {
        return Vec::new();
    };
    entries
        .into_iter()
        .filter(|(index, _, _, _)| string_indices.contains(index))
        .map(|(_, text, _, _)| text)
        .collect()
}

/// 常量池解析结果：Utf8 条目(索引,文本,start,end) + String 引用的 Utf8 索引
type PoolEntries = (Vec<(u16, String, usize, usize)>, HashSet<u16>);

/// 解析常量池
pub(super) fn parse_pool(bytes: &[u8]) -> Result<PoolEntries, String> {
    if bytes.len() < 10 || bytes[0..4] != [0xca, 0xfe, 0xba, 0xbe] {
        return Err("not a valid Java class file".to_string());
    }
    let count = u16::from_be_bytes([bytes[8], bytes[9]]);
    let mut entries = Vec::new();
    let mut string_indices = HashSet::new();
    let mut offset = 10usize;
    let mut index = 1u16;
    while index < count {
        if offset >= bytes.len() {
            return Err("class constant pool is truncated".to_string());
        }
        let tag = bytes[offset];
        offset += 1;
        match tag {
            1 => {
                if offset + 2 > bytes.len() {
                    return Err("class UTF-8 constant is truncated".to_string());
                }
                let length = u16::from_be_bytes([bytes[offset], bytes[offset + 1]]) as usize;
                offset += 2;
                if offset + length > bytes.len() {
                    return Err("class UTF-8 constant is truncated".to_string());
                }
                let text = String::from_utf8_lossy(&bytes[offset..offset + length]).into_owned();
                entries.push((index, text, offset - 3, offset + length));
                offset += length;
            }
            3 | 4 => offset += 4,
            5 | 6 => {
                offset += 8;
                index += 1;
            }
            8 => {
                if offset + 2 > bytes.len() {
                    return Err("class String constant is truncated".to_string());
                }
                string_indices.insert(u16::from_be_bytes([bytes[offset], bytes[offset + 1]]));
                offset += 2;
            }
            7 | 16 | 19 | 20 => offset += 2,
            9 | 10 | 11 | 12 | 17 | 18 => offset += 4,
            15 => offset += 3,
            _ => return Err(format!("unsupported class constant pool tag: {tag}")),
        }
        if offset > bytes.len() {
            return Err("class constant pool is truncated".to_string());
        }
        index += 1;
    }
    Ok((entries, string_indices))
}
