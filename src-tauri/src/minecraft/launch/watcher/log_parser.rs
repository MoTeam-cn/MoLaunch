//! 日志行解析与加载进度检测
//!
//! 原 `GameWatcher::parse_log_line` / `extract_log_level` / `detect_load_progress`
//! 静态方法的纯函数实现，与监控器结构体解耦。

use super::types::{LoadProgress, LogEntry, LogLevel};

/// 解析日志行
pub(crate) fn parse_log_line(line: &str, source: &str) -> LogEntry {
    let (level, _message) = extract_log_level(line);

    LogEntry {
        timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
        level,
        source: source.to_string(),
        message: line.to_string(),
    }
}

/// 提取日志级别
fn extract_log_level(line: &str) -> (LogLevel, &str) {
    let line_lower = line.to_lowercase();

    if line_lower.contains("[fatal]") || line_lower.contains("fatal error") {
        (LogLevel::Fatal, line)
    } else if line_lower.contains("[error]") || line_lower.contains("exception") {
        (LogLevel::Error, line)
    } else if line_lower.contains("[warn]") {
        (LogLevel::Warn, line)
    } else if line_lower.contains("[debug]") {
        (LogLevel::Debug, line)
    } else if line_lower.contains("[trace]") {
        (LogLevel::Trace, line)
    } else {
        (LogLevel::Info, line)
    }
}

/// 检测加载进度
pub(crate) fn detect_load_progress(line: &str) -> LoadProgress {
    let line_lower = line.to_lowercase();

    // Level 5: 材质加载
    if line_lower.contains("textures") && line_lower.contains("-atlas") {
        return LoadProgress::TextureLoaded;
    }

    // Level 4: OpenAL 初始化
    if line_lower.contains("openal initialized") {
        return LoadProgress::OpenAlInit;
    }

    // Level 3: LWJGL
    if line_lower.contains("lwjgl version") || line_lower.contains("lwjgl") {
        return LoadProgress::LwjglInit;
    }

    // Level 2: Setting user
    if line_lower.contains("setting user:") || line_lower.contains("setting user ") {
        return LoadProgress::SettingUser;
    }

    // Level 1: 任何日志输出
    LoadProgress::LogAppeared
}
