//! 加载器自动重装流程（进度推送 / 重装 / 合并）

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};

use crate::minecraft::loaders::LoaderType;
use crate::state::AppState;
use crate::{log_info, log_warn};

use super::detect::{detect_loader_damage, LoaderHealth, REPAIR_LOADER_PROGRESS_EVENT};
use super::merge::merge_loader_json_into;

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
    emit_repair_progress(
        app,
        &RepairProgress {
            version_id,
            phase,
            progress,
            damaged,
            repaired,
            loader_type: health.loader_type.as_deref(),
            loader_version: &health.loader_version,
            mc_version: &health.mc_version,
            message,
        },
    );
}

/// 构建 IPC 最终返回结果（与 RepairLoaderResult 前端类型对应）
fn build_result(
    health: &LoaderHealth,
    damaged: bool,
    repaired: bool,
    message: &str,
) -> serde_json::Value {
    serde_json::json!({
        "loaderType": health.loader_type.as_deref(),
        "loaderVersion": health.loader_version,
        "mcVersion": health.mc_version,
        "damaged": damaged,
        "repaired": repaired,
        "message": message,
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
pub(crate) fn fresh_loader_dir_name(
    loader_type: &LoaderType,
    mc_version: &str,
    loader_version: &str,
) -> String {
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
/// 扫描阶段由独立 `detect_loader_damage` IPC 完成（前端先调用并询问用户），
/// 本函数仅在确认重装后执行，从 `installing` 阶段开始推送进度；`installing`
/// 阶段复用 `install_single_loader` 内部的伪进度 ticker（写 download_state），
/// 此处轮询最后一个 stage 的进度并转发到本事件，避免重复实现进度逻辑。
pub async fn repair_version_loader(
    state: &AppState,
    app: &AppHandle,
    version_id: &str,
) -> Result<serde_json::Value, String> {
    // 重新检测以获取最新健康信息（不再推送 scanning 事件，扫描由前端独立发起）
    let health = match detect_loader_damage(state, version_id).await {
        Ok(h) => h,
        Err(e) => {
            emit_repair_progress(
                app,
                &RepairProgress {
                    version_id,
                    phase: "error",
                    progress: 100,
                    damaged: false,
                    repaired: false,
                    loader_type: None,
                    loader_version: "",
                    mc_version: "",
                    message: &e,
                },
            );
            return Err(e);
        }
    };

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
                ds.stages
                    .last()
                    .map(|s| (s.progress * 100.0) as u8)
                    .unwrap_or(0)
            };
            emit_repair_progress(
                &poll_app,
                &RepairProgress {
                    version_id: &poll_version,
                    phase: "installing",
                    progress,
                    damaged: true,
                    repaired: false,
                    loader_type: poll_loader_type.as_deref(),
                    loader_version: &poll_loader_version,
                    mc_version: &poll_mc_version,
                    message: "正在重新安装加载器...",
                },
            );
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
    let fresh_name =
        fresh_loader_dir_name(&loader_type, &health.mc_version, &health.loader_version);
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
