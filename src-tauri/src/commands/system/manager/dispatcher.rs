//! 系统模块统一分发逻辑（system 域 manager 的实现）
//! 35 个 system action 在 `once_cell::sync::Lazy` 初始化时按域分组注册：
//! game_dir / config / developer / updater 及其它（about/logger/http_logs/certs/deeplink/ws）。
//! 非 Result 返回的命令需在 handler 内用 `serde_json::to_value` 包装。

use once_cell::sync::Lazy;
use serde::Deserialize;
use tauri::AppHandle;

use super::{config, developer, game_dir, updater};
use crate::handler;
use crate::logger::{get_log_path, list_log_files, read_log_file};
use crate::state::AppState;
use crate::utils::dispatcher::{ActionRequest, Dispatcher};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReadLogFileParams {
    filename: String,
}

/// 读取 HTTP 日志的参数
///
/// - `date`: 日期字符串（`YYYY-MM-DD`），None 表示今天
/// - `limit`: 最多返回条数（从末尾截取最新的），None 表示全部
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReadHttpLogsParams {
    date: Option<String>,
    limit: Option<usize>,
}

/// 添加自定义证书的参数（源文件路径）
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AddCustomCertParams {
    path: String,
}

/// 删除自定义证书的参数（证书文件名）
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RemoveCustomCertParams {
    filename: String,
}

static DISPATCHER: Lazy<Dispatcher> = Lazy::new(|| {
    let mut d = Dispatcher::new();
    config::register(&mut d);
    game_dir::register(&mut d);
    developer::register(&mut d);
    updater::register(&mut d);

    d.register(
        "get_about_data",
        handler!(_state, _app, _params, {
            let r = crate::commands::system::about::get_about_data().await?;
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );

    // get_log_path / list_log_files 返回非 Result（String / Vec<String>），
    // handler 内用 Ok(to_value(r)?) 包装。read_log_file 返回 Result<String>。
    d.register(
        "get_log_path",
        handler!(_state, _app, _params, {
            let r = get_log_path();
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );
    d.register(
        "list_log_files",
        handler!(_state, _app, _params, {
            let r = list_log_files();
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );
    d.register(
        "read_log_file",
        handler!(_state, _app, params, {
            let p: ReadLogFileParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            let r = read_log_file(p.filename)?;
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "read_http_logs",
        handler!(_state, _app, params, {
            let p: ReadHttpLogsParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            let r = crate::minecraft::online::http_log::read_http_logs(p.date.as_deref(), p.limit);
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "list_http_log_files",
        handler!(_state, _app, _params, {
            let r = crate::minecraft::online::http_log::list_http_log_files();
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "list_custom_certs",
        handler!(_state, _app, _params, {
            let r = crate::certs::list_custom_certs();
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );
    // 添加自定义证书（从源路径复制 PEM 文件到 certs 目录）
    d.register(
        "add_custom_cert",
        handler!(_state, _app, params, {
            let p: AddCustomCertParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            crate::certs::add_custom_cert(&p.path)?;
            Ok(serde_json::Value::Null)
        }),
    );
    // 删除自定义证书（按文件名删除 certs 目录下对应文件）
    d.register(
        "remove_custom_cert",
        handler!(_state, _app, params, {
            let p: RemoveCustomCertParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            crate::certs::remove_custom_cert(&p.filename)?;
            Ok(serde_json::Value::Null)
        }),
    );

    // deeplink（3 个）：molaunch:// 协议注册状态查询 / 注册 / 卸载
    // 便携版（未安装）没有安装器自动注册，需在设置页手动触发；
    // 安装版由 NSIS 安装时注册，此处查询结果应显示"已注册（指向当前程序）"
    d.register(
        "get_deeplink_status",
        handler!(_state, _app, _params, {
            let r = crate::deeplink::protocol_status();
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );
    d.register(
        "register_deeplink",
        handler!(_state, _app, _params, {
            crate::deeplink::register_protocol()?;
            let r = crate::deeplink::protocol_status();
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );
    d.register(
        "unregister_deeplink",
        handler!(_state, _app, _params, {
            crate::deeplink::unregister_protocol()?;
            let r = crate::deeplink::protocol_status();
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );

    // ws（1 个）：获取 WebSocket 服务器端口 + 鉴权 token（前端建 WS 连接用）
    // 返回 {port: u16, token: string}，port=0 表示 WS 服务器尚未启动
    // token 用于客户端建连后首条消息鉴权，防止本机其他进程窃听下载进度
    d.register(
        "get_ws_port",
        handler!(state, _app, _params, {
            let port = state.ws_port.get().copied().unwrap_or(0u16);
            let token = state.ws_token.get().cloned().unwrap_or_default();
            Ok(serde_json::json!({
                "port": port,
                "token": token,
            }))
        }),
    );

    d
});

/// 分发入口
pub async fn dispatch(
    state: AppState,
    app: AppHandle,
    req: ActionRequest,
) -> Result<serde_json::Value, String> {
    DISPATCHER.dispatch(state, app, req).await
}