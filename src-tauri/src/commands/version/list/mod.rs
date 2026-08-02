//! 版本列表、类型检测、隔离解析
//!
//! 注：原 6 个独立 Tauri 命令已聚合为 `version_list_manager` IPC 入口，
//! 通过请求体的 `action` 字段分发。本模块函数已去掉 `#[tauri::command]` 标注，
//! 由 `list_manager::dispatch` 反序列化参数后调用。

mod detect;
mod info;
mod installed;
mod modpack;
mod remote;

pub use detect::{detect_version_type_from_dir, resolve_isolation_mode};
pub use info::{get_version_effective_dir, get_version_game_version, get_version_loader_info};
pub use installed::{
    list_installed_versions, list_installed_versions_with_type, uninstall_version,
    InstalledVersionInfo,
};
pub use modpack::{check_local_modpack, read_local_modpack_meta, CheckLocalModpackResult};
pub use remote::list_versions;

use crate::minecraft::version::state::VersionType;

/// Convert VersionType to string
pub(super) fn version_type_to_string(version_type: &VersionType) -> String {
    match version_type {
        VersionType::Release => "release".to_string(),
        VersionType::Snapshot => "snapshot".to_string(),
        VersionType::Old => "old".to_string(),
        VersionType::Fool => "fool".to_string(),
        VersionType::Forge => "forge".to_string(),
        VersionType::NeoForge => "neoforge".to_string(),
        VersionType::Fabric => "fabric".to_string(),
        VersionType::Quilt => "quilt".to_string(),
        VersionType::OptiFine => "optifine".to_string(),
        VersionType::LiteLoader => "liteloader".to_string(),
        VersionType::Unknown => "unknown".to_string(),
    }
}
