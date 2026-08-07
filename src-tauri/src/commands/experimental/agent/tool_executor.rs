use serde_json::Value;

use super::super::{ask, crash, info, logs};
use super::require_version;
use super::AgentContext;

/// 执行工具调用，返回给模型的工具结果文本
pub async fn execute_tool(name: &str, args: &Value, ctx: &AgentContext) -> Result<String, String> {
    match name {
        "get_launcher_info" => Ok(info::launcher_info(ctx)),
        "list_installed_versions" => info::list_installed_versions(ctx),
        "read_game_logs" => {
            let version = require_version(args, ctx)?;
            let lines = args.get("lines").and_then(|v| v.as_i64()).unwrap_or(200) as usize;
            let start = args.get("startLine").and_then(|v| v.as_i64());
            let end = args.get("endLine").and_then(|v| v.as_i64());
            let local_analyze = args
                .get("localAnalyze")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let keyword = args
                .get("keyword")
                .and_then(|v| v.as_str())
                .map(String::from);
            logs::read_game_logs_ex(&version, ctx, lines, start, end, local_analyze, keyword)
        }
        "read_crash_report" => {
            let version = require_version(args, ctx)?;
            let start = args.get("startLine").and_then(|v| v.as_i64());
            let end = args.get("endLine").and_then(|v| v.as_i64());
            let local_analyze = args
                .get("localAnalyze")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let keyword = args
                .get("keyword")
                .and_then(|v| v.as_str())
                .map(String::from);
            crash::read_crash_report_ex(&version, ctx, start, end, local_analyze, keyword)
        }
        "analyze_crash_log" => {
            let version = require_version(args, ctx)?;
            crash::analyze_crash_log(&version, ctx)
        }
        "read_log_lines" => {
            let version = require_version(args, ctx)?;
            let keyword = args
                .get("keyword")
                .and_then(|v| v.as_str())
                .map(String::from);
            if let Some(kw) = keyword.as_deref().filter(|k| !k.trim().is_empty()) {
                return logs::search_log_keyword(&version, ctx, kw);
            }
            let start = args
                .get("startLine")
                .and_then(|v| v.as_i64())
                .ok_or_else(|| "缺少 startLine 参数".to_string())?;
            let end = args
                .get("endLine")
                .and_then(|v| v.as_i64())
                .ok_or_else(|| "缺少 endLine 参数".to_string())?;
            logs::read_log_lines(&version, ctx, start, end)
        }
        "list_installed_mods" => {
            let version = require_version(args, ctx)?;
            info::list_installed_mods(&version, ctx)
        }
        "read_launcher_logs" => {
            let lines = args.get("lines").and_then(|v| v.as_i64()).unwrap_or(200) as usize;
            logs::read_launcher_logs(lines)
        }
        "ask_user" => ask::ask_user(args, ctx).await,
        other => Err(format!("未知工具: {}", other)),
    }
}

/// 收集上下文（手动附加上下文兜底，模型不支持工具调用时由前端调用）
///
/// 需要 `version_id`：因为启动器默认开启版本隔离，各版本数据位于各自目录。
pub fn collect_context(kind: &str, version_id: &str, ctx: &AgentContext) -> Result<String, String> {
    match kind {
        "launcher" => Ok(info::launcher_info(ctx)),
        "game_logs" => logs::read_game_logs(version_id, ctx, 300),
        "crash_report" => crash::read_crash_report(version_id, ctx),
        "mods" => info::list_installed_mods(version_id, ctx),
        "launcher_logs" => logs::read_launcher_logs(300),
        other => Err(format!("未知上下文类型: {}", other)),
    }
}
