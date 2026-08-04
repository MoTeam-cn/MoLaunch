//! frpc 启动：校验二进制 → 生成配置 → 启动子进程 → 捕获日志 → 监听退出

use crate::commands::frp::provider;
use crate::commands::frp::tunnel;
use crate::commands::frp::{ensure_dir, frp_logs_dir};
use crate::log_debug;
use crate::log_info;
use crate::state::AppState;
use tauri::{AppHandle, Emitter};

use super::capture::capture_stream;
use super::{FrpcHandle, RUNNING};

/// 生成启动用 frpc 配置文件（优先厂商原版，回退本地生成）
///
/// 流程：
/// 1. 若隧道厂商配置了 `tunnels.config` 端点（config.mode=url），调用厂商 API
///    拉取原版 frpc 配置，叠加逆向字段（`[proxies.transport]` 带宽限制等）
///    后直接写盘。
/// 2. 无 config 端点 / 拉取失败时，回退本地 `tunnel::generate_config`
///    生成 v1.x 格式 TOML。
///
/// 返回配置文件路径。
async fn prepare_config(
    state: &AppState,
    tunnel: &crate::commands::frp::Tunnel,
) -> Result<std::path::PathBuf, String> {
    let config_dir = crate::commands::frp::frp_config_dir();
    ensure_dir(&config_dir)?;
    let config_path = config_dir.join(format!("{}.toml", tunnel.id));

    // 导入隧道优先直接复用 config 接口返回的完整原文，不能重新拼装或覆盖。
    if let Some(raw) = tunnel.raw_config.as_deref().filter(|v| !v.trim().is_empty()) {
        std::fs::write(&config_path, raw)
            .map_err(|e| format!("写入厂商原版 frpc 配置失败: {}", e))?;
        log_info!("[Frp] 直接复用已保存的厂商原版配置: {}", config_path.display());
        return Ok(config_path);
    }

    // 系统默认 frpc 没有厂商 manifest：直接生成本地配置；只有外部厂商才读取 manifest。
    if tunnel.provider_id == crate::commands::frp::provider::SYSTEM_DEFAULT_ID {
        tunnel::generate_config(tunnel)?;
        return Ok(config_path);
    }

    // 旧数据没有 rawConfig：有 config 端点时启动前拉取一次；没有端点才允许本地生成。
    let manifest = crate::commands::frp::provider::read_provider_manifest(&tunnel.provider_id)?;
    let endpoints_file = manifest.api.as_ref()
        .map(|a| a.endpoints_file.as_str())
        .unwrap_or("api/endpoints.json");
    let spec = crate::commands::frp::api_spec::load_api_spec(&tunnel.provider_id, endpoints_file)?;
    let has_config_endpoint = spec.endpoints.as_ref()
        .and_then(|e| e.tunnels.as_ref())
        .and_then(|t| t.config.as_ref())
        .is_some();
    if !has_config_endpoint {
        tunnel::generate_config(tunnel)?;
        return Ok(config_path);
    }

    let remote_name = tunnel
        .remote_tunnel_name
        .as_deref()
        .unwrap_or(&tunnel.name);
    let raw = crate::commands::frp::api_spec::fetch_raw_tunnel_config(
        state,
        &tunnel.provider_id,
        &tunnel.id,
        remote_name,
    )
    .await
    .map_err(|e| format!("厂商 config 接口获取失败，已停止启动（不会回退本地配置）: {}", e))?;

    std::fs::write(&config_path, raw)
        .map_err(|e| format!("写入厂商原版 frpc 配置失败: {}", e))?;
    log_info!("[Frp] 使用厂商 config 接口原样配置启动: {}", config_path.display());
    Ok(config_path)
}

/// 启动隧道
///
/// 1. 校验 frpc 二进制就绪（按隧道 provider_id 选择对应厂商 frpc）
/// 2. 生成 frpc TOML 配置
/// 3. 启动 frpc 子进程（CREATE_NO_WINDOW）
/// 4. 异步捕获 stdout/stderr 写入日志文件 + 推送 frpc-log event
/// 5. spawn monitor task 监听进程退出，推送 frp-tunnel-status event
/// 6. 记录到全局进程表
pub async fn start_tunnel(state: &AppState, id: String, app: AppHandle) -> Result<(), String> {
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

    // 判断是否 command 直连模式（厂商魔改 frpc，无需配置文件）
    let launch_mode = crate::commands::frp::provider::read_provider_manifest(
        &tunnel.provider_id,
    )
    .ok()
    .and_then(|m| m.binary.launch)
    .filter(|l| l.mode.eq_ignore_ascii_case("command"));

    // config 模式才生成配置文件；command 模式直接走命令参数
    let config_path = if launch_mode.is_none() {
        Some(prepare_config(state, &tunnel).await?)
    } else {
        None
    };

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
        config_path
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "命令直连模式".to_string())
    );

    // 启动 frpc 子进程
    // 按厂商 manifest 的 binary.launch 决定启动方式（通用机制）：
    // - mode=config（默认）：<frpc> -c <config.toml>
    // - mode=command：厂商魔改 frpc 用命令参数直连（如 Lolia `-t <tunnelId>:<token>`），
    //   command 模板支持 {frpc}/{tunnelId}/{token} 占位符
    let mut cmd = tokio::process::Command::new(&frpc_path);
    if let Some(launch) = launch_mode {
        // command 直连模式：解析模板生成参数（不做 shell 拼接，防注入）
        // {tunnelId} = 远程隧道自增 ID（如 Lolia 的 -t 16977:<token>）
        let remote_id = tunnel
            .remote_tunnel_id
            .as_deref()
            .unwrap_or(&tunnel.id);
        let token = tunnel.token.as_deref().unwrap_or("");
        let command = launch.command.as_deref().unwrap_or("").to_string();
        let resolved = command
            .replace("{frpc}", &frpc_path.to_string_lossy())
            .replace("{tunnelId}", remote_id)
            .replace("{token}", token);
        log_info!(
            "[Frp] 使用厂商命令模式启动: {}",
            resolved
        );
        let mut parts = resolved.split_whitespace();
        // 第一个 token 应为 {frpc} 替换后的二进制路径，跳过（Command::new 已指定）
        let _ = parts.next();
        for arg in parts {
            cmd.arg(arg);
        }
    } else {
        // 默认 config 模式
        cmd.arg("-c").arg(config_path.as_ref().unwrap());
    }
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());
    cmd.stdin(std::process::Stdio::null());

    // 清空环境变量，仅保留必要项（防止敏感环境变量泄露给 frpc 子进程）
    // 对应设计文档 §7.3 进程隔离。
    // 保留项：
    // - PATH：frpc 运行必需
    // - 系统代理变量（HTTP_PROXY/HTTPS_PROXY/ALL_PROXY/NO_PROXY 及小写形式）：
    //   frpc 的 DNS 解析 / 网络连接依赖代理环境，清掉后域名解析会失败
    //   （如 `lookup xxx: getaddrinfow: A non-recoverable error`）
    // - Windows 基础变量（SystemRoot/SystemDrive/TEMP/TMP/ComSpec/WINDIR/USERPROFILE）：
    //   部分子系统依赖，清掉可能导致 getaddrinfow 等系统调用诡异失败
    let keep_keys = [
        "PATH",
        "HTTP_PROXY",
        "HTTPS_PROXY",
        "ALL_PROXY",
        "NO_PROXY",
        "http_proxy",
        "https_proxy",
        "all_proxy",
        "no_proxy",
        "SystemRoot",
        "SystemDrive",
        "SYSTEMROOT",
        "WINDIR",
        "TEMP",
        "TMP",
        "ComSpec",
        "USERPROFILE",
    ];
    let mut kept_envs: Vec<(String, String)> = Vec::new();
    for key in keep_keys.iter() {
        if let Ok(val) = std::env::var(key) {
            kept_envs.push((key.to_string(), val));
        }
    }
    cmd.env_clear();
    for (k, v) in kept_envs.iter() {
        cmd.env(k, v);
    }
    // 排障日志：打印实际传给 frpc 的环境变量名（代理值脱敏，防止泄露）
    let env_desc: Vec<String> = kept_envs
        .iter()
        .map(|(k, v)| {
            let is_proxy = k.contains("PROXY") || k.contains("proxy");
            let shown = if is_proxy && !v.is_empty() {
                "<已设置，值已脱敏>".to_string()
            } else if is_proxy {
                "<未设置>".to_string()
            } else {
                v.clone()
            };
            format!("{}={}", k, shown)
        })
        .collect();
    log_debug!(
        "[Frp] 传给 frpc 的环境变量: {}",
        env_desc.join("; ")
    );

    // Windows: CREATE_NO_WINDOW，不弹出控制台窗口
    #[cfg(target_os = "windows")]
    {
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    // Unix: pre_exec 调用 setpgid(0, 0) 让 frpc 成为新进程组 leader，
    // stop_tunnel 可通过 killpg(pid, SIGTERM) 一次性杀整个进程组
    // （含 frpc 自身派生的子进程），避免 ps 递归查询漏掉短命子进程。
    // 对应 Windows 端的 Job Object 进程隔离（设计文档 §7.3）。
    #[cfg(unix)]
    unsafe {
        cmd.pre_exec(|| {
            if libc::setpgid(0, 0) != 0 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
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
        use crate::log_warn;
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
