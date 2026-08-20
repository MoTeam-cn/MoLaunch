//! 游戏配置管理模块
//!
//! 启动前自动设置游戏语言，根据 MC 版本调整大小写（1.10- 大写，1.11+ 小写）。

use crate::{log_debug, log_warn};
use std::path::Path;

/// 设置游戏语言（写入 options.txt 的 lang 字段）
///
/// 参数：`game_dir` 游戏根目录；`version_id` 版本目录名（仅日志）；`mc_version` 真实 MC
/// 版本号（决定语言代码大小写）；`target_lang` 目标语言代码（小写，如 "zh_cn"）。
/// 行为：options.txt 不存在则创建只写 lang；lang 不存在则补充；lang 已是目标则跳过；
/// lang 是其他语言且 saves/ 不存在则覆盖（先写 `-` 清缓存再写目标值），saves/ 已存在
/// 则跳过尊重老用户选择。每个分支都有 `[Language]` 前缀日志便于排查。
pub fn set_game_language(
    game_dir: &Path,
    version_id: &str,
    mc_version: &str,
    target_lang: &str,
) -> anyhow::Result<()> {
    let options_path = game_dir.join("options.txt");
    // 根据 MC 版本调整大小写（1.10- 用大写，1.11+ 用小写）
    let required_lang = adjust_lang_case(target_lang, mc_version);

    log_debug!(
        "[Language] set_game_language called: game_dir={:?}, version_id={}, mc_version={}, target_lang={} -> required={}",
        game_dir,
        version_id,
        mc_version,
        target_lang,
        required_lang
    );

    // 如果 options.txt 不存在，创建并写入语言设置
    if !options_path.exists() {
        log_debug!(
            "[Language] options.txt not found, creating with lang={}",
            required_lang
        );
        let content = format!("lang:{}\n", required_lang);
        std::fs::write(&options_path, content)
            .map_err(|e| anyhow::anyhow!("Failed to create options.txt: {}", e))?;
        log_debug!(
            "[Language] Created options.txt with lang={} for version {}",
            required_lang,
            version_id
        );
        return Ok(());
    }

    // 读取当前语言
    let current_lang = read_ini_value(&options_path, "lang");
    log_debug!(
        "[Language] options.txt exists, current lang={:?}",
        current_lang
    );

    // 检查是否需要设置语言
    // 条件1: lang 键不存在 → 需要补充
    // 条件2: saves 文件夹不存在（新实例）→ 可以覆盖
    let saves_exist = game_dir.join("saves").exists();
    log_debug!("[Language] saves folder exists: {}", saves_exist);

    // lang 字段不存在时直接补充（无论 saves 是否存在）
    if current_lang.is_none() {
        log_debug!("[Language] lang field missing, appending to options.txt");
        write_ini_value(&options_path, "lang", &required_lang);
        log_debug!(
            "[Language] Appended lang={} for version {}",
            required_lang,
            version_id
        );
        return Ok(());
    }

    // lang 字段存在，检查是否已经是目标语言
    if let Some(ref current) = current_lang {
        if current == &required_lang {
            log_debug!(
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
    log_debug!(
        "[Language] Overwriting lang: {} -> {} (cache clear via '-')",
        current_lang.as_deref().unwrap_or("(none)"),
        required_lang
    );
    write_ini_value(&options_path, "lang", "-");
    write_ini_value(&options_path, "lang", &required_lang);

    log_debug!(
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

    log_debug!("[Language] Set forceUnicodeFont to {}", value);

    Ok(())
}

#[cfg(test)]
#[path = "language_tests.rs"]
mod tests;
