//! 红石联机动作分发（redstone_manager 的命令层实现）
//! 统一注册 redstone_get_servers / redstone_start / redstone_status / redstone_stop，
//! 内核经 extract_hongshi_core 释放后由 HongshiTunnel 子进程封装拉起。

use once_cell::sync::Lazy;
use serde::Deserialize;

use crate::handler;
use crate::log_info;
use crate::state::AppState;
use crate::utils::dispatcher::{ActionRequest, Dispatcher};

use super::tunnel::{parse_tunnel_status, HongshiTunnel};

static DISPATCHER: Lazy<Dispatcher> = Lazy::new(|| {
    let mut d = Dispatcher::new();
    register(&mut d);
    d
});

/// 分发入口
pub(crate) async fn dispatch(
    state: AppState,
    app: tauri::AppHandle,
    req: ActionRequest,
) -> Result<serde_json::Value, String> {
    DISPATCHER.dispatch(state, app, req).await
}

/// `redstone_start` 参数
#[derive(Debug, Deserialize)]
struct RedstoneStartParams {
    /// 中转服务器地址（newserver.json 的 host，可手动填写）
    server: String,
    /// 本地 MC 端口（兼容前端 camelCase 的 mcPort）
    #[serde(alias = "mcPort")]
    mc_port: u16,
}

/// newserver.json 单条服务器记录
#[derive(Debug, Deserialize)]
struct ServerEntry {
    host: String,
    #[serde(default)]
    region: Option<String>,
}

fn register(d: &mut Dispatcher) {
    // 拉取红石中转服务器列表（newserver.json，仅几 KB 轻量 JSON）
    // 失败时前端降级为手动填写，不阻塞创建流程
    d.register(
        "redstone_get_servers",
        handler!(_state, _app, _params, {
            let client = crate::http::build_client_with_redirect(
                reqwest::redirect::Policy::default(),
                Some(10_000),
            );
            let resp = client
                .get("https://hongshi.site/newserver.json")
                .send()
                .await
                .map_err(|e| format!("获取红石服务器列表失败: {e}"))?;
            let entries: Vec<ServerEntry> = resp
                .json()
                .await
                .map_err(|e| format!("解析红石服务器列表失败: {e}"))?;
            let servers: Vec<serde_json::Value> = entries
                .into_iter()
                .map(|e| serde_json::json!({ "host": e.host, "region": e.region }))
                .collect();
            Ok(serde_json::json!({ "servers": servers }))
        }),
    );

    // 创建隧道：释放内核 → 停止旧实例（单实例）→ 拉起 hongshi 子进程
    d.register(
        "redstone_start",
        handler!(state, _app, params, {
            let p: RedstoneStartParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {e}"))?;

            // 释放内核并确定同目录状态文件（tunnel.ini 由内核原子写）
            let kernel_path = crate::resources::extract_hongshi_core()
                .map_err(|e| format!("释放红石内核失败: {e}"))?;
            let status_file = kernel_path.with_file_name("tunnel.ini");

            // 单实例：先停止旧隧道，防止进程泄漏
            let old = state.redstone.lock().await.take();
            if let Some(old) = old {
                old.stop().await;
            }

            let tunnel = HongshiTunnel::spawn(&p.server, p.mc_port, &status_file).await?;
            let pid = tunnel.child.id();
            *state.redstone.lock().await = Some(tunnel);
            log_info!(
                "[Redstone] 隧道已启动: server={}, mc_port={}",
                p.server,
                p.mc_port
            );
            Ok(serde_json::json!({ "pid": pid }))
        }),
    );

    // 查询隧道状态：子进程存活 + tunnel.ini 解析结果
    d.register(
        "redstone_status",
        handler!(state, _app, _params, {
            let mut guard = state.redstone.lock().await;
            if let Some(tunnel) = guard.as_mut() {
                let running = tunnel.is_running();
                let content = std::fs::read_to_string(&tunnel.status_file).unwrap_or_default();
                let ts = parse_tunnel_status(&content);
                Ok(serde_json::json!({
                    "running": running,
                    "status": ts.status,
                    "server": ts.server,
                    "port": ts.port,
                    "created": ts.created,
                }))
            } else {
                Ok(serde_json::json!({
                    "running": false,
                    "status": "unknown",
                    "server": serde_json::Value::Null,
                    "port": serde_json::Value::Null,
                    "created": serde_json::Value::Null,
                }))
            }
        }),
    );

    // 停止隧道并清除 AppState 引用
    d.register(
        "redstone_stop",
        handler!(state, _app, _params, {
            let old = state.redstone.lock().await.take();
            if let Some(old) = old {
                old.stop().await;
                log_info!("[Redstone] 隧道已停止");
            }
            Ok(serde_json::json!({}))
        }),
    );
}
