//! Agent 工具定义与执行（供 AI 调用的诊断工具集）
//!
//! 工具均只读、输出统一截断。版本隔离下各版本数据分目录存放，
//! 读文件工具均需 `version_id`，缺失时提示先调用 `list_installed_versions`。

use serde_json::{json, Value};
use tauri::AppHandle;

use super::{ask, crash, info, logs};
use crate::ai_core::client::{ToolDef, ToolFunction};
use crate::minecraft::isolation::IsolationMode;
use crate::minecraft::version::scan::scan_installed_versions;

/// 工具执行上下文（由 chat_send 构造）
pub struct AgentContext {
    /// 游戏根目录（已解析真实路径）
    pub game_dir: std::path::PathBuf,
    /// 启动器版本号
    pub version: String,
    /// 配置摘要（仅安全字段）
    pub config_summary: String,
    /// 版本隔离模式（用于计算各版本有效目录）
    pub isolation_mode: IsolationMode,
    /// 当前会话 id（ask_user 提问按会话隔离等待）
    pub conversation_id: i64,
    /// Tauri 应用句柄（用于 emit ai-ask-user 事件）
    pub app: AppHandle,
}

/// 解析版本参数（兼容 camelCase `versionId` 与 snake_case `version_id`）
pub(crate) fn version_arg(args: &Value) -> Option<String> {
    let raw = args
        .get("version_id")
        .or_else(|| args.get("versionId"))
        .and_then(|v| v.as_str())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    raw
}

/// 计算版本有效游戏目录（按隔离模式 + 实际目录存在性判断）
///
/// 开启版本隔离时数据在 `game_dir/versions/{id}`；无独立目录（共享模式）时回退根目录。
pub(super) fn effective_dir(ctx: &AgentContext, version_id: &str) -> std::path::PathBuf {
    if ctx.isolation_mode != IsolationMode::Disabled {
        let isolated = ctx.game_dir.join("versions").join(version_id);
        if isolated.exists() {
            return isolated;
        }
    }
    ctx.game_dir.clone()
}

/// 校验版本参数；缺失时返回友好错误，引导模型先调用 list_installed_versions
pub(crate) fn require_version(args: &Value, ctx: &AgentContext) -> Result<String, String> {
    match version_arg(args) {
        Some(v) => Ok(v),
        None => {
            let versions = installed_version_ids(ctx);
            let hint = if versions.is_empty() {
                "当前未扫描到已安装的 Minecraft 版本".to_string()
            } else {
                format!(
                    "请先调用 list_installed_versions 获取版本列表（当前可用: {}）",
                    versions.join(", ")
                )
            };
            Err(format!("缺少 versionId 参数。{}", hint))
        }
    }
}

/// 已安装版本 id 列表（升序去重）
pub(super) fn installed_version_ids(ctx: &AgentContext) -> Vec<String> {
    let infos = scan_installed_versions(&ctx.game_dir);
    let mut ids: Vec<String> = infos.into_iter().map(|i| i.id).collect();
    ids.sort();
    ids.dedup();
    ids
}

/// 工具定义列表（随聊天请求下发）
pub fn tool_definitions() -> Vec<ToolDef> {
    vec![
        tool(
            "get_launcher_info",
            "获取 MoLaunch 启动器版本、游戏目录、版本隔离模式与常用配置摘要",
            json!({"type": "object", "properties": {}, "additionalProperties": false}),
        ),
        tool(
            "list_installed_versions",
            "获取启动器已安装的 Minecraft 版本列表（返回各版本 id）。在调用任何读取游戏数据的工具之前，必须先调用本工具确认版本。",
            json!({"type": "object", "properties": {}, "additionalProperties": false}),
        ),
        tool(
            "read_game_logs",
            "读取指定版本的游戏目录 logs/latest.log 内容，用于排查运行时报错。默认读取末尾 200 行（最多 800 行）。可通过 startLine/endLine 指定行范围精确定位；也可通过 keyword 搜索关键词，返回首次命中行前后各 15 行上下文（带行号）；若日志很长，可设 localAnalyze=true 让本地引擎先初检，返回定位到的问题范围（更省 token）。必须提供 versionId。",
            json!({
                "type": "object",
                "properties": {
                    "versionId": {"type": "string", "description": "已安装的游戏版本 id，必须先调用 list_installed_versions 获取"},
                    "lines": {"type": "integer", "description": "读取末尾行数（不指定 startLine 时生效）", "default": 200},
                    "startLine": {"type": "integer", "description": "起始行号（从 1 起），指定后按行范围读取"},
                    "endLine": {"type": "integer", "description": "结束行号（含），指定 startLine 后生效"},
                    "keyword": {"type": "string", "description": "关键词搜索：返回首次命中该关键词的行前后各 15 行上下文（含行号），与 startLine/endLine 互斥"},
                    "localAnalyze": {"type": "boolean", "description": "true 时先用本地规则引擎初检，返回问题范围摘要而非全文", "default": false}
                },
                "required": ["versionId"],
                "additionalProperties": false
            }),
        ),
        tool(
            "read_crash_report",
            "读取指定版本游戏目录 crash-reports 下最新崩溃报告的文本内容。默认返回全文（截断保护）；可设 localAnalyze=true 让本地引擎先初检，返回定位到的问题范围摘要；或用 startLine/endLine 读取指定行段；或用 keyword 搜索关键词返回命中行前后各 1 行上下文。必须提供 versionId。",
            json!({
                "type": "object",
                "properties": {
                    "versionId": {"type": "string", "description": "已安装的游戏版本 id，必须先调用 list_installed_versions 获取"},
                    "localAnalyze": {"type": "boolean", "description": "true 时先用本地引擎初检，返回问题范围摘要而非全文", "default": false},
                    "startLine": {"type": "integer", "description": "起始行号（从 1 起），指定后按行范围读取"},
                    "endLine": {"type": "integer", "description": "结束行号（含），指定 startLine 后生效"},
                    "keyword": {"type": "string", "description": "关键词搜索：返回首次命中的行前后各 1 3 行上下文（含行号），与 startLine/endLine 互斥"}
                },
                "required": ["versionId"],
                "additionalProperties": false
            }),
        ),
        tool(
            "analyze_crash_log",
            "读取指定版本游戏目录 crash-reports 下最新崩溃报告，用本地规则引擎做初检，返回定位到的问题范围摘要（分类、严重级别、关键行、修复建议）。比直接读全文更省 token、更聚焦；若需要更多上下文，可再调用 read_game_logs 的 startLine/endLine 或 read_log_lines 读取日志对应行段。必须提供 versionId。",
            json!({
                "type": "object",
                "properties": {
                    "versionId": {"type": "string", "description": "已安装的游戏版本 id，必须先调用 list_installed_versions 获取"}
                },
                "required": ["versionId"],
                "additionalProperties": false
            }),
        ),
        tool(
            "read_log_lines",
            "读取指定版本游戏目录 logs/latest.log 的指定行范围（startLine~endLine，从 1 起），用于拿到崩溃报告初检范围后精确读取日志对应位置的关键上下文；也可提供 keyword 按关键词搜索，返回首次命中行前后各 15 行上下文。必须提供 versionId。",
            json!({
                "type": "object",
                "properties": {
                    "versionId": {"type": "string", "description": "已安装的游戏版本 id，必须先调用 list_installed_versions 获取"},
                    "startLine": {"type": "integer", "description": "起始行号（从 1 起）"},
                    "endLine": {"type": "integer", "description": "结束行号（含）"},
                    "keyword": {"type": "string", "description": "关键词搜索：返回首次命中的行前后各 15 行上下文（含行号），提供时忽略 startLine/endLine"}
                },
                "required": ["versionId"],
                "additionalProperties": false
            }),
        ),
        tool(
            "list_installed_mods",
            "列出指定版本游戏目录 mods 文件夹中已安装的 Mod 文件列表。必须提供 versionId。",
            json!({
                "type": "object",
                "properties": {
                    "versionId": {"type": "string", "description": "已安装的游戏版本 id，必须先调用 list_installed_versions 获取"}
                },
                "required": ["versionId"],
                "additionalProperties": false
            }),
        ),
        tool(
            "read_launcher_logs",
            "读取 MoLaunch 启动器自身最新日志的末尾内容（默认 200 行，最多 800 行），用于排查启动器问题",
            json!({
                "type": "object",
                "properties": {
                    "lines": {"type": "integer", "description": "读取行数", "default": 200}
                },
                "additionalProperties": false
            }),
        ),
        tool(
            "ask_user",
            "向用户提问以确认信息（例如：用户在多个版本间未指定版本、需要用户选择或输入内容时）。question 用一句话描述问题，options 为候选答案（最多 6 个，可省略）。调用后等待用户回答，返回用户选择或输入的内容。",
            json!({
                "type": "object",
                "properties": {
                    "question": {"type": "string", "description": "需要向用户确认的问题"},
                    "options": {
                        "type": "array",
                        "description": "候选答案（最多 6 个）。推荐使用对象格式 {\"label\": \"选项文本\", \"description\": \"该选项的说明/注释，帮助用户理解与选择\"}；也可用纯字符串，可省略让用户自由输入",
                        "items": {
                            "oneOf": [
                                {"type": "string"},
                                {
                                    "type": "object",
                                    "properties": {
                                        "label": {"type": "string", "description": "选项显示文本"},
                                        "description": {"type": "string", "description": "该选项的说明/注释，简要解释选项含义，帮助用户理解后选择"}
                                    },
                                    "required": ["label"],
                                    "additionalProperties": false
                                }
                            ]
                        }
                    }
                },
                "required": ["question"],
                "additionalProperties": false
            }),
        ),
    ]
}

fn tool(name: &str, description: &str, parameters: Value) -> ToolDef {
    ToolDef {
        ty: "function".to_string(),
        function: ToolFunction {
            name: name.to_string(),
            description: description.to_string(),
            parameters,
        },
    }
}

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
