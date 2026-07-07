//! 游戏进程监控模块
//! 参考PCL2的ModWatcher实现，监控游戏状态和崩溃检测

use crate::log_info;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::{Mutex, RwLock};

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
}

/// 崩溃信息
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrashInfo {
    /// 崩溃原因
    pub reason: String,
    /// 崩溃类别
    pub category: CrashCategory,
    /// 相关日志行
    pub log_lines: Vec<String>,
    /// 建议的解决方案
    pub suggestion: String,
    /// 可能导致崩溃的Mod
    pub problematic_mod: Option<String>,
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

/// 加载进度级别 (参考PCL2)
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

/// 游戏进程监控器
pub struct GameWatcher {
    /// 进程ID
    #[allow(dead_code)]
    pid: u32,
    /// 游戏状态
    state: Arc<RwLock<GameState>>,
    /// 加载进度
    load_progress: Arc<RwLock<LoadProgress>>,
    /// 日志缓冲区
    log_buffer: Arc<Mutex<VecDeque<LogEntry>>>,
    /// 最大日志行数
    max_log_lines: usize,
    /// 游戏目录
    #[allow(dead_code)]
    game_dir: PathBuf,
    /// 版本ID
    version_id: String,
    /// 退出通知通道
    exit_tx: tokio::sync::watch::Sender<Option<ExitInfo>>,
    /// 退出接收通道（供外部监听）
    exit_rx: tokio::sync::watch::Receiver<Option<ExitInfo>>,
}

impl GameWatcher {
    /// 创建新的监控器
    pub fn new(pid: u32, game_dir: PathBuf, version_id: String) -> Self {
        let (exit_tx, exit_rx) = tokio::sync::watch::channel(None);
        Self {
            pid,
            state: Arc::new(RwLock::new(GameState::Starting)),
            load_progress: Arc::new(RwLock::new(LoadProgress::None)),
            log_buffer: Arc::new(Mutex::new(VecDeque::new())),
            exit_tx,
            exit_rx,
            max_log_lines: 10000,
            game_dir,
            version_id,
        }
    }

    /// 获取当前状态
    pub async fn state(&self) -> GameState {
        self.state.read().await.clone()
    }

    /// 获取加载进度
    pub async fn load_progress(&self) -> LoadProgress {
        *self.load_progress.read().await
    }

    /// 获取最近的日志
    pub async fn recent_logs(&self, count: usize) -> Vec<LogEntry> {
        let buffer = self.log_buffer.lock().await;
        buffer.iter().rev().take(count).cloned().collect()
    }

    /// 获取退出通知接收器
    pub fn exit_receiver(&self) -> tokio::sync::watch::Receiver<Option<ExitInfo>> {
        self.exit_rx.clone()
    }

    /// 开始监控
    pub async fn start_monitoring(
        &self,
        child: tokio::process::Child,
    ) -> Arc<Mutex<Option<tokio::process::Child>>> {
        let child_handle = Arc::new(Mutex::new(Some(child)));
        let child_clone = child_handle.clone();

        // 获取stdout和stderr
        let (stdout, stderr) = {
            let mut guard = child_clone.lock().await;
            if let Some(ref mut c) = *guard {
                (c.stdout.take(), c.stderr.take())
            } else {
                (None, None)
            }
        };

        // 启动日志读取
        if let Some(stdout) = stdout {
            let log_buffer = self.log_buffer.clone();
            let state = self.state.clone();
            let load_progress = self.load_progress.clone();
            let max_lines = self.max_log_lines;

            tokio::spawn(async move {
                let reader = BufReader::new(stdout);
                let mut lines = reader.lines();

                loop {
                    match lines.next_line().await {
                        Ok(Some(line)) => {
                            let entry = Self::parse_log_line(&line, "stdout");

                            // 检测加载进度
                            let new_progress = Self::detect_load_progress(&line);
                            {
                                let mut current = load_progress.write().await;
                                if new_progress > *current {
                                    *current = new_progress;
                                }
                            }

                            // 检测是否开始加载
                            {
                                let mut state_guard = state.write().await;
                                if *state_guard == GameState::Starting {
                                    *state_guard = GameState::Loading;
                                }
                            }

                            // 添加到缓冲区
                            let mut buffer = log_buffer.lock().await;
                            buffer.push_back(entry);
                            if buffer.len() > max_lines {
                                buffer.pop_front();
                            }
                        }
                        Ok(None) => break, // 流正常关闭
                        Err(e) => {
                            // 非 UTF-8 行或读取错误：记录后退出，避免静默吞错
                            crate::log_warn!("[Watcher] stdout 读取异常: {}", e);
                            break;
                        }
                    }
                }
            });
        }

        // 读取stderr
        if let Some(stderr) = stderr {
            let log_buffer = self.log_buffer.clone();
            let max_lines = self.max_log_lines;

            tokio::spawn(async move {
                let reader = BufReader::new(stderr);
                let mut lines = reader.lines();

                loop {
                    match lines.next_line().await {
                        Ok(Some(line)) => {
                            let entry = Self::parse_log_line(&line, "stderr");
                            let mut buffer = log_buffer.lock().await;
                            buffer.push_back(entry);
                            if buffer.len() > max_lines {
                                buffer.pop_front();
                            }
                        }
                        Ok(None) => break, // 流正常关闭
                        Err(e) => {
                            crate::log_warn!("[Watcher] stderr 读取异常: {}", e);
                            break;
                        }
                    }
                }
            });
        }

        // 启动状态检测
        let state = self.state.clone();
        let _load_progress = self.load_progress.clone();
        let log_buffer = self.log_buffer.clone();
        let _pid = self.pid;
        let exit_tx = self.exit_tx.clone();
        let version_id = self.version_id.clone();

        tokio::spawn(async move {
            // 等待进程结束
            let exit_code = {
                let mut guard = child_clone.lock().await;
                if let Some(ref mut c) = *guard {
                    c.wait().await.ok().map(|s| s.code().unwrap_or(-1))
                } else {
                    None
                }
            };

            let exit_code = exit_code.unwrap_or(-1);
            let logs = {
                let buffer = log_buffer.lock().await;
                buffer.iter().cloned().collect::<Vec<_>>()
            };

            // 分析是否崩溃
            let crash_info = Self::analyze_crash(exit_code, &logs);

            let exit_info = if let Some(info) = crash_info {
                let mut state_guard = state.write().await;
                *state_guard = GameState::Crashed(info.clone());
                ExitInfo {
                    code: exit_code,
                    is_normal: false,
                }
            } else {
                let exit_info = ExitInfo {
                    code: exit_code,
                    is_normal: exit_code == 0,
                };
                let mut state_guard = state.write().await;
                *state_guard = GameState::Exited(exit_info.clone());
                exit_info
            };

            // 发送退出通知
            let _ = exit_tx.send(Some(exit_info));

            log_info!(
                "[Watcher] Game process exited (PID: {}, code: {}, version: {})",
                _pid,
                exit_code,
                version_id
            );
        });

        child_handle
    }

    /// 解析日志行
    fn parse_log_line(line: &str, source: &str) -> LogEntry {
        let (level, _message) = Self::extract_log_level(line);

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

    /// 检测加载进度 (参考PCL2)
    fn detect_load_progress(line: &str) -> LoadProgress {
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

    /// 分析崩溃 (参考PCL2的ModCrash)
    fn analyze_crash(exit_code: i32, logs: &[LogEntry]) -> Option<CrashInfo> {
        // 正常退出
        if exit_code == 0 {
            return None;
        }

        // 收集错误日志
        let error_lines: Vec<String> = logs
            .iter()
            .filter(|e| e.level == LogLevel::Error || e.level == LogLevel::Fatal)
            .map(|e| e.message.clone())
            .collect();

        // 检查常见崩溃模式
        for line in &error_lines {
            let line_lower = line.to_lowercase();

            // Java 虚拟机创建失败
            if line_lower.contains("could not create the java virtual machine") {
                return Some(CrashInfo {
                    reason: "无法创建Java虚拟机".to_string(),
                    category: CrashCategory::Java,
                    log_lines: error_lines.clone(),
                    suggestion: "请检查JVM参数是否正确，或尝试更换Java版本".to_string(),
                    problematic_mod: None,
                });
            }

            // 内存不足
            if line_lower.contains("outofmemoryerror") || line_lower.contains("out of memory") {
                return Some(CrashInfo {
                    reason: "内存不足".to_string(),
                    category: CrashCategory::Memory,
                    log_lines: error_lines.clone(),
                    suggestion: "请增加最大内存分配，或关闭其他程序释放内存".to_string(),
                    problematic_mod: None,
                });
            }

            // OpenGL 错误
            if line_lower.contains("opengl")
                && (line_lower.contains("error") || line_lower.contains("not supported"))
            {
                return Some(CrashInfo {
                    reason: "OpenGL错误".to_string(),
                    category: CrashCategory::Graphics,
                    log_lines: error_lines.clone(),
                    suggestion: "请更新显卡驱动，或尝试降低游戏设置".to_string(),
                    problematic_mod: None,
                });
            }

            // Forge 错误
            if line_lower.contains("forge") && line_lower.contains("error") {
                return Some(CrashInfo {
                    reason: "Forge加载错误".to_string(),
                    category: CrashCategory::Forge,
                    log_lines: error_lines.clone(),
                    suggestion: "请尝试重新安装Forge，或检查Mod兼容性".to_string(),
                    problematic_mod: None,
                });
            }

            // Fabric 错误
            if line_lower.contains("fabric") && line_lower.contains("error") {
                return Some(CrashInfo {
                    reason: "Fabric加载错误".to_string(),
                    category: CrashCategory::Fabric,
                    log_lines: error_lines.clone(),
                    suggestion: "请尝试重新安装Fabric，或检查Mod兼容性".to_string(),
                    problematic_mod: None,
                });
            }
        }

        // 检查崩溃报告
        let has_crash_report = logs.iter().any(|e| {
            e.message.contains("Crash report saved to") || e.message.contains("crash-reports")
        });

        if has_crash_report {
            return Some(CrashInfo {
                reason: "游戏崩溃".to_string(),
                category: CrashCategory::Unknown,
                log_lines: error_lines,
                suggestion: "请查看崩溃报告获取详细信息".to_string(),
                problematic_mod: None,
            });
        }

        // 尝试从堆栈分析Mod
        let problematic_mod = Self::analyze_stack_for_mod(&error_lines);

        // 通用崩溃
        if exit_code != 0 {
            let reason = if let Some(ref mod_id) = problematic_mod {
                format!("可能由Mod '{}' 导致的崩溃", mod_id)
            } else {
                format!("游戏异常退出 (代码: {})", exit_code)
            };

            let category = if problematic_mod.is_some() {
                CrashCategory::Mod
            } else {
                CrashCategory::Unknown
            };

            let suggestion = if let Some(ref mod_id) = problematic_mod {
                format!("请尝试移除Mod '{}' 或更新到兼容版本", mod_id)
            } else {
                "请查看日志获取详细信息".to_string()
            };

            return Some(CrashInfo {
                reason,
                category,
                log_lines: error_lines,
                suggestion,
                problematic_mod,
            });
        }

        None
    }

    /// 从堆栈分析可能的Mod
    fn analyze_stack_for_mod(error_lines: &[String]) -> Option<String> {
        // 常见的非Mod包名
        let excluded_packages = [
            "java.",
            "javax.",
            "sun.",
            "com.sun.",
            "jdk.",
            "net.minecraft",
            "com.mojang",
            "net.minecraftforge",
            "net.fabricmc",
            "net.neoforged",
            "cpw.mods",
            "org.spongepowered",
            "org.apache",
            "com.google",
        ];

        for line in error_lines {
            // 查找 at 开头的堆栈行
            if line.trim().starts_with("at ") || line.contains("at ") {
                // 提取类名
                if let Some(at_pos) = line.find("at ") {
                    let rest = &line[at_pos + 3..];
                    if let Some(paren_pos) = rest.find('(') {
                        let class_path = &rest[..paren_pos];

                        // 检查是否是Mod包
                        let is_excluded =
                            excluded_packages.iter().any(|p| class_path.starts_with(p));
                        if !is_excluded && class_path.contains('.') {
                            // 可能是Mod包，尝试提取Mod ID
                            let parts: Vec<&str> = class_path.split('.').collect();
                            if parts.len() >= 3 {
                                // 通常格式: com.modid.xxx
                                let potential_mod_id = parts[1];
                                // 过滤掉常见的非Mod标识
                                if !["common", "core", "api", "util", "lib", "internal"]
                                    .contains(&potential_mod_id)
                                {
                                    return Some(potential_mod_id.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }

        None
    }

    /// 停止监控
    pub async fn stop(&self, child: &Arc<Mutex<Option<tokio::process::Child>>>) {
        let mut guard = child.lock().await;
        if let Some(ref mut c) = *guard {
            let _ = c.kill().await;
        }
    }
}

/// 从日志文件分析崩溃
pub async fn analyze_crash_report(game_dir: &PathBuf, _version_id: &str) -> Option<CrashInfo> {
    // 查找最新的崩溃报告
    let crash_reports_dir = game_dir.join("crash-reports");
    if !crash_reports_dir.exists() {
        return None;
    }

    // 读取最新的崩溃报告
    let mut latest_report = None;
    let mut latest_time = std::time::SystemTime::UNIX_EPOCH;

    if let Ok(entries) = std::fs::read_dir(&crash_reports_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "txt") {
                if let Ok(metadata) = entry.metadata() {
                    if let Ok(modified) = metadata.modified() {
                        if modified > latest_time {
                            latest_time = modified;
                            latest_report = Some(path);
                        }
                    }
                }
            }
        }
    }

    if let Some(report_path) = latest_report {
        if let Ok(content) = std::fs::read_to_string(&report_path) {
            return parse_crash_report(&content);
        }
    }

    None
}

/// 解析崩溃报告
fn parse_crash_report(content: &str) -> Option<CrashInfo> {
    let mut reason = "未知崩溃".to_string();
    let mut category = CrashCategory::Unknown;
    let mut suggestion = "请查看崩溃报告获取详细信息".to_string();

    // 提取描述
    if let Some(desc_start) = content.find("---- Minecraft Crash Report ----") {
        let desc_section = &content[desc_start..];
        if let Some(desc_line) = desc_section.lines().find(|l| l.contains("Description:")) {
            reason = desc_line.replace("Description:", "").trim().to_string();
        }
    }

    // 检测类别
    let content_lower = content.to_lowercase();

    if content_lower.contains("optifine") {
        category = CrashCategory::OptiFine;
        suggestion = "请尝试移除OptiFine或更换兼容版本".to_string();
    } else if content_lower.contains("forge") || content_lower.contains("neoforge") {
        category = CrashCategory::Forge;
        suggestion = "请尝试重新安装Forge/NeoForge".to_string();
    } else if content_lower.contains("fabric") {
        category = CrashCategory::Fabric;
        suggestion = "请尝试重新安装Fabric".to_string();
    } else if content_lower.contains("outofmemoryerror") {
        category = CrashCategory::Memory;
        suggestion = "请增加最大内存分配".to_string();
    } else if content_lower.contains("opengl") || content_lower.contains("pixel format") {
        category = CrashCategory::Graphics;
        suggestion = "请更新显卡驱动".to_string();
    }

    Some(CrashInfo {
        reason,
        category,
        log_lines: content.lines().take(100).map(|l| l.to_string()).collect(),
        suggestion,
        problematic_mod: None,
    })
}
