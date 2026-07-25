//! 游戏配置管理模块
//!
//! 在启动前自动设置游戏语言等配置。
//!
//! ## 语言代码大小写规则
//! MC 1.0 ~ 1.10 的 `lang` 字段必须使用大写后缀（如 `zh_CN`），否则切换回英文；
//! MC 1.11+ 必须使用小写后缀（如 `zh_cn`），大写反而切英文。
//! 调用方传入的 `target_lang` 应使用小写形式（如 `zh_cn`、`en_us`），
//! 本模块会根据 `mc_version` 自动转为正确的大小写。

use crate::{log_info, log_warn};
use std::path::Path;

/// 设置游戏语言（写入 options.txt 的 lang 字段）
///
/// ## 参数
/// - `game_dir`: 游戏根目录（或隔离后的有效游戏目录）
/// - `version_id`: 版本目录名（仅用于日志展示）
/// - `mc_version`: 真实 MC 版本号（如 "1.20.1"），用于决定语言代码大小写
/// - `target_lang`: 目标语言代码（小写形式，如 "zh_cn"、"en_us"）
///
/// ## 行为
/// 1. **options.txt 不存在**：创建文件并写入 `lang:<target>`，不写入其他字段
/// 2. **文件存在，lang 字段不存在**：补充 lang 字段到文件末尾
/// 3. **文件存在，lang 已是目标语言**：跳过，不写入
/// 4. **文件存在，lang 是其他语言且 saves/ 不存在**：覆盖为目标语言（先写 `-` 触发缓存清空，再写目标值）
/// 5. **文件存在，lang 是其他语言且 saves/ 已存在**：跳过，尊重老用户选择
///
/// ## 日志
/// 每个分支都有 `[Language]` 前缀的日志，便于排查
pub fn set_game_language(
    game_dir: &Path,
    version_id: &str,
    mc_version: &str,
    target_lang: &str,
) -> anyhow::Result<()> {
    let options_path = game_dir.join("options.txt");
    // 根据 MC 版本调整大小写（1.10- 用大写，1.11+ 用小写）
    let required_lang = adjust_lang_case(target_lang, mc_version);

    log_info!(
        "[Language] set_game_language called: game_dir={:?}, version_id={}, mc_version={}, target_lang={} -> required={}",
        game_dir,
        version_id,
        mc_version,
        target_lang,
        required_lang
    );

    // 如果 options.txt 不存在，创建并写入语言设置
    if !options_path.exists() {
        log_info!("[Language] options.txt not found, creating with lang={}", required_lang);
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
    log_info!(
        "[Language] options.txt exists, current lang={:?}",
        current_lang
    );

    // 检查是否需要设置语言
    // 条件1: lang 键不存在 → 需要补充
    // 条件2: saves 文件夹不存在（新实例）→ 可以覆盖
    let saves_exist = game_dir.join("saves").exists();
    log_info!("[Language] saves folder exists: {}", saves_exist);

    // lang 字段不存在时直接补充（无论 saves 是否存在）
    if current_lang.is_none() {
        log_info!("[Language] lang field missing, appending to options.txt");
        write_ini_value(&options_path, "lang", &required_lang);
        log_info!(
            "[Language] Appended lang={} for version {}",
            required_lang,
            version_id
        );
        return Ok(());
    }

    // lang 字段存在，检查是否已经是目标语言
    if let Some(ref current) = current_lang {
        if current == &required_lang {
            log_info!(
                "[Language] Language already {}, no change needed",
                required_lang
            );
            return Ok(());
        }
        // 语言不同，但 saves 已存在 → 老用户保护，不覆盖
        if saves_exist {
            log_warn!(
                "[Language] Language is {} (target={}), but saves/ exists — respecting user choice, skipping",
                current,
                required_lang
            );
            return Ok(());
        }
    }

    // 写入语言设置
    // 先写 "-" 触发缓存清理，再写目标值
    log_info!(
        "[Language] Overwriting lang: {} -> {} (cache clear via '-')",
        current_lang.as_deref().unwrap_or("(none)"),
        required_lang
    );
    write_ini_value(&options_path, "lang", "-");
    write_ini_value(&options_path, "lang", &required_lang);

    log_info!(
        "[Language] Set language to {} for version {}",
        required_lang,
        version_id
    );

    Ok(())
}

/// 根据 MC 版本调整语言代码的大小写
///
/// - MC 1.0 ~ 1.10：后缀大写（`zh_cn` → `zh_CN`），否则 MC 切回英文
/// - MC 1.11+：后缀小写（`zh_CN` → `zh_cn`），大写反而切英文
/// - MC 26+：小写
///
/// 输入应为小写形式（如 `zh_cn`、`en_us`、`ja_jp`）。
/// 不含下划线的语言代码（如 `none`）原样返回。
fn adjust_lang_case(lang: &str, mc_version: &str) -> String {
    // 无下划线的代码原样返回
    if !lang.contains('_') {
        return lang.to_string();
    }

    // 解析版本号
    let parts: Vec<&str> = mc_version.split('.').collect();
    let major: u32 = parts.first().and_then(|s| s.parse().ok()).unwrap_or(1);
    let minor: u32 = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);

    // 新版本格式 (26+) 使用小写
    if major >= 26 {
        return lang.to_lowercase();
    }

    // 1.x 格式
    if major == 1 {
        if minor <= 10 {
            // MC 1.0 ~ 1.10：后缀大写
            return to_upper_suffix(lang);
        }
        // MC 1.11+：小写
        return lang.to_lowercase();
    }

    // 默认使用小写
    lang.to_lowercase()
}

/// 将语言代码的后缀转为大写：`zh_cn` → `zh_CN`，`en_us` → `en_US`
fn to_upper_suffix(lang: &str) -> String {
    if let Some(pos) = lang.find('_') {
        let (prefix, suffix) = lang.split_at(pos);
        format!("{}{}", prefix, suffix.to_uppercase())
    } else {
        lang.to_string()
    }
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
///
/// 当前未在启动流程中调用，保留以备后续启用。
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
    fn test_adjust_lang_case() {
        // MC 1.0 ~ 1.10：后缀大写
        assert_eq!(adjust_lang_case("zh_cn", "1.0"), "zh_CN");
        assert_eq!(adjust_lang_case("zh_cn", "1.5.2"), "zh_CN");
        assert_eq!(adjust_lang_case("zh_cn", "1.10.2"), "zh_CN");
        assert_eq!(adjust_lang_case("en_us", "1.8.9"), "en_US");

        // MC 1.11+：小写
        assert_eq!(adjust_lang_case("zh_cn", "1.11.2"), "zh_cn");
        assert_eq!(adjust_lang_case("zh_cn", "1.12.2"), "zh_cn");
        assert_eq!(adjust_lang_case("zh_cn", "1.13.2"), "zh_cn");
        assert_eq!(adjust_lang_case("zh_cn", "1.20.1"), "zh_cn");
        assert_eq!(adjust_lang_case("zh_CN", "1.20.1"), "zh_cn");

        // MC 26+：小写
        assert_eq!(adjust_lang_case("zh_cn", "26.2"), "zh_cn");
        assert_eq!(adjust_lang_case("zh_cn", "27.1"), "zh_cn");

        // 无下划线的代码原样返回
        assert_eq!(adjust_lang_case("none", "1.20.1"), "none");
        assert_eq!(adjust_lang_case("auto", "1.20.1"), "auto");
    }

    #[test]
    fn test_to_upper_suffix() {
        assert_eq!(to_upper_suffix("zh_cn"), "zh_CN");
        assert_eq!(to_upper_suffix("en_us"), "en_US");
        assert_eq!(to_upper_suffix("ja_jp"), "ja_JP");
        assert_eq!(to_upper_suffix("ko_kr"), "ko_KR");
        // 无下划线的原样返回
        assert_eq!(to_upper_suffix("none"), "none");
    }
}
