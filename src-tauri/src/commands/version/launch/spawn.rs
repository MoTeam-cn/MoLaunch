//! 启动进程域：退出监视任务 + 运行状态/历史短命令

use crate::log_info;
use crate::minecraft::launch::{self, LaunchPipeline};
use crate::state::{AppState, LaunchHistory};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};

use super::GameExitEvent;

/// 启动退出监视任务
///
/// 异步等待游戏进程退出，清理状态并发送 game-exited 事件。
/// 如果 `current_pid` 已被 `stop_game` 清理（变为 None），则不发送事件（避免重复通知）。
pub(super) async fn spawn_exit_watcher(
    app_handle: AppHandle,
    pipeline: Arc<LaunchPipeline>,
    version_id: String,
    launched_pid: u32,
    current_pid: Arc<tokio::sync::Mutex<Option<u32>>>,
    launch_pipeline: Arc<tokio::sync::Mutex<Option<Arc<LaunchPipeline>>>>,
) {
    // 在 spawn 前获取退出接收器（exit_receiver 返回的 future 借用 pipeline，需在当前作用域 await）
    let exit_rx = pipeline.exit_receiver().await;

    tokio::spawn(async move {
        if let Some(mut rx) = exit_rx {
            // 等待退出通知（wait_for 返回 Ref，需立即 clone 出来避免持有非 Send 的 Ref 跨 await）
            let rx_fut = rx.wait_for(|val| val.is_some());
            let exit_info_opt =
                match tokio::time::timeout(std::time::Duration::from_secs(3600), rx_fut).await {
                    Ok(Ok(ref_val)) => (*ref_val).clone(),
                    _ => None,
                };

            // 检查是否被 stop_game 手动清理了
            if current_pid.lock().await.is_some() {
                let (exit_code, is_normal, crash_info) = match &exit_info_opt {
                    Some(info) => (info.code, info.is_normal, info.crash_info.clone()),
                    None => (0, true, None),
                };

                *current_pid.lock().await = None;
                *launch_pipeline.lock().await = None;

                let _ = app_handle.emit(
                    "game-exited",
                    GameExitEvent {
                        pid: launched_pid,
                        version_id,
                        exit_code,
                        is_normal,
                        crash_info,
                    },
                );
            }
        }
    });
}

/// 获取启动进度
///
/// 注：原为独立 `#[tauri::command]`，已聚合为 `version_launch_manager` IPC 入口。
pub async fn get_launch_progress(
    state: &AppState,
) -> Result<Option<launch::LaunchProgress>, String> {
    let pipeline = state.launch_pipeline.lock().await;
    if let Some(ref pipeline) = *pipeline {
        Ok(Some(pipeline.progress().await))
    } else {
        Ok(None)
    }
}

/// 取消启动
///
/// 注：原为独立 `#[tauri::command]`，已聚合为 `version_launch_manager` IPC 入口。
pub async fn cancel_launch(state: &AppState) -> Result<(), String> {
    let pipeline = state.launch_pipeline.lock().await;
    if let Some(ref pipeline) = *pipeline {
        pipeline.cancel().await;
        Ok(())
    } else {
        Err("没有正在进行的启动".to_string())
    }
}

/// 停止游戏
///
/// 注：原为独立 `#[tauri::command]`，已聚合为 `version_launch_manager` IPC 入口。
pub async fn stop_game(state: &AppState) -> Result<(), String> {
    let mut current_pid = state.current_pid.lock().await;
    if let Some(pid) = *current_pid {
        log_info!("Stopping game with PID: {}", pid);

        // 立即清理并释放 current_pid 锁，避免阻塞监控任务且防止 PID 复用误判
        *current_pid = None;
        drop(current_pid);

        // 先标记 watcher 为手动停止，避免 kill 后 watcher 误判为崩溃并触发崩溃分析
        // 修复：之前 kill_process_tree 后游戏以非 0 退出码退出，watcher 误判为崩溃
        {
            let pipeline = state.launch_pipeline.lock().await;
            if let Some(ref p) = *pipeline {
                p.mark_manual_stop().await;
            }
        }

        // 终止进程树（Windows: taskkill /T /F，Unix: kill -9），统一走 shell 模块
        crate::minecraft::system::shell::kill_process_tree(pid)?;

        // 清理 pipeline
        *state.launch_pipeline.lock().await = None;

        log_info!("Game stopped successfully");
        Ok(())
    } else {
        Err("No game is currently running".to_string())
    }
}

/// 获取当前运行的游戏PID
///
/// 注：原为独立 `#[tauri::command]`，已聚合为 `version_launch_manager` IPC 入口。
pub async fn get_running_game(state: &AppState) -> Result<Option<u32>, String> {
    let current_pid = state.current_pid.lock().await;
    Ok(*current_pid)
}

/// 获取启动历史记录
///
/// 返回最近启动过的版本记录（按时间倒序，最多 50 条）。
/// 历史记录仅在内存中累积，重启启动器后清空。
/// 供插件系统（如「启动历史」插件）和未来可能的统计功能使用。
///
/// 注：原为独立 `#[tauri::command]`，已聚合为 `version_launch_manager` IPC 入口。
pub async fn get_launch_history(state: &AppState) -> Result<Vec<LaunchHistory>, String> {
    let history = state.launch_history.lock().await;
    // 返回倒序副本（最近启动在前），最多 50 条避免历史过长
    let mut sorted: Vec<LaunchHistory> = history.iter().rev().take(50).cloned().collect();
    sorted.shrink_to_fit();
    Ok(sorted)
}
