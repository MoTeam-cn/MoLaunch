//! WS 服务器：监听端口、鉴权握手、进度推送

use crate::state::AppState;
use crate::{log_error, log_info, log_warn};
use futures_util::{SinkExt, StreamExt};
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tokio_tungstenite::accept_async;

use super::auth::{auth_ok_message, generate_ws_token, verify_auth_message, AUTH_TIMEOUT_SECS};

/// WS 推送节流间隔（毫秒）
const THROTTLE_MS: u64 = 200;

/// 启动 WS 服务器，监听 127.0.0.1:0 随机端口
pub async fn start_server(app: AppHandle, state: AppState) {
    let listener = match TcpListener::bind("127.0.0.1:0").await {
        Ok(l) => {
            log_info!(
                "WebSocket server listening on ws://{}",
                l.local_addr().unwrap()
            );
            l
        }
        Err(e) => {
            log_error!("Failed to bind WebSocket server: {}", e);
            return;
        }
    };

    let port = listener.local_addr().unwrap().port();
    let token = generate_ws_token();
    let _ = state.ws_port.set(port);
    let _ = state.ws_token.set(token.clone());
    let _ = app.emit("ws-port", port);

    while let Ok((stream, peer)) = listener.accept().await {
        let state = state.clone();
        tokio::spawn(async move {
            let ws_stream = match accept_async(stream).await {
                Ok(ws) => ws,
                Err(e) => {
                    log_warn!("Failed to accept WS connection from {}: {}", peer, e);
                    return;
                }
            };
            handle_connection(ws_stream, state).await;
        });
    }
}

/// 处理单个连接：鉴权 → 推送初始 snapshot → 200ms 节流推送后续进度
async fn handle_connection(
    ws_stream: tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
    state: AppState,
) {
    let (mut writer, mut reader) = ws_stream.split();

    // 鉴权阶段
    let expected_token = match state.ws_token.get() {
        Some(t) => t.clone(),
        None => {
            log_warn!("[WS] No token set in AppState, closing connection");
            let _ = writer
                .send(tokio_tungstenite::tungstenite::Message::Close(None))
                .await;
            return;
        }
    };

    let auth_result =
        tokio::time::timeout(Duration::from_secs(AUTH_TIMEOUT_SECS), reader.next()).await;

    let authed = match auth_result {
        Ok(Some(Ok(tokio_tungstenite::tungstenite::Message::Text(text)))) => {
            verify_auth_message(&text, &expected_token)
        }
        _ => false,
    };

    if !authed {
        log_warn!("[WS] Auth failed or timed out, closing connection");
        let _ = writer
            .send(tokio_tungstenite::tungstenite::Message::Close(None))
            .await;
        return;
    }

    if writer
        .send(tokio_tungstenite::tungstenite::Message::Text(
            auth_ok_message().to_string(),
        ))
        .await
        .is_err()
    {
        log_warn!("[WS] Failed to send auth ack, closing connection");
        return;
    }
    log_info!("[WS] Client authenticated");

    // 订阅 broadcast 并推送进度
    let mut rx = state.progress_tx.subscribe();
    log_info!(
        "[WS] Connection established, subscriber count = {}",
        state.progress_tx.receiver_count()
    );

    // 推送当前 snapshot（解决 subscribe 时序问题）
    let initial_snapshot = {
        let ds = state.download_state.lock().unwrap();
        let is_paused = state
            .download_pause_flag
            .load(std::sync::atomic::Ordering::Relaxed);
        let version_name = ds.version_name.clone();
        crate::commands::version::download::build_snapshot(&ds, &version_name, is_paused)
    };
    if writer
        .send(tokio_tungstenite::tungstenite::Message::Text(
            initial_snapshot.to_string(),
        ))
        .await
        .is_err()
    {
        log_warn!("[WS] Failed to send initial snapshot, closing connection");
        return;
    }

    // 200ms 节流推送
    let mut interval = tokio::time::interval(Duration::from_millis(THROTTLE_MS));
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    interval.tick().await; // 跳过首次立即触发

    let mut pending: Option<serde_json::Value> = None;
    let mut first_broadcast_logged = false;

    loop {
        tokio::select! {
            recv = rx.recv() => match recv {
                Ok(snapshot) => {
                    if !first_broadcast_logged {
                        log_info!("[WS] First broadcast message received");
                        first_broadcast_logged = true;
                    }
                    pending = Some(snapshot);
                }
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    log_warn!("[WS] Connection lagged, skipped {} messages", n);
                }
                Err(broadcast::error::RecvError::Closed) => {
                    log_info!("[WS] Broadcast channel closed, exiting");
                    break;
                }
            },
            _ = interval.tick() => {
                if let Some(snapshot) = pending.take() {
                    let msg = tokio_tungstenite::tungstenite::Message::Text(snapshot.to_string());
                    if writer.send(msg).await.is_err() {
                        log_warn!("[WS] Failed to send throttled message, closing connection");
                        break;
                    }
                }
            }
            msg = reader.next() => match msg {
                Some(Ok(m)) => {
                    if matches!(m, tokio_tungstenite::tungstenite::Message::Close(_)) {
                        log_info!("[WS] Client sent Close frame, exiting");
                        break;
                    }
                }
                _ => {
                    log_info!("[WS] Client stream ended, exiting");
                    break;
                }
            },
        }
    }
}
