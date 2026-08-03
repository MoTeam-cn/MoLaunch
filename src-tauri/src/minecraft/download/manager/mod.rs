//! Download manager - batch download with progress tracking
//!
//! 子模块：state（进度状态机）/ core（主实现）。

mod core;
mod state;

pub use core::DownloadManager;
