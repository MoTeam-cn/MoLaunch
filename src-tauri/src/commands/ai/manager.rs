//! AI 实现库（分析 / 状态查询）
//!
//! 自「实验性」功能上线后，AI 的 IPC action（analyze_crash / check_status /
//! save_config / load_config / list_models）已并入
//! `commands::experimental::manager` 的统一分发，本模块不再提供独立
//! Tauri 命令入口，仅保留被复用的纯实现函数。
//!
//! 配置持久化于 config.ini [AI] 段（api_key 经 SDK DES 加密），
//! 服务为本地 OpenAI 兼容 API。

use super::types::{AiAnalysisResult, AiProbeParams, AiStatusResult, AnalyzeCrashParams};
use crate::ai_core;
use crate::{log_info, log_warn};

/// 分析崩溃日志（本地 AI）
///
/// 被 `commands::experimental` 的 `analyze_crash` action 调用。
pub(crate) async fn analyze_crash(params: AnalyzeCrashParams) -> Result<serde_json::Value, String> {
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
///
/// 被 `commands::experimental` 的 `check_status` action 调用；
/// 传入 `Some(probe)` 时使用表单当前值探测，否则读取已保存配置。
pub(crate) async fn check_status(
    probe: Option<AiProbeParams>,
) -> Result<serde_json::Value, String> {
    let config = if let Some(p) = probe {
        ai_core::AiConfig {
            base_url: p.base_url,
            api_key: p.api_key,
            timeout_secs: p.timeout_secs,
            models: Vec::new(),
            default_model: String::new(),
            max_input_tokens: 184_000,
            max_output_tokens: 16_000,
            icon_color_mode: "color".to_string(),
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
