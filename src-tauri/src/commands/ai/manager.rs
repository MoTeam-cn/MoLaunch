//! AI action 分发（复用 `utils::dispatcher::Dispatcher` + `handler!` 宏）
//! action：`analyze_crash` / `check_status` / `save_config` / `load_config` / `list_models`。
//! 配置持久化于 config.ini [AI] 段（api_key 经 SDK DES 加密），服务为本地 OpenAI 兼容 API。

use once_cell::sync::Lazy;
use serde_json::Value;
use tauri::AppHandle;

use super::types::{AiAnalysisResult, AiProbeParams, AiStatusResult, AnalyzeCrashParams};
use crate::ai_core;
use crate::handler;
use crate::state::AppState;
use crate::utils::dispatcher::{ActionRequest, Dispatcher};
use crate::{log_info, log_warn};

static DISPATCHER: Lazy<Dispatcher> = Lazy::new(|| {
    let mut d = Dispatcher::new();

    d.register(
        "analyze_crash",
        handler!(_state, _app, params, {
            let p: AnalyzeCrashParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            analyze_crash(p).await
        }),
    );

    d.register(
        "check_status",
        handler!(_state, _app, params, {
            let probe = if params.is_null() {
                None
            } else {
                Some(serde_json::from_value::<AiProbeParams>(params).map_err(|e| format!("参数解析失败: {}", e))?)
            };
            check_status(probe).await
        }),
    );

    d.register(
        "save_config",
        handler!(state, _app, params, {
            let cfg: ai_core::AiConfig = serde_json::from_value(params)
                .map_err(|e| format!("配置解析失败: {}", e))?;
            ai_core::save_config(&state.sdk, &cfg).await?;
            serde_json::to_value(()).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "load_config",
        handler!(_state, _app, _params, {
            let cfg = ai_core::load_config_async().await;
            serde_json::to_value(cfg).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "list_models",
        handler!(_state, _app, params, {
            let p: AiProbeParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            let config = ai_core::AiConfig {
                base_url: p.base_url,
                api_key: p.api_key,
                timeout_secs: p.timeout_secs,
                models: Vec::new(),
                default_model: String::new(),
            };
            let models = ai_core::list_models(&config).await.map_err(|e| e.to_string())?;
            serde_json::to_value(models).map_err(|e| e.to_string())
        }),
    );

    d
});

/// action 分发入口（由 `super::ai_manager` 调用）
pub async fn dispatch(
    state: AppState,
    app: AppHandle,
    req: ActionRequest,
) -> Result<Value, String> {
    DISPATCHER.dispatch(state, app, req).await
}

/// 分析崩溃日志（本地 AI）
async fn analyze_crash(params: AnalyzeCrashParams) -> Result<Value, String> {
    let config = ai_core::load_config_async().await;
    if config.base_url.is_empty() {
        return Err("未配置 AI 服务地址".to_string());
    }
    let model = config.resolve_model(params.model.as_deref());
    if model.is_empty() {
        return Err("未启用任何模型，请先在设置页导入并选择默认模型".to_string());
    }

    let user_content = ai_core::prompt::crash_user_prompt(
        &params.runtime_log,
        &params.error_lines,
        &params.crash_report,
        &params.hs_err,
    );

    let started = std::time::Instant::now();
    log_info!("[AI] 开始本地 AI 分析（model={}）", model);
    let content = ai_core::chat(&config, ai_core::PromptKind::CrashLog, user_content, Some(&model))
        .await
        .map_err(|e| {
            log_warn!("[AI] 分析失败: {}", e);
            e.to_string()
        })?;
    let elapsed_ms = started.elapsed().as_millis() as u64;
    log_info!("[AI] 分析完成，耗时 {}ms", elapsed_ms);

    let result = AiAnalysisResult {
        content,
        model,
        elapsed_ms,
    };
    serde_json::to_value(result).map_err(|e| e.to_string())
}

/// 检测本地 AI 服务是否可用
async fn check_status(probe: Option<AiProbeParams>) -> Result<Value, String> {
    let config = if let Some(p) = probe {
        ai_core::AiConfig {
            base_url: p.base_url,
            api_key: p.api_key,
            timeout_secs: p.timeout_secs,
            models: Vec::new(),
            default_model: String::new(),
        }
    } else {
        ai_core::load_config_async().await
    };
    let base_url = config.base_url.clone();
    let model = config.resolve_model(None);

    // 未配置服务地址时不探测，直接返回不可用
    if base_url.trim().is_empty() {
        let result = AiStatusResult {
            available: false,
            base_url,
            model,
        };
        return serde_json::to_value(result).map_err(|e| e.to_string());
    }

    // 复用 `ai_core::list_models`（带 api_key 认证），外层限时 5s 探测
    let available = match tokio::time::timeout(
        std::time::Duration::from_secs(5),
        ai_core::list_models(&config),
    )
    .await
    {
        Ok(Ok(models)) => !models.is_empty(),
        _ => false,
    };

    let result = AiStatusResult {
        available,
        base_url,
        model,
    };
    serde_json::to_value(result).map_err(|e| e.to_string())
}
