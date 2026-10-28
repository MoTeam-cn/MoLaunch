//! 位图（`.partmap`）的落盘与读回：序列化、自校验、本进程信任记录
//!
//! 从 `table.rs` 拆出来的 IO/格式部分：段表本身只管状态，这里管它怎么描述到自己
//! 到磁盘、以及读回来时怎么判断「位图说完成」的数据是否真的在盘上。

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use crate::log_warn;

use super::{seg_len_for, segments_for, SegmentTable, Status, BLOCK_HASH_LEN};

/// 位图文件头（含版本，格式变更时直接判为无效而不是读出脏数据）
const MAP_MAGIC: &[u8; 6] = b"MPSEG3";

/// 文件头长度：magic(6) + 文件大小(8) + 块长(8) + 块数(8) + 续传指纹(8)
const MAP_HEADER_LEN: usize = 38;

/// 每块在位图里的字节数：状态(1) + 来源(1) + 写入时的 SHA256(32)
const ENTRY_LEN: usize = 2 + BLOCK_HASH_LEN;

impl SegmentTable {
    /// 落盘位图（每块 2 字节：状态 + 来源）
    ///
    /// `resume_key` 是「同一次下载」的指纹（任务、来源、期望哈希的摘要）：
    /// 文件大小相同但内容换了的场景（同名模组更新）靠它拒绝复用旧位图。
    pub(crate) fn save(&self, path: &Path, resume_key: u64) -> std::io::Result<()> {
        let segs = self.lock();
        let mut bytes = Vec::with_capacity(MAP_HEADER_LEN + segs.len() * ENTRY_LEN);
        bytes.extend_from_slice(MAP_MAGIC);
        bytes.extend_from_slice(&self.file_size.to_le_bytes());
        bytes.extend_from_slice(&self.seg_len.to_le_bytes());
        bytes.extend_from_slice(&(segs.len() as u64).to_le_bytes());
        bytes.extend_from_slice(&resume_key.to_le_bytes());
        let mut done = 0_u64;
        for seg in segs.iter() {
            let finished = seg.status == Status::Done;
            if finished {
                done += 1;
            }
            bytes.push(u8::from(finished));
            bytes.push(seg.source);
            bytes.extend_from_slice(&seg.hash);
        }
        drop(segs);
        // 记下「刚写过」：同一次尝试里重跑时不必把刚写下的字节再读一遍验证
        recent_saves()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .insert(path.to_path_buf(), (resume_key, done));
        std::fs::write(path, bytes)
    }

    /// 读回位图
    ///
    /// `data_len` 是数据文件当前长度：位图里有但文件里没写到的块一律作废，
    /// 避免「位图说有、文件其实是空洞」这种脏状态被当成已完成。
    pub(crate) fn load(
        path: &Path,
        file_size: u64,
        data_len: u64,
        resume_key: u64,
        data: Option<&Path>,
    ) -> Option<Self> {
        let raw = std::fs::read(path).ok()?;
        if raw.len() < MAP_HEADER_LEN || &raw[0..6] != MAP_MAGIC {
            return None;
        }
        let mut offset = 6;
        let mut take_u64 = || {
            let value = u64::from_le_bytes(raw[offset..offset + 8].try_into().ok()?);
            offset += 8;
            Some(value)
        };
        let stored_size = take_u64()?;
        let stored_seg_len = take_u64()?;
        let stored_count = take_u64()?;
        let stored_key = take_u64()?;
        if stored_size != file_size
            || stored_seg_len != seg_len_for(file_size)
            || stored_key != resume_key
        {
            return None;
        }
        let table = Self::new(file_size);
        let expected_count = segments_for(file_size, stored_seg_len).len() as u64;
        if stored_count != expected_count {
            return None;
        }
        if raw.len() < MAP_HEADER_LEN + (stored_count as usize) * ENTRY_LEN {
            return None;
        }
        let mut done_in_file = 0_u64;
        {
            let mut segs = table.lock();
            for (index, seg) in segs.iter_mut().enumerate() {
                let base = MAP_HEADER_LEN + index * ENTRY_LEN;
                let done = raw[base] == 1;
                let source = raw[base + 1];
                let mut hash = [0_u8; BLOCK_HASH_LEN];
                hash.copy_from_slice(&raw[base + 2..base + ENTRY_LEN]);
                if !done {
                    continue;
                }
                done_in_file += 1;
                // 文件里还没写到的块不算完成
                let end = table.bounds(index).map(|(_, end)| end + 1).unwrap_or(0);
                if end > data_len {
                    continue;
                }
                seg.status = Status::Done;
                seg.source = source;
                seg.hash = hash;
            }
        }
        // 本进程刚写过的位图直接信；否则逐块验证数据确实在盘上（掉电后页缓存可能丢）
        let trusted = recent_saves()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .get(path)
            .is_some_and(|(key, done)| *key == resume_key && *done == done_in_file);
        if !trusted {
            if let Some(data) = data {
                verify_done_blocks(&table, data);
            }
        }
        Some(table)
    }
}

/// 本进程刚写过的位图：(续传指纹, 已完成块数)
fn recent_saves() -> &'static Mutex<HashMap<PathBuf, (u64, u64)>> {
    static SAVES: OnceLock<Mutex<HashMap<PathBuf, (u64, u64)>>> = OnceLock::new();
    SAVES.get_or_init(|| Mutex::new(HashMap::new()))
}

/// 测试用：模拟「进程重启」，让下一次 `load` 走真实的自校验路径
#[cfg(test)]
pub(crate) fn forget_recent_save(path: &Path) {
    recent_saves()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .remove(path);
}

/// 逐块验证「位图说完成」的数据确实在盘上：读回该区间重算 SHA256 比对
fn verify_done_blocks(table: &SegmentTable, data: &Path) {
    let mut dropped = 0_usize;
    let mut kept = 0_usize;
    {
        let mut segs = table.lock();
        for index in 0..segs.len() {
            if segs[index].status != Status::Done {
                continue;
            }
            let Some((start, end)) = table.bounds(index) else {
                continue;
            };
            let len = end - start + 1;
            let verified = crate::minecraft::utils::hash::sha256_range(data, start, len)
                .map(|actual| actual == segs[index].hash)
                .unwrap_or(false);
            if verified {
                kept += 1;
            } else {
                segs[index].status = Status::Pending;
                segs[index].hash = [0; BLOCK_HASH_LEN];
                dropped += 1;
            }
        }
    }
    if dropped > 0 {
        log_warn!(
            "[Segment] 位图自校验丢弃 {} 块（数据未落盘或已损坏），保留 {} 块",
            dropped,
            kept
        );
    }
}
