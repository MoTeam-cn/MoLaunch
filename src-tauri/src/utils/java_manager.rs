//! Java 管理统一分发逻辑（java_manager 的工具实现）
//!
//! 使用 `utils::dispatcher::Dispatcher` 注册式分发，6 个 action：
//! `detect_java` / `list_java` / `select_java_for_mc` / `get_java_requirements` /
//! `check_java_compatible` / `download_java`。
//! `get_java_requirements` / `check_java_compatible` 不需要 state/app；
//! `download_java` 需要 state（读下载源）和 app（emit `java-download-progress`）。

use once_cell::sync::Lazy;
use serde::Deserialize;
use tauri::AppHandle;

use crate::commands::java;
use crate::handler;
use crate::state::AppState;
use crate::utils::dispatcher::{ActionRequest, Dispatcher};


#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SelectJavaForMcParams {
    mc_version: String,
    user_java_path: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GetJavaRequirementsParams {
    mc_version: String,
    loader: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CheckJavaCompatibleParams {
    java_path: String,
    mc_version: String,
    loader: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DownloadJavaParams {
    target_major: u32,
}


static DISPATCHER: Lazy<Dispatcher> = Lazy::new(|| {
    let mut d = Dispatcher::new();

    d.register("detect_java", handler!(state, _app, _params, {
        let r = java::detect_java(&state).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    d.register("list_java", handler!(state, _app, _params, {
        let r = java::list_java(&state).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    d.register("select_java_for_mc", handler!(state, _app, params, {
        let p: SelectJavaForMcParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let r = java::select_java_for_mc(p.mc_version, p.user_java_path, &state).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    d.register("get_java_requirements", handler!(_state, _app, params, {
        let p: GetJavaRequirementsParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let r = java::get_java_requirements(p.mc_version, p.loader).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    d.register("check_java_compatible", handler!(_state, _app, params, {
        let p: CheckJavaCompatibleParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let r = java::check_java_compatible(p.java_path, p.mc_version, p.loader).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    d.register("download_java", handler!(state, app, params, {
        let p: DownloadJavaParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let r = java::download_java(p.target_major, &app, &state).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
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
