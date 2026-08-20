//! 加载器健康检测（版本 JSON 库条目 + 磁盘文件校验）

use std::path::{Path, PathBuf};

use crate::minecraft::version::setup::VersionSetup;
use crate::minecraft::version::state::VersionType;
use crate::state::AppState;

use super::super::super::sanitize_version_id;
use super::super::{get_version_game_version, version_type_to_string};

/// 加载器健康检测结果（经 `detect_loader_damage` IPC 返回，键名 camelCase 与前端约定一致）
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoaderHealth {
    pub loader_type: Option<String>,
    pub loader_version: String,
    pub mc_version: String,
    pub healthy: bool,
    pub reason: String,
}

/// 修复进度推送事件名（前端 RepairLoaderProgress 同源常量）
pub const REPAIR_LOADER_PROGRESS_EVENT: &str = "repair-loader-progress";

/// 加载器类型对应的库名匹配模式
pub(crate) fn loader_lib_pattern(loader_type: &VersionType) -> Option<&'static str> {
    match loader_type {
        VersionType::Forge => Some("net.minecraftforge"),
        VersionType::NeoForge => Some("net.neoforged"),
        VersionType::Fabric => Some("net.fabricmc:fabric-loader"),
        VersionType::Quilt => Some("org.quiltmc:quilt-loader"),
        VersionType::OptiFine => Some("optifine:OptiFine"),
        VersionType::LiteLoader => Some("com.mumfrey:liteloader"),
        _ => None,
    }
}

/// 从版本 JSON 中查找第一个匹配的加载器库名
pub(crate) fn find_loader_lib(json: &serde_json::Value, pattern: &str) -> Option<String> {
    json["libraries"].as_array()?.iter().find_map(|lib| {
        let name = lib["name"].as_str()?;
        if name.contains(pattern) {
            Some(name.to_string())
        } else {
            None
        }
    })
}

/// 计算加载器库在磁盘上的路径（优先 downloads.artifact.path，兜底 maven 坐标）
pub(crate) fn json_lib_local_path(
    json: &serde_json::Value,
    name: &str,
    game_dir: &Path,
) -> PathBuf {
    json["libraries"]
        .as_array()
        .and_then(|libs| libs.iter().find(|l| l["name"].as_str() == Some(name)))
        .and_then(|lib| lib["downloads"]["artifact"]["path"].as_str())
        .map(|p| game_dir.join("libraries").join(p))
        .unwrap_or_else(|| crate::minecraft::utils::maven::maven_to_local_path(name, game_dir))
}

/// 检测版本加载器是否损坏
///
/// 判定标准：版本 JSON 中存在加载器库条目，且对应库文件存在且非空。
pub async fn detect_loader_damage(
    state: &AppState,
    version_id: &str,
) -> Result<LoaderHealth, String> {
    sanitize_version_id(version_id)?;
    let game_dir = crate::state::resolve_game_dir_from_state(state).await;
    let version_dir = game_dir.join("versions").join(version_id);

    let content = std::fs::read_to_string(version_dir.join(format!("{}.json", version_id)))
        .map_err(|e| format!("读取版本 JSON 失败: {}", e))?;
    let json: serde_json::Value =
        serde_json::from_str(&content).map_err(|e| format!("解析版本 JSON 失败: {}", e))?;

    let setup = VersionSetup::load_or_create(&version_dir, version_id);
    let loader_type_str = version_type_to_string(&setup.loader.version_type);
    let loader_version = match setup.loader.version_type {
        VersionType::Forge => setup.loader.forge_version.clone().unwrap_or_default(),
        VersionType::NeoForge => setup.loader.neoforge_version.clone().unwrap_or_default(),
        VersionType::Fabric => setup.loader.fabric_version.clone().unwrap_or_default(),
        VersionType::Quilt => setup.loader.quilt_version.clone().unwrap_or_default(),
        VersionType::OptiFine => setup.loader.optifine_version.clone().unwrap_or_default(),
        VersionType::LiteLoader => setup.loader.liteloader_version.clone().unwrap_or_default(),
        _ => String::new(),
    };
    let mc_version = get_version_game_version(state, version_id.to_string())
        .await?
        .unwrap_or_default();

    let Some(pattern) = loader_lib_pattern(&setup.loader.version_type) else {
        return Ok(LoaderHealth {
            loader_type: None,
            loader_version: String::new(),
            mc_version,
            healthy: true,
            reason: "该版本未安装加载器".to_string(),
        });
    };

    let mut healthy = true;
    let mut reason = String::new();
    match find_loader_lib(&json, pattern) {
        None => {
            healthy = false;
            reason = format!("版本 JSON 中缺少 {} 库文件", loader_type_str);
        }
        Some(name) => {
            let path = json_lib_local_path(&json, &name, &game_dir);
            let file_ok = path.exists() && path.metadata().map(|m| m.len() > 0).unwrap_or(false);
            if !file_ok {
                healthy = false;
                reason = format!("{} 库文件缺失或为空: {}", loader_type_str, path.display());
            }
        }
    }

    Ok(LoaderHealth {
        loader_type: Some(loader_type_str),
        loader_version,
        mc_version,
        healthy,
        reason,
    })
}
