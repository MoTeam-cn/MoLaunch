//! 导入执行：版本 JSON 生成 + 游戏数据迁移 + setup.ini 写入
//!
//! 导入目标布局（与 MoLaunch 版本扫描/隔离机制对齐）：
//! `{game_dir}/versions/{name}/` 即"版本目录 = 游戏数据目录"，
//! 写入 `{name}.json` + 游戏数据 + `setup.ini`（强制隔离 indie_type=1），
//! 保证无论全局隔离模式如何，导入实例的数据（saves/mods 等）都能在启动时生效。
//!
//! - 复制模式：完整拷贝数据，与原实例完全独立；
//! - 符号链接模式：`{versions}/{name}` 链接到源数据目录，与源实例共享数据。

use std::path::{Path, PathBuf};

use serde_json::json;

use crate::commands::version::sanitize::sanitize_version_id;
use crate::log_info;
use crate::minecraft::version::setup::VersionSetup;
use crate::minecraft::version::state::VersionType;
use crate::state::{resolve_game_dir_from_state, AppState};

use super::detect::strip_extended_prefix;
use super::parse::{detect_instance_info, find_version_json, normalize_version};
use crate::commands::tools::types::{ImportResultItem, LauncherImportRequest};

/// 启动器元数据文件（复制迁移时跳过，避免污染导入结果）
const META_ITEMS: &[&str] = &[
    "instance.cfg",
    "mmc-pack.json",
    "minecraftinstance.json",
    "instance.json",
    "config.json",
    "hmcl.json",
    "launcher-settings.json",
];

/// 执行单个实例导入
pub async fn run_import(
    state: &AppState,
    req: LauncherImportRequest,
) -> Result<ImportResultItem, String> {
    // 源路径：去除 Windows 长路径 `\\?\` 前缀（前端传入的扫描路径可能带此前缀）
    let source_dir = strip_extended_prefix(&PathBuf::from(&req.source_path));
    if !source_dir.is_dir() {
        return Err(format!("源路径不存在或不是目录: {}", source_dir.display()));
    }

    // 实例名：优先请求值，否则取源目录名
    let source_name = source_dir
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    let name = req
        .instance_name
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or(source_name.as_str())
        .to_string();
    if name.is_empty() {
        return Err("实例名为空".to_string());
    }
    sanitize_version_id(&name)?;

    // 目标版本目录
    let game_dir = resolve_game_dir_from_state(state).await;
    let target_dir = game_dir.join("versions").join(&name);
    if target_dir.exists() {
        return Err(format!("目标版本 {} 已存在，请更换实例名", name));
    }
    std::fs::create_dir_all(game_dir.join("versions"))
        .map_err(|e| format!("创建版本目录失败: {}", e))?;

    // 解析源实例（版本/加载器）
    let info = detect_instance_info(&source_dir);
    let mc_version = info.mc_version.as_deref().ok_or_else(|| {
        format!(
            "无法确定实例 {} 的 Minecraft 版本，请检查实例目录结构",
            name
        )
    })?;

    // 游戏数据目录：优先 `.minecraft` 子目录，否则实例根目录
    let data_dir = if source_dir.join(".minecraft").is_dir() {
        source_dir.join(".minecraft")
    } else {
        source_dir.clone()
    };

    if req.symlink {
        symlink_import(&source_dir, &data_dir, &target_dir, &name, mc_version)?;
    } else {
        copy_import(
            &source_dir,
            &data_dir,
            &target_dir,
            &name,
            mc_version,
            &source_name,
        )?;
    }

    write_setup(&target_dir, &name, &info, mc_version)?;

    log_info!(
        "[LauncherImport] 导入成功: {} -> {}",
        name,
        target_dir.display()
    );
    Ok(ImportResultItem {
        name,
        success: true,
        message: format!(
            "导入成功（{}，{}）",
            mc_version,
            info.loader.as_deref().unwrap_or("原版")
        ),
        mc_version: Some(mc_version.to_string()),
        loader: info.loader.clone(),
    })
}

/// 复制模式：版本 JSON + 游戏数据完整拷贝到目标版本目录
fn copy_import(
    source_dir: &Path,
    data_dir: &Path,
    target_dir: &Path,
    name: &str,
    mc_version: &str,
    source_name: &str,
) -> Result<(), String> {
    std::fs::create_dir_all(target_dir).map_err(|e| format!("创建目标目录失败: {}", e))?;

    // 版本 JSON
    let version_json = build_version_json(source_dir, name, mc_version)?;
    let json_content = serde_json::to_string_pretty(&version_json)
        .map_err(|e| format!("序列化版本 JSON 失败: {}", e))?;
    std::fs::write(target_dir.join(format!("{}.json", name)), json_content)
        .map_err(|e| format!("写入版本 JSON 失败: {}", e))?;

    // client jar（若源目录存在 {源名}.jar，复制为 {name}.jar）
    let source_jar = source_dir.join(format!("{}.jar", source_name));
    if source_jar.is_file() {
        std::fs::copy(&source_jar, target_dir.join(format!("{}.jar", name)))
            .map_err(|e| format!("复制 client jar 失败: {}", e))?;
    }

    // 游戏数据
    copy_tree(data_dir, target_dir, source_name)?;
    Ok(())
}

/// 符号链接模式：`{versions}/{name}` 链接到源数据目录（与源实例共享数据）
fn symlink_import(
    source_dir: &Path,
    data_dir: &Path,
    target_dir: &Path,
    name: &str,
    mc_version: &str,
) -> Result<(), String> {
    // 数据必须能直接作为游戏目录根（数据在 .minecraft 子层时已由调用方上提）
    if data_dir.join(".minecraft").is_dir() {
        return Err(format!(
            "实例数据位于 {} 的 .minecraft 子目录，符号链接模式不适用，请改用复制模式",
            data_dir.display()
        ));
    }

    let version_json = build_version_json(source_dir, name, mc_version)?;
    let json_content = serde_json::to_string_pretty(&version_json)
        .map_err(|e| format!("序列化版本 JSON 失败: {}", e))?;

    // 若目标 JSON 已存在于源数据目录（之前导入过），不覆盖
    let json_path = data_dir.join(format!("{}.json", name));
    if !(json_path.is_file()
        && super::detect::read_text_file(&json_path)
            .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
            .is_some())
    {
        std::fs::write(&json_path, json_content)
            .map_err(|e| format!("写入版本 JSON 失败: {}", e))?;
    }

    create_dir_link(data_dir, target_dir)?;
    Ok(())
}

/// 创建目录链接：Windows 优先 junction（无需管理员权限/开发者模式），失败回退符号链接
fn create_dir_link(target: &Path, link: &Path) -> Result<(), String> {
    if link.exists() {
        return Err(format!("目标路径已存在: {}", link.display()));
    }
    #[cfg(target_os = "windows")]
    {
        match junction::create(target, link) {
            Ok(()) => Ok(()),
            Err(junction_err) => {
                log_info!(
                    "[LauncherImport] junction 创建失败（{}），回退符号链接",
                    junction_err
                );
                std::os::windows::fs::symlink_dir(target, link).map_err(|e| {
                    format!(
                        "创建符号链接失败: {}（junction 失败: {}；Windows 需要开发者模式或管理员权限）",
                        e, junction_err
                    )
                })
            }
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        std::os::unix::fs::symlink(target, link).map_err(|e| format!("创建符号链接失败: {}", e))
    }
}

/// 生成版本 JSON：优先复用源实例版本 JSON（完整/继承式），否则构造继承原版的最小 JSON
fn build_version_json(
    source_dir: &Path,
    name: &str,
    mc_version: &str,
) -> Result<serde_json::Value, String> {
    if let Some(json_path) = find_version_json(source_dir) {
        if let Some(content) = super::detect::read_text_file(&json_path) {
            if let Ok(mut json) = serde_json::from_str::<serde_json::Value>(&content) {
                let has_libraries = json
                    .get("libraries")
                    .and_then(|l| l.as_array())
                    .is_some_and(|a| !a.is_empty());
                let has_inherits = json
                    .get("inheritsFrom")
                    .and_then(|v| v.as_str())
                    .is_some_and(|s| !s.is_empty());
                if json.get("mainClass").is_some() && (has_libraries || has_inherits) {
                    // 完整版本 JSON：保留结构，仅调整 id/type
                    json["id"] = json!(name);
                    json["type"] = json!("release");
                    return Ok(json);
                }
            }
        }
    }

    Ok(json!({
        "id": name,
        "inheritsFrom": normalize_version(mc_version),
        "type": "release",
    }))
}

/// 写入 setup.ini（强制隔离 indie_type=1，保证导入数据在启动时生效）
fn write_setup(
    target_dir: &Path,
    name: &str,
    info: &super::parse::DetectedInfo,
    mc_version: &str,
) -> Result<(), String> {
    let version_type = loader_to_version_type(info.loader.as_deref());
    let (forge, neoforge, fabric, quilt, optifine, liteloader) = match info.loader.as_deref() {
        Some("forge") => (info.loader_version.clone(), None, None, None, None, None),
        Some("neoforge") => (None, info.loader_version.clone(), None, None, None, None),
        Some("fabric") => (None, None, info.loader_version.clone(), None, None, None),
        Some("quilt") => (None, None, None, info.loader_version.clone(), None, None),
        Some("optifine") => (None, None, None, None, info.loader_version.clone(), None),
        Some("liteloader") => (None, None, None, None, None, info.loader_version.clone()),
        _ => (None, None, None, None, None, None),
    };

    let mut setup = VersionSetup::new(
        mc_version,
        version_type,
        forge.as_deref(),
        neoforge.as_deref(),
        fabric.as_deref(),
        quilt.as_deref(),
        optifine.as_deref(),
        liteloader.as_deref(),
    );
    // 强制隔离：版本目录即游戏目录（导入数据位于版本目录根）
    setup.display.indie_type = Some(1);

    setup
        .save(target_dir)
        .map_err(|e| format!("写入 setup.ini 失败: {}", e))?;
    log_info!(
        "[LauncherImport] {} 写入 setup.ini（indie_type=1 强制隔离）",
        name
    );
    Ok(())
}

/// loader 标识 → VersionType
fn loader_to_version_type(loader: Option<&str>) -> VersionType {
    match loader {
        Some("forge") => VersionType::Forge,
        Some("neoforge") => VersionType::NeoForge,
        Some("fabric") => VersionType::Fabric,
        Some("quilt") => VersionType::Quilt,
        Some("optifine") => VersionType::OptiFine,
        Some("liteloader") => VersionType::LiteLoader,
        _ => VersionType::Release,
    }
}

/// 递归复制目录（跳过启动器元数据与源版本元数据文件，已存在且相同则跳过）
fn copy_tree(src: &Path, dst: &Path, source_name: &str) -> Result<(), String> {
    let entries =
        std::fs::read_dir(src).map_err(|e| format!("读取目录失败 {}: {}", src.display(), e))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("读取目录项失败: {}", e))?;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        let target = dst.join(&name);

        // 跳过启动器元数据
        if META_ITEMS.contains(&name.to_lowercase().as_str()) {
            continue;
        }
        // 跳过源实例版本元数据（{源名}.json / {源名}.jar），避免残留
        if name == format!("{}.json", source_name) || name == format!("{}.jar", source_name) {
            continue;
        }

        if path.is_dir() {
            if !target.exists() {
                std::fs::create_dir_all(&target)
                    .map_err(|e| format!("创建目录失败 {}: {}", target.display(), e))?;
            }
            copy_tree(&path, &target, source_name)?;
        } else {
            copy_file_if_needed(&path, &target)?;
        }
    }
    Ok(())
}

/// 复制单个文件（目标存在且 size+mtime 相同则跳过）
fn copy_file_if_needed(src: &Path, dst: &Path) -> Result<(), String> {
    let src_meta = std::fs::metadata(src)
        .map_err(|e| format!("读取文件元数据失败 {}: {}", src.display(), e))?;
    let skip = dst
        .metadata()
        .map(|m| m.len() == src_meta.len() && m.modified().ok() == src_meta.modified().ok())
        .unwrap_or(false);
    if skip {
        return Ok(());
    }
    std::fs::copy(src, dst)
        .map_err(|e| format!("复制文件失败 {} -> {}: {}", src.display(), dst.display(), e))?;
    Ok(())
}
