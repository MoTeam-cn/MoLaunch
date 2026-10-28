//! 段表：以块为单位的下载进度真相源
//!
//! 块划分只依赖 (文件大小, 块长)，与源无关——换源、重试、多源并行共享同一张表，
//! 因此「哪个源下哪一块」随时可以重新分配，而不会破坏已完成的字节。
//! 位图可落盘（`.partmap`），用于失败后复用已完成块、以及按「块的来源」定位可疑块。
//! 每块还带一个写入时算出的 SHA256：崩溃/掉电后恢复时先验这段字节确实在盘上，
//! 位图说「完成」但数据其实是空洞的脏状态不会被当成已完成。

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use crate::log_warn;

/// 默认块长
pub(crate) const DEFAULT_SEG_LEN: u64 = 2 * 1024 * 1024;

/// 块数上限：超过就把块长调大，保证状态表内存恒定在几十 KB
pub(crate) const MAX_SEGS: u64 = 65536;

/// 单块失败多少次后隔离（只许官方源再领）
pub(crate) const MAX_SEG_FAILURES: u8 = 3;

/// 位图文件头（含版本，格式变更时直接判为无效而不是读出脏数据）
const MAP_MAGIC: &[u8; 6] = b"MPSEG3";

/// 文件头长度：magic(6) + 文件大小(8) + 块长(8) + 块数(8) + 续传指纹(8)
const MAP_HEADER_LEN: usize = 38;

/// 每块追加的 SHA256 长度
const BLOCK_HASH_LEN: usize = 32;

/// 每块在位图里的字节数：状态(1) + 来源(1) + 写入时的 SHA256(32)
const ENTRY_LEN: usize = 2 + BLOCK_HASH_LEN;

/// 来源未知（未完成或未记录）
const SOURCE_UNKNOWN: u8 = u8::MAX;

/// 块状态
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Status {
    Pending,
    InFlight,
    Done,
    Quarantined,
}

struct Seg {
    status: Status,
    failures: u8,
    /// 这一块是谁下下来的（源下标），用于内容校验失败时定位可疑块
    source: u8,
    /// 写入时算出的 SHA256；全零表示未知（旧位图或尚未写入）
    hash: [u8; BLOCK_HASH_LEN],
}

/// 一个待下载的块（闭区间）
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Segment {
    pub index: usize,
    pub start: u64,
    pub end: u64,
}

impl Segment {
    pub(crate) fn len(&self) -> u64 {
        self.end - self.start + 1
    }
}

/// 按文件大小推导块长
pub(crate) fn seg_len_for(file_size: u64) -> u64 {
    let needed = file_size.div_ceil(MAX_SEGS);
    DEFAULT_SEG_LEN.max(needed).max(1)
}

/// 按 (文件大小, 块长) 切块，返回每块的 (起点, 终点)
pub(crate) fn segments_for(file_size: u64, seg_len: u64) -> Vec<(u64, u64)> {
    if file_size == 0 || seg_len == 0 {
        return Vec::new();
    }
    let mut ranges = Vec::new();
    let mut start = 0_u64;
    while start < file_size {
        let end = (start + seg_len - 1).min(file_size - 1);
        ranges.push((start, end));
        start = end + 1;
    }
    ranges
}

/// 段表
pub(crate) struct SegmentTable {
    file_size: u64,
    seg_len: u64,
    segs: Mutex<Vec<Seg>>,
}

impl SegmentTable {
    pub(crate) fn new(file_size: u64) -> Self {
        let seg_len = seg_len_for(file_size);
        let count = segments_for(file_size, seg_len).len();
        let segs = (0..count)
            .map(|_| Seg {
                status: Status::Pending,
                failures: 0,
                source: SOURCE_UNKNOWN,
                hash: [0; BLOCK_HASH_LEN],
            })
            .collect();
        Self {
            file_size,
            seg_len,
            segs: Mutex::new(segs),
        }
    }

    pub(crate) fn seg_len(&self) -> u64 {
        self.seg_len
    }

    fn bounds(&self, index: usize) -> Option<(u64, u64)> {
        let start = index as u64 * self.seg_len;
        if start >= self.file_size {
            return None;
        }
        let end = (start + self.seg_len - 1).min(self.file_size - 1);
        Some((start, end))
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Vec<Seg>> {
        self.segs
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// 认领一块
    ///
    /// 从表尾往前找：尾块最难凑齐，尽早派出去能缩短收尾等待。
    /// `allow_quarantined` 只有官方源为真——镜像连续失败过的块不再交给它。
    pub(crate) fn claim(&self, allow_quarantined: bool) -> Option<Segment> {
        let mut segs = self.lock();
        for index in (0..segs.len()).rev() {
            let usable = match segs[index].status {
                Status::Pending => true,
                Status::Quarantined => allow_quarantined,
                _ => false,
            };
            if usable {
                segs[index].status = Status::InFlight;
                drop(segs);
                let (start, end) = self.bounds(index)?;
                return Some(Segment { index, start, end });
            }
        }
        None
    }

    /// 标记完成，记下这一块的来源与写入时的 SHA256
    pub(crate) fn complete(&self, index: usize, source: usize, hash: [u8; BLOCK_HASH_LEN]) {
        let mut segs = self.lock();
        if let Some(seg) = segs.get_mut(index) {
            if seg.status == Status::InFlight || seg.status == Status::Quarantined {
                seg.status = Status::Done;
                seg.source = u8::try_from(source).unwrap_or(SOURCE_UNKNOWN);
                seg.hash = hash;
            }
        }
    }

    /// 标记失败：释放回待领，累计失败次数到阈值后隔离
    pub(crate) fn fail(&self, index: usize) {
        let mut segs = self.lock();
        if let Some(seg) = segs.get_mut(index) {
            if seg.status == Status::Done {
                return;
            }
            seg.failures = seg.failures.saturating_add(1);
            seg.status = if seg.failures >= MAX_SEG_FAILURES {
                Status::Quarantined
            } else {
                Status::Pending
            };
        }
    }

    /// 把「非指定源」下过的块标回待领（内容校验失败时只重下可疑块）
    ///
    /// 返回被作废的块数。文件的既有字节保留，重下时会原地覆盖。
    pub(crate) fn invalidate_others(&self, keep_source: usize) -> usize {
        let keep = u8::try_from(keep_source).unwrap_or(SOURCE_UNKNOWN);
        let mut segs = self.lock();
        let mut invalidated = 0_usize;
        for seg in segs.iter_mut() {
            if seg.status == Status::Done && seg.source != keep {
                seg.status = Status::Pending;
                seg.source = SOURCE_UNKNOWN;
                seg.failures = 0;
                invalidated += 1;
            }
        }
        invalidated
    }

    /// 把所有已完成块标回待领（内容校验失败且无法从来源判断时，全部重下）
    ///
    /// 返回被作废的块数。文件的既有字节保留，重下时原地覆盖。
    pub(crate) fn invalidate_all(&self) -> usize {
        let mut segs = self.lock();
        let mut invalidated = 0_usize;
        for seg in segs.iter_mut() {
            if seg.status == Status::Done {
                seg.status = Status::Pending;
                seg.source = SOURCE_UNKNOWN;
                seg.failures = 0;
                invalidated += 1;
            }
        }
        invalidated
    }

    /// 已完成的字节数
    pub(crate) fn done_bytes(&self) -> u64 {
        let segs = self.lock();
        let mut bytes = 0_u64;
        for (index, seg) in segs.iter().enumerate() {
            if seg.status != Status::Done {
                continue;
            }
            if let Some((start, end)) = self.bounds(index) {
                bytes += end - start + 1;
            }
        }
        bytes
    }

    /// 已完成的块数
    pub(crate) fn done_count(&self) -> usize {
        self.lock()
            .iter()
            .filter(|seg| seg.status == Status::Done)
            .count()
    }

    /// 是否全部完成
    pub(crate) fn is_complete(&self) -> bool {
        self.lock().iter().all(|seg| seg.status == Status::Done)
    }

    /// 还有没有可认领的块（含隔离块，供判断「是否已彻底无望」）
    pub(crate) fn has_claimable(&self) -> bool {
        self.lock()
            .iter()
            .any(|seg| matches!(seg.status, Status::Pending | Status::Quarantined))
    }

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

#[cfg(test)]
#[path = "table_test.rs"]
mod table_test;
