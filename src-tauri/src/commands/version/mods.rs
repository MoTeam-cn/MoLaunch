//! 版本 Mod 管理命令
//!
//! 参考 PCL2 PageInstanceMod：
//! - list_mods：扫描版本 mods 目录，返回 Mod 列表（含启用/禁用状态）
//! - toggle_mod：启用/禁用 Mod（重命名 .jar ↔ .jar.disabled）
//! - delete_mod：删除 Mod 文件
//! - install_mod：从外部文件复制到 mods 目录
//! - is_version_modable：判断版本是否可安装 Mod（有 Forge/Fabric/NeoForge/LiteLoader 或 DisplayType=API）

use crate::minecraft::version::setup::VersionSetup;
use crate::minecraft::version::state::VersionType;
use crate::state::AppState;
use crate::{log_error, log_info};
use serde::{Deserialize, Serialize};
use tauri::State;

use super::sanitize_version_id;

/// 单个 Mod 信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModInfo {
    /// 文件名（不含路径，含扩展名）
    pub file_name: String,
    /// 启用时的文件名（去除 .disabled / .old 后缀）
    pub enabled_name: String,
    /// 是否启用
    pub is_enabled: bool,
    /// 文件大小（字节）
    pub size: u64,
    /// Mod 加载器类型（forge/fabric/neoforge/liteloader/unknown）
    /// 通过文件名和扩展名推断，简化处理
    pub loader_type: String,
}

/// 判断版本是否可以安装 Mod（参考 PCL2 McInstance.Modable）
///
/// 规则：版本含 Forge/Fabric/NeoForge/LiteLoader，或个性化分类被强制为 "可安装Mod"（display_type=2）
#[tauri::command]
pub async fn is_version_modable(
    state: State<'_, AppState>,
    version_id: String,
) -> Result<bool, String> {
    sanitize_version_id(&version_id)?;

    let config = state.config.lock().await;
    let game_dir = crate::state::resolve_game_dir(&config.game_dir);
    drop(config);

    let version_dir = game_dir.join("versions").join(&version_id);

    // 1. 检查个性化设置中是否强制为 "可安装Mod"
    let setup = VersionSetup::load_or_create(&version_dir, &version_id);
    if let Some(dt) = setup.display_type {
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

/// 列出版本的 Mod（参考 PCL2 LocalResourceLoader 扫描 mods 目录）
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

        mods.push(ModInfo {
            file_name,
            enabled_name,
            is_enabled,
            size,
            loader_type,
        });
    }

    // 启用的排前面，同状态按文件名排序
    mods.sort_by(|a, b| {
        b.is_enabled
            .cmp(&a.is_enabled)
            .then_with(|| a.enabled_name.to_lowercase().cmp(&b.enabled_name.to_lowercase()))
    });

    log_info!("Found {} mods for version {}", mods.len(), version_id);
    Ok(mods)
}

/// 启用/禁用 Mod（参考 PCL2 EDMods，重命名文件扩展名）
#[tauri::command]
pub async fn toggle_mod(
    state: State<'_, AppState>,
    version_id: String,
    file_name: String,
    enable: bool,
) -> Result<(), String> {
    sanitize_version_id(&version_id)?;
    sanitize_file_name(&file_name)?;
    log_info!(
        "Toggling mod {} for version {} (enable={})",
        file_name,
        version_id,
        enable
    );

    let mods_dir = get_mods_dir(&state, &version_id).await?;
    let src_path = mods_dir.join(&file_name);

    if !src_path.exists() {
        return Err(format!("Mod 文件不存在: {}", file_name));
    }

    let lower = file_name.to_lowercase();
    let is_currently_enabled = !(lower.ends_with(".disabled") || lower.ends_with(".old"));

    // 状态已一致，无需操作
    if is_currently_enabled == enable {
        return Ok(());
    }

    // 计算目标文件名
    let new_name = if enable {
        // 启用：去掉 .disabled 或 .old 后缀
        file_name
            .trim_end_matches(".disabled")
            .trim_end_matches(".old")
            .to_string()
    } else {
        // 禁用：优先使用 .disabled，若 .disabled 已存在则用 .old
        let disabled_name = format!("{}.disabled", file_name);
        if !mods_dir.join(&disabled_name).exists() {
            disabled_name
        } else {
            format!("{}.old", file_name)
        }
    };

    let dst_path = mods_dir.join(&new_name);

    // 目标已存在（同名文件冲突）
    if dst_path.exists() && dst_path != src_path {
        return Err(format!("目标文件已存在: {}", new_name));
    }

    std::fs::rename(&src_path, &dst_path).map_err(|e| {
        log_error!("Failed to toggle mod: {}", e);
        e.to_string()
    })?;

    log_info!("Mod renamed: {} -> {}", file_name, new_name);
    Ok(())
}

/// 删除 Mod 文件
#[tauri::command]
pub async fn delete_mod(
    state: State<'_, AppState>,
    version_id: String,
    file_name: String,
) -> Result<(), String> {
    sanitize_version_id(&version_id)?;
    sanitize_file_name(&file_name)?;
    log_info!("Deleting mod {} for version {}", file_name, version_id);

    let mods_dir = get_mods_dir(&state, &version_id).await?;
    let path = mods_dir.join(&file_name);

    if !path.exists() {
        return Err(format!("Mod 文件不存在: {}", file_name));
    }

    std::fs::remove_file(&path).map_err(|e| {
        log_error!("Failed to delete mod: {}", e);
        e.to_string()
    })?;

    log_info!("Mod deleted: {}", file_name);
    Ok(())
}

/// 从外部文件安装 Mod（复制到 mods 目录）
#[tauri::command]
pub async fn install_mod(
    state: State<'_, AppState>,
    version_id: String,
    source_path: String,
) -> Result<(), String> {
    sanitize_version_id(&version_id)?;

    // 安全校验：源路径不能包含 ..
    if source_path.contains("..") {
        return Err("源路径不能包含 ..".to_string());
    }

    log_info!("Installing mod to version {}", version_id);

    let mods_dir = get_mods_dir(&state, &version_id).await?;

    // 确保 mods 目录存在
    if !mods_dir.exists() {
        std::fs::create_dir_all(&mods_dir).map_err(|e| {
            log_error!("Failed to create mods dir: {}", e);
            e.to_string()
        })?;
    }

    let src = std::path::Path::new(&source_path);
    if !src.is_absolute() {
        return Err("源路径必须是绝对路径".to_string());
    }
    if !src.exists() {
        return Err(format!("源文件不存在: {}", source_path));
    }

    // 提取文件名，去除 .disabled / .old 后缀
    let original_name = src
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or("无法获取文件名")?
        .to_string();
    let clean_name = original_name
        .trim_end_matches(".disabled")
        .trim_end_matches(".old")
        .to_string();

    // 校验为 Mod 文件
    let lower = clean_name.to_lowercase();
    if !(lower.ends_with(".jar") || lower.ends_with(".litemod")) {
        return Err("仅支持 .jar 或 .litemod 格式的 Mod 文件".to_string());
    }

    let dst = mods_dir.join(&clean_name);

    // 若目标已存在，跳过（避免覆盖）
    if dst.exists() {
        return Err(format!("Mods 目录已存在同名文件: {}", clean_name));
    }

    log_info!("Installing mod from {} to {}", source_path, dst.display());

    std::fs::copy(src, &dst).map_err(|e| {
        log_error!("Failed to copy mod: {}", e);
        e.to_string()
    })?;

    log_info!("Mod installed: {} -> {}", source_path, clean_name);
    Ok(())
}

/// 打开版本的 mods 目录（自动创建）
#[tauri::command]
pub async fn open_mods_dir(
    state: State<'_, AppState>,
    version_id: String,
) -> Result<(), String> {
    sanitize_version_id(&version_id)?;
    let mods_dir = get_mods_dir(&state, &version_id).await?;

    if !mods_dir.exists() {
        std::fs::create_dir_all(&mods_dir).map_err(|e| e.to_string())?;
    }

    let path_str = mods_dir.to_string_lossy().to_string();
    crate::commands::system::open_path_impl(&path_str)
}

/// 获取版本的 mods 目录路径（内部辅助函数）
async fn get_mods_dir(
    state: &State<'_, AppState>,
    version_id: &str,
) -> Result<std::path::PathBuf, String> {
    let config = state.config.lock().await;
    let game_dir = crate::state::resolve_game_dir(&config.game_dir);
    let global_isolation_mode = config.isolation_mode;
    drop(config);

    // 版本独立隔离设置覆盖全局
    let isolation_mode = crate::commands::version::manage::resolve_isolation_mode(
        &game_dir,
        version_id,
        global_isolation_mode,
    );
    let version_type =
        crate::commands::version::manage::detect_version_type_from_dir(&game_dir, version_id);
    let mode = crate::minecraft::isolation::IsolationMode::from_u32(isolation_mode);
    let effective_dir = crate::minecraft::isolation::get_effective_game_dir(
        &game_dir,
        version_id,
        mode,
        version_type,
    );

    Ok(effective_dir.join("mods"))
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

/// 校验文件名，防止路径遍历
fn sanitize_file_name(name: &str) -> Result<(), String> {
    if name.is_empty()
        || name.contains('/')
        || name.contains('\\')
        || name.contains("..")
        || name.contains('\0')
    {
        return Err(format!("Invalid file name: {}", name));
    }
    Ok(())
}
