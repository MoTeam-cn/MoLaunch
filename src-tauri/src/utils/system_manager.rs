//! 系统模块统一分发逻辑（system_manager 的工具实现）
//!
//! 使用 `utils::dispatcher::Dispatcher` 注册式分发，替代每个 action 一条 Tauri 命令。
//! 24 个 system action 在 `once_cell::sync::Lazy` 初始化时注册到 DISPATCHER。
//!
//! 命令清单（24 个，按子模块分组）：
//! - game_dir（7 个）：`open_game_dir` / `open_path` / `reveal_in_explorer`
//!   / `get_game_dir` / `write_text_file` / `get_system_memory` / `set_game_dir`
//! - config（2 个）：`get_config_path` / `save_config_to_file`
//! - developer（5 个）：`is_developer_unlocked` / `unlock_developer_mode`
//!   / `get_storage_dirs` / `get_system_info` / `get_cache_stats`
//! - about（1 个）：`get_about_data`
//! - logger（3 个）：`get_log_path` / `list_log_files` / `read_log_file`
//! - http_log（2 个）：`read_http_logs` / `list_http_log_files`
//! - updater（2 个）：`check_update` / `download_and_install_update`
//! - certs（3 个）：`list_custom_certs` / `add_custom_cert` / `remove_custom_cert`
//! - ws（1 个）：`get_ws_port`（获取下载进度推送 WS 端口）
//!
//! 注意事项：
//! - 子模块函数接收 `&AppState`（或不接收 state），handler 内调用时用 `&state` / 忽略 `_state`
//! - `open_game_dir` / `get_game_dir` / `save_config_to_file` / `set_game_dir` 需要 state
//! - `open_path` / `reveal_in_explorer` / `write_text_file` / `read_log_file` 需要参数
//! - `is_developer_unlocked` 返回 bool（非 Result），handler 内用
//!   `Ok(serde_json::to_value(x).map_err(|e| e.to_string())?)` 包装
//! - `get_storage_dirs` 返回 `StorageDirs`（非 Result），同样包装
//! - `get_system_info` 返回 `SystemInfo`（非 Result），同样包装
//! - `get_log_path` 返回 `String`（非 Result），同样包装
//! - `list_log_files` 返回 `Vec<String>`（非 Result），同样包装
//! - `list_custom_certs` 返回 `Vec<CustomCertInfo>`（非 Result），同样包装

use once_cell::sync::Lazy;
use serde::Deserialize;
use tauri::AppHandle;

use crate::commands::system::{
    about::get_about_data,
    config::{get_config_path, save_config_to_file},
    developer::{
        get_cache_stats, get_storage_dirs, get_system_info, is_developer_unlocked,
        unlock_developer_mode,
    },
    game_dir::{
        get_game_dir, get_system_memory, open_game_dir, open_path, reveal_in_explorer,
        set_game_dir, write_text_file,
    },
    updater,
};
use crate::handler;
use crate::logger::{get_log_path, list_log_files, read_log_file};
use crate::state::AppState;
use crate::utils::dispatcher::{ActionRequest, Dispatcher};

// ============================================================
// action 参数
// ============================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PathParams {
    path: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WriteTextFileParams {
    path: String,
    content: String,
}

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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SetGameDirParams {
    game_dir: String,
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

// ============================================================
// Dispatcher 注册
// ============================================================

static DISPATCHER: Lazy<Dispatcher> = Lazy::new(|| {
    let mut d = Dispatcher::new();

    // === game_dir（6 个） ===
    d.register("open_game_dir", handler!(state, _app, _params, {
        open_game_dir(&state).await?;
        Ok(serde_json::Value::Null)
    }));
    d.register("open_path", handler!(_state, _app, params, {
        let p: PathParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        open_path(p.path).await?;
        Ok(serde_json::Value::Null)
    }));
    d.register("reveal_in_explorer", handler!(_state, _app, params, {
        let p: PathParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        reveal_in_explorer(p.path).await?;
        Ok(serde_json::Value::Null)
    }));
    d.register("get_game_dir", handler!(state, _app, _params, {
        let r = get_game_dir(&state).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));
    d.register("write_text_file", handler!(_state, _app, params, {
        let p: WriteTextFileParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        write_text_file(p.path, p.content).await?;
        Ok(serde_json::Value::Null)
    }));
    d.register("get_system_memory", handler!(_state, _app, _params, {
        let r = get_system_memory().await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));
    d.register("set_game_dir", handler!(state, _app, params, {
        let p: SetGameDirParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        set_game_dir(&state, p.game_dir).await?;
        Ok(serde_json::Value::Null)
    }));

    // === config（2 个） ===
    d.register("get_config_path", handler!(_state, _app, _params, {
        let r = get_config_path().await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));
    d.register("save_config_to_file", handler!(state, _app, _params, {
        save_config_to_file(&state).await?;
        Ok(serde_json::Value::Null)
    }));

    // === developer（5 个） ===
    // 注意：is_developer_unlocked / get_storage_dirs / get_system_info 返回非 Result，
    // handler 内用 Ok(to_value(r)?) 包装。
    d.register("is_developer_unlocked", handler!(_state, _app, _params, {
        let r = is_developer_unlocked();
        Ok(serde_json::to_value(r).map_err(|e| e.to_string())?)
    }));
    d.register("unlock_developer_mode", handler!(_state, _app, _params, {
        unlock_developer_mode()?;
        Ok(serde_json::Value::Null)
    }));
    d.register("get_storage_dirs", handler!(_state, _app, _params, {
        let r = get_storage_dirs();
        Ok(serde_json::to_value(r).map_err(|e| e.to_string())?)
    }));
    d.register("get_system_info", handler!(_state, _app, _params, {
        let r = get_system_info();
        Ok(serde_json::to_value(r).map_err(|e| e.to_string())?)
    }));
    d.register("get_cache_stats", handler!(_state, _app, _params, {
        let r = get_cache_stats().await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    // === about（1 个） ===
    d.register("get_about_data", handler!(_state, _app, _params, {
        let r = get_about_data().await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    // === logger（3 个） ===
    // 注意：get_log_path / list_log_files 返回非 Result（String / Vec<String>），
    // handler 内用 Ok(to_value(r)?) 包装。read_log_file 返回 Result<String>。
    d.register("get_log_path", handler!(_state, _app, _params, {
        let r = get_log_path();
        Ok(serde_json::to_value(r).map_err(|e| e.to_string())?)
    }));
    d.register("list_log_files", handler!(_state, _app, _params, {
        let r = list_log_files();
        Ok(serde_json::to_value(r).map_err(|e| e.to_string())?)
    }));
    d.register("read_log_file", handler!(_state, _app, params, {
        let p: ReadLogFileParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let r = read_log_file(p.filename)?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    // === HTTP 请求日志（联机 API 调用追踪） ===
    // 读取指定日期的 HTTP 日志（结构化），供开发者模式表格展示
    d.register("read_http_logs", handler!(_state, _app, params, {
        let p: ReadHttpLogsParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let r = crate::minecraft::online::http_log::read_http_logs(
            p.date.as_deref(),
            p.limit,
        );
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));
    // 列出所有 HTTP 日志文件名（http_*.log，最新的在前）
    d.register("list_http_log_files", handler!(_state, _app, _params, {
        let r = crate::minecraft::online::http_log::list_http_log_files();
        Ok(serde_json::to_value(r).map_err(|e| e.to_string())?)
    }));

    // === updater（2 个） ===
    // Windows 便携版自实现 + macOS/Linux 转发官方 plugin
    // See: docs/updater/design.md §4
    d.register("check_update", handler!(_state, app, _params, {
        let r = updater::check_update(&app).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));
    d.register("download_and_install_update", handler!(_state, app, params, {
        let p: updater::UpdateInfo = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        updater::download_and_install(&app, p).await?;
        Ok(serde_json::Value::Null)
    }));

    // === certs（3 个）=== 自定义 TLS 证书管理
    // 列出 certs 目录下所有 .pem 文件（含 subject / not_after 元信息）
    // 返回 Vec<CustomCertInfo>（非 Result），handler 内用 Ok(to_value(r)?) 包装
    d.register("list_custom_certs", handler!(_state, _app, _params, {
        let r = crate::certs::list_custom_certs();
        Ok(serde_json::to_value(r).map_err(|e| e.to_string())?)
    }));
    // 添加自定义证书（从源路径复制 PEM 文件到 certs 目录）
    d.register("add_custom_cert", handler!(_state, _app, params, {
        let p: AddCustomCertParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        crate::certs::add_custom_cert(&p.path)?;
        Ok(serde_json::Value::Null)
    }));
    // 删除自定义证书（按文件名删除 certs 目录下对应文件）
    d.register("remove_custom_cert", handler!(_state, _app, params, {
        let p: RemoveCustomCertParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        crate::certs::remove_custom_cert(&p.filename)?;
        Ok(serde_json::Value::Null)
    }));

    // ws（1 个）：获取 WebSocket 服务器端口 + 鉴权 token（前端建 WS 连接用）
    // 返回 {port: u16, token: string}，port=0 表示 WS 服务器尚未启动
    // token 用于客户端建连后首条消息鉴权，防止本机其他进程窃听下载进度
    d.register("get_ws_port", handler!(state, _app, _params, {
        let port = state.ws_port.get().copied().unwrap_or(0u16);
        let token = state.ws_token.get().cloned().unwrap_or_default();
        Ok(serde_json::json!({
            "port": port,
            "token": token,
        }))
    }));

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
