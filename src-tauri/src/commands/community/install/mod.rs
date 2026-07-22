//! 社区资源下载安装命令
//!
//! 下载资源文件到指定版本目录
//!
//! 模块结构：
//! - types.rs: DownloadRequest / DownloadResult / CommunityDownloadProgress /
//!   InstallModpackRequest / ModpackFormat / InstallModpackResult / ModpackInfo 数据类型
//! - helpers.rs: apply_filename_format / resolve_install_dir /
//!   parse_cf_loader_id / parse_mr_loader / extract_mr_project_id / construct_cf_edge_url 纯函数
//! - concurrent.rs: download_files_concurrent / extract_overrides / detect_modpack_format
//! - curseforge.rs: CF 整合包 manifest 数据结构 + install_cf_mods
//! - modrinth.rs: MR 整合包 index 数据结构 + install_mr_files
//! - modpack_stages.rs: install_modpack 阶段辅助（download_modpack_archive + parse_modpack_info）
//! - resource.rs: 资源文件下载命令（download_resource / download_resource_to_path /
//!   install_resource / format_download_filename / get_resource_install_path）
//! - modpack.rs: 整合包安装命令（install_modpack）
//! - mod.rs: 模块入口 + pub mod 声明 + 类型 re-export
//!
//! 注意：所有 #[tauri::command] 命令分散在 resource.rs 和 modpack.rs 中，
//! tauri::command 宏在定义处生成 __cmd__ 符号，不能通过 pub use 重导出，
//! 故 lib.rs 使用完整路径注册（commands::community::install::resource::* / ::modpack::*）

pub mod concurrent;
pub mod curseforge;
pub mod helpers;
pub mod modpack;
pub mod modpack_stages;
pub mod modrinth;
pub mod resource;
mod types;

// 对外暴露数据类型（保持向后兼容）
pub use types::{
    CommunityDownloadProgress, DownloadRequest, DownloadResult, InstallModpackRequest,
    InstallModpackResult, ModpackFormat,
};
