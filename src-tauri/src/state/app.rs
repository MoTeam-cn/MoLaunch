//! 应用全局状态（AppState）

use crate::sdk::SdkInstance;
use crate::{log_info, log_warn};
use std::sync::{Arc, Mutex};
use tokio::sync::Mutex as TokioMutex;

use super::auth::AuthState;
use super::config::AppConfig;
use super::download::DownloadState;
use super::launch::LaunchHistory;

/// 应用全局状态
pub struct AppState {
    pub sdk: Arc<TokioMutex<Option<SdkInstance>>>,
    pub config: Arc<TokioMutex<AppConfig>>,
    pub auth: Arc<TokioMutex<AuthState>>,
    pub auth_storage: Arc<crate::minecraft::auth::storage::AuthStorage>,
    pub download_state: Arc<Mutex<DownloadState>>,
    pub launch_history: Arc<TokioMutex<Vec<LaunchHistory>>>,
    pub current_pid: Arc<TokioMutex<Option<u32>>>,
    pub launch_pipeline: Arc<TokioMutex<Option<Arc<crate::minecraft::launch::LaunchPipeline>>>>,
    /// 下载取消信号（设置为 true 时，正在进行的下载任务会尽快中止）
    /// 参考 PCL2 中 LoaderTask 的 IsAborted 机制
    pub download_cancel_flag: Arc<std::sync::atomic::AtomicBool>,
    /// 下载暂停信号（设置为 true 时，新任务不再开始，已进行的任务完成当前文件后等待）
    /// 参考 PCL2 中下载暂停按钮的行为
    pub download_pause_flag: Arc<std::sync::atomic::AtomicBool>,
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

        // 创建 SDK Arc（需先创建以便共享给 auth_storage）
        let sdk_arc = Arc::new(TokioMutex::new(sdk));

        Self {
            sdk: sdk_arc.clone(),
            config: Arc::new(TokioMutex::new(config)),
            auth: Arc::new(TokioMutex::new(AuthState::default())),
            auth_storage: Arc::new(crate::minecraft::auth::storage::AuthStorage::new(sdk_arc)),
            download_state: Arc::new(Mutex::new(DownloadState::default())),
            launch_history: Arc::new(TokioMutex::new(Vec::new())),
            current_pid: Arc::new(TokioMutex::new(None)),
            launch_pipeline: Arc::new(TokioMutex::new(None)),
            download_cancel_flag: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            download_pause_flag: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }
}
