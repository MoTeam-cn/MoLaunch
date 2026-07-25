//! 社区资源下载安装命令
//!
//! 下载资源文件到指定版本目录
//!
//! 模块结构：
//! - types.rs: DownloadRequest / DownloadResult / CommunityDownloadProgress /
//!   InstallModpackRequest / ModpackFormat / InstallModpackResult / ModpackInfo 数据类型
//! - helpers.rs: apply_filename_format / resolve_install_dir /
//!   parse_cf_loader_id / parse_mr_loader / extract_mr_project_id / construct_cf_edge_url 纯函数
//! - concurrent.rs: download_files_concurrent / extract_overrides / detect_modpack_format /
//!   build_overrides_prefixes / DetectedModpack
//! - curseforge.rs: CF 整合包 manifest 数据结构 + install_cf_mods
//! - modrinth.rs: MR 整合包 index 数据结构 + install_mr_files
//! - hmcl.rs: HMCL 整合包 modpack.json 数据结构
//! - mmc.rs: MMC 整合包 mmc-pack.json 数据结构
//! - mcbbs.rs: MCBBS 整合包 mcbbs.packmeta/manifest.json 数据结构
//! - modpack_stages.rs: install_modpack 阶段辅助（download_modpack_archive + parse_modpack_info）
//! - resource.rs: 资源文件下载命令（download_resource / download_resource_to_path /
//!   install_resource / format_download_filename / get_resource_install_path）
//! - modpack.rs: 整合包安装命令（install_modpack / install_local_modpack）
//! - mod.rs: 模块入口 + pub mod 声明 + 类型 re-export
//!
//! 注意：resource.rs / modpack.rs 中的子模块函数已去掉 `#[tauri::command]` 标注，
//! 改为接收 `&AppState` / `&AppHandle`，由 `utils::community_manager::dispatch`
//! 反序列化参数后调用。lib.rs 仅注册统一入口 `community_manager`。

pub mod concurrent;
pub mod curseforge;
pub mod helpers;
pub mod hmcl;
pub mod mcbbs;
pub mod mmc;
pub mod modpack;
pub mod modpack_stages;
pub mod modrinth;
pub mod resource;
mod types;

// 对外暴露数据类型（保持向后兼容）
pub use types::{
    CommunityDownloadProgress, DownloadRequest, DownloadResult, InstallLocalModpackRequest,
    InstallModpackRequest, InstallModpackResult, ModpackFormat, ModpackPreview, OptionalModInfo,
};
