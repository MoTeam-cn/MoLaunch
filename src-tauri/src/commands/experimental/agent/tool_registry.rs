use crate::ai_core::client::{ToolDef, ToolFunction};
use serde_json::{json, Value};

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
