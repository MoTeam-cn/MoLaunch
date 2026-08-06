//! Agent 工具：供 AI 调用的诊断工具集（实验性）
//!
//! 工具均只读、不修改任何游戏/启动器文件。工具结果会被拼回对话继续请求；
//! 输出统一截断，避免超长文本挤爆上下文窗口。
//!
//! 版本约束：启动器默认开启版本隔离，各版本的数据（mods / logs / crash-reports）
//! 分别存放在各自的版本目录中。所有读文件工具都需要 `version_id` 参数，
//! 缺失时返回错误并提示模型先调用 `list_installed_versions`。

use once_cell::sync::Lazy;
use serde_json::json;
use std::collections::HashMap;
use tauri::{AppHandle, Emitter};
use tokio::sync::{oneshot, Mutex};

use crate::ai_core::client::{ToolDef, ToolFunction};
use crate::minecraft::isolation::IsolationMode;
use crate::minecraft::version::scan::scan_installed_versions;
use crate::utils::format::truncate_chars;

/// ask_user 等待超时（秒）
const ASK_USER_TIMEOUT_SECS: u64 = 120;

/// ask_user 等待队列：conversation_id → (reply 发送端, 等待端)
pub static ASK_USER_QUEUE: Lazy<Mutex<HashMap<i64, oneshot::Sender<String>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

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

/// 版本相关参数统一解析（兼容 camelCase `versionId` 与 snake_case `version_id`）
fn version_arg(args: &serde_json::Value) -> Option<String> {
    let raw = args
        .get("version_id")
        .or_else(|| args.get("versionId"))
        .and_then(|v| v.as_str())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    raw
}

/// 计算某个版本的有效游戏目录（按隔离模式 + 实际目录存在性判断）
///
/// 启动器开启版本隔离时数据在 `game_dir/versions/{id}`；未开启或该版本
/// 无独立目录（共享目录模式）时回退到 `game_dir` 根目录。
fn effective_dir(ctx: &AgentContext, version_id: &str) -> std::path::PathBuf {
    if ctx.isolation_mode != IsolationMode::Disabled {
        let isolated = ctx.game_dir.join("versions").join(version_id);
        if isolated.exists() {
            return isolated;
        }
    }
    ctx.game_dir.clone()
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
            "读取指定版本的游戏目录 logs/latest.log 内容，用于排查运行时报错。默认读取末尾 200 行（最多 800 行）。可通过 startLine/endLine 指定行范围精确定位；若日志很长，可设 localAnalyze=true 让本地引擎先初检，返回定位到的问题范围（更省 token）。必须提供 versionId。",
            json!({
                "type": "object",
                "properties": {
                    "versionId": {"type": "string", "description": "已安装的游戏版本 id，必须先调用 list_installed_versions 获取"},
                    "lines": {"type": "integer", "description": "读取末尾行数（不指定 startLine 时生效）", "default": 200},
                    "startLine": {"type": "integer", "description": "起始行号（从 1 起），指定后按行范围读取"},
                    "endLine": {"type": "integer", "description": "结束行号（含），指定 startLine 后生效"},
                    "localAnalyze": {"type": "boolean", "description": "true 时先用本地规则引擎初检，返回问题范围摘要而非全文", "default": false}
                },
                "required": ["versionId"],
                "additionalProperties": false
            }),
        ),
        tool(
            "read_crash_report",
            "读取指定版本游戏目录 crash-reports 下最新崩溃报告的文本内容。默认返回全文（截断保护）；可设 localAnalyze=true 让本地引擎先初检，返回定位到的问题范围摘要；或用 startLine/endLine 读取指定行段。必须提供 versionId。",
            json!({
                "type": "object",
                "properties": {
                    "versionId": {"type": "string", "description": "已安装的游戏版本 id，必须先调用 list_installed_versions 获取"},
                    "localAnalyze": {"type": "boolean", "description": "true 时先用本地规则引擎初检，返回问题范围摘要而非全文", "default": false},
                    "startLine": {"type": "integer", "description": "起始行号（从 1 起），指定后按行范围读取"},
                    "endLine": {"type": "integer", "description": "结束行号（含），指定 startLine 后生效"}
                },
                "required": ["versionId"],
                "additionalProperties": false
            }),
        ),
        tool(
            "analyze_crash_log",
            "读取指定版本游戏目录 crash-reports 下最新崩溃报告，用本地规则引擎做初检，返回定位到的问题范围摘要（分类、严重级别、关键行、修复建议）。比直接读全文更省 token、更聚焦；若需要更多上下文，可再用 read_game_logs 的 startLine/endLine 或 read_log_lines 读取日志对应行段。必须提供 versionId。",
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
            "读取指定版本游戏目录 logs/latest.log 的指定行范围（startLine~endLine，从 1 起）。用于拿到崩溃报告初检范围后，精确读取日志对应位置的关键上下文。必须提供 versionId。",
            json!({
                "type": "object",
                "properties": {
                    "versionId": {"type": "string", "description": "已安装的游戏版本 id，必须先调用 list_installed_versions 获取"},
                    "startLine": {"type": "integer", "description": "起始行号（从 1 起）"},
                    "endLine": {"type": "integer", "description": "结束行号（含）"}
                },
                "required": ["versionId", "startLine", "endLine"],
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
                    "options": {"type": "array", "items": {"type": "string"}, "description": "候选答案（最多 6 个），可省略让用户自由输入"}
                },
                "required": ["question"],
                "additionalProperties": false
            }),
        ),
    ]
}

fn tool(name: &str, description: &str, parameters: serde_json::Value) -> ToolDef {
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
pub async fn execute_tool(
    name: &str,
    args: &serde_json::Value,
    ctx: &AgentContext,
) -> Result<String, String> {
    match name {
        "get_launcher_info" => Ok(launcher_info(ctx)),
        "list_installed_versions" => list_installed_versions(ctx),
        "read_game_logs" => {
            let version = require_version(args, ctx)?;
            let lines = args.get("lines").and_then(|v| v.as_i64()).unwrap_or(200) as usize;
            let start = args.get("startLine").and_then(|v| v.as_i64());
            let end = args.get("endLine").and_then(|v| v.as_i64());
            let local_analyze = args.get("localAnalyze").and_then(|v| v.as_bool()).unwrap_or(false);
            read_game_logs_ex(&version, ctx, lines, start, end, local_analyze)
        }
        "read_crash_report" => {
            let version = require_version(args, ctx)?;
            let start = args.get("startLine").and_then(|v| v.as_i64());
            let end = args.get("endLine").and_then(|v| v.as_i64());
            let local_analyze = args.get("localAnalyze").and_then(|v| v.as_bool()).unwrap_or(false);
            read_crash_report_ex(&version, ctx, start, end, local_analyze)
        }
        "analyze_crash_log" => {
            let version = require_version(args, ctx)?;
            analyze_crash_log(&version, ctx)
        }
        "read_log_lines" => {
            let version = require_version(args, ctx)?;
            let start = args.get("startLine").and_then(|v| v.as_i64()).ok_or_else(|| "缺少 startLine 参数".to_string())?;
            let end = args.get("endLine").and_then(|v| v.as_i64()).ok_or_else(|| "缺少 endLine 参数".to_string())?;
            read_log_lines(&version, ctx, start, end)
        }
        "list_installed_mods" => {
            let version = require_version(args, ctx)?;
            list_installed_mods(&version, ctx)
        }
        "read_launcher_logs" => {
            let lines = args.get("lines").and_then(|v| v.as_i64()).unwrap_or(200) as usize;
            read_launcher_logs(lines)
        }
        "ask_user" => ask_user(args, ctx).await,
        other => Err(format!("未知工具: {}", other)),
    }
}

/// ask_user：emit ai-ask-user 事件并等待用户回答
async fn ask_user(args: &serde_json::Value, ctx: &AgentContext) -> Result<String, String> {
    let question = args
        .get("question")
        .and_then(|v| v.as_str())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "ask_user 缺少 question 参数".to_string())?;

    // options 兼容纯字符串 `"选项"` 与对象 `{"label": "...", "description": "备注"}`，
    // 统一归一化为 `{label, description?}` 后透传前端展示
    let options: Vec<serde_json::Value> = args
        .get("options")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| {
                    if let Some(s) = v.as_str() {
                        Some(json!({ "label": s }))
                    } else if v.get("label").and_then(|l| l.as_str()).is_some() {
                        Some(v.clone())
                    } else {
                        None
                    }
                })
                .take(6)
                .collect()
        })
        .unwrap_or_default();

    // 注册等待通道
    let (tx, rx) = oneshot::channel::<String>();
    {
        let mut queue = ASK_USER_QUEUE.lock().await;
        // 同一会话已有提问未回答时，先移除旧的（防堆积）
        let _ = queue.insert(ctx.conversation_id, tx);
    }

    // 通知前端弹窗询问
    let _ = ctx.app.emit(
        "ai-ask-user",
        json!({
            "conversationId": ctx.conversation_id,
            "question": question,
            "options": options
        }),
    );

    // 等待回答（超时保护，防止模型提问后无人回应挂死工具循环）
    let wait_outcome = tokio::time::timeout(
        std::time::Duration::from_secs(ASK_USER_TIMEOUT_SECS),
        rx,
    )
    .await;
    match wait_outcome {
        Ok(Ok(reply)) => {
            let reply = reply.trim().to_string();
            if reply.is_empty() {
                Ok("（用户未提供有效回答）".to_string())
            } else {
                Ok(format!("用户回答: {}", reply))
            }
        }
        Ok(Err(_)) => {
            // 发送端被丢弃（例如会话被切换），清理队列
            let mut queue = ASK_USER_QUEUE.lock().await;
            queue.remove(&ctx.conversation_id);
            Err("提问等待被中断".to_string())
        }
        Err(_) => {
            let mut queue = ASK_USER_QUEUE.lock().await;
            queue.remove(&ctx.conversation_id);
            Err("向用户提问超时（120s 内未收到回答）".to_string())
        }
    }
}

/// 回填 ask_user 回答（由 `reply_ask_user` action 调用）
pub async fn reply_ask_user(conversation_id: i64, reply: String) -> Result<(), String> {
    let mut queue = ASK_USER_QUEUE.lock().await;
    if let Some(sender) = queue.remove(&conversation_id) {
        let _ = sender.send(reply);
        Ok(())
    } else {
        Err("没有正在等待的回答".to_string())
    }
}

/// 校验版本参数；缺失时返回友好错误，引导模型先调用 list_installed_versions
fn require_version(args: &serde_json::Value, ctx: &AgentContext) -> Result<String, String> {
    match version_arg(args) {
        Some(v) => Ok(v),
        None => {
            let versions = installed_version_ids(ctx);
            let hint = if versions.is_empty() {
                "当前未扫描到已安装的 Minecraft 版本".to_string()
            } else {
                format!("请先调用 list_installed_versions 获取版本列表（当前可用: {}）", versions.join(", "))
            };
            Err(format!("缺少 versionId 参数。{}", hint))
        }
    }
}

/// 收集上下文（手动附加上下文兜底，模型不支持工具调用时由前端调用）
///
/// 需要 `version_id`：因为启动器默认开启版本隔离，各版本数据位于各自目录。
pub fn collect_context(kind: &str, version_id: &str, ctx: &AgentContext) -> Result<String, String> {
    match kind {
        "launcher" => Ok(launcher_info(ctx)),
        "game_logs" => read_game_logs(version_id, ctx, 300),
        "crash_report" => read_crash_report(version_id, ctx),
        "mods" => list_installed_mods(version_id, ctx),
        "launcher_logs" => read_launcher_logs(300),
        other => Err(format!("未知上下文类型: {}", other)),
    }
}

fn installed_version_ids(ctx: &AgentContext) -> Vec<String> {
    let infos = scan_installed_versions(&ctx.game_dir);
    let mut ids: Vec<String> = infos.into_iter().map(|i| i.id).collect();
    ids.sort();
    ids.dedup();
    ids
}

fn list_installed_versions(ctx: &AgentContext) -> Result<String, String> {
    let ids = installed_version_ids(ctx);
    if ids.is_empty() {
        return Ok("（未扫描到已安装的 Minecraft 版本，游戏目录可能为空）".to_string());
    }
    let mut text = format!("已安装的 Minecraft 版本共 {} 个：\n", ids.len());
    for id in ids {
        text.push_str(&format!("- {}\n", id));
    }
    Ok(text)
}

fn launcher_info(ctx: &AgentContext) -> String {
    format!(
        "MoLaunch 启动器信息\n版本: {}\n游戏目录: {}\n{}\n",
        ctx.version,
        ctx.game_dir.display(),
        ctx.config_summary
    )
}

fn read_game_logs(version_id: &str, ctx: &AgentContext, lines: usize) -> Result<String, String> {
    read_game_logs_ex(version_id, ctx, lines, None, None, false)
}

/// 读取游戏日志：支持末尾 N 行 / startLine~endLine 行范围 / localAnalyze 本地预检
fn read_game_logs_ex(
    version_id: &str,
    ctx: &AgentContext,
    lines: usize,
    start: Option<i64>,
    end: Option<i64>,
    local_analyze: bool,
) -> Result<String, String> {
    let dir = effective_dir(ctx, version_id);
    let path = dir.join("logs").join("latest.log");
    if !path.exists() {
        return Ok(format!(
            "（未找到版本 {} 的 latest.log，可能尚未启动过游戏或已清理日志）",
            version_id
        ));
    }
    let content = std::fs::read_to_string(&path).map_err(|e| format!("读取日志失败: {}", e))?;

    // 本地预检：规则引擎初检，返回问题范围摘要（不返回全文，省 token）
    if local_analyze {
        return Ok(local_precheck_summary(version_id, &content, &path));
    }

    // 指定行范围读取
    if let (Some(s), Some(e)) = (start, end) {
        let snippet = read_line_range(&content, s, e, 4000)?;
        return Ok(format!(
            "【版本 {} 的 logs/latest.log 第 {}~{} 行】\n{}",
            version_id, s, e, snippet
        ));
    }

    let tail = read_tail(&path, lines, 6000)?;
    Ok(format!(
        "【版本 {} 的 logs/latest.log 末尾 {} 行】\n{}",
        version_id, lines, tail
    ))
}

/// 本地预检摘要：用规则引擎初检日志文本，返回收敛后的问题范围（省 token）
fn local_precheck_summary(version_id: &str, content: &str, path: &std::path::Path) -> String {
    let items = crate::commands::tools::crash_analyzer::analyze_log_text(content);
    if items.is_empty() {
        return format!(
            "【版本 {} 的 {} 本地预检结果】\n未识别到已知崩溃模式（日志正常或错误特征未命中规则）。若需要，可读取原文进一步排查。",
            version_id,
            path.file_name().unwrap_or_default().to_string_lossy()
        );
    }
    let mut text = format!(
        "【版本 {} 的 {} 本地预检结果】检测到 {} 个可能问题：\n",
        version_id,
        path.file_name().unwrap_or_default().to_string_lossy(),
        items.len()
    );
    for (i, it) in items.iter().enumerate() {
        text.push_str(&format!(
            "{}. [{}][{}] {}\n",
            i + 1,
            it.category,
            it.severity,
            it.title
        ));
        if !it.detail.is_empty() {
            text.push_str(&format!("   关键行: {}\n", it.detail));
        }
        if !it.suggestion.is_empty() {
            text.push_str(&format!("   建议: {}\n", it.suggestion));
        }
    }
    text.push_str("\n若需要更多上下文，可调用 read_log_lines 读取日志对应行段。");
    text
}

/// 读取日志文件指定行范围（从 1 起，含首尾），超长截断
fn read_line_range(content: &str, start: i64, end: i64, max_chars: usize) -> Result<String, String> {
    if start < 1 || end < start {
        return Err(format!("行范围参数无效: startLine={} endLine={}（需 start>=1 且 end>=start）", start, end));
    }
    let all: Vec<&str> = content.lines().collect();
    if start as usize > all.len() {
        return Ok(format!("（日志共 {} 行，起始行 {} 超出范围）", all.len(), start));
    }
    let end_u = (end as usize).min(all.len());
    let lines = &all[(start as usize - 1)..end_u];
    let joined = lines.join("\n");
    let trimmed = truncate_chars(&joined, max_chars);
    Ok(format!("（共 {} 行）\n{}", lines.len(), trimmed))
}

/// 读取游戏日志的指定行段（日志分析页 / AI 工具复用入口）
fn read_log_lines(version_id: &str, ctx: &AgentContext, start: i64, end: i64) -> Result<String, String> {
    let dir = effective_dir(ctx, version_id);
    let path = dir.join("logs").join("latest.log");
    if !path.exists() {
        return Ok(format!(
            "（未找到版本 {} 的 latest.log，可能尚未启动过游戏或已清理日志）",
            version_id
        ));
    }
    let content = std::fs::read_to_string(&path).map_err(|e| format!("读取日志失败: {}", e))?;
    let snippet = read_line_range(&content, start, end, 6000)?;
    Ok(format!(
        "【版本 {} 的 logs/latest.log 第 {}~{} 行】\n{}",
        version_id, start, end, snippet
    ))
}

fn read_crash_report(version_id: &str, ctx: &AgentContext) -> Result<String, String> {
    read_crash_report_ex(version_id, ctx, None, None, false)
}

/// 读取最新崩溃报告：支持全文（截断）/ 行范围 / localAnalyze 本地预检
fn read_crash_report_ex(
    version_id: &str,
    ctx: &AgentContext,
    start: Option<i64>,
    end: Option<i64>,
    local_analyze: bool,
) -> Result<String, String> {
    let dir = effective_dir(ctx, version_id).join("crash-reports");
    let Some(newest) = newest_file(&dir, Some("txt")) else {
        return Ok(format!(
            "（版本 {} 的 crash-reports 目录不存在或为空，游戏可能未崩溃过）",
            version_id
        ));
    };
    let content = std::fs::read_to_string(&newest).map_err(|e| format!("读取崩溃报告失败: {}", e))?;

    if local_analyze {
        return Ok(local_precheck_summary(version_id, &content, &newest));
    }

    if let (Some(s), Some(e)) = (start, end) {
        let snippet = read_line_range(&content, s, e, 6000)?;
        return Ok(format!(
            "【版本 {} 最新崩溃报告 {} 第 {}~{} 行】\n{}",
            version_id,
            newest.file_name().unwrap_or_default().to_string_lossy(),
            s,
            e,
            snippet
        ));
    }

    let trimmed = truncate_chars(&content, 6000);
    Ok(format!(
        "【版本 {} 最新崩溃报告 {}】\n{}",
        version_id,
        newest.file_name().unwrap_or_default().to_string_lossy(),
        trimmed
    ))
}

/// 分析崩溃日志：读取最新崩溃报告 → 本地预检 → 返回问题范围摘要
fn analyze_crash_log(version_id: &str, ctx: &AgentContext) -> Result<String, String> {
    let dir = effective_dir(ctx, version_id).join("crash-reports");
    let Some(newest) = newest_file(&dir, Some("txt")) else {
        return Ok(format!(
            "（版本 {} 的 crash-reports 目录不存在或为空，游戏可能未崩溃过）",
            version_id
        ));
    };
    let content = std::fs::read_to_string(&newest).map_err(|e| format!("读取崩溃报告失败: {}", e))?;
    Ok(local_precheck_summary(version_id, &content, &newest))
}

fn list_installed_mods(version_id: &str, ctx: &AgentContext) -> Result<String, String> {
    let dir = effective_dir(ctx, version_id).join("mods");
    if !dir.exists() {
        return Ok(format!("（版本 {} 的 mods 目录不存在）", version_id));
    }
    let mut names: Vec<String> = std::fs::read_dir(&dir)
        .map_err(|e| format!("读取 mods 目录失败: {}", e))?
        .flatten()
        .filter(|e| {
            let name = e.file_name().to_string_lossy().to_lowercase();
            name.ends_with(".jar") || name.ends_with(".disabled")
        })
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();
    names.sort();
    let count = names.len();
    if count == 0 {
        return Ok(format!("（版本 {} 的 mods 目录为空）", version_id));
    }
    let mut text = format!("版本 {} 已安装 Mod 共 {} 个：\n", version_id, count);
    for n in names {
        text.push_str(&format!("- {}\n", n));
    }
    Ok(truncate_chars(&text, 3000))
}

fn read_launcher_logs(lines: usize) -> Result<String, String> {
    let storage = crate::storage::Storage::instance();
    let logs_dir = storage.logs_dir();
    let mut files: Vec<std::path::PathBuf> = std::fs::read_dir(&logs_dir)
        .map_err(|e| format!("读取启动器日志目录失败: {}", e))?
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().map(|x| x == "log").unwrap_or(false))
        .collect();
    if files.is_empty() {
        return Ok("（未找到启动器日志文件）".to_string());
    }
    files.sort_by_key(|p| p.metadata().and_then(|m| m.modified()).ok());
    let newest = files.last().expect("files 非空");
    let raw = std::fs::read_to_string(newest).map_err(|e| format!("读取启动器日志失败: {}", e))?;
    // 复用日志脱敏，避免把 token 等敏感信息传给模型
    let sanitized = crate::logger::sanitize_sensitive_info(&raw);
    // 先取末尾 N 行，再截断字符数（截断保留的是行首，即最新内容，避免只拿到日志中间一段）
    let trimmed = sanitized
        .lines()
        .rev()
        .take(lines)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<Vec<_>>()
        .join("\n");
    let tail = truncate_chars(&trimmed, 6000);
    Ok(format!(
        "【启动器日志 {} 末尾 {} 行】\n{}",
        newest.file_name().unwrap_or_default().to_string_lossy(),
        lines,
        tail
    ))
}

/// 读取文件末尾 N 行（超长时截断字符数，保留最新内容）
fn read_tail(path: &std::path::Path, lines: usize, max_chars: usize) -> Result<String, String> {
    let content = std::fs::read_to_string(path).map_err(|e| format!("读取日志失败: {}", e))?;
    // 先取末尾 N 行，再截断：`truncate_chars` 保留行首（最新内容），
    // 若先截断再取尾行，得到的是日志中间一段，会丢失真正的末尾信息
    let tail = content
        .lines()
        .rev()
        .take(lines)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<Vec<_>>()
        .join("\n");
    Ok(truncate_chars(&tail, max_chars))
}

/// 目录中修改时间最新的文件（按扩展名过滤，可选）
fn newest_file(dir: &std::path::Path, ext: Option<&str>) -> Option<std::path::PathBuf> {
    std::fs::read_dir(dir)
        .ok()?
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_file())
        .filter(|p| {
            ext.map(|e| p.extension().map(|x| x == e).unwrap_or(false))
                .unwrap_or(true)
        })
        .max_by_key(|p| p.metadata().and_then(|m| m.modified()).ok())
}
