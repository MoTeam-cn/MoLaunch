//! class UTF8 条目改写：只重建被替换条目的字节区间

use super::pool::read_pool;

/// 把常量池里所有等于 `text` 的 Utf8 条目替换为 `replacement`，其余字节原样拷贝
pub fn replace_class_utf8(bytes: &[u8], text: &str, replacement: &str) -> Result<Vec<u8>, String> {
    let pool = read_pool(bytes)?;
    if replacement == text {
        return Ok(bytes.to_vec());
    }
    let encoded = replacement.as_bytes();
    if encoded.len() > u16::MAX as usize {
        return Err("replacement text does not fit a class UTF-8 constant".to_string());
    }

    let mut patched = Vec::with_capacity(bytes.len() + encoded.len());
    let mut consumed = 0usize;
    let mut replaced = 0usize;
    for entry in pool.utf8() {
        if entry.text != text {
            continue;
        }
        patched.extend_from_slice(&bytes[consumed..entry.span.0]);
        patched.push(1);
        patched.extend_from_slice(&(encoded.len() as u16).to_be_bytes());
        patched.extend_from_slice(encoded);
        consumed = entry.span.1;
        replaced += 1;
    }
    if replaced == 0 {
        return Ok(bytes.to_vec());
    }
    patched.extend_from_slice(&bytes[consumed..]);
    Ok(patched)
}
