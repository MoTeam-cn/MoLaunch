//! 应用全局状态（AppState）

use crate::minecraft::online::bridge::VirtualLanBridge;
use crate::sdk::SdkInstance;
use crate::{log_info, log_warn};
use std::sync::{Arc, Mutex};
use tokio::sync::Mutex as TokioMutex;

use super::auth::AuthState;
use super::config::AppConfig;
use super::download::DownloadState;
use super::launch::LaunchHistory;
use crate::commands::auth::authlib::PendingAuthlibLogin;

/// 应用全局状态
///
/// 派生 `Clone`：所有字段均为 `Arc<...>`，克隆只是原子计数自增，开销极低。
/// `utils::dispatcher::Dispatcher` 的 handler 使用 owned `AppState` 参数（避免 HRTB），
/// IPC 入口通过 `state.inner().clone()` 获取 owned 实例后转发给 dispatcher。
#[derive(Clone)]
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
    pub download_cancel_flag: Arc<std::sync::atomic::AtomicBool>,
    /// 下载暂停信号（设置为 true 时，新任务不再开始，已进行的任务完成当前文件后等待）
    pub download_pause_flag: Arc<std::sync::atomic::AtomicBool>,
    /// 聊天流式回复取消信号（设置为 true 时，正在进行的 AI 回复尽快中断）
    pub chat_cancel_flag: Arc<std::sync::atomic::AtomicBool>,
    /// 日志分析流式取消信号（设置为 true 时，正在进行的 ai_analyze_log SSE 流尽快中断）
    pub analyze_cancel_flag: Arc<std::sync::atomic::AtomicBool>,
    /// authlib 多角色登录的待处理上下文
    ///
    /// `authlib_login` 返回 `NeedSelect` 时暂存，前端选定 profile 后
    /// `authlib_select_profile` 取出使用。同一时间只允许一个待处理登录。
    pub authlib_pending: Arc<TokioMutex<Option<PendingAuthlibLogin>>>,
    /// 联机虚拟网卡桥接器（阶段三子任务 5：TUN ↔ DataChannel 桥接）
    ///
    /// 房主与加入方共用同一实例，每次进入房间时 `tun_start` 创建并替换，
    /// `tun_stop` 关闭并置 None。同一时间仅允许一个桥接实例。
    pub virtual_lan_bridge: Arc<TokioMutex<Option<VirtualLanBridge>>>,
    /// MC 局域网服务器伪装（加入方本地伪装 LAN 服务器，多人游戏界面直接发现房主房间）
    ///
    /// 加入方进入房间且 TUN 就绪后由 `lan_fake_server_start` 创建并替换，
    /// 退出房间/停 TUN 时 `lan_fake_server_stop` 关闭并置 None。
    pub lan_fake_server: Arc<TokioMutex<Option<crate::commands::online::manager::LanFakeServer>>>,
    /// 应用句柄（Tauri setup 钩子中注入）
    ///
    /// 供后台任务/进度回调向前端 emit 事件（如 `download-progress`）。
    /// 在 setup 之前不会被使用（下载只能经 IPC 触发，IPC 在 setup 后可用）。
    pub app_handle: Arc<std::sync::OnceLock<tauri::AppHandle>>,
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
            Ok(Some(config)) => config,
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
            chat_cancel_flag: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            analyze_cancel_flag: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            authlib_pending: Arc::new(TokioMutex::new(None)),
            virtual_lan_bridge: Arc::new(TokioMutex::new(None)),
            lan_fake_server: Arc::new(TokioMutex::new(None)),
            app_handle: Arc::new(std::sync::OnceLock::new()),
        }
    }
}
