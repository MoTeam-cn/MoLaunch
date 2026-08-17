//! 版本管理模块（模块入口）：loaders 加载器检测 + version_extract 原版版本号提取
//! 扫描逻辑（VersionInfo / 扫描 / 继承链 / 卸载）在 scanner.rs。

mod loaders;
mod scanner;
mod version_extract;

// Re-export 公共 API（供 commands 层复用）
pub(crate) use loaders::detect_loaders;
pub(crate) use version_extract::extract_original_version;

pub use scanner::{get_version_chain, scan_installed_versions, uninstall_version, VersionInfo};
