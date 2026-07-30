//! 系统模块统一分发逻辑（system_manager 的工具实现）
//!
//! 使用 `utils::dispatcher::Dispatcher` 注册式分发，24 个 system action 在
//! `once_cell::sync::Lazy` 初始化时注册到 DISPATCHER。
//! 非 Result 返回的命令（`is_developer_unlocked` / `get_storage_dirs` /
//! `get_system_info` / `get_log_path` / `list_log_files` / `list_custom_certs`）
//! 需在 handler 内用 `serde_json::to_value` 包装。

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

static DISPATCHER: Lazy<Dispatcher> = Lazy::new(|| {
    let mut d = Dispatcher::new();

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

    d.register("get_config_path", handler!(_state, _app, _params, {
        let r = get_config_path().await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));
    d.register("save_config_to_file", handler!(state, _app, _params, {
        save_config_to_file(&state).await?;
        Ok(serde_json::Value::Null)
    }));

    // is_developer_unlocked / get_storage_dirs / get_system_info 返回非 Result，
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

    d.register("get_about_data", handler!(_state, _app, _params, {
        let r = get_about_data().await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    // get_log_path / list_log_files 返回非 Result（String / Vec<String>），
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

    d.register("read_http_logs", handler!(_state, _app, params, {
        let p: ReadHttpLogsParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let r = crate::minecraft::online::http_log::read_http_logs(
            p.date.as_deref(),
            p.limit,
        );
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    d.register("list_http_log_files", handler!(_state, _app, _params, {
        let r = crate::minecraft::online::http_log::list_http_log_files();
        Ok(serde_json::to_value(r).map_err(|e| e.to_string())?)
    }));

    // Windows 便携版自实现 + macOS/Linux 转发官方 plugin
    d.register("check_update", handler!(state, app, _params, {
        let r = updater::check_update(&state, &app).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));
    d.register("download_and_install_update", handler!(_state, app, params, {
        let p: updater::UpdateInfo = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        updater::download_and_install(&app, p).await?;
        Ok(serde_json::Value::Null)
    }));

    // Windows 便携版后台静默下载新版本到 appdata/last.exe
    d.register("download_update_to_appdata", handler!(_state, _app, params, {
        let p: updater::UpdateInfo = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let downloaded = updater::download_update_to_appdata(p).await?;
        Ok(serde_json::to_value(downloaded).map_err(|e| e.to_string())?)
    }));

    // 退出时检查并应用待安装更新（last.exe → 替换主 exe）
    d.register("apply_pending_update", handler!(_state, app, _params, {
        let has_update = updater::apply_pending_update(&app).await?;
        Ok(serde_json::to_value(has_update).map_err(|e| e.to_string())?)
    }));

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
