//! 启动失败处理
//!
//! LaunchProcess 阶段失败时等待 watcher 崩溃分析、发送 game-exited 事件、清理状态

use crate::minecraft::launch::pipeline::LaunchError;
use crate::minecraft::launch::{CrashCategory, CrashInfo, LaunchPipeline, LaunchStage};
use crate::state::AppState;
use crate::{log_debug, log_error, log_info};
use std::sync::Arc;
use tauri::{Emitter, State};

use super::GameExitEvent;

/// 处理启动失败
///
/// 仅对 LaunchProcess 阶段失败做崩溃分析（其他阶段如 GetJava/Login 失败不需要）。
/// 崩溃分析流程：
/// 1. 等待 watcher 完成崩溃分析（最多 15 秒）
/// 2. 若无分析结果，构造基本 CrashInfo
/// 3. 清理启动状态（current_pid / launch_pipeline）
/// 4. 发送 game-exited 事件让前端展示崩溃对话框
/// 5. 返回错误字符串（调用方包装为 Err）
pub(super) async fn handle_launch_failure(
    state: &State<'_, AppState>,
    app_handle: &tauri::AppHandle,
    pipeline: &Arc<LaunchPipeline>,
    version_id: &str,
    launch_err: LaunchError,
) -> String {
    log_error!("Launch failed: {}", launch_err);

    // 只对 LaunchProcess 阶段的失败做崩溃分析（其他阶段如 GetJava/Login 失败不需要）
    if launch_err.stage != LaunchStage::LaunchProcess {
        return launch_err.to_string();
    }

    log_debug!("[Launch] LaunchProcess 阶段失败，等待 watcher 崩溃分析...");
    // 等待 watcher 完成崩溃分析（watcher 在进程退出后延迟 2 秒开始分析）
    // 这里最多等 15 秒，避免无限等待
    let exit_rx = pipeline.exit_receiver().await;
    let mut crash_info: Option<CrashInfo> = None;
    let exit_code: i32;
    if let Some(mut rx) = exit_rx {
        log_debug!("[Launch] 已获取 exit_rx，等待退出信号...");
        let rx_fut = rx.wait_for(|val| val.is_some());
        match tokio::time::timeout(std::time::Duration::from_secs(15), rx_fut).await {
            Ok(Ok(ref_val)) => {
                log_debug!("[Launch] 收到退出信号");
                if let Some(info) = (*ref_val).clone() {
                    exit_code = info.code;
                    crash_info = info.crash_info.clone();
                } else {
                    exit_code = 1;
                }
            }
            Ok(Err(_)) => {
                log_debug!("[Launch] wait_for 返回错误");
                exit_code = 1;
            }
            Err(_) => {
                log_debug!("[Launch] 等待退出信号超时（15秒）");
                exit_code = 1;
            }
        }
    } else {
        log_debug!("[Launch] 无法获取 exit_rx（watcher 未初始化）");
        exit_code = 1;
    }

    // 如果崩溃分析没结果，构造一个基本的 CrashInfo（用 launch_err.message 作为 reason）
    let crash_info = crash_info.unwrap_or_else(|| {
        log_debug!("[Launch] 崩溃分析无结果，构造基本 CrashInfo");
        CrashInfo {
            reason: "游戏启动失败".to_string(),
            category: CrashCategory::Unknown,
            log_lines: vec![launch_err.message.clone()],
            suggestion: "游戏进程启动后立即退出，可能是 Java 环境或版本文件问题。\n请查看日志详情了解具体原因。".to_string(),
            problematic_mod: None,
            crash_report_path: None,
            log_tail: Vec::new(),
        }
    });

    log_info!(
        "[Watcher] 崩溃分析完成（启动失败路径）: {}（类别: {:?}）",
        crash_info.reason,
        crash_info.category
    );

    // 清理启动状态
    *state.current_pid.lock().await = None;
    *state.launch_pipeline.lock().await = None;

    // 发送 game-exited 事件，让前端展示崩溃对话框
    match app_handle.emit(
        "game-exited",
        GameExitEvent {
            pid: 0,
            version_id: version_id.to_string(),
            exit_code,
            is_normal: false,
            crash_info: Some(crash_info),
        },
    ) {
        Ok(_) => log_debug!("[Launch] game-exited 事件发送成功"),
        Err(e) => log_error!("[Launch] game-exited 事件发送失败: {}", e),
    }

    launch_err.to_string()
}
