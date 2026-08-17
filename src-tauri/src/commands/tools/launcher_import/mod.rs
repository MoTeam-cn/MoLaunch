//! 启动器数据导入：探测外部启动器（PCL2/HMCL/MultiMC/CurseForge 等）实例并迁移到 MoLaunch
//!
//! 模块划分：`detect` 探测与枚举、`parse` 实例信息解析、`migrate` 导入执行。

mod curseforge;
mod detect;
mod hmcl;
mod migrate;
mod multimc;
mod parse;
mod pcl;

pub use detect::{list_sources, scan_generic_path};
pub use migrate::run_import;
