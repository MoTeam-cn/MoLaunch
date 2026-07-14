//! Launch pipeline - 完整的Minecraft启动流程
//! 复刻PCL2的启动架构，支持并行执行和进度追踪

use crate::{log_info, log_warn};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

use super::watcher::{ExitInfo, GameState, GameWatcher, LoadProgress, LogEntry};
use super::AuthInfo;

/// 启动阶段枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LaunchStage {
    /// 初始化
    Init,
    /// 获取Java
    GetJava,
    /// 登录验证
    Login,
    /// 文件检查/补全
    ValidateFiles,
    /// 构建参数
    BuildArgs,
    /// 启动前命令
    PreLaunch,
    /// 解压Natives
    ExtractNatives,
    /// 启动进程
    LaunchProcess,
    /// 等待窗口
    WaitWindow,
    /// 完成
    Finished,
    /// 失败
    Failed,
}

impl LaunchStage {
    pub fn weight(&self) -> f64 {
        match self {
            LaunchStage::Init => 0.0,
            LaunchStage::GetJava => 4.0,
            LaunchStage::Login => 15.0,
            LaunchStage::ValidateFiles => 15.0,
            LaunchStage::BuildArgs => 2.0,
            LaunchStage::PreLaunch => 1.0,
            LaunchStage::ExtractNatives => 2.0,
            LaunchStage::LaunchProcess => 2.0,
            LaunchStage::WaitWindow => 1.0,
            LaunchStage::Finished => 0.0,
            LaunchStage::Failed => 0.0,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            LaunchStage::Init => "初始化",
            LaunchStage::GetJava => "获取Java",
            LaunchStage::Login => "登录验证",
            LaunchStage::ValidateFiles => "文件检查",
            LaunchStage::BuildArgs => "构建参数",
            LaunchStage::PreLaunch => "启动前命令",
            LaunchStage::ExtractNatives => "解压原生库",
            LaunchStage::LaunchProcess => "启动进程",
            LaunchStage::WaitWindow => "等待窗口",
            LaunchStage::Finished => "完成",
            LaunchStage::Failed => "失败",
        }
    }
}

/// 启动进度
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchProgress {
    /// 当前阶段
    pub stage: LaunchStage,
    /// 阶段内进度 (0.0-1.0)
    pub stage_progress: f64,
    /// 总体进度 (0.0-1.0)
    pub overall_progress: f64,
    /// 状态消息
    pub message: String,
}

/// 启动配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchConfig {
    /// 游戏目录
    pub game_dir: PathBuf,
    /// 版本ID
    pub version_id: String,
    /// 认证信息
    pub auth_info: AuthInfo,
    /// 最小内存(MB)
    pub min_memory: u32,
    /// 最大内存(MB)
    pub max_memory: u32,
    /// 窗口宽度
    pub window_width: Option<u32>,
    /// 窗口高度
    pub window_height: Option<u32>,
    /// 服务器地址
    pub server_address: Option<String>,
    /// 服务器端口
    pub server_port: Option<u32>,
    /// 隔离模式
    pub isolation_mode: u32,
    /// 用户指定的Java路径(空=自动)
    pub java_path: Option<String>,
    /// Java 选择模式：None/空/auto=自动选择, "auto_version"=自动选择指定版本范围, "folder"=使用版本文件夹中的 Java, "custom"=使用指定的 Java
    pub java_mode: Option<String>,
    /// 自动选择时的最小 Java 主版本（仅 auto_version 模式生效，0=不限）
    pub java_version_min: u32,
    /// 自动选择时的最大 Java 主版本（仅 auto_version 模式生效，0=不限）
    pub java_version_max: u32,
    /// 下载源模式（"official"/"mirror"/"smart"），用于 Java 自动下载
    pub download_source: String,
    /// 自定义镜像源 URL（None 或空则用 BMCLAPI）
    pub mirror_url: Option<String>,
    /// 额外JVM参数
    pub extra_jvm_args: Vec<String>,
    /// 额外游戏参数
    pub extra_game_args: Vec<String>,
    /// 启动前执行命令（None=不执行）
    pub pre_launch_cmd: Option<String>,
    /// Tauri AppHandle（用于 Java 自动下载时推送进度事件）
    #[serde(skip)]
    pub app_handle: Option<tauri::AppHandle>,
}

/// 启动结果
#[derive(Debug, Clone)]
pub struct LaunchResult {
    /// 进程ID
    pub pid: u32,
    /// 使用的Java路径
    pub java_path: PathBuf,
    /// 游戏目录
    pub game_dir: PathBuf,
    /// 启动参数
    pub args: Vec<String>,
}

/// 启动错误
#[derive(Debug, Clone)]
pub struct LaunchError {
    /// 错误阶段
    pub stage: LaunchStage,
    /// 错误消息
    pub message: String,
    /// 是否用户可见
    pub is_user_facing: bool,
}

impl std::fmt::Display for LaunchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.stage.name(), self.message)
    }
}

impl std::error::Error for LaunchError {}

/// 启动流水线
pub struct LaunchPipeline {
    config: LaunchConfig,
    progress: Arc<RwLock<LaunchProgress>>,
    #[allow(dead_code)]
    current_stage: Arc<Mutex<LaunchStage>>,
    cancel_flag: Arc<Mutex<bool>>,
    watcher: Arc<Mutex<Option<GameWatcher>>>,
    child_process: Arc<Mutex<Option<Arc<tokio::sync::Mutex<Option<tokio::process::Child>>>>>>,
}

impl LaunchPipeline {
    /// 创建新的启动流水线
    pub fn new(config: LaunchConfig) -> Self {
        Self {
            config,
            progress: Arc::new(RwLock::new(LaunchProgress {
                stage: LaunchStage::Init,
                stage_progress: 0.0,
                overall_progress: 0.0,
                message: "初始化中...".to_string(),
            })),
            current_stage: Arc::new(Mutex::new(LaunchStage::Init)),
            cancel_flag: Arc::new(Mutex::new(false)),
            watcher: Arc::new(Mutex::new(None)),
            child_process: Arc::new(Mutex::new(None)),
        }
    }

    /// 获取游戏状态
    pub async fn game_state(&self) -> Option<GameState> {
        let watcher = self.watcher.lock().await;
        if let Some(ref w) = *watcher {
            Some(w.state().await)
        } else {
            None
        }
    }

    /// 获取加载进度
    pub async fn load_progress(&self) -> Option<LoadProgress> {
        let watcher = self.watcher.lock().await;
        if let Some(ref w) = *watcher {
            Some(w.load_progress().await)
        } else {
            None
        }
    }

    /// 获取最近日志
    pub async fn recent_logs(&self, count: usize) -> Vec<LogEntry> {
        let watcher_guard = self.watcher.lock().await;
        if let Some(ref w) = *watcher_guard {
            w.recent_logs(count).await
        } else {
            Vec::new()
        }
    }

    /// 获取退出通知接收器
    pub async fn exit_receiver(
        &self,
    ) -> Option<tokio::sync::watch::Receiver<Option<super::watcher::ExitInfo>>> {
        let watcher_guard = self.watcher.lock().await;
        watcher_guard.as_ref().map(|w| w.exit_receiver())
    }

    /// 停止游戏
    pub async fn stop_game(&self) {
        let child = self.child_process.lock().await;
        if let Some(ref child) = *child {
            let watcher = self.watcher.lock().await;
            if let Some(ref w) = *watcher {
                w.stop(child).await;
            }
        }
    }

    /// 获取当前进度
    pub async fn progress(&self) -> LaunchProgress {
        self.progress.read().await.clone()
    }

    /// 取消启动
    pub async fn cancel(&self) {
        *self.cancel_flag.lock().await = true;
    }

    /// 更新进度
    async fn update_progress(
        &self,
        stage: LaunchStage,
        stage_progress: f64,
        message: impl Into<String>,
    ) {
        let mut progress = self.progress.write().await;
        let stages = vec![
            LaunchStage::GetJava,
            LaunchStage::ValidateFiles,
            LaunchStage::BuildArgs,
            LaunchStage::PreLaunch,
            LaunchStage::ExtractNatives,
            LaunchStage::LaunchProcess,
            LaunchStage::WaitWindow,
        ];
        let total_weight: f64 = stages.iter().map(|s| s.weight()).sum();

        let mut completed_weight = 0.0;
        for s in &stages {
            if *s == stage {
                completed_weight += s.weight() * stage_progress;
                break;
            } else {
                completed_weight += s.weight();
            }
        }

        progress.stage = stage;
        progress.stage_progress = stage_progress;
        progress.overall_progress = completed_weight / total_weight;
        progress.message = message.into();
    }

    /// 执行启动流程
    pub async fn execute(&self) -> Result<LaunchResult, LaunchError> {
        log_info!(
            "Starting launch pipeline for version: {}",
            self.config.version_id
        );

        // 检查取消
        if *self.cancel_flag.lock().await {
            return Err(LaunchError {
                stage: LaunchStage::Init,
                message: "启动已取消".to_string(),
                is_user_facing: true,
            });
        }

        // 阶段1: 获取Java
        self.update_progress(LaunchStage::GetJava, 0.0, "正在检测Java...")
            .await;
        let java_path = self.detect_java().await?;
        self.update_progress(LaunchStage::GetJava, 1.0, "Java检测完成")
            .await;

        // 检查取消
        if *self.cancel_flag.lock().await {
            return Err(LaunchError {
                stage: LaunchStage::GetJava,
                message: "启动已取消".to_string(),
                is_user_facing: true,
            });
        }

        // 阶段2: 文件检查和补全
        self.update_progress(LaunchStage::ValidateFiles, 0.0, "正在检查游戏文件...")
            .await;
        self.validate_and_fix_files().await?;
        self.update_progress(LaunchStage::ValidateFiles, 1.0, "文件检查完成")
            .await;

        // 检查取消
        if *self.cancel_flag.lock().await {
            return Err(LaunchError {
                stage: LaunchStage::ValidateFiles,
                message: "启动已取消".to_string(),
                is_user_facing: true,
            });
        }

        // 阶段3: 构建参数（内包含语言设置）
        self.update_progress(LaunchStage::BuildArgs, 0.0, "正在构建启动参数...")
            .await;
        let launch_args = self.build_arguments(&java_path).await?;
        self.update_progress(LaunchStage::BuildArgs, 1.0, "参数构建完成")
            .await;

        // 检查取消
        if *self.cancel_flag.lock().await {
            return Err(LaunchError {
                stage: LaunchStage::BuildArgs,
                message: "启动已取消".to_string(),
                is_user_facing: true,
            });
        }

        // 阶段4: 启动前命令（advance_run_cmd，参考 PCL2 的 PreLaunch）
        if self.config.pre_launch_cmd.is_some() {
            self.update_progress(LaunchStage::PreLaunch, 0.0, "正在执行启动前命令...")
                .await;
            self.run_pre_launch().await?;
            self.update_progress(LaunchStage::PreLaunch, 1.0, "启动前命令执行完成")
                .await;
        }

        // 阶段5: 解压Natives
        self.update_progress(LaunchStage::ExtractNatives, 0.0, "正在解压原生库...")
            .await;
        self.extract_natives().await?;
        self.update_progress(LaunchStage::ExtractNatives, 1.0, "原生库解压完成")
            .await;

        // 阶段6: 启动进程
        self.update_progress(LaunchStage::LaunchProcess, 0.0, "正在启动游戏...")
            .await;
        let result = self.launch_process(&java_path, &launch_args).await?;
        self.update_progress(
            LaunchStage::LaunchProcess,
            1.0,
            format!("游戏已启动 PID: {}", result.pid),
        )
        .await;

        // 阶段7: 等待窗口 (监控进程)
        self.update_progress(LaunchStage::WaitWindow, 0.0, "等待游戏加载...")
            .await;
        // 监控已在launch_process中启动

        // 完成
        self.update_progress(LaunchStage::Finished, 1.0, "启动完成")
            .await;

        Ok(result)
    }

    /// 执行启动前命令（语法同 Windows cmd，不等待退出，失败仅记录日志）
    async fn run_pre_launch(&self) -> Result<(), LaunchError> {
        let cmd_str = match self.config.pre_launch_cmd.as_ref() {
            Some(s) if !s.is_empty() => s.clone(),
            _ => return Ok(()),
        };

        // 安全检测：检查命令字符串中的危险字符/关键词（仅警告，不阻止执行）
        // 保留底层执行方式（cmd /C 或 sh -c）以维持向后兼容
        match validate_pre_launch_cmd(&cmd_str) {
            Err(reason) => log_warn!(
                "PreLaunch executing command: {} (warning: contains potentially dangerous characters: {})",
                cmd_str,
                reason
            ),
            Ok(()) => log_warn!("PreLaunch executing command: {}", cmd_str),
        }

        #[cfg(target_os = "windows")]
        let (program, args) = ("cmd", vec!["/C".to_string(), cmd_str.clone()]);
        #[cfg(not(target_os = "windows"))]
        let (program, args) = ("sh", vec!["-c".to_string(), cmd_str.clone()]);

        let game_dir = self.config.game_dir.clone();
        let result = tokio::task::spawn_blocking(move || {
            let mut cmd = std::process::Command::new(program);
            cmd.args(&args).current_dir(&game_dir);
            #[cfg(target_os = "windows")]
            {
                use std::os::windows::process::CommandExt;
                cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
            }
            cmd.output()
        })
        .await;

        match result {
            Ok(Ok(output)) => {
                if !output.status.success() {
                    log_info!(
                        "[PreLaunch] Command exited with status: {}",
                        output.status
                    );
                }
                Ok(())
            }
            Ok(Err(e)) => {
                log_info!("[PreLaunch] Failed to execute command: {}", e);
                // 启动前命令失败不中断启动流程（与 PCL2 行为一致）
                Ok(())
            }
            Err(e) => {
                log_info!("[PreLaunch] Task spawn failed: {}", e);
                Ok(())
            }
        }
    }

    /// 检测Java
    async fn detect_java(&self) -> Result<PathBuf, LaunchError> {
        // 获取版本目录
        let version_dir = self
            .config
            .game_dir
            .join("versions")
            .join(&self.config.version_id);

        // 读取 MC 版本号和加载器类型（从 setup.ini 或 JSON）
        let (mc_version, loader) = self.read_mc_version_and_loader(&version_dir);

        // 读取版本 JSON 中的 Mojang 官方 Java 要求
        let mojang_req = self.read_mojang_java_requirement(&version_dir);

        // 计算 Java 版本约束区间 [min, max]
        let (mut min_req, mut max_req) =
            crate::minecraft::java_selector::get_java_version_range(&mc_version, loader.as_deref());

        // Mojang 官方要求覆盖规则表的下限（取更严格者）
        if let Some(mojang_min) = mojang_req {
            min_req = Some(min_req.map_or(mojang_min, |m| m.max(mojang_min)));
        }

        // Java 选择模式（来自 setup.ini 的 JavaMode 字段）
        let java_mode = self.config.java_mode.as_deref().unwrap_or("").trim();
        // auto_version 模式：用用户指定的版本范围覆盖规则表的约束
        if java_mode.eq_ignore_ascii_case("auto_version") {
            if self.config.java_version_min > 0 {
                min_req = Some(self.config.java_version_min);
            }
            // max=0 表示不限上限（清除规则表的上限约束）
            if self.config.java_version_max > 0 {
                max_req = Some(self.config.java_version_max);
            } else {
                max_req = None;
            }
        }

        log_info!(
            "[DetectJava] MC {} (loader: {:?}) requires Java {}-{} (mojang: {:?}, mode: {:?})",
            mc_version,
            loader,
            min_req.unwrap_or(0),
            max_req.map(|m| m.to_string()).unwrap_or("∞".to_string()),
            mojang_req,
            java_mode
        );

        // folder 模式：优先使用版本文件夹下的 Java（runtime/jre 子目录）
        if java_mode.eq_ignore_ascii_case("folder") {
            if let Some(folder_java) = self.find_version_folder_java(&version_dir) {
                log_info!("[DetectJava] Using Java from version folder: {}", folder_java.display());
                return Ok(folder_java);
            }
            log_warn!("[DetectJava] folder 模式未在版本文件夹下找到 Java，回退到自动选择");
        }

        // custom 模式：使用用户指定的 Java
        if java_mode.eq_ignore_ascii_case("custom") {
            if let Some(ref path) = self.config.java_path {
                if !path.is_empty() {
                    let java_path = PathBuf::from(path);
                    if java_path.exists() {
                        // 校验版本兼容性（参考 PCL2：不兼容时阻断启动并提示）
                        if let Some(java_ver) =
                            crate::minecraft::java::detect_java_version(path)
                        {
                            if let Err((_cur, cur_min, cur_max)) =
                                crate::minecraft::java_selector::check_java_compatible(
                                    java_ver,
                                    &mc_version,
                                    loader.as_deref(),
                                )
                            {
                                let req_desc = match (cur_min, cur_max) {
                                    (Some(mn), Some(mx)) if mn == mx => format!("需要 Java {}", mn),
                                    (Some(mn), Some(mx)) => format!("需要 Java {}~{}", mn, mx),
                                    (Some(mn), None) => format!("至少需要 Java {}", mn),
                                    (None, Some(mx)) => format!("最高兼容到 Java {}", mx),
                                    _ => String::new(),
                                };
                                return Err(LaunchError {
                                    stage: LaunchStage::GetJava,
                                    message: format!(
                                        "Java 版本不兼容：当前版本{}，{}。\n请前往 版本设置 → 游戏 Java 重新选择，或切换为「自动选择」",
                                        java_ver, req_desc
                                    ),
                                    is_user_facing: true,
                                });
                            }
                        }
                        return Ok(java_path);
                    }
                    log_warn!("[DetectJava] User-specified Java not found: {}", path);
                }
            }
        }

        self.update_progress(LaunchStage::GetJava, 0.3, "正在搜索系统Java...")
            .await;

        // 搜索Java (使用同步函数在spawn_blocking中运行)
        let mc_version_clone = mc_version.clone();
        let loader_clone = loader.clone();
        let game_dir_clone = self.config.game_dir.clone();
        let java_list = tokio::task::spawn_blocking(move || {
            crate::minecraft::java::search_java_with_paths(&[game_dir_clone])
        })
            .await
            .map_err(|e| LaunchError {
                stage: LaunchStage::GetJava,
                message: format!("Java搜索失败: {}", e),
                is_user_facing: false,
            })?;

        self.update_progress(LaunchStage::GetJava, 0.6, "正在选择最佳Java...")
            .await;

        // 选择最佳Java（支持加载器约束）
        let selected_path = match crate::minecraft::java_selector::select_best_java_with_loader(
            &mc_version_clone,
            loader_clone.as_deref(),
            &java_list,
            None,
        ) {
            Some(path) => path,
            None => {
                // 自动下载补全：用约束区间的下限作为下载目标
                // 优先用 min_req（已被 mojang JSON 覆盖的值），其次用规则表推荐版本
                let target_major = min_req.unwrap_or_else(|| {
                    crate::minecraft::java_selector::get_recommended_java_version(&mc_version_clone)
                });
                // 校验推荐版本落在需求区间内
                let in_range = min_req.map_or(true, |m| target_major >= m)
                    && max_req.map_or(true, |m| target_major <= m);
                if !in_range {
                    return Err(LaunchError {
                        stage: LaunchStage::GetJava,
                        message: format!(
                            "未找到满足要求的Java (需要Java {}-{})，且无法自动下载匹配版本。\n请在 版本设置 → 游戏 Java 中手动选择或下载。",
                            min_req.unwrap_or(0),
                            max_req.map(|m| m.to_string()).unwrap_or("∞".to_string())
                        ),
                        is_user_facing: true,
                    });
                }

                self.update_progress(
                    LaunchStage::GetJava,
                    0.7,
                    &format!("未找到兼容 Java，正在自动下载 Java {}...", target_major),
                )
                .await;

                let app_handle = self.config.app_handle.clone();
                let dl_mode =
                    crate::minecraft::sources::DownloadSourceMode::from_str(
                        &self.config.download_source,
                    );
                let mirror_url = self.config.mirror_url.clone();
                let downloaded = crate::minecraft::java::download::download_java_runtime(
                    target_major,
                    dl_mode,
                    mirror_url.as_deref(),
                    app_handle.as_ref(),
                )
                .await
                .map_err(|e| LaunchError {
                    stage: LaunchStage::GetJava,
                    message: format!(
                        "自动下载 Java {} 失败：{}\n请在 版本设置 → 游戏 Java 中手动下载或选择。",
                        target_major, e
                    ),
                    is_user_facing: true,
                })?;

                log_info!("[DetectJava] Auto-downloaded Java: {}", downloaded.display());
                downloaded.to_string_lossy().to_string()
            }
        };

        log_info!("Selected Java: {}", selected_path);
        Ok(PathBuf::from(&selected_path))
    }

    /// 在版本文件夹下查找 Java 可执行文件（folder 模式）
    /// 查找路径：{version_dir}/runtime/{任意子目录}/bin/javaw.exe（Windows）或 bin/java（Unix）
    /// 也兼容 {version_dir}/jre/ 等常见命名
    fn find_version_folder_java(&self, version_dir: &std::path::Path) -> Option<PathBuf> {
        let exe_name = if cfg!(windows) { "javaw.exe" } else { "java" };
        // 候选根目录：runtime/、jre/、java/
        let candidates = ["runtime", "jre", "java"];
        for dir in candidates {
            let root = version_dir.join(dir);
            if !root.exists() {
                continue;
            }
            // 遍历 root 下的所有子目录（包括 root 本身），查找 bin/{exe_name}
            if let Some(found) = Self::search_java_in_dir(&root, exe_name) {
                return Some(found);
            }
        }
        None
    }

    /// 递归查找目录下的 bin/{exe_name}（最多 4 层深度，避免遍历过大）
    fn search_java_in_dir(dir: &std::path::Path, exe_name: &str) -> Option<PathBuf> {
        // 直接检查 dir/bin/{exe_name}
        let direct = dir.join("bin").join(exe_name);
        if direct.exists() {
            return Some(direct);
        }
        // 限制深度为 4 层
        fn walk(dir: &std::path::Path, exe_name: &str, depth: u32) -> Option<PathBuf> {
            if depth > 4 {
                return None;
            }
            let entries = match std::fs::read_dir(dir) {
                Ok(e) => e,
                Err(_) => return None,
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }
                // 优先检查 path/bin/{exe_name}
                let candidate = path.join("bin").join(exe_name);
                if candidate.exists() {
                    return Some(candidate);
                }
                if let Some(found) = walk(&path, exe_name, depth + 1) {
                    return Some(found);
                }
            }
            None
        }
        walk(dir, exe_name, 0)
    }

    /// 读取 MC 版本号和加载器类型（从 setup.ini 或 JSON）
    fn read_mc_version_and_loader(&self, version_dir: &std::path::Path) -> (String, Option<String>) {
        // 优先从 setup.ini 读取 OriginalVersion 和 Type
        let setup_path = version_dir.join("setup.ini");
        let mut mc_version = None;
        let mut loader = None;

        if setup_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&setup_path) {
                for line in content.lines() {
                    if let Some(value) = line.strip_prefix("OriginalVersion=") {
                        let v = value.trim().to_string();
                        if !v.is_empty() {
                            mc_version = Some(v);
                        }
                    } else if let Some(value) = line.strip_prefix("Type=") {
                        let t = value.trim().to_lowercase();
                        if !t.is_empty() && t != "release" && t != "snapshot" {
                            loader = Some(t);
                        }
                    }
                }
            }
        }

        let mc_version = mc_version.unwrap_or_else(|| self.read_mc_version_from_json(version_dir));
        (mc_version, loader)
    }

    /// 从版本 JSON 读取 Mojang 官方 Java 版本要求
    fn read_mojang_java_requirement(&self, version_dir: &std::path::Path) -> Option<u32> {
        let json_path = version_dir.join(format!("{}.json", self.config.version_id));
        if !json_path.exists() {
            return None;
        }
        let content = std::fs::read_to_string(&json_path).ok()?;
        let json: serde_json::Value = serde_json::from_str(&content).ok()?;
        crate::minecraft::java_selector::get_mojang_java_requirement(&json)
    }

    /// 从JSON读取MC版本号（从inheritsFrom或id）
    fn read_mc_version_from_json(&self, version_dir: &std::path::Path) -> String {
        let json_path = version_dir.join(format!("{}.json", self.config.version_id));
        if json_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&json_path) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    // 优先使用inheritsFrom
                    if let Some(inherits_from) = json.get("inheritsFrom").and_then(|v| v.as_str()) {
                        if !inherits_from.is_empty() {
                            return inherits_from.to_string();
                        }
                    }
                    // 否则使用id
                    if let Some(id) = json.get("id").and_then(|v| v.as_str()) {
                        return id.to_string();
                    }
                }
            }
        }
        self.config.version_id.clone()
    }

    /// 检查文件完整性并自动补全
    async fn validate_and_fix_files(&self) -> Result<(), LaunchError> {
        let version_dir = self
            .config
            .game_dir
            .join("versions")
            .join(&self.config.version_id);
        let json_path = version_dir.join(format!("{}.json", self.config.version_id));

        // 检查版本是否存在
        if !json_path.exists() {
            return Err(LaunchError {
                stage: LaunchStage::ValidateFiles,
                message: format!("版本 {} 不存在", self.config.version_id),
                is_user_facing: true,
            });
        }

        self.update_progress(LaunchStage::ValidateFiles, 0.2, "正在读取版本信息...")
            .await;

        // 读取版本JSON
        let _json_content =
            tokio::fs::read_to_string(&json_path)
                .await
                .map_err(|e| LaunchError {
                    stage: LaunchStage::ValidateFiles,
                    message: format!("读取版本JSON失败: {}", e),
                    is_user_facing: false,
                })?;

        self.update_progress(LaunchStage::ValidateFiles, 0.4, "正在检查并补全文件...")
            .await;

        // 使用配置中的参数
        let game_dir = self.config.game_dir.clone();
        let version_id = self.config.version_id.clone();
        let source_mode = crate::minecraft::sources::DownloadSourceMode::Smart;

        // 直接调用异步函数，使用默认参数
        crate::minecraft::download::fix_version_files(
            &version_id,
            &game_dir,
            None, // mirror_url
            8,    // max_threads
            4,    // chunk_count
            0,    // speed_limit
            source_mode,
        )
        .await
        .map_err(|e| LaunchError {
            stage: LaunchStage::ValidateFiles,
            message: format!("文件补全失败: {}", e),
            is_user_facing: true,
        })?;

        self.update_progress(LaunchStage::ValidateFiles, 0.9, "文件补全完成")
            .await;

        Ok(())
    }

    /// 构建启动参数
    async fn build_arguments(
        &self,
        java_path: &PathBuf,
    ) -> Result<super::LaunchArguments, LaunchError> {
        super::build_launch_arguments(
            &self.config.game_dir,
            &self.config.version_id,
            java_path,
            &self.config.auth_info,
            self.config.min_memory,
            self.config.max_memory,
            self.config.window_width,
            self.config.window_height,
            self.config.server_address.as_deref(),
            self.config.server_port,
            self.config.isolation_mode,
            &self.config.extra_jvm_args,
            &self.config.extra_game_args,
        )
        .map_err(|e| LaunchError {
            stage: LaunchStage::BuildArgs,
            message: format!("构建参数失败: {}", e),
            is_user_facing: false,
        })
    }

    /// 解压Natives
    async fn extract_natives(&self) -> Result<(), LaunchError> {
        let version_dir = self
            .config
            .game_dir
            .join("versions")
            .join(&self.config.version_id);
        let natives_dir = version_dir.join(format!("{}-natives", self.config.version_id));

        // 创建natives目录
        tokio::fs::create_dir_all(&natives_dir)
            .await
            .map_err(|e| LaunchError {
                stage: LaunchStage::ExtractNatives,
                message: format!("创建natives目录失败: {}", e),
                is_user_facing: false,
            })?;

        // 读取版本JSON
        let json_path = version_dir.join(format!("{}.json", self.config.version_id));
        let json_content =
            tokio::fs::read_to_string(&json_path)
                .await
                .map_err(|e| LaunchError {
                    stage: LaunchStage::ExtractNatives,
                    message: format!("读取版本JSON失败: {}", e),
                    is_user_facing: false,
                })?;

        let json: serde_json::Value =
            serde_json::from_str(&json_content).map_err(|e| LaunchError {
                stage: LaunchStage::ExtractNatives,
                message: format!("解析版本JSON失败: {}", e),
                is_user_facing: false,
            })?;

        // 查找natives库
        if let Some(libraries) = json["libraries"].as_array() {
            let total = libraries.len();
            for (i, lib) in libraries.iter().enumerate() {
                // 应用 rules 过滤（平台适配）
                let rules: Option<Vec<serde_json::Value>> = lib
                    .get("rules")
                    .and_then(|v| v.as_array())
                    .map(|a| a.clone());
                if !crate::minecraft::version::libraries::check_rules(&rules) {
                    continue;
                }

                // 模式 A（旧版）：library 有 "natives" 字段 + "downloads.classifiers"
                if let Some(natives_field) = lib.get("natives").and_then(|v| v.as_object()) {
                    let platform_key = if cfg!(target_os = "windows") {
                        "windows"
                    } else if cfg!(target_os = "macos") {
                        "osx"
                    } else {
                        "linux"
                    };

                    let classifier_key = match natives_field.get(platform_key).and_then(|v| v.as_str()) {
                        Some(c) => c.to_string(),
                        None => continue,
                    };

                    if let Some(classifiers) = lib["downloads"]["classifiers"].as_object() {
                        let artifact = classifiers.get(&classifier_key).or_else(|| {
                            let base = classifier_key
                                .split('-')
                                .take(2)
                                .collect::<Vec<_>>()
                                .join("-");
                            if base != classifier_key {
                                classifiers.get(&base)
                            } else {
                                None
                            }
                        });

                        if let Some(artifact) = artifact {
                            if let Some(path) = artifact["path"].as_str() {
                                let jar_path = self.config.game_dir.join("libraries").join(path);
                                if jar_path.exists() {
                                    let jar_sha1 = artifact["sha1"].as_str();
                                    log_info!(
                                        "[Natives] Processing native JAR: {} (expected sha1: {:?})",
                                        jar_path.display(),
                                        jar_sha1
                                    );
                                    self.extract_native_jar(&jar_path, &natives_dir, jar_sha1)
                                        .await?;
                                }
                            }
                        }
                    }
                    self.update_progress(
                        LaunchStage::ExtractNatives,
                        (i + 1) as f64 / total as f64,
                        "正在解压原生库...",
                    )
                    .await;
                    continue;
                }

                // 模式 B（Forge 26.2+ 新格式）：library 无 "natives" 字段，但 name 含 classifier（如 "natives-windows-x86"）
                // 这类直接用 downloads.artifact.path 解压
                if let Some(name) = lib["name"].as_str() {
                    let parts: Vec<&str> = name.split(':').collect();
                    if parts.len() > 3 {
                        let classifier = parts[3];
                        if classifier.starts_with("natives-") {
                            // 架构过滤：避免解压错误架构的 native
                            if !crate::minecraft::version::libraries::is_native_matching_arch(
                                classifier,
                            ) {
                                self.update_progress(
                                    LaunchStage::ExtractNatives,
                                    (i + 1) as f64 / total as f64,
                                    "正在解压原生库...",
                                )
                                .await;
                                continue;
                            }
                            if let Some(path) = lib["downloads"]["artifact"]["path"].as_str() {
                                let jar_path = self.config.game_dir.join("libraries").join(path);
                                if jar_path.exists() {
                                    let jar_sha1 = lib["downloads"]["artifact"]["sha1"].as_str();
                                    log_info!(
                                        "[Natives] Processing native JAR: {} (expected sha1: {:?})",
                                        jar_path.display(),
                                        jar_sha1
                                    );
                                    self.extract_native_jar(&jar_path, &natives_dir, jar_sha1)
                                        .await?;
                                }
                            }
                        }
                    }
                }

                self.update_progress(
                    LaunchStage::ExtractNatives,
                    (i + 1) as f64 / total as f64,
                    "正在解压原生库...",
                )
                .await;
            }
        }

        Ok(())
    }

    /// 解压单个native jar
    ///
    /// `expected_sha1` 为版本 JSON 中记录的 JAR 文件 SHA1（可选）。
    /// - 若提供：先校验 JAR 文件 SHA1，匹配才解压；不匹配则跳过提取并记录警告。
    /// - 若为 None：记录警告（无法校验），仍按原逻辑解压。
    /// 每个提取出的 DLL/SO/DYLIB 会计算并记录其 SHA1，便于审计。
    async fn extract_native_jar(
        &self,
        jar_path: &PathBuf,
        natives_dir: &PathBuf,
        expected_sha1: Option<&str>,
    ) -> Result<(), LaunchError> {
        let jar_path = jar_path.clone();
        let natives_dir = natives_dir.clone();
        let expected_sha1 = expected_sha1.map(|s| s.to_string());

        tokio::task::spawn_blocking(move || {
            use sha1::Digest;
            use std::fs::File;
            use std::io::Read;

            // SHA1 校验：如果提供了预期 SHA1，先校验 JAR 文件完整性（CWE-494/CWE-345）
            if let Some(ref expected) = expected_sha1 {
                let jar_bytes = match std::fs::read(&jar_path) {
                    Ok(b) => b,
                    Err(e) => return Err(format!("读取jar文件失败: {}", e)),
                };
                let mut hasher = sha1::Sha1::new();
                hasher.update(&jar_bytes);
                let actual = hex::encode(hasher.finalize());
                if actual.eq_ignore_ascii_case(expected) {
                    log_info!(
                        "[Natives] JAR SHA1 verified: {} (sha1={})",
                        jar_path.display(),
                        actual
                    );
                } else {
                    log_warn!(
                        "[Natives] JAR SHA1 mismatch for {}: expected={}, actual={} — skipping extraction",
                        jar_path.display(),
                        expected,
                        actual
                    );
                    return Ok(());
                }
            } else {
                log_warn!(
                    "[Natives] No expected SHA1 for JAR {}, skipping verification",
                    jar_path.display()
                );
            }

            log_info!(
                "[Natives] Extracting native JAR: {}",
                jar_path.display()
            );

            let file = File::open(&jar_path).map_err(|e| format!("打开jar失败: {}", e))?;
            let mut archive =
                zip::ZipArchive::new(file).map_err(|e| format!("读取zip失败: {}", e))?;

            for i in 0..archive.len() {
                let mut entry = archive
                    .by_index(i)
                    .map_err(|e| format!("读取zip条目失败: {}", e))?;

                let entry_name = entry.name().to_string();

                // 只提取dll/so/dylib文件
                if entry_name.ends_with(".dll")
                    || entry_name.ends_with(".so")
                    || entry_name.ends_with(".dylib")
                {
                    let out_path =
                        natives_dir.join(std::path::Path::new(&entry_name).file_name().unwrap());

                    let mut buffer = Vec::new();
                    entry
                        .read_to_end(&mut buffer)
                        .map_err(|e| format!("读取文件失败: {}", e))?;

                    // 计算提取文件的 SHA1 用于审计日志
                    let mut hasher = sha1::Sha1::new();
                    hasher.update(&buffer);
                    let file_sha1 = hex::encode(hasher.finalize());

                    std::fs::write(&out_path, &buffer)
                        .map_err(|e| format!("写入文件失败: {}", e))?;

                    log_info!(
                        "[Natives] Extracted: {} (size={}, sha1={})",
                        out_path.display(),
                        buffer.len(),
                        file_sha1
                    );
                }
            }

            Ok(())
        })
        .await
        .map_err(|e| LaunchError {
            stage: LaunchStage::ExtractNatives,
            message: format!("任务执行失败: {}", e),
            is_user_facing: false,
        })?
        .map_err(|e: String| LaunchError {
            stage: LaunchStage::ExtractNatives,
            message: e,
            is_user_facing: false,
        })
    }

    /// 启动游戏进程
    async fn launch_process(
        &self,
        java_path: &PathBuf,
        args: &super::LaunchArguments,
    ) -> Result<LaunchResult, LaunchError> {
        use tokio::process::Command;

        let mut cmd = Command::new(java_path);

        // 添加JVM参数
        for arg in &args.jvm_args {
            cmd.arg(arg);
        }

        // 添加主类
        cmd.arg(&args.main_class);

        // 添加游戏参数
        for arg in &args.game_args {
            cmd.arg(arg);
        }

        // 设置工作目录
        cmd.current_dir(&self.config.game_dir);

        // 设置环境变量
        cmd.env("appdata", &self.config.game_dir);

        // 重定向stdout和stderr以便监控
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());

        // Windows: 不显示控制台窗口
        #[cfg(target_os = "windows")]
        {
            cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
        }

        log_info!("Launching: {} {:?}", java_path.display(), args.jvm_args);

        let child = cmd.spawn().map_err(|e| LaunchError {
            stage: LaunchStage::LaunchProcess,
            message: format!("启动进程失败: {}", e),
            is_user_facing: true,
        })?;

        let pid = child.id().unwrap_or(0);
        log_info!("Game process started with PID: {}", pid);

        // 创建监控器
        let watcher = GameWatcher::new(
            pid,
            self.config.game_dir.clone(),
            self.config.version_id.clone(),
        );

        // 启动监控
        let child_handle = watcher.start_monitoring(child).await;

        // 保存监控器和子进程引用
        *self.watcher.lock().await = Some(watcher);
        *self.child_process.lock().await = Some(child_handle.clone());

        // 等待一段时间检查进程是否立即崩溃
        // Forge 启动较慢，等待 5 秒覆盖早期崩溃
        let exit_rx = {
            let watcher_guard = self.watcher.lock().await;
            if let Some(ref w) = *watcher_guard {
                w.exit_receiver()
            } else {
                return Err(LaunchError {
                    stage: LaunchStage::LaunchProcess,
                    message: "Watcher not available".to_string(),
                    is_user_facing: false,
                });
            }
        };

        // 早期崩溃检测：轮询最多 2 秒，每 200ms 检查一次
        // - 进程退出且非 0 → 报告崩溃
        // - 日志出现 Java 异常 → 报告崩溃
        // - 日志出现正常启动标志（LWJGL/GL info/Setting user 等）→ 立即返回
        let fatal_errors = [
            "A Java Exception has occurred",
            "Error: A JNI error has occurred",
            "Could not create the Java Virtual Machine",
            "Exception in thread",
            "java.lang.NoClassDefFoundError",
            "java.lang.ClassNotFoundException",
            "java.lang.UnsupportedClassVersionError",
        ];
        // 正常启动标志：出现这些说明游戏已开始正常加载，不再需要等待
        let healthy_signs = [
            "LWJGL",
            "Setting user",
            "GL info",
            "OpenAL",
            "lwjgl",
            "ModLauncher",
            "EARLYDISPLAY",
            "Launching target",
        ];

        let mut exit_info_caught: Option<ExitInfo> = None;
        let mut error_logs: Option<Vec<String>> = None;

        let poll_deadline = tokio::time::Instant::now() + tokio::time::Duration::from_secs(2);
        loop {
            if tokio::time::Instant::now() >= poll_deadline {
                break;
            }

            // 先检查进程是否退出（非阻塞：借用 watch 的已接收值）
            // exit_rx 每轮 clone 一次用于 changed 超时探测
            {
                let mut rx = exit_rx.clone();
                match tokio::time::timeout(
                    tokio::time::Duration::from_millis(200),
                    rx.changed(),
                )
                .await
                {
                    Ok(Ok(())) => {
                        if let Some(ref info) = *rx.borrow() {
                            exit_info_caught = Some(info.clone());
                            break; // 进程已退出，跳出处理
                        }
                    }
                    _ => {}
                }
            }

            // 检查日志
            let logs = {
                let watcher_guard = self.watcher.lock().await;
                if let Some(ref w) = *watcher_guard {
                    w.recent_logs(80).await
                } else {
                    Vec::new()
                }
            };

            let logs_chronological: Vec<&LogEntry> = logs.iter().rev().collect();

            // 先检查是否有 Java 异常
            for (idx, log) in logs_chronological.iter().enumerate() {
                for error in &fatal_errors {
                    if log.message.contains(error) {
                        let tail: Vec<String> = logs_chronological
                            .iter()
                            .skip(idx)
                            .take(30)
                            .map(|l| l.message.clone())
                            .collect();
                        error_logs = Some(tail);
                        break;
                    }
                }
                if error_logs.is_some() {
                    break;
                }
            }
            if let Some(tail) = error_logs.take() {
                return Err(LaunchError {
                    stage: LaunchStage::LaunchProcess,
                    message: format!("Java启动失败:\n{}", tail.join("\n")),
                    is_user_facing: true,
                });
            }

            // 检查是否有正常启动标志 → 立即返回
            let has_healthy = logs_chronological
                .iter()
                .any(|l| healthy_signs.iter().any(|s| l.message.contains(s)));
            if has_healthy {
                break;
            }
        }

        // 处理轮询期间捕获的进程退出
        if let Some(exit_info) = exit_info_caught {
            if exit_info.code != 0 {
                let logs = {
                    let watcher_guard = self.watcher.lock().await;
                    if let Some(ref w) = *watcher_guard {
                        w.recent_logs(40).await
                    } else {
                        Vec::new()
                    }
                };
                let tail: Vec<String> = logs.iter().take(40).map(|l| l.message.clone()).collect();
                return Err(LaunchError {
                    stage: LaunchStage::LaunchProcess,
                    message: format!(
                        "游戏进程退出（代码: {}）\n最近日志:\n{}",
                        exit_info.code,
                        tail.join("\n")
                    ),
                    is_user_facing: true,
                });
            }
        }

        Ok(LaunchResult {
            pid,
            java_path: java_path.clone(),
            game_dir: self.config.game_dir.clone(),
            args: args
                .jvm_args
                .iter()
                .chain(std::iter::once(&args.main_class))
                .chain(args.game_args.iter())
                .cloned()
                .collect(),
        })
    }
}

/// 检测 PreLaunch 命令字符串中的危险字符/关键词。
/// 返回 `Err(reason)` 表示检测到危险模式（reason 为具体原因），`Ok(())` 表示未检测到。
/// 注意：仅用于日志警告，不阻止命令执行（保持向后兼容，用户可能确实需要这些命令）。
fn validate_pre_launch_cmd(cmd: &str) -> Result<(), String> {
    // 命令分隔符：&、&&、|
    if cmd.contains('&') || cmd.contains('|') {
        return Err("command separator (& or |)".to_string());
    }
    // 重定向：>、<
    if cmd.contains('>') || cmd.contains('<') {
        return Err("redirection (> or <)".to_string());
    }
    // 命令替换：反引号、$(
    if cmd.contains('`') || cmd.contains("$(") {
        return Err("command substitution (` or $()".to_string());
    }
    // 常见攻击载荷关键词（不区分大小写）
    let lower = cmd.to_lowercase();
    for keyword in ["powershell", "curl", "wget", "iex", "invoke-"] {
        if lower.contains(keyword) {
            return Err(format!("suspicious keyword: {}", keyword));
        }
    }
    Ok(())
}

/// 快捷启动函数
pub async fn launch_game(config: LaunchConfig) -> Result<LaunchResult, LaunchError> {
    let pipeline = LaunchPipeline::new(config);
    pipeline.execute().await
}
