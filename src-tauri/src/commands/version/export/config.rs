//! 配置文件保存/读取
//!
//! 参考 PCL2 PageInstanceExport 的"保存配置到文件"功能：
//! 用户可以保存当前导出选项（pack_name/pack_version/勾选状态等）到 .ini 文件，
//! 下次导出时读取该文件恢复勾选状态。
//!
//! 配置文件格式（INI 风格）：
//! ```ini
//! [General]
//! packName=MyPack
//! packVersion=1.0.0
//! checkHostedAssets=true
//! modrinthUploadMode=false
//! packPath=D:\xxx.zip
//!
//! [Options]
//! basic=true
//! mods=false
//! resourcepacks=true
//! ```

use std::collections::HashMap;
use std::path::Path;

use crate::log_info;

use super::types::{ExportOption, LoadConfigResult, SaveConfigParams};

/// 保存配置到文件
pub fn save_config(params: &SaveConfigParams) -> Result<(), String> {
    let config_path = Path::new(&params.config_path);
    if let Some(parent) = config_path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("创建配置目录失败: {} ({})", parent.display(), e))?;
        }
    }

    let mut content = String::new();
    content.push_str("[General]\n");
    content.push_str(&format!("packName={}\n", escape_ini(&params.pack_name)));
    content.push_str(&format!("packVersion={}\n", escape_ini(&params.pack_version)));
    content.push_str(&format!(
        "checkHostedAssets={}\n",
        params.check_hosted_assets
    ));
    content.push_str(&format!(
        "modrinthUploadMode={}\n",
        params.modrinth_upload_mode
    ));
    if let Some(p) = &params.pack_path {
        content.push_str(&format!("packPath={}\n", escape_ini(p)));
    }
    content.push('\n');

    content.push_str("[Options]\n");
    for opt in &params.options {
        // 只保存可勾选选项（不保存 visible=false 的项，因为下次扫描可能不同）
        if !opt.visible {
            continue;
        }
        content.push_str(&format!(
            "{}={}\n",
            escape_ini(&opt.id),
            opt.checked
        ));
    }

    std::fs::write(config_path, content)
        .map_err(|e| format!("写入配置文件失败: {} ({})", config_path.display(), e))?;

    log_info!(
        "[Export] 配置已保存: {}（{} 个选项）",
        config_path.display(),
        params.options.iter().filter(|o| o.visible).count()
    );

    Ok(())
}

/// 从文件读取配置
///
/// 返回 `LoadConfigResult`，其中 `rules_override` 是用户上次保存的勾选状态
/// 对应的 rules 列表（用于直接覆盖当前扫描的默认勾选）。
///
/// 实现策略：读取 .ini 后，对每个 `optionId=true/false` 设置对应选项的 checked 字段。
/// 调用方（前端）拿到 LoadConfigResult 后，自己将其应用到当前 options 列表。
pub fn load_config(config_path: &str) -> Result<LoadConfigResult, String> {
    let path = Path::new(config_path);
    if !path.exists() {
        return Err(format!("配置文件不存在: {}", path.display()));
    }

    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("读取配置文件失败: {} ({})", path.display(), e))?;

    let mut general: HashMap<String, String> = HashMap::new();
    let mut options: HashMap<String, bool> = HashMap::new();
    let mut current_section = String::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            current_section = line[1..line.len() - 1].to_string();
            continue;
        }
        if let Some(eq_pos) = line.find('=') {
            let key = unescape_ini(&line[..eq_pos]);
            let value = unescape_ini(&line[eq_pos + 1..]);
            match current_section.as_str() {
                "General" => {
                    general.insert(key, value);
                }
                "Options" => {
                    let checked = value == "true" || value == "1";
                    options.insert(key, checked);
                }
                _ => {}
            }
        }
    }

    let pack_name = general.get("packName").cloned().unwrap_or_default();
    let pack_version = general.get("packVersion").cloned().unwrap_or_default();
    let check_hosted_assets = general
        .get("checkHostedAssets")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(true);
    let modrinth_upload_mode = general
        .get("modrinthUploadMode")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false);
    let pack_path = general.get("packPath").cloned();

    // 将 options map 序列化为 "id=true|id=false|..." 形式的 rules_override
    // 调用方解析这个字符串列表来恢复勾选状态
    let rules_override: Vec<String> = options
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect();

    log_info!(
        "[Export] 配置已读取: {}（{} 个选项状态）",
        path.display(),
        rules_override.len()
    );

    Ok(LoadConfigResult {
        pack_name,
        pack_version,
        check_hosted_assets,
        modrinth_upload_mode,
        pack_path,
        rules_override,
    })
}

/// 转义 INI 值中的特殊字符（\n / \r / =）
fn escape_ini(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('=', "\\=")
}

/// 反转义 INI 值
fn unescape_ini(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('\\') => result.push('\\'),
                Some('n') => result.push('\n'),
                Some('r') => result.push('\r'),
                Some('=') => result.push('='),
                Some(other) => {
                    result.push('\\');
                    result.push(other);
                }
                None => result.push('\\'),
            }
        } else {
            result.push(c);
        }
    }
    result
}

/// 将 LoadConfigResult.rules_override（"id=true|id=false"）应用到 options 列表
///
/// 公共工具函数：前端读取配置后调用此函数恢复勾选状态。
pub fn apply_config_to_options(options: &mut [ExportOption], rules_override: &[String]) {
    for rule in rules_override {
        if let Some(eq_pos) = rule.find('=') {
            let id = &rule[..eq_pos];
            let checked_str = &rule[eq_pos + 1..];
            let checked = checked_str == "true" || checked_str == "1";
            for opt in options.iter_mut() {
                if opt.id == id {
                    // 必选项（enabled=false）不允许取消
                    if !opt.enabled && !checked {
                        opt.checked = true;
                    } else {
                        opt.checked = checked;
                    }
                    break;
                }
            }
        }
    }
}
