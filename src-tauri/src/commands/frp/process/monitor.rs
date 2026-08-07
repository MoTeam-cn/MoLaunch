//! frpc 退出监听、日志捕获与运行状态注册。

use crate::commands::frp::process::capture::capture_stream;
use crate::commands::frp::process::{FrpcHandle, RUNNING};
use crate::log_info;
use tauri::{AppHandle, Emitter};

use super::spawn::SpawnedFrpc;

pub(super) async fn register_and_monitor(
    mut spawned: SpawnedFrpc,
    app: AppHandle,
) -> Result<u32, String> {
    let pid = spawned
        .child
        .id()
        .ok_or_else(|| "无法获取 frpc 进程 PID".to_string())?;
    let tunnel_id = spawned.tunnel.id.clone();
    let tunnel_name = spawned.tunnel.name.clone();
    let stdout = spawned.child.stdout.take();
    let stderr = spawned.child.stderr.take();
    spawn_capture(
        stdout,
        &spawned.log_path,
        &tunnel_id,
        &tunnel_name,
        "stdout",
        &app,
    );
    spawn_capture(
        stderr,
        &spawned.log_path,
        &tunnel_id,
        &tunnel_name,
        "stderr",
        &app,
    );

    #[cfg(target_os = "windows")]
    {
        use crate::log_warn;
        if let Err(e) = super::start::assign_process_to_job_object(pid) {
            log_warn!("[Frp] 关联 Job Object 失败 ({}): {}", tunnel_id, e);
        }
    }

    let (stop_tx, stop_rx) = tokio::sync::oneshot::channel::<()>();
    {
        let mut running = RUNNING.lock().await;
        running.insert(
            tunnel_id.clone(),
            FrpcHandle {
                pid,
                stop_tx: Some(stop_tx),
            },
        );
    }

    let app_for_monitor = app.clone();
    tokio::spawn(async move {
        let status = tokio::select! {
            result = spawned.child.wait() => result,
            _ = stop_rx => {
                let _ = spawned.child.kill().await;
                spawned.child.wait().await
            }
        };
        RUNNING.lock().await.remove(&tunnel_id);
        let (exit_code, error) = match status {
            Ok(status) => {
                let code = status.code();
                (
                    code,
                    (code != Some(0)).then(|| format!("frpc 退出，代码 {:?}", code)),
                )
            }
            Err(e) => (None, Some(format!("frpc 等待失败: {}", e))),
        };
        let _ = app_for_monitor.emit(
            "frp-tunnel-status",
            serde_json::json!({
                "tunnelId": tunnel_id, "tunnelName": tunnel_name, "status": "stopped",
                "pid": pid, "exitCode": exit_code, "error": error,
            }),
        );
        log_info!("[Frp] 隧道 {} ({}) frpc 进程已退出", tunnel_name, tunnel_id);
    });
    Ok(pid)
}

fn spawn_capture<R>(
    reader: Option<R>,
    log_path: &std::path::Path,
    tunnel_id: &str,
    tunnel_name: &str,
    source: &'static str,
    app: &AppHandle,
) where
    R: tokio::io::AsyncRead + Unpin + Send + 'static,
{
    if let Some(reader) = reader {
        let log_path = log_path.to_path_buf();
        let tunnel_id = tunnel_id.to_owned();
        let tunnel_name = tunnel_name.to_owned();
        let app = app.clone();
        tokio::spawn(async move {
            capture_stream(reader, log_path, &tunnel_id, &tunnel_name, source, app).await;
        });
    }
}
