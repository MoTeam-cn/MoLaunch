//! Java 检测和管理模块

pub mod download;

mod detect;
mod search;
mod select;
mod types;

// 重新导出公共 API，保持 `crate::minecraft::java::*` 路径稳定
pub use detect::{detect_java, detect_java_version};
pub use search::{search_java, search_java_with_paths};
pub use select::select_best_java;
pub use types::JavaRuntime;