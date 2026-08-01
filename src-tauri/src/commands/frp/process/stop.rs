//! frpc 停止：drop stop_tx + 进程树兜底清理
//!
//! - Windows: taskkill /T /F（递归杀子进程）
//! - Unix: killpg(pid, SIGTERM)（start.rs 已通过 setpgid 让 frpc 成为进程组 leader，
//!   killpg 一次性杀整个进程组，含 frpc 派生的子进程，比 ps 递归查询更可靠高效）

use crate::log_info;
use crate::log_warn;

use super::RUNNING;

/// 停止隧道
///
/// 1. 从全局进程表取出 stop_tx 并 drop（通知 monitor task）
/// 2. 兜底清理进程树（monitor task 可能仍在 wait）：
///    - Windows: taskkill /T /F
///    - Unix: killpg(pid, SIGTERM)
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

    // 兜底清理进程树
    #[cfg(target_os = "windows")]
    {
        if let Err(e) = crate::minecraft::system::shell::kill_process_tree(pid) {
            log_warn!("[Frp] taskkill 兜底清理失败 (PID {}): {}", pid, e);
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        // start.rs 已通过 setpgid(0, 0) 让 frpc 成为进程组 leader（PGID = frpc PID），
        // killpg 一次性发送 SIGTERM 给整个进程组，含 frpc 派生的所有子进程。
        // SIGTERM 而非 SIGKILL，给 frpc 优雅退出（关闭连接、刷新日志）的机会；
        // 若 frpc 5s 内未退出，monitor task 的 child.wait() 仍会持续，由 tokio runtime
        // 退出时强制回收。
        let pgid = pid as i32;
        let rc = unsafe { libc::killpg(pgid, libc::SIGTERM) };
        if rc != 0 {
            let err = std::io::Error::last_os_error();
            // ESRCH (No such process) 不算错误：frpc 可能已自行退出
            if err.raw_os_error() != Some(libc::ESRCH) {
                log_warn!("[Frp] killpg 兜底清理失败 (PGID {}): {}", pgid, err);
            }
        }
    }

    log_info!("[Frp] 隧道已停止: {} (PID {})", id, pid);
    Ok(())
}
