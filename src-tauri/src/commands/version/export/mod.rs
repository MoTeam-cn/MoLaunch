//! 版本整合包导出模块
//! 支持整合包名称/版本号、~20 个可勾选导出选项（游戏本体/设置/Mod/资源包/光影包/存档等）、
//! 动态子选项扫描、联网检查（Modrinth hash + CurseForge fingerprint）、配置文件保存/读取、
//! 生成 Modrinth `modrinth.index.json` + overrides 打包 zip。不含「打包启动器本体」和
//! 「启动器个性化内容」（MoLaunch 无此需求）。

pub mod config;
pub mod network;
pub mod options;
pub mod scan;
pub mod types;
pub mod zip;

use std::path::PathBuf;

use tauri::{AppHandle, Emitter};

use crate::log_error;
use crate::log_info;
use crate::state::AppState;

use self::scan::collect_export_files;
use self::types::{ExportModpackParams, ExportModpackResult, ExportOption, ExportProgress, ExportStage};

/// 导出进度事件名（与前端 listen 的 eventName 一致）
pub const EXPORT_PROGRESS_EVENT: &str = "export-progress";

/// 获取当前版本可用的导出选项列表（含动态子选项扫描）
///
/// 前端进入导出 Tab 时调用，返回所有选项（含根据版本目录实际情况
/// 动态生成的子选项，如资源包/存档/光影包下的具体项目）。
pub async fn get_export_options(
    state: &AppState,
    _app: &AppHandle,
    version_id: String,
) -> Result<Vec<ExportOption>, String> {
    super::sanitize_version_id(&version_id)?;
    let instance_dir = resolve_instance_dir(state, &version_id).await?;
    log_info!("[Export] 获取导出选项，版本: {}，实例目录: {}", version_id, instance_dir.display());
    let opts = options::build_all_options(&instance_dir);
    Ok(opts)
}

/// 推送导出进度事件
fn emit_progress(app: &AppHandle, version_id: &str, stage: ExportStage, percent: u8, message: impl Into<String>) {
    let payload = ExportProgress::new(stage, percent, message, version_id);
    if let Err(e) = app.emit(EXPORT_PROGRESS_EVENT, payload) {
        log_error!("[Export] 推送进度事件失败: {}", e);
    }
}

/// 执行整合包导出
pub async fn export_modpack(
    state: &AppState,
    app: &AppHandle,
    params: ExportModpackParams,
) -> Result<ExportModpackResult, String> {
    super::sanitize_version_id(&params.version_id)?;
    emit_progress(app, &params.version_id, ExportStage::Init, 1, "正在定位版本目录...");

    let instance_dir = resolve_instance_dir(state, &params.version_id).await?;
    log_info!(
        "[Export] 开始导出整合包：{} v{}，版本: {}",
        params.pack_name,
        params.pack_version,
        params.version_id
    );

    // 1. 收集需要导出的文件（应用规则）— 0-10%
    emit_progress(app, &params.version_id, ExportStage::Scan, 3, "正在扫描文件...");
    let files = collect_export_files(&instance_dir, &params.options)?;
    log_info!("[Export] 文件扫描完成，共 {} 个文件", files.len());
    emit_progress(
        app,
        &params.version_id,
        ExportStage::Scan,
        10,
        format!("扫描完成，共 {} 个文件", files.len()),
    );

    // 2. 联网检查 Mod 文件（仅 Modrinth/CurseForge 格式且用户勾选时执行）— 10-50%
    //
    // 其他格式（HMCL/MMC/MCBBS/Compress）没有 mods 下载列表，所有 mod 直接打包，
    // 即使前端误传 check_hosted_assets=true 也强制跳过联网检查。
    let should_check_online = params.check_hosted_assets && params.format.requires_online_check();
    let (files, mut mod_infos) = if should_check_online {
        emit_progress(
            app,
            &params.version_id,
            ExportStage::Network,
            12,
            "正在联网检查 Mod 下载地址...",
        );
        let result = network::check_mod_files_online(&app, &files).await;
        match result {
            Ok(infos) => {
                log_info!("[Export] 联网检查完成，{} 个 mod 获取到下载地址", infos.len());
                emit_progress(
                    app,
                    &params.version_id,
                    ExportStage::Network,
                    50,
                    format!("联网检查完成，{} 个 mod 获取到下载地址", infos.len()),
                );
                (files, infos)
            }
            Err(e) => {
                log_error!("[Export] 联网检查失败: {}，继续导出（mod 将直接打包）", e);
                emit_progress(
                    app,
                    &params.version_id,
                    ExportStage::Network,
                    50,
                    format!("联网检查失败，将直接打包: {}", e),
                );
                (files, Vec::new())
            }
        }
    } else {
        if params.check_hosted_assets && !params.format.requires_online_check() {
            log_info!(
                "[Export] 格式 {:?} 不支持联网检查，强制打包所有文件",
                params.format
            );
        } else {
            log_info!("[Export] 跳过联网检查（用户选择打包资源文件）");
        }
        emit_progress(app, &params.version_id, ExportStage::Network, 50, "跳过联网检查");
        (files, Vec::new())
    };

    // 3. 确定导出路径
    let pack_path = if let Some(p) = &params.config_pack_path {
        PathBuf::from(p)
    } else {
        emit_progress(
            app,
            &params.version_id,
            ExportStage::Failed,
            0,
            "未指定导出路径",
        );
        return Err("未指定导出路径".to_string());
    };

    // 4. 生成 modrinth.index.json + 打包 zip — 50-95%
    emit_progress(app, &params.version_id, ExportStage::Zip, 52, "正在打包 zip...");
    let summary = instance_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(&params.pack_name)
        .to_string();

    let zip_result = zip::build_modpack_zip(
        &instance_dir,
        &files,
        &mut mod_infos,
        &params,
        &summary,
        &pack_path,
        app, // 传递 app 用于按文件数 emit 进度
    );

    if let Err(e) = zip_result {
        emit_progress(app, &params.version_id, ExportStage::Failed, 0, format!("打包失败: {}", e));
        return Err(e);
    }

    log_info!("[Export] 导出完成: {}", pack_path.display());
    emit_progress(
        app,
        &params.version_id,
        ExportStage::Done,
        100,
        format!("导出完成: {}", pack_path.file_name().and_then(|n| n.to_str()).unwrap_or("")),
    );

    Ok(ExportModpackResult {
        success: true,
        file_path: pack_path.to_string_lossy().to_string(),
        file_size: std::fs::metadata(&pack_path)
            .map(|m| m.len())
            .unwrap_or(0),
        file_count: files.len(),
        mod_count: mod_infos.len(),
    })
}

/// 解析版本对应的实例目录
async fn resolve_instance_dir(state: &AppState, version_id: &str) -> Result<PathBuf, String> {
    let game_dir = crate::state::resolve_game_dir_from_state(state).await;
    let instance_dir = game_dir.join("versions").join(version_id);
    if !instance_dir.is_dir() {
        return Err(format!(
            "版本目录不存在: {}",
            instance_dir.display()
        ));
    }
    Ok(instance_dir)
}
