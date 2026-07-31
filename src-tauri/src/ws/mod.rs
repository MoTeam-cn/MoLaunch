//! WebSocket 服务器（下载进度推送）
//! 监听 127.0.0.1:0 随机端口，前端通过 `get_ws_port` IPC 获取端口 + token 后建连。
//! 客户端首条消息需携带 token 鉴权，通过后服务器以 200ms 节流推送进度 snapshot。
//! 替代前端 300ms 轮询 `get_download_progress` IPC，devtools 网络面板更干净。

mod auth;
mod server;

use crate::state::AppState;

pub use server::start_server;

/// 向所有 WS 连接广播进度 snapshot
///
/// 在 progress_callback / stage_callback / cancel / pause / resume 中调用。
pub fn broadcast_progress(state: &AppState, snapshot: serde_json::Value) {
    let _ = state.progress_tx.send(snapshot);
}