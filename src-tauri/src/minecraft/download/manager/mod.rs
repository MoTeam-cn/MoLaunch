//! Download manager - batch download with progress tracking
//!
//! 子模块：state（进度状态机）/ core（主实现）/ batch（批次调度）/ lease（面板持有）。

mod batch;
mod core;
mod lease;
mod state;

pub use core::DownloadManager;
pub use lease::PanelLease;
