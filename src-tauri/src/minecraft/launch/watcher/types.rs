//! 监控器数据结构（GameState / ExitInfo / CrashInfo / CrashCategory / LogLevel / LogEntry / LoadProgress）

use serde::{Deserialize, Serialize};

/// 游戏状态
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameState {
    /// 启动中
    Starting,
    /// 加载中
    Loading,
    /// 运行中
    Running,
    /// 已退出
    Exited(ExitInfo),
    /// 崩溃
    Crashed(CrashInfo),
}

/// 退出信息
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExitInfo {
    pub code: i32,
    pub is_normal: bool,
    /// 崩溃详情（仅 is_normal=false 时可能有值）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crash_info: Option<CrashInfo>,
}

/// 崩溃信息
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrashInfo {
    /// 崩溃原因摘要
    pub reason: String,
    /// 崩溃类别
    pub category: CrashCategory,
    /// 相关日志行（错误/致命级别）
    #[serde(default)]
    pub log_lines: Vec<String>,
    /// 建议的解决方案
    pub suggestion: String,
    /// 可能导致崩溃的Mod
    pub problematic_mod: Option<String>,
    /// 崩溃报告文件路径（供用户查看，如 crash-reports/crash-xxx.txt）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crash_report_path: Option<String>,
    /// 游戏日志尾部（最近 N 行，供弹窗展示）
    #[serde(default)]
    pub log_tail: Vec<String>,
}

/// 崩溃类别
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CrashCategory {
    /// Java相关
    Java,
    /// Mod相关
    Mod,
    /// 显卡相关
    Graphics,
    /// 内存相关
    Memory,
    /// Forge相关
    Forge,
    /// Fabric相关
    Fabric,
    /// OptiFine相关
    OptiFine,
    /// 资源包相关
    ResourcePack,
    /// 光影相关
    Shader,
    /// 未知
    Unknown,
}

/// 日志级别
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
}

/// 日志条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: LogLevel,
    pub source: String,
    pub message: String,
}

/// 加载进度级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum LoadProgress {
    /// 无输出
    None = 0,
    /// 有日志输出
    LogAppeared = 1,
    /// Setting user (设置用户)
    SettingUser = 2,
    /// LWJGL 初始化
    LwjglInit = 3,
    /// OpenAL 初始化
    OpenAlInit = 4,
    /// 材质加载
    TextureLoaded = 5,
    /// 游戏窗口出现
    WindowAppeared = 6,
}

impl LoadProgress {
    pub fn name(&self) -> &str {
        match self {
            LoadProgress::None => "准备中",
            LoadProgress::LogAppeared => "开始加载",
            LoadProgress::SettingUser => "设置用户",
            LoadProgress::LwjglInit => "初始化图形",
            LoadProgress::OpenAlInit => "初始化音频",
            LoadProgress::TextureLoaded => "加载材质",
            LoadProgress::WindowAppeared => "游戏窗口",
        }
    }
}
