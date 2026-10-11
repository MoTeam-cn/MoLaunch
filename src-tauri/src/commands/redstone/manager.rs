//! 红石联机动作分发（redstone_manager 的命令层实现）
//! 统一注册 redstone_get_servers / redstone_start / redstone_status / redstone_stop
//! / redstone_log_files / redstone_read_log。
//! 内核由 kernel::ensure_kernel 解析（缓存 → 运行时下载 → 编译期兜底），
//! 再由 HongshiTunnel 子进程封装拉起；隧道地址取自内核 stdout 的 endpoint=。

use once_cell::sync::Lazy;
use serde::Deserialize;
use std::time::Duration;

use crate::handler;
use crate::log_info;
use crate::state::AppState;
use crate::utils::dispatcher::{ActionRequest, Dispatcher};

use super::tunnel::{split_endpoint, HongshiTunnel};

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
    /// 中转服务器地址（/api/server/list 的 host，可手动填写）
    server: String,
    /// 本地 MC 端口（兼容前端 camelCase 的 mcPort）
    #[serde(alias = "mcPort")]
    mc_port: u16,
}

/// 中继列表解析：官方 /api/server/list 返回 map（region → host），
/// 旧文档示例为 array（[{host, region}]），统一转换为 `{ host, region }` 列表。
fn parse_new_server(value: serde_json::Value) -> Vec<serde_json::Value> {
    match value {
        // map 格式（线上实际）：{ "南京": "nanjing.hongshi.site", "成都": "chengdu.hongshi.site" }
        serde_json::Value::Object(map) => map
            .into_iter()
            .map(|(region, host)| {
                let host = host.as_str().unwrap_or_default().to_string();
                serde_json::json!({ "host": host, "region": region })
            })
            .collect(),
        // array 格式（官方文档示例）：[{ "host": "relay-1.hongshi.site", "region": "cn-east" }]
        serde_json::Value::Array(arr) => arr
            .into_iter()
            .map(|e| {
                let host = e
                    .get("host")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string();
                let region = e.get("region").and_then(|v| v.as_str());
                serde_json::json!({ "host": host, "region": region })
            })
            .collect(),
        _ => Vec::new(),
    }
}

fn register(d: &mut Dispatcher) {
    // 拉取红石中转服务器列表（官方 /api/server/list，仅几 KB 轻量 JSON）
    // 失败时前端降级为手动填写，不阻塞创建流程
    d.register(
        "redstone_get_servers",
        handler!(_state, _app, _params, {
            // 复用全局主 client（重定向策略一致），单请求 10s 超时
            let resp = crate::http::get_client()
                .get("https://hongshi.site/api/server/list")
                .timeout(Duration::from_secs(10))
                .send()
                .await
                .map_err(|e| format!("获取红石服务器列表失败: {e}"))?;
            let value: serde_json::Value = resp
                .json()
                .await
                .map_err(|e| format!("解析红石服务器列表失败: {e}"))?;
            let servers = parse_new_server(value);
            if servers.is_empty() {
                return Err("红石服务器列表格式异常".to_string());
            }
            Ok(serde_json::json!({ "servers": servers }))
        }),
    );

    // 创建隧道：解析内核 → 停止旧实例（单实例）→ 拉起 hongshic 子进程
    d.register(
        "redstone_start",
        handler!(state, _app, params, {
            let p: RedstoneStartParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {e}"))?;

            // 单实例：先停止旧隧道，防止进程泄漏
            let old = state.redstone.lock().await.take();
            if let Some(old) = old {
                old.stop().await;
            }

            let tunnel = HongshiTunnel::spawn(&p.server, p.mc_port).await?;
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

    // 查询隧道状态：子进程存活 + 内核 stdout 解析出的 endpoint
    //  - 进程已退出 → closed
    //  - 进程在但还没拿到 endpoint → unknown（前端保持"连接中"）
    //  - 拿到 endpoint → open，server/port 即该地址
    d.register(
        "redstone_status",
        handler!(state, _app, _params, {
            let mut guard = state.redstone.lock().await;
            if let Some(tunnel) = guard.as_mut() {
                let running = tunnel.is_running();
                let endpoint = tunnel.endpoint();
                let (host, port) = match endpoint.as_deref() {
                    Some(ep) => split_endpoint(ep),
                    None => (String::new(), None),
                };
                let status = if !running {
                    "closed"
                } else if port.is_some() {
                    "open"
                } else {
                    "unknown"
                };
                let server = if host.is_empty() {
                    serde_json::Value::Null
                } else {
                    serde_json::Value::String(host)
                };
                Ok(serde_json::json!({
                    "running": running,
                    "status": status,
                    "server": server,
                    "port": port,
                    "created": tunnel.created_at,
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

    // 列出红石内核日志文件（logs/ 目录，按时间倒序）
    d.register(
        "redstone_log_files",
        handler!(_state, _app, _params, {
            let files = crate::commands::redstone::log::list_log_files()?;
            Ok(serde_json::json!({ "files": files }))
        }),
    );

    // 读取指定红石内核日志文件尾部内容
    d.register(
        "redstone_read_log",
        handler!(_state, _app, params, {
            let file_name = params
                .get("file_name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let max_lines = params
                .get("max_lines")
                .and_then(|v| v.as_u64())
                .map(|v| v as usize);
            let content = crate::commands::redstone::log::read_log_file(file_name, max_lines)?;
            Ok(serde_json::json!({ "content": content }))
        }),
    );
}
