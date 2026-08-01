//! 整合包元数据读取与本地校验（联机大厅阶段 3/4 新增）

use crate::minecraft::version::scan as version_scan;
use crate::state::AppState;
use serde::Serialize;

use super::super::sanitize_version_id;

/// 读取本地整合包元数据（联机大厅阶段 3 新增）
///
/// 从 `versions/{id}/modpack.meta.json` 读取整合包来源元数据，
/// 用于创建联机房间时上报 `modpack` 字段。
///
/// 返回 `Option<ModpackMetaFile>`：
/// - `Some`：文件存在且解析成功，含 source/project_id/file_id/name/... 等字段
/// - `None`：文件不存在（非平台安装的版本，如手动导入或原版）
///
/// 文件存在但解析失败时返回错误（提示用户 modpack.meta.json 可能损坏）。
pub async fn read_local_modpack_meta(
    state: &AppState,
    version_id: String,
) -> Result<Option<crate::minecraft::version::modpack_meta::ModpackMetaFile>, String> {
    use crate::minecraft::version::modpack_meta::ModpackMetaFile;

    sanitize_version_id(&version_id)?;

    let game_dir = crate::state::resolve_game_dir_from_state(state).await;
    let version_dir = game_dir.join("versions").join(&version_id);

    ModpackMetaFile::load(&version_dir)
        .map_err(|e| format!("Failed to read modpack.meta.json: {}", e))
}

/// 校验本地是否已安装指定整合包的检测结果（联机大厅阶段 4 新增）
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckLocalModpackResult {
    /// 是否已安装
    pub installed: bool,
    /// 匹配的 version_id（`installed=false` 时为 None）
    pub version_id: Option<String>,
}

/// 校验本地是否已安装指定整合包（联机大厅阶段 4 新增）
///
/// 扫描所有已安装版本的 `modpack.meta.json`，按以下优先级匹配：
/// 1. `manifest_hash` 优先匹配（双方都有且一致）
/// 2. 回退三元组匹配：`(source, project_id, file_id)`
///
/// 用于加入方加入房间后判断本地是否已装房主要求的整合包。
pub async fn check_local_modpack(
    state: &AppState,
    manifest_hash: Option<String>,
    source: String,
    project_id: String,
    file_id: String,
) -> Result<CheckLocalModpackResult, String> {
    use crate::minecraft::version::modpack_meta::ModpackMetaFile;

    let game_dir = crate::state::resolve_game_dir_from_state(state).await;
    let versions = version_scan::scan_installed_versions(&game_dir);

    for version in &versions {
        let version_dir = game_dir.join("versions").join(&version.id);
        if let Ok(Some(meta)) = ModpackMetaFile::load(&version_dir) {
            // 优先 manifest_hash 匹配
            if let (Some(req_hash), Some(local_hash)) = (&manifest_hash, &meta.manifest_hash) {
                if req_hash == local_hash {
                    return Ok(CheckLocalModpackResult {
                        installed: true,
                        version_id: Some(version.id.clone()),
                    });
                }
            }
            // 回退三元组匹配
            if meta.source == source && meta.project_id == project_id && meta.file_id == file_id {
                return Ok(CheckLocalModpackResult {
                    installed: true,
                    version_id: Some(version.id.clone()),
                });
            }
        }
    }

    Ok(CheckLocalModpackResult {
        installed: false,
        version_id: None,
    })
}
