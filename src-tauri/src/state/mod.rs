//! 应用状态管理

use crate::{log_info, log_warn};
use crate::sdk::SdkInstance;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

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
    pub sdk: Arc<Mutex<Option<SdkInstance>>>,
    pub config: Arc<Mutex<AppConfig>>,
    pub auth: Arc<Mutex<AuthState>>,
}

/// 应用配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub game_dir: String,
    pub max_download_threads: u32,
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
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            game_dir: get_default_game_dir(),
            max_download_threads: 8,
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
            sdk: Arc::new(Mutex::new(sdk)),
            config: Arc::new(Mutex::new(config)),
            auth: Arc::new(Mutex::new(AuthState::default())),
        }
    }
}
