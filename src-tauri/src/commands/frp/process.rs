//! frpc 进程管理：启动 / 停止 / 状态查询 + 日志捕获 + 退出监听
//!
//! 运行中的 frpc 进程通过全局 `HashMap<tunnel_id, FrpcHandle>` 管理。
//! stdout/stderr 被异步捕获并写入 `<base_dir>/frp/logs/<tunnel_id>.log`，
//! 同时通过 `frpc-log` event 实时推送给前端。
//! frpc 进程退出时通过 `frp-tunnel-status` event 通知前端。
//! 停止隧道时先 kill 子进程，再用 taskkill /T /F 兜底清理进程树。

use super::tunnel;
use super::{
    ensure_dir, frp_logs_dir, LogFileContent, LogFileInfo, TunnelStatus, TunnelWithStatus,
};
use crate::commands::frp::provider;
use crate::log_info;
use crate::log_warn;
use crate::state::AppState;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::Mutex as TokioMutex;

/// 运行中的 frpc 进程句柄
///
/// `child` 已移入 monitor task 等待退出，这里只保留 pid 和
/// stop_tx（drop 时通知 monitor task 停止）。
struct FrpcHandle {
    pid: u32,
    stop_tx: Option<tokio::sync::oneshot::Sender<()>>,
}

/// 全局运行中进程表（tunnel_id → FrpcHandle）
static RUNNING: Lazy<TokioMutex<HashMap<String, FrpcHandle>>> =
    Lazy::new(|| TokioMutex::new(HashMap::new()));

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
        return Err(format!(
            "frpc 二进制不存在: {}",
            frpc_path.display()
        ));
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

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("启动 frpc 失败: {}", e))?;

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
            capture_stream(stdout, log_path, &tunnel_id, &tunnel_name, "stdout", app_for_stream)
                .await;
        });
    }
    if let Some(stderr) = stderr {
        let log_path = log_path.clone();
        let tunnel_id = tunnel.id.clone();
        let tunnel_name = tunnel.name.clone();
        let app_for_stream = app.clone();
        tokio::spawn(async move {
            capture_stream(stderr, log_path, &tunnel_id, &tunnel_name, "stderr", app_for_stream)
                .await;
        });
    }

    // 创建 stop channel
    let (stop_tx, stop_rx) = tokio::sync::oneshot::channel::<()>();

    // 先插入 RUNNING 表（保证 monitor task 退出时能找到条目）
    {
        let mut running = RUNNING.lock().await;
        running.insert(id.clone(), FrpcHandle { pid, stop_tx: Some(stop_tx) });
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

/// 停止隧道
///
/// 1. 从全局进程表取出 stop_tx 并 drop（通知 monitor task）
/// 2. 用 taskkill /T /F 兜底清理进程树（monitor task 可能仍在 wait）
pub async fn stop_tunnel(id: String) -> Result<(), String> {
    let (pid, stop_tx) = {
        let mut running = RUNNING.lock().await;
        let handle = running
            .remove(&id)
            .ok_or_else(|| format!("隧道未在运行: {}", id))?;
        (handle.pid, handle.stop_tx)
    };

    // drop stop_tx 通知 monitor task
    drop(stop_tx);

    // 兜底：用 taskkill /T /F 清理进程树
    if let Err(e) = crate::minecraft::system::shell::kill_process_tree(pid) {
        log_warn!("[Frp] taskkill 兜底清理失败 (PID {}): {}", pid, e);
    }

    log_info!("[Frp] 隧道已停止: {} (PID {})", id, pid);
    Ok(())
}

/// 查询所有隧道状态（附加运行状态 + PID）
pub async fn list_tunnels_with_status() -> Result<Vec<TunnelWithStatus>, String> {
    let tunnels = tunnel::list_tunnels().await?;
    let running = RUNNING.lock().await;

    let result = tunnels
        .into_iter()
        .map(|t| {
            let (status, pid) = if running.contains_key(&t.id) {
                let pid = running.get(&t.id).map(|h| h.pid);
                (TunnelStatus::Running, pid)
            } else {
                (TunnelStatus::Stopped, None)
            };
            TunnelWithStatus {
                tunnel: t,
                status,
                pid,
            }
        })
        .collect();

    Ok(result)
}

/// 查询单个隧道状态
pub async fn get_tunnel_status(id: String) -> Result<TunnelStatus, String> {
    let running = RUNNING.lock().await;
    if running.contains_key(&id) {
        Ok(TunnelStatus::Running)
    } else {
        Ok(TunnelStatus::Stopped)
    }
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
        JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
        SetInformationJobObject,
    };
    use windows::Win32::System::Threading::{
        OpenProcess, PROCESS_SET_QUOTA, PROCESS_TERMINATE,
    };

    unsafe {
        // 1. 创建 Job Object
        let job = CreateJobObjectW(None, None)
            .map_err(|e| format!("创建 Job Object 失败: {}", e))?;

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

/// 单流日志捕获上限（1MB），超过后停止捕获防止内存膨胀
const MAX_STREAM_BYTES: usize = 1024 * 1024;

/// 异步捕获 frpc stdout/stderr 并写入日志文件 + 推送 frpc-log event
///
/// 每行格式：`[HH:MM:SS.ms] [LEVEL] <line>`。
/// LEVEL 推断：行内含 [E]/error/panic → ERROR，stderr → WARN，stdout → INFO。
///
/// 安全措施（对应设计文档 §7.3）：
/// - 每行经 `log_redact::redact_log` 脱敏后再写入文件/推送前端
/// - 单流捕获上限 1MB，超过后写入截断提示并停止捕获
async fn capture_stream(
    reader: impl tokio::io::AsyncRead + Unpin,
    log_path: std::path::PathBuf,
    tunnel_id: &str,
    tunnel_name: &str,
    source: &str,
    app: AppHandle,
) {
    let mut reader = BufReader::new(reader);
    let mut lines = Vec::new();
    let mut total_bytes: usize = 0;
    let mut buf = String::new();

    loop {
        buf.clear();
        match reader.read_line(&mut buf).await {
            Ok(0) => break, // EOF
            Ok(_) => {
                let raw = buf.trim_end();
                if raw.is_empty() {
                    continue;
                }
                // 日志脱敏：将 token / 密码等敏感值替换为 ***
                let line = super::log_redact::redact_log(raw);

                // 1MB 截断检查：超过上限后写入截断提示并停止捕获该流
                let line_len = line.len();
                if total_bytes + line_len > MAX_STREAM_BYTES {
                    let timestamp = chrono_now();
                    let truncated = format!(
                        "[{}] [WARN] 日志输出已超过 1MB 上限，停止捕获该流\n",
                        timestamp
                    );
                    lines.push(truncated.clone());
                    flush_log(&log_path, &mut lines, tunnel_id);
                    let _ = app.emit(
                        "frpc-log",
                        serde_json::json!({
                            "tunnelId": tunnel_id,
                            "tunnelName": tunnel_name,
                            "line": truncated.trim_end(),
                            "timestamp": now_ms(),
                            "level": "WARN",
                        }),
                    );
                    log_warn!(
                        "[Frp] {} 日志超过 1MB 上限，停止捕获 ({})",
                        source,
                        tunnel_id
                    );
                    return;
                }
                total_bytes += line_len;

                let level = infer_log_level(source, &line);
                let timestamp = chrono_now();
                let formatted = format!("[{}] [{}] {}\n", timestamp, level, line);
                lines.push(formatted.clone());

                // 推送 frpc-log event
                let _ = app.emit(
                    "frpc-log",
                    serde_json::json!({
                        "tunnelId": tunnel_id,
                        "tunnelName": tunnel_name,
                        "line": formatted.trim_end(),
                        "timestamp": now_ms(),
                        "level": level,
                    }),
                );

                // 批量写入（每 50 行或达到一定大小写一次）
                if lines.len() >= 50 {
                    flush_log(&log_path, &mut lines, tunnel_id);
                }
            }
            Err(e) => {
                log_warn!("[Frp] 读取 {} 日志失败 ({}): {}", source, tunnel_id, e);
                break;
            }
        }
    }

    // 刷新剩余行
    flush_log(&log_path, &mut lines, tunnel_id);
}

/// 推断日志级别
///
/// - 行内含 "[E]" / "error" / "panic"（大小写不敏感） → ERROR
/// - source=stderr → WARN
/// - source=stdout → INFO
fn infer_log_level(source: &str, line: &str) -> &'static str {
    let lower = line.to_lowercase();
    if line.contains("[E]") || lower.contains("error") || lower.contains("panic") {
        "ERROR"
    } else if source == "stderr" {
        "WARN"
    } else {
        "INFO"
    }
}

/// 批量写入日志文件（追加模式）
fn flush_log(log_path: &std::path::Path, lines: &mut Vec<String>, tunnel_id: &str) {
    if lines.is_empty() {
        return;
    }
    use std::io::Write;
    let mut file = match std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)
    {
        Ok(f) => f,
        Err(e) => {
            log_warn!("[Frp] 写入日志失败 ({}): {}", tunnel_id, e);
            return;
        }
    };
    for line in lines.drain(..) {
        let _ = file.write_all(line.as_bytes());
    }
}

/// 当前时间字符串（用于日志前缀，格式 HH:MM:SS.ms）
fn chrono_now() -> String {
    chrono::Local::now().format("%H:%M:%S%.3f").to_string()
}

/// 当前 Unix 毫秒时间戳
fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

// ============================================================
// 日志读取
// ============================================================

/// 列出所有日志文件
///
/// 扫描 `<base_dir>/frp/logs/` 目录，返回 `.log` 文件列表，
/// 按修改时间倒序排列。
pub async fn list_log_files() -> Result<Vec<LogFileInfo>, String> {
    let logs_dir = frp_logs_dir();
    if !logs_dir.exists() {
        return Ok(Vec::new());
    }
    let mut files = Vec::new();
    for entry in std::fs::read_dir(&logs_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("log") {
            continue;
        }
        let tunnel_id = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();
        let metadata = entry.metadata().map_err(|e| e.to_string())?;
        let modified = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        files.push(LogFileInfo {
            tunnel_id,
            size_bytes: metadata.len(),
            modified_at: modified,
        });
    }
    // 按修改时间倒序
    files.sort_by(|a, b| b.modified_at.cmp(&a.modified_at));
    Ok(files)
}

/// 读取日志文件内容（尾部 maxLines 行）
///
/// `tunnel_id` 为空时合并所有隧道日志（按行内时间戳排序，限 max 行）。
/// `max_lines` 为 None 时默认返回最后 500 行。
/// 文件不存在时返回空内容。
pub async fn read_log_file(
    tunnel_id: String,
    max_lines: Option<usize>,
) -> Result<LogFileContent, String> {
    let max = max_lines.unwrap_or(500);

    // 空隧道 ID：合并所有日志文件
    if tunnel_id.trim().is_empty() {
        return read_all_logs(max).await;
    }

    let path = frp_logs_dir().join(format!("{}.log", tunnel_id));
    if !path.exists() {
        return Ok(LogFileContent {
            lines: Vec::new(),
            has_more: false,
        });
    }
    let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let all_lines: Vec<&str> = content.lines().collect();
    let (lines, has_more) = if all_lines.len() > max {
        let start = all_lines.len() - max;
        (
            all_lines[start..].iter().map(|s| s.to_string()).collect(),
            true,
        )
    } else {
        (all_lines.iter().map(|s| s.to_string()).collect(), false)
    };
    Ok(LogFileContent { lines, has_more })
}

/// 合并所有隧道日志文件，按行内时间戳排序，返回最后 max 行
///
/// 日志行格式：`[HH:MM:SS.ms] [LEVEL] message` 或 `[YYYY-MM-DD HH:MM:SS.ms] [LEVEL] ...`。
/// 无时间戳的行按文件顺序排在一起。
/// 跨文件合并后总行数超过 max 时截取尾部 max 行，并标记 has_more=true。
async fn read_all_logs(max: usize) -> Result<LogFileContent, String> {
    let logs_dir = frp_logs_dir();
    if !logs_dir.exists() {
        return Ok(LogFileContent {
            lines: Vec::new(),
            has_more: false,
        });
    }

    // 收集 (时间戳 sortable key, tunnel_id, 行内容)
    // 时间戳提取：行首 `[...]` 内的内容；若无法提取则用空字符串（排最前）
    let mut entries: Vec<(String, String, String)> = Vec::new();

    for entry in std::fs::read_dir(&logs_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("log") {
            continue;
        }
        let tid = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        for line in content.lines() {
            if line.trim().is_empty() {
                continue;
            }
            let ts = extract_log_timestamp(line);
            entries.push((ts, tid.clone(), line.to_string()));
        }
    }

    if entries.is_empty() {
        return Ok(LogFileContent {
            lines: Vec::new(),
            has_more: false,
        });
    }

    // 按时间戳排序（相同时间戳保持稳定排序）
    entries.sort_by(|a, b| a.0.cmp(&b.0));

    let total = entries.len();
    let has_more = total > max;
    let start = if has_more { total - max } else { 0 };
    let lines: Vec<String> = entries[start..]
        .iter()
        .map(|(ts, tid, line)| {
            // 行内若已含时间戳则原样返回；否则补前缀 `[tid]`
            // 这里所有行都已有时间戳前缀（capture_stream 写入时统一加），直接返回
            let _ = (ts, tid);
            line.clone()
        })
        .collect();

    Ok(LogFileContent { lines, has_more })
}

/// 从日志行提取时间戳排序键
///
/// 支持格式：
/// - `[HH:MM:SS.ms] [LEVEL] ...` → 当天时间，按字符串排序即可
/// - `[YYYY-MM-DD HH:MM:SS.ms] [LEVEL] ...` → 完整时间戳
/// - `2026-07-31 16:47:21.286 [I] ...` → frpc 原生格式，无方括号
///
/// 提取失败返回空字符串（排在前面）。
fn extract_log_timestamp(line: &str) -> String {
    let trimmed = line.trim_start();
    // 形式 1/2：以 [ 开头
    if trimmed.starts_with('[') {
        if let Some(end) = trimmed.find(']') {
            return trimmed[1..end].to_string();
        }
    }
    // 形式 3：frpc 原生格式 "YYYY-MM-DD HH:MM:SS.ms ..."
    // 取前 23 个字符（"YYYY-MM-DD HH:MM:SS.mmm" 长度）
    if trimmed.len() >= 23 {
        let prefix = &trimmed[..23];
        // 简单校验是否符合日期格式
        if prefix.chars().nth(4) == Some('-')
            && prefix.chars().nth(7) == Some('-')
            && prefix.chars().nth(10) == Some(' ')
            && prefix.chars().nth(13) == Some(':')
        {
            return prefix.to_string();
        }
    }
    String::new()
}
