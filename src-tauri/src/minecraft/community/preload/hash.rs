//! CurseForge MurmurHash2 + Modrinth SHA1 指纹计算
//!
//! 跳过空白字符（0x09/0x0A/0x0D/0x20）后做 MurmurHash2（seed=1，与 CF 官方一致）

use crate::error_util::log_err;
use std::path::Path;

/// 计算 CurseForge 用的 MurmurHash2 指纹
pub fn compute_curseforge_fingerprint(path: &Path) -> Result<u32, String> {
    let bytes = std::fs::read(path).map_err(log_err("Failed to read file for fingerprint"))?;
    let filtered: Vec<u8> = bytes
        .iter()
        .copied()
        .filter(|&b| b != 0x09 && b != 0x0A && b != 0x0D && b != 0x20)
        .collect();
    Ok(murmur_hash2(&filtered, 1))
}

/// Modrinth 用的 SHA1 哈希（hex 字符串）
pub fn compute_modrinth_sha1(path: &Path) -> Result<String, String> {
    let bytes = std::fs::read(path).map_err(log_err("Failed to read file for hash"))?;
    Ok(crate::minecraft::utils::file_checker::compute_sha1_hex(
        &bytes,
    ))
}

/// MurmurHash2 算法
fn murmur_hash2(data: &[u8], seed: u32) -> u32 {
    let m: u32 = 0x5bd1_e995;
    let r: u32 = 24;
    let len = data.len();

    let mut h: u32 = seed ^ (len as u32);

    let mut i = 0;
    while i + 4 <= len {
        let mut k = u32::from_le_bytes([data[i], data[i + 1], data[i + 2], data[i + 3]]);
        k = k.wrapping_mul(m);
        k ^= k >> r;
        k = k.wrapping_mul(m);

        h = h.wrapping_mul(m);
        h ^= k;

        i += 4;
    }

    let remaining = len - i;
    if remaining >= 3 {
        h ^= (data[i + 2] as u32) << 16;
    }
    if remaining >= 2 {
        h ^= (data[i + 1] as u32) << 8;
    }
    if remaining >= 1 {
        h ^= data[i] as u32;
        h = h.wrapping_mul(m);
    }

    h ^= h >> 13;
    h = h.wrapping_mul(m);
    h ^= h >> 15;

    h
}
