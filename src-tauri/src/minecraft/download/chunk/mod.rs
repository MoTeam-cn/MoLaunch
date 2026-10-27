//! 单文件分片并发下载模块
//! 将大文件拆分为多个 chunk，使用 HTTP Range 请求并发下载，最后合并。
//! 子模块：`probe`（Range 检测/大小探测）、`download`（分片下载）、`merge`（合并）、`api`（调度）

mod api;
pub mod download;
pub mod merge;
pub mod probe;

// 对外暴露探测能力：retry 需要「是否支持 Range + 声明大小」一起拿到，
// 避免为了判断分片可行性多发一次请求
pub use probe::{
    probe_file_size, probe_range, range_support_known, remember_range_support, supports_range,
    RangeProbe,
};

pub use api::{download_chunked, ChunkDownloadResult};
