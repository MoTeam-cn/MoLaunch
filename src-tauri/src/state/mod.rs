//! 应用状态管理

use crate::{log_info, log_warn};
use crate::sdk::SdkInstance;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tokio::sync::Mutex as TokioMutex;

/// 本地认证结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalAuthResult {
    /// 用户名
    pub name: String,
    /// UUID
    pub uuid: String,
    /// 访问令牌
    pub access_token: String,
    /// 客户端令牌
    pub client_token: String,
    /// 登录类型
    pub login_type: String,
    /// 微软登录时的档案信息
    pub profile_json: Option<String>,
}

/// 应用全局状态
pub struct AppState {
    pub sdk: Arc<TokioMutex<Option<SdkInstance>>>,
    pub config: Arc<TokioMutex<AppConfig>>,
    pub auth: Arc<TokioMutex<AuthState>>,
    pub download_state: Arc<Mutex<DownloadState>>,
}

/// 阶段状态
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub enum StageStatus {
    #[default]
    Waiting,
    Loading,
    Finished,
    Failed,
}

/// 下载阶段
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadStage {
    pub name: String,
    pub progress: f64,
    pub weight: f64,
    pub status: StageStatus,
    pub bytes_downloaded: u64,
    pub bytes_total: u64,
    pub files_downloaded: usize,
    pub files_total: usize,
}

impl DownloadStage {
    pub fn new(name: impl Into<String>, weight: f64) -> Self {
        Self {
            name: name.into(),
            progress: 0.0,
            weight,
            status: StageStatus::Waiting,
            bytes_downloaded: 0,
            bytes_total: 0,
            files_downloaded: 0,
            files_total: 0,
        }
    }
}

/// 下载状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadState {
    pub is_active: bool,
    pub is_complete: bool,
    pub stages: Vec<DownloadStage>,
    pub current_stage_index: usize,
    pub global_speed: u64,
    pub global_bytes_downloaded: u64,
    pub global_bytes_total: u64,
    pub error_code: i32,
}

impl Default for DownloadState {
    fn default() -> Self {
        Self {
            is_active: false,
            is_complete: false,
            stages: vec![
                DownloadStage::new("版本清单", 2.0),
                DownloadStage::new("版本信息", 3.0),
                DownloadStage::new("客户端", 5.0),
                DownloadStage::new("库文件", 15.0),
                DownloadStage::new("资源文件", 20.0),
                DownloadStage::new("加载器安装", 30.0),
            ],
            current_stage_index: 0,
            global_speed: 0,
            global_bytes_downloaded: 0,
            global_bytes_total: 0,
            error_code: 0,
        }
    }
}

/// 应用配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub game_dir: String,
    pub max_download_threads: u32,
    pub chunk_count: u32,
    pub isolation_mode: u32,
    pub log_level: u32,
    pub min_memory: u32,
    pub max_memory: u32,
    pub theme: String,
    pub language: String,
    pub mirror_url: Option<String>,
    pub mirror_url_meta: Option<String>,
    pub mirror_url_download: Option<String>,
    pub mirror_mode: u32,
    pub max_download_speed: u64,
    pub download_source: String,  // "mirror" | "official" | "smart"
    pub proxy_mode: String,       // "none" | "system" | "custom"
    pub proxy_type: String,       // "http" | "https" | "socks5"
    pub proxy_url: String,        // 自定义代理地址，如 "127.0.0.1:7890"
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            game_dir: get_default_game_dir(),
            max_download_threads: 8,
            chunk_count: 4,
            isolation_mode: 4,
            log_level: 3,
            min_memory: 512,
            max_memory: 2048,
            theme: "system".to_string(),
            language: "zh-CN".to_string(),
            mirror_url: None,
            mirror_url_meta: None,
            mirror_url_download: None,
            mirror_mode: 0,
            max_download_speed: 0,
            download_source: "smart".to_string(),
            proxy_mode: "none".to_string(),
            proxy_type: "http".to_string(),
            proxy_url: String::new(),
        }
    }
}

/// 认证状态
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuthState {
    pub current_user: Option<LocalAuthResult>,
    pub is_logged_in: bool,
}

/// 获取默认游戏目录：启动器同级目录下的 .minecraft
fn get_default_game_dir() -> String {
    // 优先使用可执行文件所在目录
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let minecraft_dir = exe_dir.join(".minecraft");
            return minecraft_dir.to_string_lossy().to_string();
        }
    }
    // 兜底：当前工作目录
    if let Ok(cwd) = std::env::current_dir() {
        let minecraft_dir = cwd.join(".minecraft");
        return minecraft_dir.to_string_lossy().to_string();
    }
    ".minecraft".to_string()
}

/// 解析游戏目录：如果是相对路径，则相对于可执行文件目录
pub fn resolve_game_dir(game_dir: &str) -> std::path::PathBuf {
    let path = std::path::Path::new(game_dir);
    if path.is_absolute() {
        return path.to_path_buf();
    }
    // 相对路径：优先相对于可执行文件目录
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            return exe_dir.join(game_dir);
        }
    }
    // 兜底：当前工作目录
    std::env::current_dir().unwrap_or_default().join(game_dir)
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        // 尝试从文件加载配置
        let config = match crate::config::load_config() {
            Ok(Some(config)) => {
                log_info!("Loaded config from file");
                config
            }
            Ok(None) => {
                log_info!("No config file found, using defaults");
                AppConfig::default()
            }
            Err(e) => {
                log_warn!("Failed to load config: {}, using defaults", e);
                AppConfig::default()
            }
        };

        // 尝试加载 SDK lite
        let sdk = match crate::sdk::SdkInstance::load() {
            Ok(sdk) => {
                log_info!("SDK lite loaded successfully");
                Some(sdk)
            }
            Err(e) => {
                log_warn!("Failed to load SDK lite: {}", e);
                None
            }
        };

        Self {
            sdk: Arc::new(TokioMutex::new(sdk)),
            config: Arc::new(TokioMutex::new(config)),
            auth: Arc::new(TokioMutex::new(AuthState::default())),
            download_state: Arc::new(Mutex::new(DownloadState::default())),
        }
    }
}
