//! Agent 信息工具：启动器信息 / 已安装版本 / 已安装 Mod

use crate::commands::experimental::agent::AgentContext;
use crate::utils::format::truncate_chars;

use super::tools::{effective_dir, installed_version_ids};

/// 启动器信息（版本 + 游戏目录 + 配置摘要）
pub(super) fn launcher_info(ctx: &AgentContext) -> String {
    format!(
        "MoLaunch 启动器信息\n版本: {}\n游戏目录: {}\n{}\n",
        ctx.version,
        ctx.game_dir.display(),
        ctx.config_summary
    )
}

/// 已安装版本列表（文本形式返回给模型）
pub(super) fn list_installed_versions(ctx: &AgentContext) -> Result<String, String> {
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

/// 列出指定版本 mods 目录中的 Mod 文件
pub(super) fn list_installed_mods(version_id: &str, ctx: &AgentContext) -> Result<String, String> {
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
