//! class UTF8 条目改写：只重建被替换条目的字节区间

use super::pool::parse_pool;

/// 只重建被替换 Utf8 条目的字节区间，其余原样拷贝
pub fn replace_class_utf8(bytes: &[u8], text: &str, replacement: &str) -> Result<Vec<u8>, String> {
    let (entries, _) = parse_pool(bytes)?;
    let encoded = replacement.as_bytes();
    if encoded.len() > 65_535 {
        return Err("replacement class text is too long".to_string());
    }
    let mut out = Vec::with_capacity(bytes.len());
    let mut cursor = 0usize;
    let mut changed = 0usize;
    for (_, entry_text, start, end) in &entries {
        if entry_text != text || replacement == text {
            continue;
        }
        out.extend_from_slice(&bytes[cursor..*start]);
        out.push(1);
        out.extend_from_slice(&(encoded.len() as u16).to_be_bytes());
        out.extend_from_slice(encoded);
        cursor = *end;
        changed += 1;
    }
    if changed == 0 {
        return Ok(bytes.to_vec());
    }
    out.extend_from_slice(&bytes[cursor..]);
    Ok(out)
}
