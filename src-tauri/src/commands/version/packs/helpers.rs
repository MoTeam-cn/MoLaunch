//! Packs 命令共享辅助函数
//! `resolve_packs_dir` 复用 `pack_common::resolve_version_subdir`。

use std::path::PathBuf;

use crate::state::AppState;

use super::types::PackKind;

/// 获取版本的内容目录（resourcepacks / shaderpacks）
pub(crate) async fn resolve_packs_dir(
    state: &AppState,
    version_id: &str,
    kind: PackKind,
) -> Result<PathBuf, String> {
    super::super::pack_common::resolve_version_subdir(state, version_id, kind.subdir()).await
}
