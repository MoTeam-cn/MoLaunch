//! 社区资源下载安装命令
//!
//! 由 `utils::community_manager::dispatch` 反序列化参数后调用，lib.rs 仅注册统一入口 `community_manager`。
//! 子模块：types / helpers / concurrent / modpack_stages / modpack / curseforge / modrinth / hmcl / mmc / mcbbs / resource

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
