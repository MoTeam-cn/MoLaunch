//! class UTF8 条目改写（规格 §4）
//!
//! 只重建目标条目的字节区间（tag + 长度 + 内容），其余字节原样拷贝；
//! 未命中时返回原始字节，替换文本超长时报错。

use super::pool::read_pool;

/// 把常量池里所有等于 `text` 的 UTF8 条目替换为 `replacement`
pub fn replace_class_utf8(bytes: &[u8], text: &str, replacement: &str) -> Result<Vec<u8>, String> {
    if replacement == text {
        return Ok(bytes.to_vec());
    }
    let encoded = replacement.as_bytes();
    if encoded.len() > u16::MAX as usize {
        return Err("replacement text does not fit a class UTF-8 constant".to_string());
    }

    let pool = read_pool(bytes)?;
    let targets: Vec<(usize, usize)> = pool
        .utf8()
        .iter()
        .filter(|entry| entry.text == text)
        .map(|entry| entry.span)
        .collect();
    if targets.is_empty() {
        return Ok(bytes.to_vec());
    }

    let removed: usize = targets.iter().map(|(start, end)| end - start).sum();
    let mut patched = Vec::with_capacity(bytes.len() - removed + encoded.len() * targets.len());
    let mut cursor = 0usize;
    for (start, end) in targets {
        patched.extend_from_slice(&bytes[cursor..start]);
        patched.push(super::pool::tag::UTF8);
        patched.extend_from_slice(&(encoded.len() as u16).to_be_bytes());
        patched.extend_from_slice(encoded);
        cursor = end;
    }
    patched.extend_from_slice(&bytes[cursor..]);
    Ok(patched)
}
