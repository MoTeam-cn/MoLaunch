//! Java 管理统一分发逻辑（java_manager 的工具实现）
//!
//! 使用 `utils::dispatcher::Dispatcher` 注册式分发。
//! 6 个 java action 在 `once_cell::sync::Lazy` 初始化时注册到 DISPATCHER。
//!
//! 命令清单（6 个）：
//! - `detect_java`：检测系统 Java（自动选最佳）
//! - `list_java`：列出所有可用 Java
//! - `select_java_for_mc`：按 MC 版本选最佳 Java
//! - `get_java_requirements`：获取 MC 版本的 Java 需求（支持加载器约束）
//! - `check_java_compatible`：检查指定 Java 是否兼容 MC 版本
//! - `download_java`：下载 Mojang 官方 Java Runtime
//!
//! 注意：
//! - `detect_java` / `list_java` / `select_java_for_mc` 保留 `&AppState` 参数（原签名一致），
//!   handler 内用 `state` 传入；`detect_java` / `list_java` 实际未使用 state
//! - `get_java_requirements` / `check_java_compatible` 不需要 state/app，handler 内用 `_state` / `_app`
//! - `download_java` 需要 state（读取下载源配置）和 app（推送 `java-download-progress` 事件）

use once_cell::sync::Lazy;
use serde::Deserialize;
use tauri::AppHandle;

use crate::commands::java;
use crate::handler;
use crate::state::AppState;
use crate::utils::dispatcher::{ActionRequest, Dispatcher};

// ============================================================
// action 参数
// ============================================================

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

// ============================================================
// Dispatcher 注册
// ============================================================

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
