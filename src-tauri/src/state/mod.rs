//! 应用状态管理

use crate::sdk::SdkInstance;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

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
        }
    }
}

/// 认证状态
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuthState {
    pub current_user: Option<crate::sdk::AuthResult>,
    pub is_logged_in: bool,
}

/// 获取默认游戏目录
fn get_default_game_dir() -> String {
    if let Some(home_dir) = dirs::home_dir() {
        let minecraft_dir = home_dir.join(".minecraft");
        minecraft_dir.to_string_lossy().to_string()
    } else {
        ".minecraft".to_string()
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        Self {
            sdk: Arc::new(Mutex::new(None)),
            config: Arc::new(Mutex::new(AppConfig::default())),
            auth: Arc::new(Mutex::new(AuthState::default())),
        }
    }
}
