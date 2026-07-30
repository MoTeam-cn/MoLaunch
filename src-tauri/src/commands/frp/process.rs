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
pub async fn start_tunnel(id: String, app: AppHandle) -> Result<(), String> {
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
    crate::commands::frp::binary::ensure_frpc(Some(tunnel.provider_id.clone())).await?;
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

/// 异步捕获 frpc stdout/stderr 并写入日志文件 + 推送 frpc-log event
///
/// 每行格式：`[HH:MM:SS.ms] [LEVEL] <line>`。
/// LEVEL 推断：行内含 [E]/error/panic → ERROR，stderr → WARN，stdout → INFO。
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
    let mut buf = String::new();

    loop {
        buf.clear();
        match reader.read_line(&mut buf).await {
            Ok(0) => break, // EOF
            Ok(_) => {
                let line = buf.trim_end();
                if line.is_empty() {
                    continue;
                }
                let level = infer_log_level(source, line);
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
/// `max_lines` 为 None 时默认返回最后 500 行。
/// 文件不存在时返回空内容。
pub async fn read_log_file(
    tunnel_id: String,
    max_lines: Option<usize>,
) -> Result<LogFileContent, String> {
    let path = frp_logs_dir().join(format!("{}.log", tunnel_id));
    if !path.exists() {
        return Ok(LogFileContent {
            lines: Vec::new(),
            has_more: false,
        });
    }
    let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let all_lines: Vec<&str> = content.lines().collect();
    let max = max_lines.unwrap_or(500);
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
