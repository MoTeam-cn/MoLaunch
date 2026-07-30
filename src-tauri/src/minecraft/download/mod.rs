//! 下载管理模块 - 完整版本下载流程
//! 子模块：config / session / version_list / full_download / stages / fix / util
//! assets / chunk / downloader / manager / rate_limiter / types

pub mod assets;
pub mod chunk;
pub mod config;
pub mod downloader;
pub mod fix;
pub mod full_download;
pub mod manager;
pub mod rate_limiter;
pub mod session;
pub mod stages;
pub mod types;
pub mod util;
pub mod version_list;

// 公共 API 再导出（保持外部调用路径 `crate::minecraft::download::*` 稳定）
pub use fix::fix_version_files;
pub use full_download::{download_version_full, VersionDownloadResult};
pub use manager::DownloadManager;
pub use session::DownloadSession;
pub use util::fetch_url;
pub use version_list::{
    fetch_version_list, get_latest_versions, get_version_json_url, parse_version_list,
    VersionEntry, VersionListResult,
};
