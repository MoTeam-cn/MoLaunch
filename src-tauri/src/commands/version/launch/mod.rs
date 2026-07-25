//! 版本启动命令
//!
//! 模块结构：
//! - mod.rs: 共享类型 + 共享 helper + launch_game 编排 + 其他短命令
//! - build_config.rs: build_launch_config（从全局配置+版本设置+前端入参构建 LaunchConfig）
//! - failure.rs: handle_launch_failure（启动失败崩溃分析+事件通知+状态清理）

mod build_config;
mod failure;

use crate::minecraft::launch::{self, LaunchPipeline};
use crate::state::{AppState, LaunchHistory};
use crate::log_info;
use crate::utils::dispatcher::ActionRequest;
use std::sync::Arc;
use tauri::{Emitter, State};

use super::sanitize_version_id;
use build_config::build_launch_config;
use failure::handle_launch_failure;

/// 游戏退出事件数据
#[derive(Clone, serde::Serialize)]
pub struct GameExitEvent {
    pub pid: u32,
    pub version_id: String,
    pub exit_code: i32,
    pub is_normal: bool,
    /// 崩溃详情（仅异常退出时可能有值）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crash_info: Option<crate::minecraft::launch::watcher::CrashInfo>,
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

/// 解析游戏默认语言配置为实际写入 options.txt 的语言代码
///
/// - `game_language="none"` 或空 → None（不设置，保留玩家游戏内选择）
/// - `game_language="zh_cn"` / `"en_us"` 等 → 直接返回该值
/// - `game_language="auto"` → 旧配置兼容：跟随启动器 UI 语言（zh-CN → zh_cn，en-US → en_us）
///   新版本前端已移除 auto 选项，默认改为 "zh_cn"，此处保留仅为兼容旧配置文件
pub(super) fn resolve_game_language(game_language: &str, launcher_language: &str) -> Option<String> {
    let gl = game_language.trim();
    if gl.is_empty() || gl == "none" {
        return None;
    }
    if gl == "auto" {
        // 旧配置兼容：跟随启动器 UI 语言（BCP 47 → MC 小写代码）
        let mc_lang = match launcher_language {
            "zh-CN" => "zh_cn",
            "en-US" => "en_us",
            _ => "zh_cn", // 默认中文
        };
        return Some(mc_lang.to_string());
    }
    // 直接使用用户指定的语言代码
    Some(gl.to_string())
}

/// 启动游戏
///
/// 编排层：sanitize → build_launch_config → pipeline.execute → 失败时 handle_launch_failure →
/// 成功时保存历史 + spawn 退出监视任务
///
/// 安全修复：移除 access_token 参数，改为后端从 auth_storage 自行获取 token
/// 前端只传 username 和 uuid，避免 token 在 IPC 请求体中明文传输
///
/// 注：原为独立 `#[tauri::command]`，已聚合为 `version_launch_manager` IPC 入口，
/// 由 `utils::version_launch_manager::dispatch` 反序列化参数后调用。
pub async fn launch_game(
    state: &AppState,
    app_handle: &tauri::AppHandle,
    version_id: String,
    java_path: Option<String>,
    username: String,
    uuid: String,
    login_type: Option<String>,
    window_width: Option<u32>,
    window_height: Option<u32>,
    server_address: Option<String>,
    server_port: Option<u32>,
) -> Result<u32, String> {
    sanitize_version_id(&version_id)?;
    log_info!("Launching game version: {}", version_id);

    let launch_config = build_launch_config(
        state,
        app_handle,
        &version_id,
        java_path,
        username,
        uuid,
        login_type,
        window_width,
        window_height,
        server_address,
        server_port,
    )
    .await;

    // 在移入 pipeline 前保存 username（用于启动历史记录）
    let history_username = launch_config.auth_info.username.clone();

    // 使用启动流水线
    let pipeline = Arc::new(LaunchPipeline::new(launch_config));

    // 存储 pipeline 以便后续取消，立即释放锁，后续通过 Arc 访问
    // 避免阻塞 cancel_launch/stop_game/get_launch_progress
    *state.launch_pipeline.lock().await = Some(pipeline.clone());

    // 执行启动
    let result = match pipeline.execute().await {
        Ok(r) => r,
        Err(launch_err) => {
            return Err(handle_launch_failure(state, app_handle, &pipeline, &version_id, launch_err).await);
        }
    };

    log_info!("Game launched with PID: {}", result.pid);

    // 保存到启动历史
    {
        let mut history = state.launch_history.lock().await;
        history.push(LaunchHistory {
            version_id: version_id.clone(),
            username: history_username,
            launch_time: chrono::Local::now().to_rfc3339(),
            pid: result.pid,
            exit_code: None,
        });
    }

    // 更新当前运行状态
    *state.current_pid.lock().await = Some(result.pid);

    // 启动退出监视任务（pipeline 现在是 Arc<LaunchPipeline>，无需持有锁）
    // app_handle 在此函数中是借用，spawn_exit_watcher 需要 owned，故 clone（Arc 廉价）
    spawn_exit_watcher(
        app_handle.clone(),
        pipeline,
        version_id,
        result.pid,
        state.current_pid.clone(),
        state.launch_pipeline.clone(),
    )
    .await;

    Ok(result.pid)
}

/// 启动退出监视任务
///
/// 异步等待游戏进程退出，清理状态并发送 game-exited 事件。
/// 如果 `current_pid` 已被 `stop_game` 清理（变为 None），则不发送事件（避免重复通知）。
async fn spawn_exit_watcher(
    app_handle: tauri::AppHandle,
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

// ============================================================
// 统一 IPC 入口（dispatcher 模式）
// ============================================================

/// 版本启动管理统一 IPC 入口
///
/// 接收 `ActionRequest { action, params }` 请求体，转发到
/// `crate::utils::version_launch_manager::dispatch` 进行 action 分发。
/// 原 7 个独立 Tauri 命令（6 个 launch + 1 个 script_export）均通过此入口聚合调用。
#[tauri::command]
pub async fn version_launch_manager(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
    req: ActionRequest,
) -> Result<serde_json::Value, String> {
    let state = state.inner().clone();
    crate::utils::version_launch_manager::dispatch(state, app, req).await
}
