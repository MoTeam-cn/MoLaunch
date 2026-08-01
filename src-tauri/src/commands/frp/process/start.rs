//! frpc 启动：校验二进制 → 生成配置 → 启动子进程 → 捕获日志 → 监听退出

use crate::commands::frp::provider;
use crate::commands::frp::tunnel;
use crate::commands::frp::{ensure_dir, frp_logs_dir};
use crate::log_info;
use crate::log_warn;
use crate::state::AppState;
use tauri::{AppHandle, Emitter};

use super::capture::capture_stream;
use super::{FrpcHandle, RUNNING};

/// 启动隧道
///
/// 1. 校验 frpc 二进制就绪（按隧道 provider_id 选择对应厂商 frpc）
/// 2. 生成 frpc TOML 配置
/// 3. 启动 frpc 子进程（CREATE_NO_WINDOW）
/// 4. 异步捕获 stdout/stderr 写入日志文件 + 推送 frpc-log event
/// 5. spawn monitor task 监听进程退出，推送 frp-tunnel-status event
/// 6. 记录到全局进程表
pub async fn start_tunnel(state: &AppState, id: String, app: AppHandle) -> Result<(), String> {
    // 检查是否已在运行
    {
        let running = RUNNING.lock().await;
        if running.contains_key(&id) {
            return Err(format!("隧道已在运行: {}", id));
        }
    }

    // 读取隧道配置
    let tunnels = tunnel::list_tunnels().await?;
    let tunnel = tunnels
        .into_iter()
        .find(|t| t.id == id)
        .ok_or_else(|| format!("隧道不存在: {}", id))?;

    // 校验 frpc 就绪（按 provider_id 选择厂商）
    crate::commands::frp::binary::ensure_frpc(state, Some(tunnel.provider_id.clone())).await?;
    let frpc_path = provider::get_frpc_path_for_provider(&tunnel.provider_id)?;
    if !frpc_path.exists() {
        return Err(format!("frpc 二进制不存在: {}", frpc_path.display()));
    }

    let config_path = tunnel::generate_config(&tunnel)?;

    // 准备日志文件
    let logs_dir = frp_logs_dir();
    ensure_dir(&logs_dir)?;
    let log_path = logs_dir.join(format!("{}.log", tunnel.id));
    // 清空旧日志
    std::fs::write(&log_path, "").ok();

    log_info!(
        "[Frp] 启动隧道: {} ({}), frpc={}, config={}",
        tunnel.name,
        tunnel.id,
        frpc_path.display(),
        config_path.display()
    );

    // 启动 frpc 子进程
    let mut cmd = tokio::process::Command::new(&frpc_path);
    cmd.arg("-c").arg(&config_path);
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());
    cmd.stdin(std::process::Stdio::null());

    // 清空环境变量，仅保留 PATH（防止敏感环境变量泄露给 frpc 子进程）
    // 对应设计文档 §7.3 进程隔离
    let path_env = std::env::var("PATH").unwrap_or_default();
    cmd.env_clear();
    cmd.env("PATH", path_env);

    // Windows: CREATE_NO_WINDOW，不弹出控制台窗口
    #[cfg(target_os = "windows")]
    {
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    let mut child = cmd.spawn().map_err(|e| format!("启动 frpc 失败: {}", e))?;

    let pid = child
        .id()
        .ok_or_else(|| "无法获取 frpc 进程 PID".to_string())?;

    // Windows: 关联到 Job Object，启动器退出时 frpc 自动终止（防止僵尸进程）
    // 对应设计文档 §7.3 进程隔离。失败仅记录警告，不阻断启动（stop_tunnel 仍可用
    // taskkill /T /F 兜底清理）。
    #[cfg(target_os = "windows")]
    {
        if let Err(e) = assign_process_to_job_object(pid) {
            log_warn!("[Frp] 关联 Job Object 失败 ({}): {}", tunnel.id, e);
        }
    }

    // 取出 stdout/stderr 管道
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    // 异步捕获日志（带 frpc-log event 推送）
    if let Some(stdout) = stdout {
        let log_path = log_path.clone();
        let tunnel_id = tunnel.id.clone();
        let tunnel_name = tunnel.name.clone();
        let app_for_stream = app.clone();
        tokio::spawn(async move {
            capture_stream(
                stdout,
                log_path,
                &tunnel_id,
                &tunnel_name,
                "stdout",
                app_for_stream,
            )
            .await;
        });
    }
    if let Some(stderr) = stderr {
        let log_path = log_path.clone();
        let tunnel_id = tunnel.id.clone();
        let tunnel_name = tunnel.name.clone();
        let app_for_stream = app.clone();
        tokio::spawn(async move {
            capture_stream(
                stderr,
                log_path,
                &tunnel_id,
                &tunnel_name,
                "stderr",
                app_for_stream,
            )
            .await;
        });
    }

    // 创建 stop channel
    let (stop_tx, stop_rx) = tokio::sync::oneshot::channel::<()>();

    // 先插入 RUNNING 表（保证 monitor task 退出时能找到条目）
    {
        let mut running = RUNNING.lock().await;
        running.insert(
            id.clone(),
            FrpcHandle {
                pid,
                stop_tx: Some(stop_tx),
            },
        );
    }

    // spawn monitor task：等待 child 退出，清理 RUNNING 表并推送 event
    let app_for_monitor = app.clone();
    let tunnel_id_for_monitor = id.clone();
    let tunnel_name_for_monitor = tunnel.name.clone();
    let pid_for_monitor = pid;
    tokio::spawn(async move {
        let status = tokio::select! {
            s = child.wait() => s,
            _ = stop_rx => {
                // stop_tunnel 通知停止，尝试 kill child
                let _ = child.kill().await;
                child.wait().await
            }
        };

        // 从 RUNNING 表移除
        {
            let mut running = RUNNING.lock().await;
            running.remove(&tunnel_id_for_monitor);
        }

        // 推送退出事件
        let (status_str, exit_code, error) = match status {
            Ok(s) => {
                let code = s.code();
                if code == Some(0) {
                    ("stopped", code, None)
                } else {
                    ("stopped", code, Some(format!("frpc 退出，代码 {:?}", code)))
                }
            }
            Err(e) => ("stopped", None, Some(format!("frpc 等待失败: {}", e))),
        };
        let _ = app_for_monitor.emit(
            "frp-tunnel-status",
            serde_json::json!({
                "tunnelId": tunnel_id_for_monitor,
                "tunnelName": tunnel_name_for_monitor,
                "status": status_str,
                "pid": pid_for_monitor,
                "exitCode": exit_code,
                "error": error,
            }),
        );
        log_info!(
            "[Frp] 隧道 {} ({}) frpc 进程已退出",
            tunnel_name_for_monitor,
            tunnel_id_for_monitor
        );
    });

    log_info!("[Frp] 隧道已启动: {} (PID {})", tunnel.name, pid);
    Ok(())
}

/// Windows: 将 frpc 进程关联到 Job Object
///
/// 创建带 `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE` 标志的 Job Object 并关联子进程，
/// 确保启动器退出时 frpc 自动终止，防止僵尸进程。
///
/// 故意不关闭 job 句柄：保持 Job Object 存活直到启动器进程退出。
/// 启动器退出时 OS 自动关闭所有句柄，Job Object 销毁触发 KILL_ON_JOB_CLOSE，
/// 所有关联的 frpc 进程被强制终止。
///
/// 依赖 `windows` crate 的 `Win32_System_JobObjects` feature（需在 Cargo.toml 启用）。
#[cfg(target_os = "windows")]
fn assign_process_to_job_object(pid: u32) -> Result<(), String> {
    use std::ffi::c_void;
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::JobObjects::{
        AssignProcessToJobObject, CreateJobObjectW, JobObjectExtendedLimitInformation,
        SetInformationJobObject, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
        JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
    };
    use windows::Win32::System::Threading::{OpenProcess, PROCESS_SET_QUOTA, PROCESS_TERMINATE};

    unsafe {
        // 1. 创建 Job Object
        let job =
            CreateJobObjectW(None, None).map_err(|e| format!("创建 Job Object 失败: {}", e))?;

        // 2. 配置 KILL_ON_JOB_CLOSE：Job 句柄关闭时杀掉所有关联进程
        let mut info = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
        info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        SetInformationJobObject(
            job,
            JobObjectExtendedLimitInformation,
            &info as *const _ as *const c_void,
            std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
        )
        .map_err(|e| format!("设置 Job Object 信息失败: {}", e))?;

        // 3. 打开子进程句柄
        let process = OpenProcess(PROCESS_SET_QUOTA | PROCESS_TERMINATE, false, pid)
            .map_err(|e| format!("打开 frpc 进程失败: {}", e))?;

        // 4. 关联到 Job Object
        AssignProcessToJobObject(job, process)
            .map_err(|e| format!("关联进程到 Job Object 失败: {}", e))?;

        // 5. 关闭 process 句柄（已关联到 Job，句柄不再需要）
        let _ = CloseHandle(process);

        // 6. 故意不关闭 job 句柄：保持 Job Object 存活直到启动器退出
        //    HANDLE 在 windows 0.58 为 Copy 类型（无 Drop），不调用 CloseHandle 即保持开启
    }
    Ok(())
}
