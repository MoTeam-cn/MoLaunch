//! 配置持久化模块
//!
//! 负责将应用配置保存到文件和从文件加载配置
//! 配置文件位置: 运行目录/.Molaunch/config.json
//!
//! 策略：内存 + 文件双备份
//! - 启动时从文件加载到内存
//! - 运行时所有操作在内存中进行
//! - 退出时将内存中的配置保存到文件

use crate::state::AppConfig;
use std::path::PathBuf;

/// 获取配置目录路径 (.Molaunch)
pub fn get_config_dir() -> PathBuf {
    // 优先使用可执行文件所在目录
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            return exe_dir.join(".Molaunch");
        }
    }
    // 兜底：当前工作目录
    if let Ok(cwd) = std::env::current_dir() {
        return cwd.join(".Molaunch");
    }
    PathBuf::from(".Molaunch")
}

/// 获取配置文件路径 (config.json)
pub fn get_config_file_path() -> PathBuf {
    get_config_dir().join("config.json")
}

/// 确保配置目录存在
pub fn ensure_config_dir() -> std::io::Result<()> {
    let config_dir = get_config_dir();
    if !config_dir.exists() {
        std::fs::create_dir_all(&config_dir)?;
        log::info!("Created config directory: {}", config_dir.display());
    }
    Ok(())
}

/// 可持久化的配置结构体
/// 不包含敏感信息（如 API Key、认证 Token 等）
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PersistConfig {
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
    pub download_source: String,
}

impl From<&AppConfig> for PersistConfig {
    fn from(config: &AppConfig) -> Self {
        Self {
            game_dir: config.game_dir.clone(),
            max_download_threads: config.max_download_threads,
            log_level: config.log_level,
            min_memory: config.min_memory,
            max_memory: config.max_memory,
            theme: config.theme.clone(),
            language: config.language.clone(),
            mirror_url: config.mirror_url.clone(),
            mirror_url_meta: config.mirror_url_meta.clone(),
            mirror_url_download: config.mirror_url_download.clone(),
            mirror_mode: config.mirror_mode,
            max_download_speed: config.max_download_speed,
            download_source: config.download_source.clone(),
        }
    }
}

impl PersistConfig {
    /// 应用到 AppConfig
    pub fn apply_to(&self, config: &mut AppConfig) {
        config.game_dir = self.game_dir.clone();
        config.max_download_threads = self.max_download_threads;
        config.log_level = self.log_level;
        config.min_memory = self.min_memory;
        config.max_memory = self.max_memory;
        config.theme = self.theme.clone();
        config.language = self.language.clone();
        config.mirror_url = self.mirror_url.clone();
        config.mirror_url_meta = self.mirror_url_meta.clone();
        config.mirror_url_download = self.mirror_url_download.clone();
        config.mirror_mode = self.mirror_mode;
        config.max_download_speed = self.max_download_speed;
        config.download_source = self.download_source.clone();
    }
}

/// 保存配置到文件（备份）
pub fn save_config(config: &AppConfig) -> Result<(), String> {
    ensure_config_dir().map_err(|e| format!("Failed to create config directory: {}", e))?;

    let persist_config = PersistConfig::from(config);
    let json = serde_json::to_string_pretty(&persist_config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    let config_path = get_config_file_path();
    std::fs::write(&config_path, json)
        .map_err(|e| format!("Failed to write config file: {}", e))?;

    log::info!("Config saved to: {}", config_path.display());
    Ok(())
}

/// 从文件加载配置
pub fn load_config() -> Result<Option<AppConfig>, String> {
    let config_path = get_config_file_path();

    // 确保配置目录存在
    ensure_config_dir().map_err(|e| format!("Failed to create config directory: {}", e))?;

    if !config_path.exists() {
        log::info!("Config file not found, creating with defaults");
        // 创建默认配置文件
        let default_config = AppConfig::default();
        save_config(&default_config)?;
        return Ok(None);
    }

    let json = std::fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config file: {}", e))?;

    let persist_config: PersistConfig = serde_json::from_str(&json)
        .map_err(|e| format!("Failed to parse config file: {}", e))?;

    let mut config = AppConfig::default();
    persist_config.apply_to(&mut config);

    log::info!("Config loaded from: {}", config_path.display());
    Ok(Some(config))
}
