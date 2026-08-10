//! 加载器损坏检测与自动重装
//!
//! `detect_loader_damage` 检查版本 JSON 中加载器库文件是否缺失/损坏；
//! `repair_version_loader` 检测到损坏后复用 `install_single_loader` 重装加载器，
//! 并将新生成的加载器库合并回当前版本 JSON。

use crate::minecraft::loaders::LoaderType;
use crate::minecraft::version::json_merge::merge_version_json;
use crate::minecraft::version::setup::VersionSetup;
use crate::minecraft::version::state::VersionType;
use crate::state::AppState;
use crate::{log_info, log_warn};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};

use super::super::sanitize_version_id;
use super::{get_version_game_version, version_type_to_string};

/// 加载器健康检测结果
#[derive(Debug, serde::Serialize)]
pub struct LoaderHealth {
    pub loader_type: Option<String>,
    pub loader_version: String,
    pub mc_version: String,
    pub healthy: bool,
    pub reason: String,
}

/// 修复进度推送事件名（前端 RepairLoaderProgress 同源常量）
pub const REPAIR_LOADER_PROGRESS_EVENT: &str = "repair-loader-progress";

/// 修复进度事件负载
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct RepairProgress<'a> {
    version_id: &'a str,
    phase: &'a str,
    progress: u8,
    damaged: bool,
    repaired: bool,
    loader_type: Option<&'a str>,
    loader_version: &'a str,
    mc_version: &'a str,
    message: &'a str,
}

/// 推送修复进度事件
fn emit_repair_progress(app: &AppHandle, payload: &RepairProgress) {
    let _ = app.emit(REPAIR_LOADER_PROGRESS_EVENT, payload);
}

/// 基于健康检测结果推送指定阶段事件
#[allow(clippy::too_many_arguments)]
fn emit_phase(
    app: &AppHandle,
    version_id: &str,
    health: &LoaderHealth,
    phase: &str,
    progress: u8,
    damaged: bool,
    repaired: bool,
    message: &str,
) {
    emit_repair_progress(app, &RepairProgress {
        version_id,
        phase,
        progress,
        damaged,
        repaired,
        loader_type: health.loader_type.as_deref(),
        loader_version: &health.loader_version,
        mc_version: &health.mc_version,
        message,
    });
}

/// 构建 IPC 最终返回结果（与 RepairLoaderResult 前端类型对应）
fn build_result(health: &LoaderHealth, damaged: bool, repaired: bool, message: &str) -> serde_json::Value {
    serde_json::json!({
        "loaderType": health.loader_type.as_deref(),
        "loaderVersion": health.loader_version,
        "mcVersion": health.mc_version,
        "damaged": damaged,
        "repaired": repaired,
        "message": message,
    })
}

/// 加载器类型对应的库名匹配模式
fn loader_lib_pattern(loader_type: &VersionType) -> Option<&'static str> {
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
fn find_loader_lib(json: &serde_json::Value, pattern: &str) -> Option<String> {
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
fn json_lib_local_path(
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
            let file_ok =
                path.exists() && path.metadata().map(|m| m.len() > 0).unwrap_or(false);
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

/// 加载器显示名
fn loader_display_name(loader_type: &LoaderType) -> &'static str {
    match loader_type {
        LoaderType::Forge => "Forge",
        LoaderType::NeoForge => "NeoForge",
        LoaderType::Fabric => "Fabric",
        LoaderType::OptiFine => "OptiFine",
        LoaderType::LiteLoader => "LiteLoader",
    }
}

/// 重装后加载器安装器生成的版本目录名
fn fresh_loader_dir_name(loader_type: &LoaderType, mc_version: &str, loader_version: &str) -> String {
    match loader_type {
        LoaderType::Forge => format!("{}-forge-{}", mc_version, loader_version),
        LoaderType::NeoForge => format!("{}-neoforge-{}", mc_version, loader_version),
        LoaderType::Fabric => format!("fabric-{}-{}", loader_version, mc_version),
        LoaderType::LiteLoader => format!("{}-LiteLoader", mc_version),
        LoaderType::OptiFine => format!("{}-OptiFine", mc_version),
    }
}

/// 检测并自动重装损坏的加载器
///
/// 执行过程中按 `scanning → installing → merging → done/error` 阶段通过
/// `repair-loader-progress` 事件推送进度；`installing` 阶段复用
/// `install_single_loader` 内部的伪进度 ticker（写 download_state），此处轮询
/// 最后一个 stage 的进度并转发到本事件，避免重复实现进度逻辑。
pub async fn repair_version_loader(
    state: &AppState,
    app: &AppHandle,
    version_id: &str,
) -> Result<serde_json::Value, String> {
    // 阶段 1：扫描
    emit_repair_progress(app, &RepairProgress {
        version_id,
        phase: "scanning",
        progress: 0,
        damaged: false,
        repaired: false,
        loader_type: None,
        loader_version: "",
        mc_version: "",
        message: "正在扫描加载器...",
    });
    let health = match detect_loader_damage(state, version_id).await {
        Ok(h) => h,
        Err(e) => {
            emit_repair_progress(app, &RepairProgress {
                version_id,
                phase: "error",
                progress: 100,
                damaged: false,
                repaired: false,
                loader_type: None,
                loader_version: "",
                mc_version: "",
                message: &e,
            });
            return Err(e);
        }
    };
    let damaged = !health.healthy;
    emit_phase(
        app,
        version_id,
        &health,
        "scanning",
        100,
        damaged,
        false,
        if damaged { "检测到加载器损坏" } else { "扫描完成，加载器无损坏" },
    );

    if health.loader_type.is_none() {
        let msg = "该版本未安装加载器";
        emit_phase(app, version_id, &health, "done", 100, false, false, msg);
        return Ok(build_result(&health, false, false, msg));
    }
    if health.healthy {
        let msg = "当前文件无损坏";
        emit_phase(app, version_id, &health, "done", 100, false, false, msg);
        return Ok(build_result(&health, false, false, msg));
    }

    let loader_type = match health.loader_type.as_deref() {
        Some("forge") => LoaderType::Forge,
        Some("neoforge") => LoaderType::NeoForge,
        Some("fabric") => LoaderType::Fabric,
        Some("liteloader") => LoaderType::LiteLoader,
        _ => {
            let msg = format!(
                "检测到加载器损坏，但 {} 暂不支持自动重装",
                health.loader_type.as_deref().unwrap_or("该加载器")
            );
            emit_phase(app, version_id, &health, "done", 100, true, false, &msg);
            return Ok(build_result(&health, true, false, &msg));
        }
    };

    if health.loader_version.is_empty() || health.mc_version.is_empty() {
        let msg = format!(
            "无法确定加载器版本（loader={} mc={}），请尝试重新安装",
            health.loader_version, health.mc_version
        );
        emit_phase(app, version_id, &health, "error", 100, true, false, &msg);
        return Err(msg);
    }

    log_info!(
        "[RepairLoader] {} 检测到加载器损坏，开始自动重装 {} {} (MC {})",
        version_id,
        health.loader_type.as_deref().unwrap_or(""),
        health.loader_version,
        health.mc_version
    );

    // 阶段 2：重新安装
    emit_phase(
        app,
        version_id,
        &health,
        "installing",
        0,
        true,
        false,
        "正在重新安装加载器...",
    );
    let stop_poll = Arc::new(AtomicBool::new(false));
    let poll_stop = stop_poll.clone();
    let poll_state = state.clone();
    let poll_app = app.clone();
    let poll_version = version_id.to_string();
    let poll_loader_type = health.loader_type.clone();
    let poll_loader_version = health.loader_version.clone();
    let poll_mc_version = health.mc_version.clone();
    let _poller = tokio::spawn(async move {
        while !poll_stop.load(Ordering::Relaxed) {
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
            let progress = {
                let ds = poll_state.download_state.lock().unwrap();
                ds.stages.last().map(|s| (s.progress * 100.0) as u8).unwrap_or(0)
            };
            emit_repair_progress(&poll_app, &RepairProgress {
                version_id: &poll_version,
                phase: "installing",
                progress,
                damaged: true,
                repaired: false,
                loader_type: poll_loader_type.as_deref(),
                loader_version: &poll_loader_version,
                mc_version: &poll_mc_version,
                message: "正在重新安装加载器...",
            });
        }
    });

    let game_dir = crate::state::resolve_game_dir_from_state(state).await;
    let version_dir = game_dir.join("versions").join(version_id);
    let json_path = version_dir.join(format!("{}.json", version_id));
    // 备份当前版本 JSON（加载器安装器可能覆盖标准命名目录）
    let backup_json = std::fs::read_to_string(&json_path)
        .ok()
        .and_then(|c| serde_json::from_str::<serde_json::Value>(&c).ok());

    // 重装后加载器安装器生成的版本目录（在移动 loader_type 前确定）
    let fresh_name = fresh_loader_dir_name(&loader_type, &health.mc_version, &health.loader_version);
    let fresh_dir = game_dir.join("versions").join(&fresh_name);

    let (mirror_url, source_mode) = crate::state::resolve_mirror_and_source(state).await;
    let max_threads = {
        let config = state.config.lock().await;
        config.download.max_threads as usize
    };
    let display_name = loader_display_name(&loader_type);

    let install_result = crate::commands::version::install::loader_helpers::install_single_loader(
        state,
        loader_type,
        display_name,
        &health.loader_version,
        &health.mc_version,
        &game_dir,
        mirror_url.as_deref(),
        max_threads,
        source_mode,
    )
    .await;
    stop_poll.store(true, Ordering::Relaxed);
    if let Err(e) = install_result {
        let msg = format!("加载器重装失败: {}", e);
        emit_phase(app, version_id, &health, "error", 100, true, false, &msg);
        return Err(msg);
    }
    emit_phase(
        app,
        version_id,
        &health,
        "installing",
        100,
        true,
        false,
        "加载器重装完成",
    );

    // 阶段 3：合并资源文件
    let existing = match backup_json {
        Some(v) => v,
        None => {
            let msg = "读取版本 JSON 失败，无法完成修复".to_string();
            emit_phase(app, version_id, &health, "error", 100, true, false, &msg);
            return Err(msg);
        }
    };
    emit_phase(
        app,
        version_id,
        &health,
        "merging",
        0,
        true,
        false,
        "正在合并资源文件，请稍后...",
    );
    if let Err(e) = merge_loader_json_into(&game_dir, version_id, &existing, &fresh_dir) {
        emit_phase(app, version_id, &health, "error", 100, true, false, &e);
        return Err(e);
    }

    // 补全缺失库文件（含加载器库，复用补全文件逻辑，幂等）
    if let Err(e) =
        crate::commands::version::manage::fix_version_files(state, app, version_id.to_string())
            .await
    {
        let msg = format!("补全加载器文件失败: {}", e);
        emit_phase(app, version_id, &health, "error", 100, true, false, &msg);
        return Err(msg);
    }

    // 清理重装产生的临时加载器版本目录
    if fresh_dir != version_dir {
        if let Err(e) = std::fs::remove_dir_all(&fresh_dir) {
            log_warn!("[RepairLoader] 清理临时版本目录失败: {}", e);
        }
    }

    log_info!("[RepairLoader] {} 加载器重装完成", version_id);

    // 阶段 4：完成
    let msg = "加载器已重新安装";
    emit_phase(app, version_id, &health, "done", 100, true, true, msg);
    Ok(build_result(&health, true, true, msg))
}

/// 将新生成的加载器 JSON 合并进当前版本 JSON
///
/// - minecraftArguments：token 去重合并
/// - arguments：jvm/game 数组追加去重
/// - libraries：同名库以加载器为准，其余保留
/// - 其余字段：加载器覆盖（保留当前版本 id，去除继承）
fn merge_loader_json_into(
    game_dir: &Path,
    version_id: &str,
    existing: &serde_json::Value,
    fresh_dir: &Path,
) -> Result<(), String> {
    let fresh_dir_name = fresh_dir
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| "无法解析加载器版本目录名".to_string())?;
    let fresh_json_path = fresh_dir.join(format!("{}.json", fresh_dir_name));
    let fresh_content = std::fs::read_to_string(&fresh_json_path)
        .map_err(|e| format!("读取加载器 JSON 失败: {}", e))?;
    let fresh_json: serde_json::Value = serde_json::from_str(&fresh_content)
        .map_err(|e| format!("解析加载器 JSON 失败: {}", e))?;

    // 尝试解析继承链（原版目录存在时），失败则保留加载器 JSON 原样
    let fresh_merged = merge_version_json(&fresh_json, game_dir).unwrap_or_else(|_| fresh_json.clone());

    let mut target = existing.clone();

    merge_minecraft_args(&mut target, &fresh_merged);
    merge_fields(&mut target, &fresh_merged);
    merge_argument_arrays(&mut target, &fresh_merged);
    merge_libraries_dedup(&mut target, &fresh_merged);

    target["id"] = serde_json::Value::String(version_id.to_string());
    if let Some(obj) = target.as_object_mut() {
        obj.remove("inheritsFrom");
    }

    let json_path = game_dir
        .join("versions")
        .join(version_id)
        .join(format!("{}.json", version_id));
    let new_content = serde_json::to_string_pretty(&target)
        .map_err(|e| format!("序列化版本 JSON 失败: {}", e))?;
    std::fs::write(&json_path, new_content).map_err(|e| format!("写入版本 JSON 失败: {}", e))?;

    log_info!("[RepairLoader] 已合并加载器 JSON: {}", version_id);
    Ok(())
}

/// minecraftArguments：按空格 token 去重合并
fn merge_minecraft_args(target: &mut serde_json::Value, fresh: &serde_json::Value) {
    let base = target["minecraftArguments"].as_str().unwrap_or("").to_string();
    let Some(other) = fresh["minecraftArguments"].as_str() else {
        return;
    };
    if other.is_empty() {
        return;
    }
    let mut merged: Vec<&str> = base.split(' ').collect();
    for arg in other.split(' ') {
        if !merged.contains(&arg) {
            merged.push(arg);
        }
    }
    target["minecraftArguments"] = serde_json::Value::String(merged.join(" "));
}

/// 其余字段递归合并（source 覆盖 target），跳过单独处理的键
fn merge_fields(target: &mut serde_json::Value, source: &serde_json::Value) {
    let (Some(target_map), Some(source_map)) = (target.as_object_mut(), source.as_object()) else {
        return;
    };
    for (key, value) in source_map {
        match key.as_str() {
            "libraries" | "arguments" | "minecraftArguments" | "id" | "inheritsFrom" => continue,
            _ => {}
        }
        if let Some(target_value) = target_map.get_mut(key) {
            if target_value.is_object() && value.is_object() {
                merge_fields(target_value, value);
            } else {
                target_value.clone_from(value);
            }
        } else {
            target_map.insert(key.clone(), value.clone());
        }
    }
}

/// arguments：jvm/game 数组追加去重，避免覆盖原版参数
fn merge_argument_arrays(target: &mut serde_json::Value, fresh: &serde_json::Value) {
    let Some(fresh_args) = fresh["arguments"].as_object() else {
        return;
    };
    if !target["arguments"].is_object() {
        target["arguments"] = fresh["arguments"].clone();
        return;
    }
    let target_args = target["arguments"].as_object_mut().unwrap();
    for (key, fresh_val) in fresh_args {
        if let Some(fresh_arr) = fresh_val.as_array() {
            if let Some(target_arr) = target_args.get_mut(key).and_then(|v| v.as_array_mut()) {
                for item in fresh_arr {
                    if !target_arr.contains(item) {
                        target_arr.push(item.clone());
                    }
                }
            } else {
                target_args.insert(key.clone(), fresh_val.clone());
            }
        } else if !target_args.contains_key(key) {
            target_args.insert(key.clone(), fresh_val.clone());
        }
    }
}

/// libraries：同名库以 fresh 为准，其余保留
fn merge_libraries_dedup(target: &mut serde_json::Value, fresh: &serde_json::Value) {
    let Some(fresh_libs) = fresh["libraries"].as_array() else {
        return;
    };
    if !target["libraries"].is_array() {
        target["libraries"] = serde_json::Value::Array(fresh_libs.clone());
        return;
    }
    let target_libs = target["libraries"].as_array_mut().unwrap();
    for lib in fresh_libs {
        let name = lib["name"].as_str().unwrap_or_default();
        if let Some(existing) = target_libs
            .iter_mut()
            .find(|l| l["name"].as_str() == Some(name))
        {
            *existing = lib.clone();
        } else {
            target_libs.push(lib.clone());
        }
    }
}

#[cfg(test)]
#[path = "repair_loader_test.rs"]
mod tests;
