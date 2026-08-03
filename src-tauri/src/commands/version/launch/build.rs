//! 启动参数构建域：launch_game 编排 + 参数解析 helper

use crate::log_info;
use crate::minecraft::launch::LaunchPipeline;
use crate::state::AppState;
use std::sync::Arc;

use super::super::sanitize_version_id;
use super::build_config::build_launch_config;

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
pub(super) fn resolve_game_language(
    game_language: &str,
    launcher_language: &str,
) -> Option<String> {
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

#[allow(clippy::too_many_arguments)]
/// 启动游戏
///
/// 编排层：sanitize → build_launch_config → pipeline.execute → 失败时 handle_launch_failure →
/// 成功时保存历史 + spawn 退出监视任务
///
/// 安全修复：移除 access_token 参数，改为后端从 auth_storage 自行获取 token
/// 前端只传 username 和 uuid，避免 token 在 IPC 请求体中明文传输
///
/// 注：原为独立 `#[tauri::command]`，已聚合为 `version_launch_manager` IPC 入口，
/// 由 `manager::dispatch` 反序列化参数后调用。
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
    extra_jvm_args: Option<Vec<String>>,
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
        extra_jvm_args,
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
            return Err(super::failure::handle_launch_failure(
                state,
                app_handle,
                &pipeline,
                &version_id,
                launch_err,
            )
            .await);
        }
    };

    log_info!("Game launched with PID: {}", result.pid);

    // 保存到启动历史
    {
        let mut history = state.launch_history.lock().await;
        history.push(crate::state::LaunchHistory {
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
    super::spawn::spawn_exit_watcher(
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