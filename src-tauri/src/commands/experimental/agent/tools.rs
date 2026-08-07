//! Agent 工具上下文与公共辅助（供 AI 调用的诊断工具集）
//!
//! 工具均只读、输出统一截断。版本隔离下各版本数据分目录存放，
//! 读文件工具均需 `version_id`，缺失时提示先调用 `list_installed_versions`。

use serde_json::Value;
use tauri::AppHandle;

use crate::minecraft::isolation::IsolationMode;
use crate::minecraft::version::scan::scan_installed_versions;

#[path = "tool_executor.rs"]
mod tool_executor;
#[path = "tool_registry.rs"]
mod tool_registry;

pub use tool_executor::{collect_context, execute_tool};
pub use tool_registry::tool_definitions;

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
