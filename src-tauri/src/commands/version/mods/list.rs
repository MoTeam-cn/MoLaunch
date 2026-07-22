//! Mod 列表查询命令（list_mods / is_version_modable）

use crate::minecraft::version::setup::VersionSetup;
use crate::minecraft::version::state::VersionType;
use crate::state::AppState;
use crate::{log_error, log_info};
use tauri::State;

use super::super::sanitize_version_id;
use super::helpers::get_mods_dir;
use super::types::ModInfo;

/// 判断版本是否可以安装 Mod
///
/// 规则：版本含 Forge/Fabric/NeoForge/LiteLoader，或个性化分类被强制为 "可安装Mod"（display_type=2）
#[tauri::command]
pub async fn is_version_modable(
    state: State<'_, AppState>,
    version_id: String,
) -> Result<bool, String> {
    sanitize_version_id(&version_id)?;

    let game_dir = crate::state::resolve_game_dir_from_state(&state).await;

    let version_dir = game_dir.join("versions").join(&version_id);

    // 1. 检查个性化设置中是否强制为 "可安装Mod"
    let setup = VersionSetup::load_or_create(&version_dir, &version_id);
    if let Some(dt) = setup.display.display_type {
        if dt == 2 {
            return Ok(true);
        }
    }

    // 2. 从 version JSON 检测加载器
    let json_path = version_dir.join(format!("{}.json", version_id));
    if json_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&json_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                let vtype = VersionType::detect_from_json(&version_id, &json);
                if vtype.is_modded() {
                    return Ok(true);
                }
            }
        }
    }

    // 3. 从版本 ID 推断
    let id_lower = version_id.to_lowercase();
    if id_lower.contains("forge")
        || id_lower.contains("neoforge")
        || id_lower.contains("fabric")
        || id_lower.contains("quilt")
        || id_lower.contains("liteloader")
        || id_lower.contains("optifine")
    {
        return Ok(true);
    }

    Ok(false)
}

/// 列出版本的 Mod（扫描 mods 目录）
///
/// **两阶段加载设计**：
/// - 本函数是同步阶段，**只做文件枚举**，不读 JAR 内容，保证瞬间返回
/// - 元数据（译名、描述、版本、logo、slug）全部返回空，由 `preload_mods_detail_cmd` 后台异步补全
/// - 排序规则：只按 `file_name`（含扩展名）字母序升序，**禁用状态不参与排序**
///   （按 `ModList.OrderBy(Function(m) m.File.Name)` 的方式）
#[tauri::command]
pub async fn list_mods(
    state: State<'_, AppState>,
    version_id: String,
) -> Result<Vec<ModInfo>, String> {
    sanitize_version_id(&version_id)?;
    log_info!("Listing mods for version: {}", version_id);

    let mods_dir = get_mods_dir(&state, &version_id).await?;

    if !mods_dir.exists() {
        return Ok(Vec::new());
    }

    let mut mods = Vec::new();
    let entries = std::fs::read_dir(&mods_dir).map_err(|e| {
        log_error!("Failed to read mods dir: {}", e);
        e.to_string()
    })?;

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let file_name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };
        let lower = file_name.to_lowercase();

        // 只处理 jar/litemod 及其禁用变体
        let is_mod = lower.ends_with(".jar")
            || lower.ends_with(".litemod")
            || lower.ends_with(".jar.disabled")
            || lower.ends_with(".jar.old")
            || lower.ends_with(".litemod.disabled")
            || lower.ends_with(".litemod.old");
        if !is_mod {
            continue;
        }

        let is_enabled = !(lower.ends_with(".disabled") || lower.ends_with(".old"));
        let enabled_name = file_name
            .trim_end_matches(".disabled")
            .trim_end_matches(".old")
            .to_string();

        let size = entry.metadata().map(|m| m.len()).unwrap_or(0);

        let loader_type = infer_loader_type(&file_name);

        // 同步阶段不读 JAR 内容！元数据字段全部返回空，由 preload 阶段异步补全
        mods.push(ModInfo {
            file_name,
            enabled_name,
            is_enabled,
            size,
            loader_type,
            translated_name: String::new(),
            description: String::new(),
            version: String::new(),
            slug: String::new(),
        });
    }

    // 只按 file_name（含扩展名）字母序升序，禁用状态不参与排序
    // （ModList.OrderBy(Function(m) m.File.Name)）
    mods.sort_by(|a, b| a.file_name.to_lowercase().cmp(&b.file_name.to_lowercase()));

    log_info!("Found {} mods for version {}", mods.len(), version_id);
    Ok(mods)
}

/// 根据文件名推断 Mod 加载器类型（简化版，仅基于文件名特征）
fn infer_loader_type(file_name: &str) -> String {
    let lower = file_name.to_lowercase();
    if lower.contains("fabric") {
        "fabric".to_string()
    } else if lower.contains("neoforge") {
        "neoforge".to_string()
    } else if lower.contains("forge") {
        "forge".to_string()
    } else if lower.contains("liteloader") {
        "liteloader".to_string()
    } else if lower.contains("quilt") {
        "quilt".to_string()
    } else if lower.contains("optifine") {
        "optifine".to_string()
    } else {
        "unknown".to_string()
    }
}
