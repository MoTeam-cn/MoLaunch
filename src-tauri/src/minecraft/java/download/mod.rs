//! Java 自动下载模块 - 编排入口
//!
//! 从 Mojang Java Runtime 索引下载到 `{APPDATA}\.minecraft\runtime\{component}\`（跨游戏目录共享）
//! 5 阶段流水线：fetching → matching → manifest → downloading → verifying

mod constants;
mod fetch;
mod files;
mod r#match;
mod pipeline;
mod progress;
mod types;
mod verify;

pub use constants::JAVA_DOWNLOAD_PROGRESS_EVENT;
pub use pipeline::download_java_runtime;
pub use progress::JavaDownloadProgress;
pub use types::RuntimeManifest;
