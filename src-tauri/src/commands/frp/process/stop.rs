//! frpc 停止：drop stop_tx + taskkill /T /F 兜底清理进程树

use crate::log_info;
use crate::log_warn;

use super::RUNNING;

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
