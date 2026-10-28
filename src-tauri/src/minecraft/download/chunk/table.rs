//! 段表：以块为单位的下载进度真相源
//!
//! 块划分只依赖 (文件大小, 块长)，与源无关——换源、重试、多源并行共享同一张表，
//! 因此「哪个源下哪一块」随时可以重新分配，而不会破坏已完成的字节。
//! 位图可落盘（`.partmap`），用于失败后复用已完成块、以及按「块的来源」定位可疑块。
//! 每块还带一个写入时算出的 SHA256：崩溃/掉电后恢复时先验这段字节确实在盘上，
//! 位图说「完成」但数据其实是空洞的脏状态不会被当成已完成。

use std::sync::Mutex;

/// 默认块长
pub(crate) const DEFAULT_SEG_LEN: u64 = 2 * 1024 * 1024;

/// 块数上限：超过就把块长调大，保证状态表内存恒定在几十 KB
pub(crate) const MAX_SEGS: u64 = 65536;

/// 单块失败多少次后隔离（只许官方源再领）
pub(crate) const MAX_SEG_FAILURES: u8 = 3;

/// 每块写入时算出的 SHA256 长度
pub(crate) const BLOCK_HASH_LEN: usize = 32;

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
}

mod table_map;

#[cfg(test)]
pub(crate) use table_map::forget_recent_save;

#[cfg(test)]
#[path = "table_test.rs"]
mod table_test;
