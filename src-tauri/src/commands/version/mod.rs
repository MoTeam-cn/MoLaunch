//! Version management commands

pub mod download;
pub mod folder;
pub mod install;
pub mod launch;
pub mod list;
pub mod loaders;
pub mod manage;
pub mod mods;
pub mod personalization;
pub mod progress;
pub mod script_export;
pub mod types;

// Re-export types
pub use types::{DownloadProgressSnapshot, DownloadStageSnapshot, VersionInfo, VersionListResult};
// Re-export commands (保持 lib.rs 中 commands::version::* 路径兼容)
pub use list::{
    detect_version_type_from_dir, get_version_effective_dir, list_installed_versions,
    list_installed_versions_with_type, list_versions, resolve_isolation_mode, uninstall_version,
    InstalledVersionInfo,
};
pub use manage::{fix_version_files, get_selected_version, rename_version, set_selected_version};
pub use personalization::{
    get_version_personalization, update_version_personalization, VersionPersonalization,
};
pub use script_export::export_launch_script;

/// 校验版本 ID / 实例名，防止路径遍历
pub fn sanitize_version_id(id: &str) -> Result<(), String> {
    if id.is_empty()
        || id.contains('/')
        || id.contains('\\')
        || id.contains("..")
        || id.contains('\0')
        || id.contains(':')
    {
        return Err(format!("Invalid version id: {}", id));
    }
    // 额外用 components 验证只含 Normal 分量
    let path = std::path::Path::new(id);
    for comp in path.components() {
        if !matches!(comp, std::path::Component::Normal(_)) {
            return Err(format!("Invalid version id: {}", id));
        }
    }
    Ok(())
}

/// 校验 MC 版本号（与 version_id 同样规则）
pub fn sanitize_mc_version(v: &str) -> Result<(), String> {
    sanitize_version_id(v)
}
