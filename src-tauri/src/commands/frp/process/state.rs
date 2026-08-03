//! frpc 进程运行状态：进程句柄 + 全局运行中进程表

use once_cell::sync::Lazy;
use std::collections::HashMap;
use tokio::sync::Mutex as TokioMutex;

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
