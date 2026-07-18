//! Launch pipeline - 完整的Minecraft启动流程
//! 复刻PCL2的启动架构，支持并行执行和进度追踪
//!
//! 结构：
//! - pipeline.rs: 结构体定义 + execute 编排 + 公共 API
//! - pipeline/java_check.rs: Java 检测与校验
//! - pipeline/natives.rs: Natives 原生库解压
//! - pipeline/pre_launch.rs: 启动前命令执行
//! - pipeline/process_spawn.rs: 游戏进程启动与早期崩溃检测

use crate::{log_info, log_warn};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

use super::watcher::{GameState, GameWatcher, LoadProgress, LogEntry};
use super::AuthInfo;

mod java_check;
mod natives;
mod pre_launch;
mod process_spawn;

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
    /// 禁用 Java Launch Wrapper（修复 Java 18- 中文路径启动问题）
    pub disable_jlw: bool,
    /// 禁用 LWJGL Unsafe Agent（修复 LWJGL 3.4.1 性能问题）
    pub disable_lua: bool,
    /// 忽略 Java 兼容性警告（custom 模式下跳过版本兼容性校验，强制使用用户指定的 Java）
    pub ignore_java_warning: bool,
    /// 关闭文件校验（跳过 libraries/assets/主 jar 文件的校验和补全）
    pub disable_assets_verify: bool,
    /// 使用高性能显卡（启动前将 Java 和 PCL exe 写入 Windows 注册表 GpuPreference=2）
    /// 参考 PCL2 ModLaunch.vb McLaunchPrerun 中 SetGPUPreference
    pub use_dedicated_gpu: bool,
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
    pub(super) config: LaunchConfig,
    progress: Arc<RwLock<LaunchProgress>>,
    #[allow(dead_code)]
    current_stage: Arc<Mutex<LaunchStage>>,
    cancel_flag: Arc<Mutex<bool>>,
    pub(super) watcher: Arc<Mutex<Option<GameWatcher>>>,
    pub(super) child_process: Arc<Mutex<Option<Arc<tokio::sync::Mutex<Option<tokio::process::Child>>>>>>,
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
    pub(super) async fn update_progress(
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
        // 高性能显卡设置也在这一阶段执行（参考 PCL2 McLaunchPrerun）
        if self.config.use_dedicated_gpu {
            self.update_progress(LaunchStage::PreLaunch, 0.0, "正在设置高性能显卡...")
                .await;
            if let Err(e) = self.set_gpu_preference(&java_path).await {
                log_warn!("[Launch] 设置高性能显卡失败: {}", e);
            }
        }
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

        // 版本独立设置 advance_disable_assets_verify：跳过文件校验和补全
        // 参考 PCL2 VersionAdvanceAssetsV2：完全不更改 assets；不校验 libraries、第三方登录库与版本主 jar 文件
        if self.config.disable_assets_verify {
            log_info!("[ValidateFiles] disable_assets_verify=true，跳过文件校验和补全");
            self.update_progress(LaunchStage::ValidateFiles, 1.0, "已跳过文件校验")
                .await;
            return Ok(());
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
            self.config.disable_jlw,
            self.config.disable_lua,
        )
        .map_err(|e| LaunchError {
            stage: LaunchStage::BuildArgs,
            message: format!("构建参数失败: {}", e),
            is_user_facing: false,
        })
    }
}
