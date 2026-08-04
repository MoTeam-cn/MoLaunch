//! frpc 进程管理：启动/停止/状态查询 + 日志捕获 + 退出监听
//! 子模块：start（启动）/ stop（停止）/ status（状态查询）/
//! capture（stdout/stderr 捕获）/ log（日志文件读取）/ state（运行状态）

mod capture;
mod log;
mod start;
mod state;
mod status;
mod stop;

use crate::log_info;
use crate::log_warn;
pub use log::{clear_log_file, list_log_files, read_log_file};
pub use start::start_tunnel;
use state::{FrpcHandle, RUNNING};
pub use status::{get_tunnel_status, list_tunnels_with_status};
pub use stop::stop_tunnel;

/// 停止所有运行中的 frpc 隧道（应用退出前统一清理，避免残留 frpc.exe）
pub async fn stop_all_tunnels() {
    let ids: Vec<String> = {
        let running = RUNNING.lock().await;
        running.keys().cloned().collect()
    };
    if ids.is_empty() {
        return;
    }
    log_info!("[Frp] 应用退出，停止 {} 个运行中的隧道", ids.len());
    for id in ids {
        if let Err(e) = stop_tunnel(id).await {
            log_warn!("[Frp] 退出清理隧道失败: {}", e);
        }
    }
}
