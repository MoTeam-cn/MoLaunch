//! 房主后台监视循环与自动关房

use std::time::Duration;

use tauri::Emitter;

use crate::log_debug;
use crate::log_error;
use crate::log_info;
use crate::log_warn;
use crate::minecraft::online::scaffolding::server::ScaffoldingServerState;
use crate::state::AppState;

use super::emit_easytier_status;
use super::host::rebuild_host_easytier;

/// 房主自动关闭房间事件（后端→房主前端，触发房间清理登记）
pub const HOST_AUTO_CLOSE_EVENT: &str = "scaffolding-host-auto-close";

/// 房主 MC 端口变更事件（后端→房主前端，展示实时端口）
pub const MC_PORT_CHANGE_EVENT: &str = "scaffolding-mc-port-change";

/// 房主后台监视循环周期（5s）
const WATCH_INTERVAL: Duration = Duration::from_secs(5);

/// 端口连续不可达次数阈值（6 次 = 30s，触发自动关房）
const AUTO_CLOSE_FAIL_LIMIT: u32 = 6;

/// 房主后台监视循环：每 5s 扫描游戏监听端口并回写联机中心。
///
/// - 手动端口设置时跳过自动更新（最高权重），不自动关房；
/// - 从未探测到端口（游戏尚未开局域网）时无限等待（支持先开房后开局域网）；
///   仅「已探测到过端口后再不可达」连续 `AUTO_CLOSE_FAIL_LIMIT` 次（30s）
///   自动关闭房间并推送事件；
/// - 外部 `easytier_stop`/`scaffolding_host_stop` 抢先时（easytier 为 None）直接退出。
pub(super) async fn host_watch_loop(
    center_state: ScaffoldingServerState,
    app: tauri::AppHandle,
    state: AppState,
) {
    let mut current_mc_port: Option<u16> = None;
    let mut fail_count: u32 = 0;
    loop {
        if state.easytier.lock().await.is_none() {
            return;
        }
        // 手动端口最高权重：始终同步手动值，不自动覆盖、不自动关房
        if let Some(manual) = *state.manual_mc_port.lock().await {
            if current_mc_port != Some(manual) {
                current_mc_port = Some(manual);
                center_state.set_mc_port(Some(manual));
                if let Err(e) = rebuild_host_easytier(&state, &app, Some(manual)).await {
                    log_error!("[Online] 房主监视: 重建 easytier 白名单失败: {e}");
                    return;
                }
                log_debug!("[Online] 房主监视: 手动 MC 端口 {manual} 已同步");
            }
            tokio::time::sleep(WATCH_INTERVAL).await;
            continue;
        }
        let pid = *state.current_pid.lock().await;
        let ports = pid
            .map(crate::minecraft::launch::watcher::ports::listening_tcp_ports)
            .unwrap_or_default();
        if ports.is_empty() {
            // 从未探测到端口（游戏尚未开局域网）→ 无限等待，不累计失败，
            // 支持「先开房后开局域网」场景；仅「已探测到过端口后再不可达」触发自动关房
            if current_mc_port.is_some() {
                fail_count += 1;
                if fail_count >= AUTO_CLOSE_FAIL_LIMIT {
                    log_warn!("[Online] 房主监视: MC 端口连续 {fail_count} 次不可达，自动关闭房间");
                    auto_close_room(&state, &app).await;
                    return;
                }
            }
        } else {
            fail_count = 0;
            // 已知端口仍在监听则保持不变；否则取与已知端口最接近者（无已知时取升序第一个）
            let chosen = match current_mc_port {
                Some(cur) if ports.contains(&cur) => cur,
                _ => {
                    let target = current_mc_port.unwrap_or(0);
                    *ports
                        .iter()
                        .min_by_key(|p| p.abs_diff(target))
                        .unwrap_or(&ports[0])
                }
            };
            if current_mc_port != Some(chosen) {
                current_mc_port = Some(chosen);
                center_state.set_mc_port(Some(chosen));
                let _ = app.emit(
                    MC_PORT_CHANGE_EVENT,
                    serde_json::json!({ "mcPort": chosen }),
                );
                if let Err(e) = rebuild_host_easytier(&state, &app, Some(chosen)).await {
                    log_error!("[Online] 房主监视: 重建 easytier 白名单失败: {e}");
                    return;
                }
                log_debug!("[Online] 房主监视: MC 端口已更新为 {chosen}");
            }
        }
        tokio::time::sleep(WATCH_INTERVAL).await;
    }
}

/// 自动关闭房间：停 easytier + 停联机中心 + 清空状态 + 推送前端事件。
///
/// 幂等：各状态为 None 时 no-op，重复调用安全。
async fn auto_close_room(state: &AppState, app: &tauri::AppHandle) {
    let old_easytier = state.easytier.lock().await.take();
    if let Some(old_easytier) = old_easytier {
        old_easytier.stop().await;
    }
    let old_server = state.scaffolding_server.lock().await.take();
    if let Some(old_server) = old_server {
        old_server.stop().await;
    }
    *state.scaffolding_host_watch.lock().await = None;
    *state.manual_mc_port.lock().await = None;
    *state.host_network_cred.lock().await = None;
    *state.client_port_forwards.lock().await = Vec::new();
    emit_easytier_status(app, state).await;
    let _ = app.emit(
        HOST_AUTO_CLOSE_EVENT,
        serde_json::json!({ "reason": "mc_unreachable" }),
    );
    log_info!("[Online] 房间已自动关闭（MC 服务 30s 不可达）");
}
