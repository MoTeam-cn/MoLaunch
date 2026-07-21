//! 版本启动命令

use crate::minecraft::launch::{self, AuthInfo, CrashCategory, CrashInfo, LaunchConfig, LaunchPipeline, LaunchStage};
use crate::minecraft::version::setup::VersionSetup;
use crate::state::{resolve_game_dir, AppState};
use crate::{log_debug, log_error, log_info, log_warn};
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

/// 启动游戏
///
/// 安全修复：移除 access_token 参数，改为后端从 auth_storage 自行获取 token
/// 前端只传 username 和 uuid，避免 token 在 IPC 请求体中明文传输
#[tauri::command]
pub async fn launch_game(
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
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

    // 构建认证信息
    // 安全修复：从后端 auth_storage 获取 access_token，避免前端 IPC 明文传输 token
    // 前端只传 username 和 uuid，后端根据 uuid 从注册表加载对应账号的 token
    let login_type_str = login_type.clone().unwrap_or_else(|| "Legacy".to_string());
    let is_legacy = login_type_str == "Legacy";
    let (access_token, client_token) = {
        match state.auth_storage.load().await {
            Ok(auth_state) => {
                if let Some(ref current) = auth_state.current_user {
                    // 验证 current_user 的 uuid 与前端传入的 uuid 一致（防止越权）
                    if current.uuid == uuid {
                        (
                            current.access_token.clone(),
                            current.client_token.clone(),
                        )
                    } else {
                        log_warn!(
                            "当前登录账号 UUID ({}) 与请求的 UUID ({}) 不一致，使用空 token",
                            current.uuid,
                            uuid
                        );
                        (String::new(), String::new())
                    }
                } else {
                    // 未登录或离线模式，token 为空
                    (String::new(), String::new())
                }
            }
            Err(e) => {
                log_warn!("从 auth_storage 加载 token 失败: {}，使用空 token", e);
                (String::new(), String::new())
            }
        }
    };

    let auth_info = AuthInfo {
        username,
        uuid,
        access_token,
        client_token,
        login_type: login_type_str,
    };

    // 离线账号皮肤：根据用户选择的皮肤变体调整 UUID
    // PCL2 方案 A：通过递增 UUID 末位让 MC 离线模式哈希到目标皮肤模型（Steve/Alex）
    let auth_info = if is_legacy {
        match state.auth_storage.load().await {
            Ok(auth_state) => {
                if let Some(acc) = auth_state
                    .offline_accounts
                    .iter()
                    .find(|a| a.uuid == auth_info.uuid)
                {
                    if let Some(ref skin_name) = acc.skin {
                        // 判断目标皮肤变体：slim → true（Alex 模型），classic → false（Steve 模型）
                        // 自定义皮肤格式 custom:/path|slim 或 custom:/path|classic
                        let slim = if skin_name.starts_with("custom:") {
                            skin_name.contains("|slim")
                        } else {
                            matches!(
                                skin_name.as_str(),
                                "Alex" | "Ari" | "Efe" | "Makena" | "Noor" | "Sunny" | "Zuri"
                            )
                        };
                        let adjusted_uuid =
                            crate::minecraft::auth::adjust_uuid_for_skin_variant(&auth_info.uuid, slim);
                        if adjusted_uuid != auth_info.uuid {
                            log_info!(
                                "离线皮肤 UUID 调整: {} -> {} (skin={}, slim={})",
                                auth_info.uuid,
                                adjusted_uuid,
                                skin_name,
                                slim
                            );
                        }
                        AuthInfo {
                            uuid: adjusted_uuid,
                            ..auth_info
                        }
                    } else {
                        auth_info
                    }
                } else {
                    auth_info
                }
            }
            Err(e) => {
                log_warn!("加载离线账号皮肤失败: {}, 使用原始 UUID", e);
                auth_info
            }
        }
    } else {
        auth_info
    };

    // 方案 B：离线账号皮肤资源包替换
    // 生成资源包 zip 替换原版玩家纹理，确保 1.19.3+ 也精确显示选定角色
    if is_legacy {
        let skin_to_apply = state
            .auth_storage
            .load()
            .await
            .ok()
            .and_then(|s| {
                s.offline_accounts
                    .iter()
                    .find(|a| a.uuid == auth_info.uuid)
                    .and_then(|a| a.skin.clone())
            });

        match crate::minecraft::launch::skin_resourcepack::apply_skin_resourcepack(
            &game_dir,
            &version_id,
            skin_to_apply.as_deref(),
        ) {
            Ok(_) => {}
            Err(e) => log_warn!("离线皮肤资源包生成失败: {}", e),
        }
    } else {
        // 非离线账号：清理可能存在的离线皮肤资源包
        crate::minecraft::launch::skin_resourcepack::remove_skin_resourcepack(&game_dir);
    }

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
        // 启动高级选项：版本独立覆盖全局（两者都未禁用才启用）
        disable_jlw: config.launch_disable_jlw || setup.advance_disable_jlw.unwrap_or(false),
        disable_lua: config.launch_disable_lua || setup.advance_disable_lua.unwrap_or(false),
        // 忽略 Java 兼容性警告（仅版本独立设置，custom 模式下跳过兼容性校验）
        ignore_java_warning: setup.advance_ignore_java_warning.unwrap_or(false),
        // 关闭文件校验（仅版本独立设置，跳过 libraries/assets/主 jar 文件校验和补全）
        disable_assets_verify: setup.advance_disable_assets_verify.unwrap_or(false),
        // 使用高性能显卡（仅全局设置，启动前写注册表 GpuPreference=2）
        use_dedicated_gpu: config.launch_use_dedicated_gpu,
        // 自定义信息（→ ${version_type} 替换）
        custom_info: setup.custom_info.clone(),
        // 自定义窗口标题（→ Win32 SetWindowText）
        window_title: setup.window_title.clone(),
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
    let result = pipeline.execute().await;

    // 启动失败时的处理：如果是 LaunchProcess 阶段失败（如 ClassNotFoundException），
    // 仍然等待 watcher 完成崩溃分析，并通过 game-exited 事件通知前端展示崩溃对话框
    // 无论进程是否成功启动，只要异常退出都做崩溃分析
    let result = match result {
        Ok(r) => r,
        Err(launch_err) => {
            log_error!("Launch failed: {}", launch_err);

            // 只对 LaunchProcess 阶段的失败做崩溃分析（其他阶段如 GetJava/Login 失败不需要）
            if launch_err.stage != LaunchStage::LaunchProcess {
                return Err(launch_err.to_string());
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

            log_info!("[Watcher] 崩溃分析完成（启动失败路径）: {}（类别: {:?}）", crash_info.reason, crash_info.category);

            // 清理启动状态
            *state.current_pid.lock().await = None;
            *state.launch_pipeline.lock().await = None;

            // 发送 game-exited 事件，让前端展示崩溃对话框
            match app_handle.emit(
                "game-exited",
                GameExitEvent {
                    pid: 0,
                    version_id: version_id.clone(),
                    exit_code,
                    is_normal: false,
                    crash_info: Some(crash_info),
                },
            ) {
                Ok(_) => log_debug!("[Launch] game-exited 事件发送成功"),
                Err(e) => log_error!("[Launch] game-exited 事件发送失败: {}", e),
            }

            return Err(launch_err.to_string());
        }
    };

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
                let (exit_code, is_normal, crash_info) = match &exit_info_opt {
                    Some(info) => (info.code, info.is_normal, info.crash_info.clone()),
                    None => (0, true, None),
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
                        crash_info,
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
