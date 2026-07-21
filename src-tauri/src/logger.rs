//! 日志系统模块
//!
//! 将日志写入 storage/logs 目录，同时可选输出到控制台

use crate::storage::Storage;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

/// 全局日志器
static LOGGER: once_cell::sync::Lazy<Mutex<Logger>> =
    once_cell::sync::Lazy::new(|| Mutex::new(Logger::new()));

/// 日志级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Error = 1,
    Warn = 2,
    Info = 3,
    Debug = 4,
    Trace = 5,
}

impl LogLevel {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "error" => Self::Error,
            "warn" | "warning" => Self::Warn,
            "info" => Self::Info,
            "debug" => Self::Debug,
            "trace" => Self::Trace,
            _ => Self::Info,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Error => "ERROR",
            Self::Warn => "WARN",
            Self::Info => "INFO",
            Self::Debug => "DEBUG",
            Self::Trace => "TRACE",
        }
    }
}

/// 日志器
struct Logger {
    file: Option<std::fs::File>,
    level: LogLevel,
    console_output: bool,
}

impl Logger {
    fn new() -> Self {
        Self {
            file: None,
            level: LogLevel::Info,
            console_output: true,
        }
    }

    fn init(&mut self, level: LogLevel, console_output: bool) {
        self.level = level;
        self.console_output = console_output;

        // 创建日志文件
        let storage = Storage::instance();
        let logs_dir = storage.logs_dir();

        // 确保日志目录存在
        let _ = std::fs::create_dir_all(&logs_dir);

        // 日志文件名：molaunch_YYYY-MM-DD.log
        let now = chrono::Local::now();
        let filename = format!("molaunch_{}.log", now.format("%Y-%m-%d"));
        let log_path = logs_dir.join(filename);

        match OpenOptions::new().create(true).append(true).open(&log_path) {
            Ok(file) => {
                self.file = Some(file);
                // 写入启动标记
                self.write_raw(&format!(
                    "\n{} === MoLaunch Started ===\n",
                    now.format("%Y-%m-%d %H:%M:%S")
                ));
            }
            Err(e) => {
                eprintln!("Failed to open log file: {}", e);
            }
        }
    }

    fn log(&mut self, level: LogLevel, file: &str, line: u32, message: &str) {
        if level > self.level {
            return;
        }

        let now = chrono::Local::now();
        let timestamp = now.format("%H:%M:%S%.3f");
        let level_str = level.as_str();

        // 将绝对路径裁剪为项目相对路径（src-tauri/src/xxx.rs）
        // file!() 在 Rust 中返回相对路径如 "src/logger.rs" 或 "src/commands/sdk.rs"
        let rel_path = strip_to_src_relative(file);

        // 安全脱敏：过滤 message 中的敏感信息，避免 token 明文写入日志文件
        // 识别 JWT 格式（eyJ 开头）、Minecraft token（含 "eyJ" 子串）、长 hex/base64 token
        let sanitized = sanitize_sensitive_info(message);
        let log_line = format!(
            "[{}] [{}] [{}:{}] {}\n",
            timestamp, level_str, rel_path, line, sanitized
        );

        // 写入文件
        if let Some(ref mut file) = self.file {
            let _ = file.write_all(log_line.as_bytes());
            let _ = file.flush();
        }

        // 输出到控制台（带颜色）
        if self.console_output {
            // 时间：灰色
            let time_colored = format!("\x1b[90m[{}]\x1b[0m", timestamp);
            // 级别：不同颜色
            let level_colored = match level {
                LogLevel::Error => format!("\x1b[1;31m[{}]\x1b[0m", level_str), // 红色加粗
                LogLevel::Warn => format!("\x1b[1;33m[{}]\x1b[0m", level_str),  // 黄色加粗
                LogLevel::Info => format!("\x1b[1;36m[{}]\x1b[0m", level_str),  // 青色加粗
                LogLevel::Debug => format!("\x1b[1;35m[{}]\x1b[0m", level_str), // 紫色加粗
                LogLevel::Trace => format!("\x1b[1;90m[{}]\x1b[0m", level_str), // 灰色加粗
            };
            // 路径：灰色
            let path_colored = format!("\x1b[90m[{}:{}]\x1b[0m", rel_path, line);
            // 内容：默认颜色（使用脱敏后的内容）
            let content = &sanitized;

            eprintln!("{} {} {} {}", time_colored, level_colored, path_colored, content);
        }
    }

    fn write_raw(&mut self, text: &str) {
        if let Some(ref mut file) = self.file {
            let _ = file.write_all(text.as_bytes());
            let _ = file.flush();
        }
    }
}

/// 初始化日志系统
pub fn init(level: LogLevel, console_output: bool) {
    // 即使锁被 poison（持有者 panic），仍取回内部数据继续工作，避免日志系统拖垮启动
    let mut logger = LOGGER.lock().unwrap_or_else(|e| e.into_inner());
    logger.init(level, console_output);
}

/// 设置日志级别（热重载）
pub fn set_level(level: LogLevel) {
    if let Ok(mut logger) = LOGGER.lock() {
        logger.level = level;
    }
    // 在锁外记录日志，避免死锁
    log(
        LogLevel::Info,
        "logger.rs",
        line!(),
        &format!("Log level changed to: {:?}", level),
    );
}

/// 初始化日志系统（从配置）
pub fn init_from_config() {
    let storage = Storage::instance();
    let level_str = storage
        .get_config("Log", "level")
        .unwrap_or_else(|| "3".to_string());

    let level = match level_str.parse::<u32>().unwrap_or(3) {
        0 => LogLevel::Error,
        1 => LogLevel::Error,
        2 => LogLevel::Warn,
        3 => LogLevel::Info,
        4 => LogLevel::Debug,
        5 => LogLevel::Trace,
        _ => LogLevel::Info,
    };

    // Debug 模式下输出到控制台
    let console_output = cfg!(debug_assertions);
    init(level, console_output);
}

/// 日志宏
#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        $crate::logger::log(
            $crate::logger::LogLevel::Error,
            file!(),
            line!(),
            &format!($($arg)*)
        )
    };
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        $crate::logger::log(
            $crate::logger::LogLevel::Warn,
            file!(),
            line!(),
            &format!($($arg)*)
        )
    };
}

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        $crate::logger::log(
            $crate::logger::LogLevel::Info,
            file!(),
            line!(),
            &format!($($arg)*)
        )
    };
}

#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        $crate::logger::log(
            $crate::logger::LogLevel::Debug,
            file!(),
            line!(),
            &format!($($arg)*)
        )
    };
}

#[macro_export]
macro_rules! log_trace {
    ($($arg:tt)*) => {
        $crate::logger::log(
            $crate::logger::LogLevel::Trace,
            file!(),
            line!(),
            &format!($($arg)*)
        )
    };
}

#[macro_export]
macro_rules! log_separator {
    ($title:expr) => {
        $crate::logger::separator($title)
    };
}

/// 记录日志（供外部调用）
pub fn log(level: LogLevel, file: &str, line: u32, message: &str) {
    if let Ok(mut logger) = LOGGER.lock() {
        logger.log(level, file, line, message);
    }
}

/// 记录分割线
pub fn separator(title: &str) {
    let line = format!("========== {} ==========", title);
    // 分割线没有调用方信息，用空字符串占位
    log(LogLevel::Info, "logger.rs", 0, &line);
}

/// 将 `file!()` 返回的路径裁剪为项目内相对路径（以 src/ 开头）
///
/// `file!()` 在 cargo 编译时返回相对于 crate root 的路径，通常已经是
/// `src/logger.rs` / `src/commands/sdk.rs` 形式。但有时会带 `src-tauri/` 前缀，
/// 此函数统一裁剪到 `src/...` 形式，便于日志阅读。
fn strip_to_src_relative(file: &str) -> String {
    // 优先裁剪到 `src/` 之后
    if let Some(pos) = file.find("src/") {
        file[pos..].to_string()
    } else {
        // 兜底：直接使用原路径
        file.to_string()
    }
}

/// 获取日志文件路径
pub fn get_log_path_inner() -> PathBuf {
    let storage = Storage::instance();
    let logs_dir = storage.logs_dir();
    let now = chrono::Local::now();
    let filename = format!("molaunch_{}.log", now.format("%Y-%m-%d"));
    logs_dir.join(filename)
}

/// 获取所有日志文件
pub fn list_log_files_inner() -> Vec<String> {
    let storage = Storage::instance();
    let logs_dir = storage.logs_dir();

    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&logs_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with(".log") {
                files.push(name);
            }
        }
    }
    files.sort();
    files.reverse(); // 最新的在前
    files
}

/// 读取日志文件内容
pub fn read_log_file_inner(filename: &str) -> anyhow::Result<String> {
    let storage = Storage::instance();
    let path = storage.logs_dir().join(filename);
    Ok(std::fs::read_to_string(&path)?)
}

// ============================================================
// Tauri 命令包装（供开发者模式「日志查看」调用）
// ============================================================
//
// 原函数返回 PathBuf / Vec<String> / anyhow::Result<String>，
// 不能直接作为 #[tauri::command]（PathBuf 需要序列化、&str 参数需要 owned）。
// 这里提供薄包装层转换为 Tauri 友好的返回类型。

/// 获取今日日志文件完整路径（字符串形式）
#[tauri::command]
pub fn get_log_path() -> String {
    get_log_path_inner().to_string_lossy().to_string()
}

/// 获取所有日志文件名列表（最新的在前）
#[tauri::command]
pub fn list_log_files() -> Vec<String> {
    list_log_files_inner()
}

/// 读取指定日志文件内容
///
/// `filename` 仅允许 `.log` 后缀，且不得包含路径分隔符（防止路径遍历）。
/// 安全修复：返回前对内容进行脱敏，避免前端日志查看器显示 token 等敏感信息
#[tauri::command]
pub fn read_log_file(filename: String) -> Result<String, String> {
    if filename.is_empty()
        || filename.contains('/')
        || filename.contains('\\')
        || filename.contains("..")
        || !filename.ends_with(".log")
    {
        return Err(format!("非法日志文件名: {}", filename));
    }
    let content = read_log_file_inner(&filename).map_err(|e| format!("读取日志文件失败: {}", e))?;
    Ok(sanitize_sensitive_info(&content))
}

/// 对日志内容进行敏感信息脱敏
///
/// 识别并替换以下模式：
/// 1. JWT 格式 token：`eyJxxx.yyy.zzz`（三段，点分隔）
/// 2. Minecraft access_token：通常以 "eyJ" 开头的长字符串
/// 3. 长度 >= 40 的 hex/base64 字符串（可能是 token）
/// 4. JSON 中的 token 字段：`"access_token":"xxx"` / `"accessToken":"xxx"`
///
/// 保留短字符串和普通日志内容，只替换明显的 token 特征。
pub fn sanitize_sensitive_info(s: &str) -> String {
    use regex::Regex;
    use std::sync::OnceLock;

    static JWT_RE: OnceLock<Regex> = OnceLock::new();
    static JSON_TOKEN_RE: OnceLock<Regex> = OnceLock::new();
    static LONG_TOKEN_RE: OnceLock<Regex> = OnceLock::new();

    let jwt_re = JWT_RE.get_or_init(|| {
        // JWT 格式：eyJ 开头，三段点分隔，每段至少 10 字符
        Regex::new(r"eyJ[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}").unwrap()
    });

    let json_token_re = JSON_TOKEN_RE.get_or_init(|| {
        // JSON 字段：access_token / accessToken / refresh_token / client_token / token
        // 匹配 "key":"value" 中的 value（支持空格）
        Regex::new(
            r#"(?i)"(access_token|accesstoken|refresh_token|refreshtoken|client_token|clienttoken|session|token)"\s*:\s*"[^"]{8,}""#,
        ).unwrap()
    });

    let long_token_re = LONG_TOKEN_RE.get_or_init(|| {
        // 长度 >= 40 的连续 base64/hex 字符串（可能是 token）
        // 排除路径、UUID（含连字符）等
        Regex::new(r"\b[A-Za-z0-9+/=_-]{40,}\b").unwrap()
    });

    let mut result = s.to_string();

    // 1. 替换 JWT 格式 token
    result = jwt_re.replace_all(&result, "***").to_string();

    // 2. 替换 JSON 中的 token 字段值
    result = json_token_re
        .replace_all(&result, r#""$1":"***""#)
        .to_string();

    // 3. 替换超长 token 字符串（最后执行，避免误伤已脱敏的 ***）
    result = long_token_re.replace_all(&result, "***").to_string();

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_jwt() {
        let input = "Launching with token eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";
        let result = sanitize_sensitive_info(input);
        assert!(!result.contains("eyJhbGciOiJIUzI1NiJ9"));
        assert!(result.contains("***"));
    }

    #[test]
    fn test_sanitize_json_token() {
        let input = r#"Auth response: {"access_token":"eyJsecret12345678","username":"player"}"#;
        let result = sanitize_sensitive_info(input);
        assert!(result.contains(r#""access_token":"***""#));
        assert!(!result.contains("eyJsecret12345678"));
        // username 不应被脱敏
        assert!(result.contains("player"));
    }

    #[test]
    fn test_sanitize_preserves_short_strings() {
        let input = "Game version: 1.16.5, Java path: C:/java/javaw.exe";
        let result = sanitize_sensitive_info(input);
        assert_eq!(input, result);
    }

    #[test]
    fn test_sanitize_long_token() {
        let input = "Token: abc123def456ghi789jkl012mno345pqr678stu901vwx234yz";
        let result = sanitize_sensitive_info(input);
        assert!(result.contains("***"));
        assert!(!result.contains("abc123def456ghi789"));
    }
}
