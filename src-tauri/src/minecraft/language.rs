//! 游戏配置管理模块
//!
//! 在启动前自动设置游戏语言等配置

use crate::log_info;
use std::path::Path;

/// 设置游戏语言为中文
pub fn set_game_language(
    game_dir: &Path,
    version_id: &str,
    mc_version: &str,
) -> anyhow::Result<()> {
    let options_path = game_dir.join("options.txt");
    let required_lang = determine_lang_code(mc_version);

    // 如果 options.txt 不存在，创建并写入语言设置
    if !options_path.exists() {
        log_info!("[Language] Creating options.txt at {:?}", options_path);
        let content = format!("lang:{}\n", required_lang);
        std::fs::write(&options_path, content)
            .map_err(|e| anyhow::anyhow!("Failed to create options.txt: {}", e))?;
        log_info!(
            "[Language] Created options.txt with lang={} for version {}",
            required_lang,
            version_id
        );
        return Ok(());
    }

    // 读取当前语言
    let current_lang = read_ini_value(&options_path, "lang");

    // 检查是否需要设置语言
    // 条件1: lang 键不存在
    // 条件2: saves 文件夹不存在（新实例）
    let saves_exist = game_dir.join("saves").exists();
    let need_set_lang = current_lang.is_none() || !saves_exist;

    if !need_set_lang {
        log_info!(
            "[Language] Language already set to {:?}, skipping",
            current_lang
        );
        return Ok(());
    }

    // 检查当前语言是否已经是目标语言
    if let Some(ref current) = current_lang {
        if current == &required_lang {
            log_info!(
                "[Language] Language is already {}, no change needed",
                required_lang
            );
            return Ok(());
        }
    }

    // 写入语言设置
    // 先写 "-" 触发缓存清理，再写目标值
    write_ini_value(&options_path, "lang", "-");
    write_ini_value(&options_path, "lang", &required_lang);

    log_info!(
        "[Language] Set language to {} for version {}",
        required_lang,
        version_id
    );

    Ok(())
}

/// 根据 MC 版本确定语言代码
fn determine_lang_code(mc_version: &str) -> String {
    // 解析版本号
    let parts: Vec<&str> = mc_version.split('.').collect();
    let major: u32 = parts.first().and_then(|s| s.parse().ok()).unwrap_or(1);
    let minor: u32 = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);

    // 新版本格式 (26+) 使用小写
    if major >= 26 {
        return "zh_cn".to_string();
    }

    // 1.x 格式
    if major == 1 {
        // MC 1.0 及更早：无语言选项（minor 为 u32，不会小于 0）
        if minor == 0 {
            return "zh_CN".to_string();
        }
        // MC 1.1 ~ 1.10：最后两位必须大写（否则崩溃或切换英文）
        if minor <= 10 {
            return "zh_CN".to_string();
        }
        // MC 1.11 ~ 1.12：小写正常
        // MC 1.13+：小写正常（大写反而切英文）
        return "zh_cn".to_string();
    }

    // 默认使用小写
    "zh_cn".to_string()
}

/// 读取 INI 文件中的值
fn read_ini_value(path: &Path, key: &str) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('#') || line.is_empty() {
            continue;
        }

        if let Some(pos) = line.find(':') {
            let k = line[..pos].trim();
            let v = line[pos + 1..].trim();
            if k == key {
                return Some(v.to_string());
            }
        }
    }

    None
}

/// 写入 INI 文件中的值
fn write_ini_value(path: &Path, key: &str, value: &str) {
    let content = std::fs::read_to_string(path).unwrap_or_default();
    let mut lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
    let mut found = false;

    // 查找并更新现有键
    for line in &mut lines {
        let trimmed = line.trim();
        if trimmed.starts_with('#') || trimmed.is_empty() {
            continue;
        }

        if let Some(pos) = trimmed.find(':') {
            let k = trimmed[..pos].trim();
            if k == key {
                *line = format!("{}:{}", key, value);
                found = true;
                break;
            }
        }
    }

    // 如果键不存在，添加到文件末尾
    if !found {
        lines.push(format!("{}:{}", key, value));
    }

    // 写回文件
    let new_content = lines.join("\n");
    let _ = std::fs::write(path, new_content);
}

/// 设置 forceUnicodeFont（改善中文字体显示）
pub fn set_force_unicode_font(game_dir: &Path, enable: bool) -> anyhow::Result<()> {
    let options_path = game_dir.join("options.txt");

    if !options_path.exists() {
        return Ok(());
    }

    let value = if enable { "true" } else { "false" };
    write_ini_value(&options_path, "forceUnicodeFont", value);

    log_info!("[Language] Set forceUnicodeFont to {}", value);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_determine_lang_code() {
        // 新版本格式
        assert_eq!(determine_lang_code("26.2"), "zh_cn");
        assert_eq!(determine_lang_code("27.1"), "zh_cn");

        // 1.x 格式
        assert_eq!(determine_lang_code("1.0"), "zh_CN");
        assert_eq!(determine_lang_code("1.5.2"), "zh_CN");
        assert_eq!(determine_lang_code("1.10.2"), "zh_CN");
        assert_eq!(determine_lang_code("1.11.2"), "zh_cn");
        assert_eq!(determine_lang_code("1.12.2"), "zh_cn");
        assert_eq!(determine_lang_code("1.13.2"), "zh_cn");
        assert_eq!(determine_lang_code("1.20.1"), "zh_cn");
    }
}
