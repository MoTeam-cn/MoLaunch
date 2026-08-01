//! Mod 依赖检测 + 去重扫描
//! `mod_dependency_check` 扫描版本 mods 目录下所有 .jar 读依赖列表，找出依赖 mod_id 不在
//! 已安装列表中的项（排除 minecraft/java/fabricloader/fabric-api 等内置依赖）。
//! `mod_dedup_scan` 按 slug 分组找出多版本 mod 组装成 DuplicateMod 列表。
//! 路径安全：本模块只读取不删除，无需 path safety 检查。

use std::collections::HashMap;
use std::path::Path;

use crate::commands::version::mods::read_mod_metadata;
use crate::error_util::log_err;
use crate::log_info;
use crate::log_warn;
use crate::state::resolve_game_dir;
use crate::state::AppState;

use super::types::{
    DuplicateMod, DuplicateVersion, MissingDep, ModDedupResult, ModDedupScanParams,
    ModDependencyCheckParams, ModDependencyResult,
};

/// 内置依赖白名单：这些 mod_id 视为始终存在，不视为缺失依赖
///
/// - `minecraft`：游戏本体
/// - `java`：Java 运行时
/// - `fabricloader`：Fabric 加载器自身
/// - `fabric-api`：Fabric API（独立安装流程管理）
/// - `quilt_loader`：Quilt 加载器自身
/// - `quilted_fabric_api`：Quilt Fabric API
/// - `forge`：Forge 加载器自身
/// - `neoforge`：NeoForge 加载器自身
const BUILTIN_DEPS: &[&str] = &[
    "minecraft",
    "java",
    "fabricloader",
    "fabric-api",
    "quilt_loader",
    "quilted_fabric_api",
    "forge",
    "neoforge",
];

/// 判断 mod_id 是否为内置依赖（无需安装）
fn is_builtin_dep(id: &str) -> bool {
    let id = id.trim().to_lowercase();
    BUILTIN_DEPS.contains(&id.as_str())
}

/// 判断文件名是否为 mod 文件（.jar / .litemod，含禁用变体）
fn is_mod_file(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.ends_with(".jar")
        || lower.ends_with(".litemod")
        || lower.ends_with(".jar.disabled")
        || lower.ends_with(".jar.old")
        || lower.ends_with(".litemod.disabled")
        || lower.ends_with(".litemod.old")
}

/// 读取 mods 目录下所有 mod 文件路径 + 文件名
///
/// 返回 (file_name, full_path) 列表，跳过非 mod 文件。
fn list_mod_files(mods_dir: &Path) -> Vec<(String, std::path::PathBuf)> {
    let mut out = Vec::new();
    let read = match std::fs::read_dir(mods_dir) {
        Ok(r) => r,
        Err(_) => return out,
    };
    for entry in read.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };
        if !is_mod_file(&name) {
            continue;
        }
        out.push((name, path));
    }
    out
}

/// Mod 依赖检测：找出已安装 mod 缺失的依赖
///
/// 步骤：
/// 1. 拼接 `versions/{version_id}/mods/` 路径
/// 2. 遍历 .jar 文件，调用 `read_mod_metadata` 读取每个 mod 的 slug + dependencies
/// 3. 构建"已安装 mod_id 集合"（slug 非空的 mod）
/// 4. 遍历每个 mod 的 dependencies，不在集合中且非内置依赖的，加入 missing
pub async fn mod_dependency_check(
    state: &AppState,
    params: ModDependencyCheckParams,
) -> Result<serde_json::Value, String> {
    let version_id = params.version_id;
    if version_id.is_empty() {
        return Err("version_id 不能为空".to_string());
    }

    let game_dir = {
        let config = state.config.lock().await;
        resolve_game_dir(&config.game_dir)
    };
    let mods_dir = game_dir.join("versions").join(&version_id).join("mods");

    log_info!(
        "[ModTools] 依赖检测: version_id={}, mods_dir={}",
        version_id,
        mods_dir.display()
    );

    if !mods_dir.exists() {
        log_warn!("[ModTools] mods 目录不存在: {}", mods_dir.display());
        let result = ModDependencyResult {
            missing: Vec::new(),
            conflicts: Vec::new(),
        };
        return serde_json::to_value(&result).map_err(|e| e.to_string());
    }

    // 在 spawn_blocking 中执行同步 IO（zip 读取）
    let mods_dir_clone = mods_dir.clone();
    let (installed_slugs, mod_dep_pairs) = tokio::task::spawn_blocking(move || {
        let files = list_mod_files(&mods_dir_clone);
        let mut slugs: Vec<String> = Vec::with_capacity(files.len());
        let mut dep_pairs: Vec<(String, Vec<String>)> = Vec::with_capacity(files.len());

        for (file_name, path) in files {
            let metadata = read_mod_metadata(&path);
            let slug = metadata.slug.trim().to_lowercase();
            if !slug.is_empty() {
                slugs.push(slug);
            }
            // 记录 (file_name, dependencies) 用于后续缺失检测
            dep_pairs.push((file_name, metadata.dependencies));
        }

        (slugs, dep_pairs)
    })
    .await
    .map_err(log_err("ModTools 依赖检测任务失败"))?;

    // 构建已安装 mod_id 集合
    let installed_set: std::collections::HashSet<&str> =
        installed_slugs.iter().map(|s| s.as_str()).collect();

    // 找出缺失依赖
    let mut missing: Vec<MissingDep> = Vec::new();
    let mut seen_missing: std::collections::HashSet<(String, String)> =
        std::collections::HashSet::new();

    for (file_name, deps) in &mod_dep_pairs {
        for dep_id in deps {
            let dep_lower = dep_id.trim().to_lowercase();
            if dep_lower.is_empty() || is_builtin_dep(&dep_lower) {
                continue;
            }
            if installed_set.contains(dep_lower.as_str()) {
                continue;
            }
            // 去重：同一 (required_by, mod_id) 只报一次
            let key = (file_name.clone(), dep_lower.clone());
            if seen_missing.contains(&key) {
                continue;
            }
            seen_missing.insert(key);
            missing.push(MissingDep {
                required_by: file_name.clone(),
                mod_id: dep_lower,
            });
        }
    }

    log_info!(
        "[ModTools] 依赖检测完成: 共 {} 个 mod, 缺失 {} 个依赖",
        mod_dep_pairs.len(),
        missing.len()
    );

    let result = ModDependencyResult {
        missing,
        conflicts: Vec::new(),
    };
    serde_json::to_value(&result).map_err(|e| e.to_string())
}

/// Mod 去重扫描：找出有多个版本的 mod
///
/// 步骤：
/// 1. 拼接 `versions/{version_id}/mods/` 路径
/// 2. 遍历 .jar 文件，读取每个 mod 的 slug + version
/// 3. 按 slug 分组，找出有多个版本的 mod
/// 4. 组装成 DuplicateMod 列表（slug 为空的 mod 不参与去重）
pub async fn mod_dedup_scan(
    state: &AppState,
    params: ModDedupScanParams,
) -> Result<serde_json::Value, String> {
    let version_id = params.version_id;
    if version_id.is_empty() {
        return Err("version_id 不能为空".to_string());
    }

    let game_dir = {
        let config = state.config.lock().await;
        resolve_game_dir(&config.game_dir)
    };
    let mods_dir = game_dir.join("versions").join(&version_id).join("mods");

    log_info!(
        "[ModTools] 去重扫描: version_id={}, mods_dir={}",
        version_id,
        mods_dir.display()
    );

    if !mods_dir.exists() {
        log_warn!("[ModTools] mods 目录不存在: {}", mods_dir.display());
        let result = ModDedupResult {
            duplicates: Vec::new(),
        };
        return serde_json::to_value(&result).map_err(|e| e.to_string());
    }

    // 在 spawn_blocking 中执行同步 IO
    let mods_dir_clone = mods_dir.clone();
    let groups = tokio::task::spawn_blocking(
        move || -> Result<HashMap<String, Vec<DuplicateVersion>>, String> {
            let files = list_mod_files(&mods_dir_clone);
            let mut groups: HashMap<String, Vec<DuplicateVersion>> = HashMap::new();

            for (file_name, path) in files {
                let metadata = read_mod_metadata(&path);
                let slug = metadata.slug.trim().to_lowercase();
                if slug.is_empty() {
                    // slug 为空的 mod 不参与去重
                    continue;
                }
                let file_size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
                let version = if metadata.version.is_empty() {
                    // 版本号空时回退到文件名
                    file_name.clone()
                } else {
                    metadata.version
                };
                groups.entry(slug).or_default().push(DuplicateVersion {
                    version,
                    file_name,
                    file_size,
                });
            }

            Ok(groups)
        },
    )
    .await
    .map_err(log_err("ModTools 去重扫描任务失败"))??;

    // 只保留有多个版本的 mod，并按 slug 排序保证输出稳定
    let mut duplicates: Vec<DuplicateMod> = groups
        .into_iter()
        .filter(|(_, versions)| versions.len() > 1)
        .map(|(mod_id, mut versions)| {
            // 同一 mod 内按 version 排序，便于前端展示
            versions.sort_by(|a, b| a.version.cmp(&b.version));
            DuplicateMod { mod_id, versions }
        })
        .collect();
    duplicates.sort_by(|a, b| a.mod_id.cmp(&b.mod_id));

    log_info!(
        "[ModTools] 去重扫描完成: 发现 {} 个重复 mod",
        duplicates.len()
    );

    let result = ModDedupResult { duplicates };
    serde_json::to_value(&result).map_err(|e| e.to_string())
}
