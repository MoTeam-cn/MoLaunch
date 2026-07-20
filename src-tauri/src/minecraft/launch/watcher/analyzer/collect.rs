//! 信息收集 — 读取 crash-reports、hs_err、latest.log 等文件
//!
//! 在崩溃发生后 3 分钟窗口内搜索最新的崩溃报告文件。

use super::super::types::{LogEntry, LogLevel};
use super::util::truncate_head_tail;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// 收集到的崩溃分析源文本
pub(super) struct CollectedSources {
    /// 运行时日志（全部行）
    pub(super) runtime_log: String,
    /// 错误/致命级别日志行
    pub(super) error_lines: Vec<String>,
    /// 崩溃报告文件路径与内容
    pub(super) crash_report: Option<(PathBuf, String)>,
    /// 崩溃报告文本
    pub(super) crash_report_text: String,
    /// hs_err 日志文本
    pub(super) hs_err_text: String,
    /// latest.log 尾部行
    pub(super) latest_log_tail: Vec<String>,
}

/// 收集各源文本
pub(super) fn collect_sources(logs: &[LogEntry], game_dir: &Path) -> CollectedSources {
    let runtime_log: String = logs
        .iter()
        .map(|e| format!("[{:?}] {}", e.level, e.message))
        .collect::<Vec<_>>()
        .join("\n");

    // 收集错误/致命级别日志行
    let error_lines: Vec<String> = logs
        .iter()
        .filter(|e| e.level == LogLevel::Error || e.level == LogLevel::Fatal)
        .map(|e| e.message.clone())
        .collect();

    // 读取 crash-reports 目录中最新的崩溃报告（3分钟内）
    let crash_report = read_latest_crash_report(game_dir);
    let crash_report_text: String = crash_report
        .as_ref()
        .and_then(|(path, content)| {
            crate::log_info!("[CrashAnalyzer] 找到崩溃报告: {}", path.display());
            Some(content.clone())
        })
        .unwrap_or_default();

    // 读取 hs_err_pid*.log（JVM 崩溃报告，3分钟内）
    let hs_err_text = read_latest_hs_err(game_dir);
    if !hs_err_text.is_empty() {
        crate::log_info!("[CrashAnalyzer] 找到 hs_err_pid 日志（{}字符）", hs_err_text.len());
    }

    // 读取 logs/latest.log 尾部（500行）
    let latest_log_tail = read_latest_log_tail(game_dir, 500);
    if !latest_log_tail.is_empty() {
        crate::log_info!("[CrashAnalyzer] 读取 latest.log 尾部（{}行）", latest_log_tail.len());
    }

    CollectedSources {
        runtime_log,
        error_lines,
        crash_report,
        crash_report_text,
        hs_err_text,
        latest_log_tail,
    }
}

/// 读取 crash-reports 目录中最新的崩溃报告（3分钟内修改过）
fn read_latest_crash_report(game_dir: &Path) -> Option<(PathBuf, String)> {
    let crash_dir = game_dir.join("crash-reports");
    if !crash_dir.exists() {
        return None;
    }

    let now = SystemTime::now();
    let mut latest: Option<(PathBuf, SystemTime)> = None;

    if let Ok(entries) = std::fs::read_dir(&crash_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            // 只看 crash-*.txt 文件
            let name = path.file_name()?.to_string_lossy();
            if !name.starts_with("crash-") || path.extension().map_or(true, |e| e != "txt") {
                continue;
            }
            if let Ok(meta) = entry.metadata() {
                if let Ok(modified) = meta.modified() {
                    // 只看 3 分钟内修改过的文件
                    if let Ok(age) = now.duration_since(modified) {
                        if age.as_secs() < 180 {
                            if latest.as_ref().map_or(true, |(_, t)| modified > *t) {
                                latest = Some((path, modified));
                            }
                        }
                    }
                }
            }
        }
    }

    if let Some((path, _)) = latest {
        if let Ok(content) = std::fs::read_to_string(&path) {
            return Some((path, content));
        }
    }
    None
}

/// 读取最新的 hs_err_pid*.log 文件（3分钟内）
fn read_latest_hs_err(game_dir: &Path) -> String {
    let now = SystemTime::now();
    let mut latest: Option<(PathBuf, SystemTime)> = None;

    // hs_err_pid*.log 可能在游戏根目录或版本目录
    let search_dirs = [game_dir.to_path_buf()];

    for dir in &search_dirs {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let name = match path.file_name() {
                    Some(n) => n.to_string_lossy().to_string(),
                    None => continue,
                };
                if !name.starts_with("hs_err_pid") {
                    continue;
                }
                if let Ok(meta) = entry.metadata() {
                    if let Ok(modified) = meta.modified() {
                        if let Ok(age) = now.duration_since(modified) {
                            if age.as_secs() < 180 {
                                if latest.as_ref().map_or(true, |(_, t)| modified > *t) {
                                    latest = Some((path, modified));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if let Some((path, _)) = latest {
        if let Ok(content) = std::fs::read_to_string(&path) {
            // 截取头 200 行 + 尾 100 行
            return truncate_head_tail(&content, 200, 100);
        }
    }
    String::new()
}

/// 读取 logs/latest.log 的尾部 N 行
fn read_latest_log_tail(game_dir: &Path, tail_lines: usize) -> Vec<String> {
    let log_path = game_dir.join("logs").join("latest.log");
    if !log_path.exists() {
        return Vec::new();
    }

    let content = match std::fs::read_to_string(&log_path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let lines: Vec<&str> = content.lines().collect();
    let start = if lines.len() > tail_lines {
        lines.len() - tail_lines
    } else {
        0
    };
    lines[start..].iter().map(|s| s.to_string()).collect()
}
