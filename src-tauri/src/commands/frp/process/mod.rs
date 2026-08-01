//! frpc 进程管理：启动/停止/状态查询 + 日志捕获 + 退出监听
//! 子模块：start（启动）/ stop（停止）/ status（状态查询）/
//! capture（stdout/stderr 捕获）/ log（日志文件读取）

mod capture;
mod log;
mod start;
mod status;
mod stop;

use once_cell::sync::Lazy;
use std::collections::HashMap;
use tokio::sync::Mutex as TokioMutex;

pub use log::{list_log_files, read_log_file};
pub use start::start_tunnel;
pub use status::{get_tunnel_status, list_tunnels_with_status};
pub use stop::stop_tunnel;

/// 运行中的 frpc 进程句柄
///
/// `child` 已移入 monitor task 等待退出，这里只保留 pid 和
/// stop_tx（drop 时通知 monitor task 停止）。
pub(super) struct FrpcHandle {
    pub(super) pid: u32,
    pub(super) stop_tx: Option<tokio::sync::oneshot::Sender<()>>,
}

/// 全局运行中进程表（tunnel_id → FrpcHandle）
pub(super) static RUNNING: Lazy<TokioMutex<HashMap<String, FrpcHandle>>> =
    Lazy::new(|| TokioMutex::new(HashMap::new()));
