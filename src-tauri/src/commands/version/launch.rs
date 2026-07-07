//! 版本启动命令

use crate::{log_info, log_error};
use crate::minecraft::launch::{self, AuthInfo, LaunchConfig, LaunchPipeline};
use crate::state::{AppState, resolve_game_dir};
use std::sync::Arc;
use tauri::{State, Emitter};

use super::sanitize_version_id;

/// 游戏退出事件数据
#[derive(Clone, serde::Serialize)]
pub struct GameExitEvent {
    pub pid: u32,
    pub version_id: String,
    pub exit_code: i32,
    pub is_normal: bool,
}

/// 启动游戏
#[tauri::command]
pub async fn launch_game(
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
    version_id: String,
    java_path: Option<String>,
    username: String,
    uuid: String,
    access_token: String,
    window_width: Option<u32>,
    window_height: Option<u32>,
    server_address: Option<String>,
    server_port: Option<u32>,
) -> Result<u32, String> {
    sanitize_version_id(&version_id)?;
    log_info!("Launching game version: {}", version_id);

    let config = state.config.lock().await;
    let game_dir = resolve_game_dir(&config.game_dir);

    // 构建认证信息
    let auth_info = AuthInfo {
        username,
        uuid,
        access_token: access_token.clone(),
        client_token: access_token,
        login_type: "Legacy".to_string(),
    };

    // 创建启动配置
    let launch_config = LaunchConfig {
        game_dir,
        version_id: version_id.clone(),
        auth_info: auth_info.clone(),
        min_memory: config.min_memory,
        max_memory: config.max_memory,
        window_width,
        window_height,
        server_address,
        server_port,
        isolation_mode: config.isolation_mode,
        java_path,
        extra_jvm_args: Vec::new(),
        extra_game_args: Vec::new(),
    };

    // 释放锁，避免阻塞其他操作
    drop(config);

    // 使用启动流水线
    let pipeline = Arc::new(LaunchPipeline::new(launch_config));

    // 存储pipeline以便后续取消，立即释放锁，后续通过 Arc 访问
    // 避免阻塞 cancel_launch/stop_game/get_launch_progress
    *state.launch_pipeline.lock().await = Some(pipeline.clone());

    // 执行启动
    let result = pipeline.execute().await
        .map_err(|e| {
            log_error!("Launch failed: {}", e);
            e.to_string()
        })?;

    log_info!("Game launched with PID: {}", result.pid);

    // 保存到启动历史
    let mut history = state.launch_history.lock().await;
    history.push(crate::state::LaunchHistory {
        version_id: version_id.clone(),
        username: auth_info.username.clone(),
        launch_time: chrono::Local::now().to_rfc3339(),
        pid: result.pid,
        exit_code: None,
    });

    // 更新当前运行状态
    *state.current_pid.lock().await = Some(result.pid);

    // 获取退出接收器（pipeline 现在是 Arc<LaunchPipeline>，无需持有锁）
    let exit_rx = pipeline.exit_receiver().await;

    let version_id_clone = version_id.clone();
    let app_handle_clone = app_handle.clone();
    let launched_pid = result.pid;
    let current_pid_arc = state.current_pid.clone();
    let launch_pipeline_arc = state.launch_pipeline.clone();

    tokio::spawn(async move {
        if let Some(mut rx) = exit_rx {
            // 等待退出通知（wait_for 返回 Ref，需立即 clone 出来避免持有非 Send 的 Ref 跨 await）
            let rx_fut = rx.wait_for(|val| val.is_some());
            let exit_info_opt = match tokio::time::timeout(
                std::time::Duration::from_secs(3600), rx_fut
            ).await {
                Ok(Ok(ref_val)) => {
                    let val = (*ref_val).clone();
                    val
                }
                _ => None,
            };

            // 检查是否被 stop_game 手动清理了
            if current_pid_arc.lock().await.is_some() {
                let (exit_code, is_normal) = match &exit_info_opt {
                    Some(info) => (info.code, info.is_normal),
                    None => (0, true),
                };

                *current_pid_arc.lock().await = None;
                *launch_pipeline_arc.lock().await = None;

                let _ = app_handle_clone.emit("game-exited", GameExitEvent {
                    pid: launched_pid,
                    version_id: version_id_clone,
                    exit_code,
                    is_normal,
                });
            }
        }
    });

    Ok(result.pid)
}

/// 获取启动进度
#[tauri::command]
pub async fn get_launch_progress(state: State<'_, AppState>) -> Result<Option<launch::LaunchProgress>, String> {
    let pipeline = state.launch_pipeline.lock().await;
    if let Some(ref pipeline) = *pipeline {
        Ok(Some(pipeline.progress().await))
    } else {
        Ok(None)
    }
}

/// 取消启动
#[tauri::command]
pub async fn cancel_launch(state: State<'_, AppState>) -> Result<(), String> {
    let pipeline = state.launch_pipeline.lock().await;
    if let Some(ref pipeline) = *pipeline {
        pipeline.cancel().await;
        Ok(())
    } else {
        Err("没有正在进行的启动".to_string())
    }
}

/// 停止游戏
#[tauri::command]
pub async fn stop_game(state: State<'_, AppState>) -> Result<(), String> {
    let mut current_pid = state.current_pid.lock().await;
    if let Some(pid) = *current_pid {
        log_info!("Stopping game with PID: {}", pid);

        // 立即清理并释放 current_pid 锁，避免阻塞监控任务且防止 PID 复用误判
        *current_pid = None;
        drop(current_pid);

        // 在 Windows 上终止进程树（/T 杀子进程，/F 强制结束）
        #[cfg(target_os = "windows")]
        {
            use std::process::Command;
            Command::new("taskkill")
                .args(&["/PID", &pid.to_string(), "/T", "/F"])
                .output()
                .map_err(|e| format!("Failed to stop game: {}", e))?;
        }

        // 在 Unix 上终止进程
        #[cfg(not(target_os = "windows"))]
        {
            use std::process::Command;
            Command::new("kill")
                .args(&["-9", &pid.to_string()])
                .output()
                .map_err(|e| format!("Failed to stop game: {}", e))?;
        }

        // 清理pipeline
        *state.launch_pipeline.lock().await = None;

        log_info!("Game stopped successfully");
        Ok(())
    } else {
        Err("No game is currently running".to_string())
    }
}

/// 获取当前运行的游戏PID
#[tauri::command]
pub async fn get_running_game(state: State<'_, AppState>) -> Result<Option<u32>, String> {
    let current_pid = state.current_pid.lock().await;
    Ok(*current_pid)
}
