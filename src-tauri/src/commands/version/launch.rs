//! 版本启动命令

use crate::minecraft::launch::{self, AuthInfo, LaunchConfig, LaunchPipeline};
use crate::minecraft::version::setup::VersionSetup;
use crate::state::{resolve_game_dir, AppState};
use crate::{log_error, log_info};
use std::sync::Arc;
use tauri::{Emitter, State};

use super::sanitize_version_id;

/// 游戏退出事件数据
#[derive(Clone, serde::Serialize)]
pub struct GameExitEvent {
    pub pid: u32,
    pub version_id: String,
    pub exit_code: i32,
    pub is_normal: bool,
}

/// 解析 "IP:Port" 字符串为 (address, port)，无端口时 port=None
pub fn parse_server_enter(s: &str) -> (Option<String>, Option<u32>) {
    let s = s.trim();
    if s.is_empty() {
        return (None, None);
    }
    if let Some((ip, port_str)) = s.rsplit_once(':') {
        if let Ok(port) = port_str.parse::<u32>() {
            return (Some(ip.to_string()), Some(port));
        }
    }
    (Some(s.to_string()), None)
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
    login_type: Option<String>,
    window_width: Option<u32>,
    window_height: Option<u32>,
    server_address: Option<String>,
    server_port: Option<u32>,
) -> Result<u32, String> {
    sanitize_version_id(&version_id)?;
    log_info!("Launching game version: {}", version_id);

    let config = state.config.lock().await;
    let game_dir = resolve_game_dir(&config.game_dir);

    // 读取版本独立设置（setup.ini）
    let version_dir = game_dir.join("versions").join(&version_id);
    let setup = VersionSetup::load_or_create(&version_dir, &version_id);

    // Java 路径解析（根据 setup.java_mode 决定策略）：
    // - 前端传入的 java_path 优先级最高（兼容旧调用方）
    // - 否则按版本独立设置的 JavaMode 处理：
    //   - auto/空 → 自动选择（resolved_java = None，pipeline 按规则表选）
    //   - auto_version → 自动选择指定版本范围（pipeline 用 java_version_min/max 约束）
    //   - folder → 使用版本文件夹下的 Java（pipeline 查找 version_dir/runtime/）
    //   - custom → 使用 setup.java_path
    let resolved_java = java_path.or_else(|| {
        let mode = setup.java_mode.as_deref().unwrap_or("").trim();
        if mode.eq_ignore_ascii_case("custom") {
            setup.java_path.clone().filter(|s| !s.is_empty())
        } else {
            None
        }
    });
    let resolved_java_mode = setup.java_mode.clone();
    let resolved_java_version_min = setup.java_version_min.unwrap_or(0);
    let resolved_java_version_max = setup.java_version_max.unwrap_or(0);

    // 服务器：前端未传则用版本独立的 server_enter（"IP:Port" 格式需解析）
    let (resolved_server_addr, resolved_server_port) =
        if server_address.is_some() || server_port.is_some() {
            (server_address, server_port)
        } else if let Some(ref enter) = setup.server_enter {
            if !enter.is_empty() {
                parse_server_enter(enter)
            } else {
                (None, None)
            }
        } else {
            (None, None)
        };

    // 额外参数：按空白拆分
    let split_args = |s: &Option<String>| -> Vec<String> {
        s.as_deref()
            .filter(|s| !s.is_empty())
            .map(|s| s.split_whitespace().map(String::from).collect())
            .unwrap_or_default()
    };
    let extra_jvm_args = split_args(&setup.advance_jvm_args);
    let extra_game_args = split_args(&setup.advance_game_args);
    let pre_launch_cmd = setup.advance_run_cmd.clone().filter(|s| !s.is_empty());

    // 内存：版本独立设置 > 全局
    // - setup.memory_mode = Some("auto") → 根据系统内存动态计算（版本独立自动）
    // - setup.memory_mode = Some("custom") → 使用 setup.min_memory/max_memory
    // - None / 空 / 其他 → 回退到全局 config.min_memory/max_memory
    let (resolved_min_mem, resolved_max_mem) = match setup
        .memory_mode
        .as_deref()
        .filter(|s| !s.is_empty())
    {
        Some("auto") => crate::minecraft::system::suggest_memory(),
        Some("custom") => {
            let max = setup.max_memory.unwrap_or(config.max_memory);
            let min = setup.min_memory.unwrap_or_else(|| max / 2);
            (min, max)
        }
        _ => (config.min_memory, config.max_memory),
    };

    // 构建认证信息（login_type 从前端传入，默认 Legacy）
    let auth_info = AuthInfo {
        username,
        uuid,
        access_token: access_token.clone(),
        client_token: access_token,
        login_type: login_type.unwrap_or_else(|| "Legacy".to_string()),
    };

    // 创建启动配置
    let launch_config = LaunchConfig {
        game_dir: game_dir.clone(),
        version_id: version_id.clone(),
        auth_info: auth_info.clone(),
        min_memory: resolved_min_mem,
        max_memory: resolved_max_mem,
        window_width,
        window_height,
        server_address: resolved_server_addr,
        server_port: resolved_server_port,
        // 版本独立隔离设置覆盖全局
        isolation_mode: super::list::resolve_isolation_mode(
            &game_dir,
            &version_id,
            config.isolation_mode,
        ),
        java_path: resolved_java,
        java_mode: resolved_java_mode,
        java_version_min: resolved_java_version_min,
        java_version_max: resolved_java_version_max,
        download_source: config.download_source.clone(),
        mirror_url: config.mirror_url.clone(),
        extra_jvm_args,
        extra_game_args,
        pre_launch_cmd,
        app_handle: Some(app_handle.clone()),
    };

    // 释放锁，避免阻塞其他操作
    drop(config);

    // 使用启动流水线
    let pipeline = Arc::new(LaunchPipeline::new(launch_config));

    // 存储pipeline以便后续取消，立即释放锁，后续通过 Arc 访问
    // 避免阻塞 cancel_launch/stop_game/get_launch_progress
    *state.launch_pipeline.lock().await = Some(pipeline.clone());

    // 执行启动
    let result = pipeline.execute().await.map_err(|e| {
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
            let exit_info_opt =
                match tokio::time::timeout(std::time::Duration::from_secs(3600), rx_fut).await {
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

                let _ = app_handle_clone.emit(
                    "game-exited",
                    GameExitEvent {
                        pid: launched_pid,
                        version_id: version_id_clone,
                        exit_code,
                        is_normal,
                    },
                );
            }
        }
    });

    Ok(result.pid)
}

/// 获取启动进度
#[tauri::command]
pub async fn get_launch_progress(
    state: State<'_, AppState>,
) -> Result<Option<launch::LaunchProgress>, String> {
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

        // 终止进程树（Windows: taskkill /T /F，Unix: kill -9），统一走 shell 模块
        crate::minecraft::system::shell::kill_process_tree(pid)?;

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
