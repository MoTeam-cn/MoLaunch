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

    fn log(&mut self, level: LogLevel, _target: &str, message: &str) {
        if level > self.level {
            return;
        }

        let now = chrono::Local::now();
        let timestamp = now.format("%H:%M:%S%.3f");
        let level_str = level.as_str();

        let log_line = format!("[{}] [{}] {}\n", timestamp, level_str, message);

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
            // 内容：默认颜色
            let content = message;

            eprintln!("{} {} {}", time_colored, level_colored, content);
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
        "logger",
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
            module_path!(),
            &format!($($arg)*)
        )
    };
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        $crate::logger::log(
            $crate::logger::LogLevel::Warn,
            module_path!(),
            &format!($($arg)*)
        )
    };
}

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        $crate::logger::log(
            $crate::logger::LogLevel::Info,
            module_path!(),
            &format!($($arg)*)
        )
    };
}

#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        $crate::logger::log(
            $crate::logger::LogLevel::Debug,
            module_path!(),
            &format!($($arg)*)
        )
    };
}

#[macro_export]
macro_rules! log_trace {
    ($($arg:tt)*) => {
        $crate::logger::log(
            $crate::logger::LogLevel::Trace,
            module_path!(),
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
pub fn log(level: LogLevel, target: &str, message: &str) {
    if let Ok(mut logger) = LOGGER.lock() {
        logger.log(level, target, message);
    }
}

/// 记录分割线
pub fn separator(title: &str) {
    let line = format!("========== {} ==========", title);
    log(LogLevel::Info, "", &line);
}

/// 获取日志文件路径
pub fn get_log_path() -> PathBuf {
    let storage = Storage::instance();
    let logs_dir = storage.logs_dir();
    let now = chrono::Local::now();
    let filename = format!("molaunch_{}.log", now.format("%Y-%m-%d"));
    logs_dir.join(filename)
}

/// 获取所有日志文件
pub fn list_log_files() -> Vec<String> {
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
pub fn read_log_file(filename: &str) -> anyhow::Result<String> {
    let storage = Storage::instance();
    let path = storage.logs_dir().join(filename);
    Ok(std::fs::read_to_string(&path)?)
}
