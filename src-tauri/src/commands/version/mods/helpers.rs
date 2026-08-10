//! Mod 命令共享辅助函数
//! `get_mods_dir` 获取版本的 mods 目录路径（pub(crate)，供 preload 命令复用）。

use crate::state::AppState;

/// 获取版本的 mods 目录路径（内部辅助函数，pub(crate) 供 preload 命令复用）
pub(crate) async fn get_mods_dir(
    state: &AppState,
    version_id: &str,
) -> Result<std::path::PathBuf, String> {
    super::super::pack_common::resolve_version_subdir(state, version_id, "mods").await
}
