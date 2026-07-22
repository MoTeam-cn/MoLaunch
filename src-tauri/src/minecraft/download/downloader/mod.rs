//! 单文件下载逻辑（模块入口）
//!
//! 模块拆分：
//! - `single`: 单文件下载编排（URL 顺序循环 + 重试 + 分片/单流选择 + 校验）
//! - `stream`: 单 URL 流式下载（HTTP 请求 + 限速 + 暂停/取消 + 进度更新）

mod single;
mod stream;

/// 未校验下载流的最大字节数上限，防止被劫持镜像源返回无限流导致磁盘耗尽
pub(crate) const MAX_UNVERIFIED_BYTES: u64 = 2 * 1024 * 1024 * 1024; // 2 GiB

// Re-export 公共 API（与原 downloader.rs 完全向后兼容）
pub use single::download_single;
