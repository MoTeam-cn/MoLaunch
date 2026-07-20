//! 启动流水线数据类型定义
//!
//! 包含启动阶段枚举、进度、配置、结果与错误类型。

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::super::AuthInfo;

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
    /// 自定义信息（对应 PCL2 VersionArgumentInfo，替换 ${version_type}）
    /// 非空时显示在游戏主界面左下角和 F3 左上角，空则删除 --versionType 参数
    #[serde(default)]
    pub custom_info: Option<String>,
    /// 自定义窗口标题（对应 PCL2 VersionArgumentTitle）
    /// 启动后通过 Win32 SetWindowText 强制改写游戏窗口标题，空则不改
    #[serde(default)]
    pub window_title: Option<String>,
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
