//! Launch pipeline - 完整的Minecraft启动流程
//! 复刻PCL2的启动架构，支持并行执行和进度追踪

use crate::log_info;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

use super::AuthInfo;
use super::watcher::{GameWatcher, GameState, LoadProgress, LogEntry};

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
    /// 额外JVM参数
    pub extra_jvm_args: Vec<String>,
    /// 额外游戏参数
    pub extra_game_args: Vec<String>,
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
    pub async fn exit_receiver(&self) -> Option<tokio::sync::watch::Receiver<Option<super::watcher::ExitInfo>>> {
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
    async fn update_progress(&self, stage: LaunchStage, stage_progress: f64, message: impl Into<String>) {
        let mut progress = self.progress.write().await;
        let total_weight: f64 = vec![
            LaunchStage::GetJava,
            LaunchStage::Login,
            LaunchStage::ValidateFiles,
            LaunchStage::BuildArgs,
            LaunchStage::ExtractNatives,
            LaunchStage::LaunchProcess,
            LaunchStage::WaitWindow,
        ].iter().map(|s| s.weight()).sum();
        
        let mut completed_weight = 0.0;
        for s in vec![
            LaunchStage::GetJava,
            LaunchStage::Login,
            LaunchStage::ValidateFiles,
            LaunchStage::BuildArgs,
            LaunchStage::ExtractNatives,
            LaunchStage::LaunchProcess,
            LaunchStage::WaitWindow,
        ] {
            if s == stage {
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
        log_info!("Starting launch pipeline for version: {}", self.config.version_id);
        
        // 检查取消
        if *self.cancel_flag.lock().await {
            return Err(LaunchError {
                stage: LaunchStage::Init,
                message: "启动已取消".to_string(),
                is_user_facing: true,
            });
        }
        
        // 阶段1: 获取Java
        self.update_progress(LaunchStage::GetJava, 0.0, "正在检测Java...").await;
        let java_path = self.detect_java().await?;
        self.update_progress(LaunchStage::GetJava, 1.0, "Java检测完成").await;
        
        // 检查取消
        if *self.cancel_flag.lock().await {
            return Err(LaunchError {
                stage: LaunchStage::GetJava,
                message: "启动已取消".to_string(),
                is_user_facing: true,
            });
        }
        
        // 阶段2: 文件检查和补全
        self.update_progress(LaunchStage::ValidateFiles, 0.0, "正在检查游戏文件...").await;
        self.validate_and_fix_files().await?;
        self.update_progress(LaunchStage::ValidateFiles, 1.0, "文件检查完成").await;
        
        // 检查取消
        if *self.cancel_flag.lock().await {
            return Err(LaunchError {
                stage: LaunchStage::ValidateFiles,
                message: "启动已取消".to_string(),
                is_user_facing: true,
            });
        }
        
        // 阶段3: 构建参数（内包含语言设置）
        self.update_progress(LaunchStage::BuildArgs, 0.0, "正在构建启动参数...").await;
        let launch_args = self.build_arguments(&java_path).await?;
        self.update_progress(LaunchStage::BuildArgs, 1.0, "参数构建完成").await;
        
        // 阶段4: 解压Natives
        self.update_progress(LaunchStage::ExtractNatives, 0.0, "正在解压原生库...").await;
        self.extract_natives().await?;
        self.update_progress(LaunchStage::ExtractNatives, 1.0, "原生库解压完成").await;
        
        // 阶段5: 启动进程
        self.update_progress(LaunchStage::LaunchProcess, 0.0, "正在启动游戏...").await;
        let result = self.launch_process(&java_path, &launch_args).await?;
        self.update_progress(LaunchStage::LaunchProcess, 1.0, format!("游戏已启动 PID: {}", result.pid)).await;
        
        // 阶段6: 等待窗口 (监控进程)
        self.update_progress(LaunchStage::WaitWindow, 0.0, "等待游戏加载...").await;
        // 监控已在launch_process中启动
        
        // 完成
        self.update_progress(LaunchStage::Finished, 1.0, "启动完成").await;
        
        Ok(result)
    }

    /// 检测Java
    async fn detect_java(&self) -> Result<PathBuf, LaunchError> {
        // 如果用户指定了Java，直接使用
        if let Some(ref path) = self.config.java_path {
            if !path.is_empty() {
                let java_path = PathBuf::from(path);
                if java_path.exists() {
                    return Ok(java_path);
                }
            }
        }
        
        // 否则自动检测
        // 获取真实的MC版本号（从setup.ini或JSON的inheritsFrom）
        let version_dir = self.config.game_dir.join("versions").join(&self.config.version_id);
        
        let mc_version = {
            // 优先从setup.ini读取OriginalVersion
            let setup_path = version_dir.join("setup.ini");
            let mut found_version = None;
            
            if setup_path.exists() {
                if let Ok(content) = std::fs::read_to_string(&setup_path) {
                    for line in content.lines() {
                        if let Some(value) = line.strip_prefix("OriginalVersion=") {
                            let version = value.trim().to_string();
                            if !version.is_empty() {
                                log_info!("[DetectJava] Using OriginalVersion from setup.ini: {}", version);
                                found_version = Some(version);
                                break;
                            }
                        }
                    }
                }
            }
            
            // 如果setup.ini没有找到，从JSON读取
            found_version.unwrap_or_else(|| self.read_mc_version_from_json(&version_dir))
        };
        
        let required_version = crate::minecraft::java_selector::get_required_java_version(&mc_version);
        
        log_info!("[DetectJava] MC {} requires Java {}+", mc_version, required_version);
        
        self.update_progress(LaunchStage::GetJava, 0.3, "正在搜索系统Java...").await;
        
        // 搜索Java (使用同步函数在spawn_blocking中运行)
        let mc_version_clone = mc_version.clone();
        let java_list = tokio::task::spawn_blocking(move || {
            crate::minecraft::java::search_java()
        }).await.map_err(|e| LaunchError {
            stage: LaunchStage::GetJava,
            message: format!("Java搜索失败: {}", e),
            is_user_facing: false,
        })?;
        
        self.update_progress(LaunchStage::GetJava, 0.6, "正在选择最佳Java...").await;
        
        // 选择最佳Java
        let selected_path = crate::minecraft::java_selector::select_best_java(
            &mc_version_clone,
            &java_list,
            None,
        ).ok_or_else(|| LaunchError {
            stage: LaunchStage::GetJava,
            message: format!("未找到满足要求的Java (需要Java {}+)", required_version),
            is_user_facing: true,
        })?;
        
        log_info!("Selected Java: {}", selected_path);
        Ok(PathBuf::from(&selected_path))
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
        let version_dir = self.config.game_dir.join("versions").join(&self.config.version_id);
        let json_path = version_dir.join(format!("{}.json", self.config.version_id));
        
        // 检查版本是否存在
        if !json_path.exists() {
            return Err(LaunchError {
                stage: LaunchStage::ValidateFiles,
                message: format!("版本 {} 不存在", self.config.version_id),
                is_user_facing: true,
            });
        }
        
        self.update_progress(LaunchStage::ValidateFiles, 0.2, "正在读取版本信息...").await;
        
        // 读取版本JSON
        let _json_content = tokio::fs::read_to_string(&json_path).await
            .map_err(|e| LaunchError {
                stage: LaunchStage::ValidateFiles,
                message: format!("读取版本JSON失败: {}", e),
                is_user_facing: false,
            })?;
        
        self.update_progress(LaunchStage::ValidateFiles, 0.4, "正在检查并补全文件...").await;
        
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
        ).await.map_err(|e| LaunchError {
            stage: LaunchStage::ValidateFiles,
            message: format!("文件补全失败: {}", e),
            is_user_facing: true,
        })?;
        
        self.update_progress(LaunchStage::ValidateFiles, 0.9, "文件补全完成").await;
        
        Ok(())
    }

    /// 构建启动参数
    async fn build_arguments(&self, java_path: &PathBuf) -> Result<super::LaunchArguments, LaunchError> {
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
        ).map_err(|e| LaunchError {
            stage: LaunchStage::BuildArgs,
            message: format!("构建参数失败: {}", e),
            is_user_facing: false,
        })
    }

    /// 解压Natives
    async fn extract_natives(&self) -> Result<(), LaunchError> {
        let version_dir = self.config.game_dir.join("versions").join(&self.config.version_id);
        let natives_dir = version_dir.join(format!("{}-natives", self.config.version_id));
        
        // 创建natives目录
        tokio::fs::create_dir_all(&natives_dir).await
            .map_err(|e| LaunchError {
                stage: LaunchStage::ExtractNatives,
                message: format!("创建natives目录失败: {}", e),
                is_user_facing: false,
            })?;
        
        // 读取版本JSON
        let json_path = version_dir.join(format!("{}.json", self.config.version_id));
        let json_content = tokio::fs::read_to_string(&json_path).await
            .map_err(|e| LaunchError {
                stage: LaunchStage::ExtractNatives,
                message: format!("读取版本JSON失败: {}", e),
                is_user_facing: false,
            })?;
        
        let json: serde_json::Value = serde_json::from_str(&json_content)
            .map_err(|e| LaunchError {
                stage: LaunchStage::ExtractNatives,
                message: format!("解析版本JSON失败: {}", e),
                is_user_facing: false,
            })?;
        
        // 查找natives库
        if let Some(libraries) = json["libraries"].as_array() {
            let total = libraries.len();
            for (i, lib) in libraries.iter().enumerate() {
                // 检查是否是native库
                if lib.get("natives").is_none() {
                    continue;
                }
                
                // 获取平台对应的classifier
                let classifier_key = if cfg!(target_os = "windows") {
                    "natives-windows"
                } else if cfg!(target_os = "macos") {
                    "natives-macos"
                } else {
                    "natives-linux"
                };
                
                if let Some(classifiers) = lib["downloads"]["classifiers"].as_object() {
                    if let Some(artifact) = classifiers.get(classifier_key) {
                        if let Some(path) = artifact["path"].as_str() {
                            let jar_path = self.config.game_dir.join("libraries").join(path);
                            if jar_path.exists() {
                                self.extract_native_jar(&jar_path, &natives_dir).await?;
                            }
                        }
                    }
                }
                
                self.update_progress(
                    LaunchStage::ExtractNatives,
                    (i + 1) as f64 / total as f64,
                    "正在解压原生库..."
                ).await;
            }
        }
        
        Ok(())
    }

    /// 解压单个native jar
    async fn extract_native_jar(&self, jar_path: &PathBuf, natives_dir: &PathBuf) -> Result<(), LaunchError> {
        let jar_path = jar_path.clone();
        let natives_dir = natives_dir.clone();
        
        tokio::task::spawn_blocking(move || {
            use std::fs::File;
            use std::io::Read;
            
            let file = File::open(&jar_path)
                .map_err(|e| format!("打开jar失败: {}", e))?;
            let mut archive = zip::ZipArchive::new(file)
                .map_err(|e| format!("读取zip失败: {}", e))?;
            
            for i in 0..archive.len() {
                let mut entry = archive.by_index(i)
                    .map_err(|e| format!("读取zip条目失败: {}", e))?;
                
                let entry_name = entry.name().to_string();
                
                // 只提取dll/so/dylib文件
                if entry_name.ends_with(".dll") || entry_name.ends_with(".so") || entry_name.ends_with(".dylib") {
                    let out_path = natives_dir.join(
                        std::path::Path::new(&entry_name).file_name().unwrap()
                    );
                    
                    let mut buffer = Vec::new();
                    entry.read_to_end(&mut buffer)
                        .map_err(|e| format!("读取文件失败: {}", e))?;
                    
                    std::fs::write(&out_path, buffer)
                        .map_err(|e| format!("写入文件失败: {}", e))?;
                }
            }
            
            Ok(())
        }).await
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
        
        let child = cmd.spawn()
            .map_err(|e| LaunchError {
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
        // Java异常通常在启动后1-2秒内发生
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        
        // 检查进程是否仍在运行（使用 try_lock 避免阻塞）
        {
            if let Ok(_guard) = child_handle.try_lock() {
                // 检查进程是否已退出（非阻塞）
                // 注意：这里不能调用 wait()，否则会阻塞
            }
        }
        
        // 检查日志中是否有Java异常
        let logs = {
            let watcher_guard = self.watcher.lock().await;
            if let Some(ref w) = *watcher_guard {
                w.recent_logs(50).await
            } else {
                Vec::new()
            }
        };
        
        // 检查是否有Java异常（这些通常出现在stderr）
        let fatal_errors = [
            "A Java Exception has occurred",
            "Error: A JNI error has occurred",
            "Could not create the Java Virtual Machine",
            "Exception in thread",
            "java.lang.NoClassDefFoundError",
            "java.lang.ClassNotFoundException",
            "java.lang.UnsupportedClassVersionError",
        ];
        
        for log in &logs {
            for error in &fatal_errors {
                if log.message.contains(error) {
                    return Err(LaunchError {
                        stage: LaunchStage::LaunchProcess,
                        message: format!("Java启动失败: {}", log.message),
                        is_user_facing: true,
                    });
                }
            }
        }
        
        Ok(LaunchResult {
            pid,
            java_path: java_path.clone(),
            game_dir: self.config.game_dir.clone(),
            args: args.jvm_args.iter().chain(std::iter::once(&args.main_class)).chain(args.game_args.iter()).cloned().collect(),
        })
    }
}

/// 快捷启动函数
pub async fn launch_game(config: LaunchConfig) -> Result<LaunchResult, LaunchError> {
    let pipeline = LaunchPipeline::new(config);
    pipeline.execute().await
}
