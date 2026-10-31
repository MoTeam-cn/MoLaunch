//! 安装后处理：JSON 合并、原版目录删除、版本重命名
//!
//! 安装加载器后合并加载器 JSON（删除 inheritsFrom）、
//! 删除原版目录、按用户自定义名称重命名版本目录/JAR/JSON。

use crate::{log_info, log_warn};

use super::instance_files;
use super::version_naming::find_loader_version_dir;

/// 安装后处理：合并 JSON + 删除原版 + 重命名版本目录
///
/// 返回最终的版本目录名（actual_version_id）。
/// - 无加载器时直接返回 mc_version
/// - 有加载器时查找加载器版本目录，合并 JSON，删除原版，按 instance 重命名
pub(crate) fn merge_and_rename_version(
    game_dir: &std::path::Path,
    mc_version: &str,
    instance: &str,
    has_any_loader: bool,
) -> String {
    if has_any_loader {
        // 顺序不能反：合并失败时加载器 JSON 里的 inheritsFrom 仍指向原版目录，
        // 先删就会留下一个永远起不来的版本（classpath 递归收集不到父版本库）。
        match merge_loader_json(game_dir, mc_version) {
            Ok(()) => delete_original_dir(game_dir, mc_version),
            Err(e) => {
                crate::log_error!("[Merged] 合并加载器 JSON 失败，保留原版目录: {}", e);
            }
        }
    }

    // 确定最终的版本目录名
    let final_version_id = if has_any_loader {
        let versions_dir = game_dir.join("versions");
        find_loader_version_dir(&versions_dir, mc_version).unwrap_or_else(|| mc_version.to_string())
    } else {
        mc_version.to_string()
    };

    // 如果用户自定义了名称，需要重命名版本目录和修改 JSON
    let final_id = if instance != final_version_id {
        rename_version_dir(game_dir, &final_version_id, instance)
    } else {
        final_version_id
    };

    // 装好之后记一份文件事实表，并顺带看看有没有安装残留
    instance_files::record(game_dir, &final_id);
    report_orphans(game_dir);
    final_id
}

/// 看一眼 versions 下有没有安装残留——用户不会主动去翻这些目录
fn report_orphans(game_dir: &std::path::Path) {
    let versions_dir = crate::minecraft::version::orphans::versions_dir_of(game_dir);
    let orphans = crate::minecraft::version::orphans::scan_version_orphans(&versions_dir);
    if let Some(summary) = crate::minecraft::version::orphans::summarize(&orphans) {
        crate::log_warn!("[Install] {}（不自动删除，需要时可手动清理）", summary);
    }
}

/// 合并加载器版本的 JSON（删除 inheritsFrom，合并原版 JSON）
fn merge_loader_json(game_dir: &std::path::Path, mc_version: &str) -> Result<(), String> {
    let versions_dir = game_dir.join("versions");
    let dir_name = find_loader_version_dir(&versions_dir, mc_version)
        .ok_or_else(|| format!("找不到 {} 的加载器版本目录", mc_version))?;

    let loader_json_path = versions_dir
        .join(&dir_name)
        .join(format!("{}.json", dir_name));

    if !loader_json_path.exists() {
        return Err(format!(
            "加载器版本 JSON 不存在: {}",
            loader_json_path.display()
        ));
    }

    let content = std::fs::read_to_string(&loader_json_path)
        .map_err(|e| format!("读取 {} 失败: {}", loader_json_path.display(), e))?;
    let json = serde_json::from_str::<serde_json::Value>(&content)
        .map_err(|e| format!("解析 {} 失败: {}", loader_json_path.display(), e))?;

    if json.get("inheritsFrom").is_none() {
        // 已经是自包含的 JSON，不需要合并
        return Ok(());
    }

    let merged = crate::minecraft::version::json_merge::merge_version_json(&json, game_dir)
        .map_err(|e| format!("合并版本 JSON 失败: {}", e))?;
    let new_content =
        serde_json::to_string_pretty(&merged).map_err(|e| format!("序列化合并结果失败: {}", e))?;
    // 写不进去就不能声称已合并——以前这里 let _ 之后照样打「已合并」
    std::fs::write(&loader_json_path, new_content)
        .map_err(|e| format!("写入 {} 失败: {}", loader_json_path.display(), e))?;
    log_info!("[Merged] 已合并JSON并删除inheritsFrom: {}", dir_name);
    Ok(())
}

fn delete_original_dir(game_dir: &std::path::Path, mc_version: &str) {
    let mc_version_dir = game_dir.join("versions").join(mc_version);
    if mc_version_dir.exists() {
        match std::fs::remove_dir_all(&mc_version_dir) {
            Ok(_) => log_info!("[Merged] 已删除原版目录: {}", mc_version_dir.display()),
            Err(e) => log_warn!("[Merged] 删除原版目录失败: {}", e),
        }
    }
}

/// 重命名版本目录（含整合包半成品目录合并场景）+ 重命名 JSON/JAR
///
/// 返回重命名后的版本目录名（成功 = instance，失败 = final_version_id）
fn rename_version_dir(
    game_dir: &std::path::Path,
    final_version_id: &str,
    instance: &str,
) -> String {
    log_info!("[Merged] 重命名版本: {} -> {}", final_version_id, instance);
    let old_dir = game_dir.join("versions").join(final_version_id);
    let new_dir = game_dir.join("versions").join(instance);

    // 重命名目录：如果目标已存在（整合包半成品目录），改为合并文件
    let rename_ok = if new_dir.exists() {
        merge_dir_contents(&old_dir, &new_dir)
    } else {
        std::fs::rename(&old_dir, &new_dir).is_ok()
    };

    if !rename_ok {
        log_warn!("[Merged] 重命名/合并目录失败");
        return final_version_id.to_string();
    }

    // 重命名 JSON 文件
    let old_json = new_dir.join(format!("{}.json", final_version_id));
    let new_json = new_dir.join(format!("{}.json", instance));
    if old_json.exists() {
        if let Err(e) = std::fs::rename(&old_json, &new_json) {
            log_warn!("[Merged] 重命名 JSON 失败: {}", e);
        }
    }

    // 修改 JSON 中的 id 字段
    if new_json.exists() {
        if let Ok(content) = std::fs::read_to_string(&new_json) {
            if let Ok(mut json) = serde_json::from_str::<serde_json::Value>(&content) {
                json["id"] = serde_json::Value::String(instance.to_string());
                if let Ok(new_content) = serde_json::to_string_pretty(&json) {
                    let _ = std::fs::write(&new_json, new_content);
                }
            }
        }
    }

    // 重命名 JAR 文件
    let old_jar = new_dir.join(format!("{}.jar", final_version_id));
    let new_jar = new_dir.join(format!("{}.jar", instance));
    if old_jar.exists() {
        let _ = std::fs::rename(&old_jar, &new_jar);
    }

    instance.to_string()
}

/// 整合包半成品目录：移动 MC 本体相关文件到目标目录，跳过已存在的文件
fn merge_dir_contents(old_dir: &std::path::Path, new_dir: &std::path::Path) -> bool {
    log_info!(
        "[Merged] 目标目录已存在（整合包半成品），改为合并文件: {} -> {}",
        old_dir.display(),
        new_dir.display()
    );
    let mut merged_ok = true;
    if let Ok(entries) = std::fs::read_dir(old_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let file_name = entry.file_name();
            let target = new_dir.join(&file_name);
            // 如果目标已存在同名文件（如 config 目录），跳过避免覆盖整合包配置
            if target.exists() {
                log_info!("[Merged] 跳过已存在的文件: {}", target.display());
                continue;
            }
            if let Err(e) = std::fs::rename(&path, &target) {
                log_warn!(
                    "[Merged] 移动文件失败: {} -> {} : {}",
                    path.display(),
                    target.display(),
                    e
                );
                merged_ok = false;
            }
        }
    }
    // 删除空的 old_dir
    let _ = std::fs::remove_dir(old_dir);
    merged_ok
}
