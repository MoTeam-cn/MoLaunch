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
use std::io::Read;
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
    /// 中文译名（来自 mcmod 数据库，可能为空）
    /// 由 community_mod_local_name_style 控制在 UI 中的显示方式：
    ///   0 = 标题显示译名，详情显示文件名
    ///   1 = 标题显示文件名，详情显示译名
    pub translated_name: String,
    /// Mod 描述（来自 jar 内 metadata，可能为空）
    /// fabric.mod.json 的 description / mods.toml 的 description / mcmod.info 的 description
    #[serde(default)]
    pub description: String,
    /// Mod 版本号（来自 jar 内 metadata，可能为空）
    /// fabric.mod.json 的 version / mods.toml 的 version / mcmod.info 的 version
    #[serde(default)]
    pub version: String,
    /// Mod 图标（base64 data URL，从 jar 内 logo 文件提取，可能为 None）
    /// 前端可直接用作 <img src> 加载
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logo_data: Option<String>,
    /// Mod slug（来自 jar 内 metadata：fabric.mod.json 的 id / mods.toml 的 modId / mcmod.info 的 modid）
    /// 用于「详情」按钮关联 CF/MR 平台工程和「前往百科」按钮查 mcmod.cn 直链
    #[serde(default)]
    pub slug: String,
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
///
/// **两阶段加载设计**（参考 PCL2 `LocalResourceLoaders.vb` 第 5-102 行）：
/// - 本函数是同步阶段，**只做文件枚举**，不读 JAR 内容，保证瞬间返回
/// - 元数据（译名、描述、版本、logo、slug）全部返回空，由 `preload_mods_detail_cmd` 后台异步补全
/// - 排序规则与 PCL2 一致：只按 `file_name`（含扩展名）字母序升序，**禁用状态不参与排序**
///   （参考 PCL2 `ModList.OrderBy(Function(m) m.File.Name)`）
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
            logo_data: None,
            slug: String::new(),
        });
    }

    // 只按 file_name（含扩展名）字母序升序，禁用状态不参与排序
    // （参考 PCL2 LocalResourceLoaders.vb 第 88 行：ModList.OrderBy(Function(m) m.File.Name)）
    mods.sort_by(|a, b| a.file_name.to_lowercase().cmp(&b.file_name.to_lowercase()));

    log_info!("Found {} mods for version {}", mods.len(), version_id);
    Ok(mods)
}

/// 启用/禁用 Mod（参考 PCL2 EDMods，重命名文件扩展名）
///
/// 返回重命名后的新文件名（前端据此原地更新 mod 字段，避免重新加载列表丢失预加载的 project 等信息）。
#[tauri::command]
pub async fn toggle_mod(
    state: State<'_, AppState>,
    version_id: String,
    file_name: String,
    enable: bool,
) -> Result<String, String> {
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
        return Ok(file_name);
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
    Ok(new_name)
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
    log_info!("Opening mods dir: {}", path_str);
    crate::minecraft::system::shell::open_path(&path_str)
}

/// 获取版本的 mods 目录路径（不打开，用于前端下载 mod 时指定保存位置）
#[tauri::command]
pub async fn get_version_mods_dir(
    state: State<'_, AppState>,
    version_id: String,
) -> Result<String, String> {
    sanitize_version_id(&version_id)?;
    let mods_dir = get_mods_dir(&state, &version_id).await?;
    if !mods_dir.exists() {
        std::fs::create_dir_all(&mods_dir).map_err(|e| e.to_string())?;
    }
    Ok(mods_dir.to_string_lossy().to_string())
}

/// 在资源管理器中打开并选中指定 Mod 文件（参考 PCL2 Open_Click）
#[tauri::command]
pub async fn reveal_mod_file(
    state: State<'_, AppState>,
    version_id: String,
    file_name: String,
) -> Result<(), String> {
    sanitize_version_id(&version_id)?;
    sanitize_file_name(&file_name)?;
    let mods_dir = get_mods_dir(&state, &version_id).await?;
    let mod_path = mods_dir.join(&file_name);
    if !mod_path.exists() {
        return Err(format!("Mod 文件不存在: {}", file_name));
    }
    let path_str = mod_path.to_string_lossy().to_string();
    log_info!("Revealing mod file: {}", path_str);
    crate::minecraft::system::shell::reveal_in_file_manager(&path_str)
}

/// 获取版本的 mods 目录路径（内部辅助函数，pub(crate) 供 preload 命令复用）
pub(crate) async fn get_mods_dir(
    state: &State<'_, AppState>,
    version_id: &str,
) -> Result<std::path::PathBuf, String> {
    let config = state.config.lock().await;
    let game_dir = crate::state::resolve_game_dir(&config.game_dir);
    let global_isolation_mode = config.isolation_mode;
    drop(config);

    // 版本独立隔离设置覆盖全局
    let isolation_mode = crate::commands::version::list::resolve_isolation_mode(
        &game_dir,
        version_id,
        global_isolation_mode,
    );
    let version_type =
        crate::commands::version::list::detect_version_type_from_dir(&game_dir, version_id);
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

/// 从 jar 文件内读取 mod 元数据：译名、描述、版本号、logo data URL、slug
///
/// 读取顺序（参考 PCL2 LocalMod.Read):
/// 1. fabric.mod.json → id / description / version / iconPath（Fabric/Quilt）
/// 2. META-INF/mods.toml → modId / description / version / logoFile（Forge 1.13+/NeoForge）
/// 3. mcmod.info → modid / description / version / logoFile（Forge 1.12-）
///
/// 查到 slug 后用 mcmod 数据库查询译名，查不到返回空字符串
/// logo 从 jar 内提取并编码为 base64 data URL，未找到则返回 None
/// slug 也一并返回，用于前端关联 CF/MR 平台工程和查 mcmod.cn 直链
pub(crate) fn read_mod_metadata(path: &std::path::Path) -> ModMetadata {
    let file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return ModMetadata::default(),
    };
    let mut archive = match zip::ZipArchive::new(file) {
        Ok(a) => a,
        Err(_) => return ModMetadata::default(),
    };

    // 尝试 fabric.mod.json
    if let Some(meta) = read_fabric_mod_meta(&mut archive) {
        return finalize_metadata(meta, &mut archive);
    }
    // 尝试 META-INF/mods.toml（Forge 1.13+/NeoForge）
    if let Some(meta) = read_forge_mods_toml_meta(&mut archive) {
        return finalize_metadata(meta, &mut archive);
    }
    // 尝试 mcmod.info（Forge 1.12-）
    if let Some(meta) = read_mcmod_info_meta(&mut archive) {
        return finalize_metadata(meta, &mut archive);
    }

    ModMetadata::default()
}

/// 把中间结构 ModMeta 转换为最终 ModMetadata（提取 logo + 查译名）
fn finalize_metadata<R: std::io::Read + std::io::Seek>(
    meta: ModMeta,
    archive: &mut zip::ZipArchive<R>,
) -> ModMetadata {
    let slug = meta.slug.clone().unwrap_or_default();
    let translated = meta
        .slug
        .as_deref()
        .and_then(lookup_translated)
        .unwrap_or_default();
    // fabric 用 icon_path，forge/mcmod 用 logo_file
    let logo_path = meta.icon_path.or(meta.logo_file);
    let logo = logo_path
        .as_deref()
        .and_then(|p| extract_logo_data_url(archive, p));
    ModMetadata {
        slug,
        description: meta.description,
        version: meta.version,
        logo_data: logo,
        translated_name: translated,
    }
}

/// jar 内 mod metadata 最终结果（供 preload 模块使用）
#[derive(Debug, Clone, Default, serde::Serialize)]
pub(crate) struct ModMetadata {
    pub slug: String,
    pub description: String,
    pub version: String,
    pub logo_data: Option<String>,
    pub translated_name: String,
}

/// jar 内 mod metadata 中间结构
struct ModMeta {
    slug: Option<String>,
    description: String,
    version: String,
    icon_path: Option<String>,
    logo_file: Option<String>,
}

/// 从 jar 内提取 logo 文件并编码为 base64 data URL
/// 支持 png/jpg/jpeg/gif，根据扩展名推断 MIME
/// logo 路径可能是绝对路径（jar 内）或相对路径（fabric.mod.json 的 iconPath 通常相对于 jar 根）
fn extract_logo_data_url<R: std::io::Read + std::io::Seek>(
    archive: &mut zip::ZipArchive<R>,
    logo_path: &str,
) -> Option<String> {
    // 清理路径：去除前导 /
    let clean_path = logo_path.trim_start_matches('/');

    // 尝试直接路径
    let mut logo_bytes = None;
    let mut mime = "image/png";

    // 尝试原路径
    if let Ok(mut file) = archive.by_name(clean_path) {
        let mut buf = Vec::new();
        if file.read_to_end(&mut buf).is_ok() && !buf.is_empty() {
            logo_bytes = Some(buf);
            mime = guess_mime(clean_path);
        }
    }

    // 如果原路径失败，尝试常见 logo 路径
    if logo_bytes.is_none() {
        let candidates = [
            clean_path.to_string(),
            format!("assets/{}", clean_path),
            format!("META-INF/{}", clean_path),
            "logo.png".to_string(),
            "icon.png".to_string(),
            "pack.png".to_string(),
        ];
        for path in &candidates {
            if let Ok(mut file) = archive.by_name(path) {
                let mut buf = Vec::new();
                if file.read_to_end(&mut buf).is_ok() && !buf.is_empty() {
                    logo_bytes = Some(buf);
                    mime = guess_mime(path);
                    break;
                }
            }
        }
    }

    let bytes = logo_bytes?;
    // 限制 256KB 防止过大图标
    if bytes.len() > 256 * 1024 {
        return None;
    }

    use base64::{engine::general_purpose, Engine as _};
    let b64 = general_purpose::STANDARD.encode(&bytes);
    Some(format!("data:{};base64,{}", mime, b64))
}

/// 根据文件扩展名猜测 MIME 类型
fn guess_mime(path: &str) -> &'static str {
    let lower = path.to_lowercase();
    if lower.ends_with(".png") {
        "image/png"
    } else if lower.ends_with(".jpg") || lower.ends_with(".jpeg") {
        "image/jpeg"
    } else if lower.ends_with(".gif") {
        "image/gif"
    } else if lower.ends_with(".webp") {
        "image/webp"
    } else {
        "image/png"
    }
}

/// 读取 fabric.mod.json 的 id / description / version / iconPath
fn read_fabric_mod_meta<R: std::io::Read + std::io::Seek>(
    archive: &mut zip::ZipArchive<R>,
) -> Option<ModMeta> {
    let mut file = archive.by_name("fabric.mod.json").ok()?;
    let mut content = String::new();
    file.read_to_string(&mut content).ok()?;
    let json: serde_json::Value = serde_json::from_str(&content).ok()?;

    let slug = json.get("id")?.as_str()?.trim().to_lowercase();
    let slug = if slug.is_empty() { None } else { Some(slug) };

    let description = json
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let version = json
        .get("version")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let icon_path = json
        .get("iconPath")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    Some(ModMeta {
        slug,
        description,
        version,
        icon_path,
        logo_file: None,
    })
}

/// 读取 META-INF/mods.toml 的 modId / description / version / logoFile（Forge 1.13+/NeoForge）
fn read_forge_mods_toml_meta<R: std::io::Read + std::io::Seek>(
    archive: &mut zip::ZipArchive<R>,
) -> Option<ModMeta> {
    // 先把 mods.toml 内容读到 String，drop 掉 ZipFile 借用，避免后续 read_manifest_version 二次借用
    let content = {
        let mut file = archive.by_name("META-INF/mods.toml").ok()?;
        let mut s = String::new();
        file.read_to_string(&mut s).ok()?;
        s
    };

    let mut slug: Option<String> = None;
    let mut description = String::new();
    let mut version = String::new();
    let mut logo_file: Option<String> = None;

    // 简化解析 TOML（避免引入 toml crate）
    // 检查 [[mods]] 块内的字段
    let mut in_mods_block = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("[[") {
            in_mods_block = trimmed == "[[mods]]";
            continue;
        }
        if !in_mods_block {
            continue;
        }
        // 解析 key = "value" 或 key = "value" # comment
        if let Some((key, value)) = parse_toml_kv(trimmed) {
            match key.as_str() {
                "modId" => {
                    if !value.is_empty() {
                        slug = Some(value.to_lowercase());
                    }
                }
                "description" => description = value,
                "version" => version = value,
                "logoFile" => logo_file = Some(value),
                _ => {}
            }
        }
    }

    // mods.toml 中 version 常为 "${file.jarVersion}" 占位符
    // 需从 JAR 内 META-INF/MANIFEST.MF 的 Implementation-Version 解析
    if version.contains("${") {
        if let Some(manifest_ver) = read_manifest_version(archive) {
            version = manifest_ver;
        } else {
            version = String::new();
        }
    }

    Some(ModMeta {
        slug,
        description,
        version,
        icon_path: None,
        logo_file,
    })
}

/// 从 META-INF/MANIFEST.MF 读取 Implementation-Version
/// 用于替换 mods.toml 中的 ${file.jarVersion} 占位符
fn read_manifest_version<R: std::io::Read + std::io::Seek>(
    archive: &mut zip::ZipArchive<R>,
) -> Option<String> {
    let mut file = archive.by_name("META-INF/MANIFEST.MF").ok()?;
    let mut content = String::new();
    file.read_to_string(&mut content).ok()?;

    // MANIFEST.MF 格式：每行 "Key: Value"
    // Implementation-Version 可能跨行续行（前导空格），但简化处理只看单行
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("Implementation-Version:") {
            let v = rest.trim();
            if !v.is_empty() {
                return Some(v.to_string());
            }
        }
    }
    None
}

/// 读取 mcmod.info 的 modid / description / version / logoFile（Forge 1.12-）
fn read_mcmod_info_meta<R: std::io::Read + std::io::Seek>(
    archive: &mut zip::ZipArchive<R>,
) -> Option<ModMeta> {
    let mut file = archive.by_name("mcmod.info").ok()?;
    let mut content = String::new();
    file.read_to_string(&mut content).ok()?;
    let json: serde_json::Value = serde_json::from_str(&content).ok()?;
    let arr = json
        .as_array()
        .or_else(|| json.get("modList")?.as_array())?;
    let first = arr.first()?;

    let slug = first
        .get("modid")
        .and_then(|v| v.as_str())
        .map(|s| s.trim().to_lowercase());
    let slug = slug.filter(|s| !s.is_empty());

    let description = first
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let version = first
        .get("version")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let logo_file = first
        .get("logoFile")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    Some(ModMeta {
        slug,
        description,
        version,
        icon_path: None,
        logo_file,
    })
}

/// 简化解析 TOML 单行 key = "value"（去除注释）
fn parse_toml_kv(line: &str) -> Option<(String, String)> {
    // 去除行尾注释（# 不在字符串内时才视为注释）
    let line = line.split('#').next()?.trim();
    let eq_pos = line.find('=')?;
    let key = line[..eq_pos].trim().to_string();
    let value_raw = line[eq_pos + 1..].trim();
    // 去除引号
    let value = value_raw
        .trim_matches('"')
        .trim_matches('\'')
        .to_string();
    if key.is_empty() {
        return None;
    }
    Some((key, value))
}

/// 查询 mcmod 译名（先查 CurseForge slug，再查 Modrinth slug）
fn lookup_translated(slug: &str) -> Option<String> {
    let slug = slug.trim().to_lowercase();
    if let Some(name) = crate::minecraft::community::mcmod::lookup_cf(&slug) {
        return Some(name.to_string());
    }
    if let Some(name) = crate::minecraft::community::mcmod::lookup_mr(&slug) {
        return Some(name.to_string());
    }
    None
}
