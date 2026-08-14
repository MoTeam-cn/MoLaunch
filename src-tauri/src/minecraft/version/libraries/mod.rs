//! Libraries dependency resolution module

mod download;
mod filter;
mod parse;
mod types;

// Re-export 公共 API（与原 libraries.rs 完全向后兼容）
pub use download::build_download_urls;
pub use filter::find_missing_libs;
pub use parse::{check_rules, check_rules_with_features, is_native_matching_arch, parse_libraries};
pub use types::{maven_to_path, LibEntry};
