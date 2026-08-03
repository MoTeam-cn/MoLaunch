//! 单文件分片并发下载模块
//! 将大文件拆分为多个 chunk，使用 HTTP Range 请求并发下载，最后合并。
//! 子模块：`probe`（Range 检测/大小探测）、`download`（分片下载）、`merge`（合并）、`api`（调度）

mod api;
pub mod download;
pub mod merge;
pub mod probe;

// 对外保持 `super::chunk::supports_range` 调用路径稳定
pub use probe::supports_range;

pub use api::{download_chunked, ChunkDownloadResult};
