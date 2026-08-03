//! 社区资源下载安装 - 并发下载与 zip 操作
//!
//! 子模块：detect（格式检测）/ extract（overrides 解压）/ run（并发下载 + 检测结果类型）

mod detect;
mod extract;
mod run;

pub(super) use detect::detect_modpack_format;
pub(super) use extract::{build_overrides_prefixes, extract_overrides};
pub(super) use run::{download_files_concurrent, DetectedModpack};
