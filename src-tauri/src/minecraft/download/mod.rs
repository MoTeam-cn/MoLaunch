//! 下载管理模块 - 完整版本下载流程
//!
//! 模块拆分：
//! - `version_list`: 版本清单获取与解析
//! - `full_download`: 完整版本下载主流程
//! - `stages`: 客户端 JAR / 库文件 / 资源文件等下载阶段
//! - `fix`: 补全版本文件
//! - `util`: URL 构建、带重试抓取等工具函数
//! - `assets` / `chunk` / `downloader` / `manager` / `rate_limiter` / `types`: 已有子模块

pub mod assets;
pub mod chunk;
pub mod downloader;
pub mod fix;
pub mod full_download;
pub mod manager;
pub mod rate_limiter;
pub mod stages;
pub mod types;
pub mod util;
pub mod version_list;

// 公共 API 再导出（保持外部调用路径 `crate::minecraft::download::*` 稳定）
pub use fix::fix_version_files;
pub use full_download::{download_version_full, VersionDownloadResult};
pub use util::fetch_url;
pub use version_list::{
    fetch_version_list, get_latest_versions, get_version_json_url, parse_version_list,
    VersionEntry, VersionListResult,
};
