//! 下载块引擎：段表 + 按块多源并行
//!
//! 子模块：`table`（段表与位图）、`block`（单块下载）、`multi`（多源调度）、`probe`（大小探测）。
//! 单一引擎：小文件与探测不到大小的文件由下载层直接走单流，不再有第二套分片实现。

mod block;
mod map_io;
mod multi;
pub mod probe;
mod schedule;
mod table;

pub(crate) use multi::{download_multi, RetryScope, SegmentOutcome};
pub use probe::probe_file_size;
