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

/// 下载进度广播通道容量
///
/// broadcast channel 满时丢弃旧消息（lagged），只保留最新 16 条。
/// WS 连接端 200ms 节流，正常情况下不会 lag。
const PROGRESS_CHANNEL_CAPACITY: usize = 16;

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
    /// 下载进度广播通道（WS 服务器订阅后推送给前端）
    ///
    /// 后端在 progress_callback / stage_callback / cancel / pause / resume 时
    /// 通过 `broadcast_progress` 发送 snapshot，WS 服务器订阅后以 200ms 节流推送。
    pub progress_tx: Arc<tokio::sync::broadcast::Sender<serde_json::Value>>,
    /// WebSocket 服务器端口（启动后写入，前端通过 `get_ws_port` IPC 查询）
    ///
    /// `OnceLock` 保证只写入一次（WS 服务器启动时），后续读取无需锁。
    pub ws_port: Arc<std::sync::OnceLock<u16>>,
    /// WebSocket 鉴权 token（启动时随机生成，前端建连后首条消息需携带）
    ///
    /// 防止本机其他进程或恶意软件直接连接 WS 端口窃取下载进度数据。
    /// 与 `ws_port` 一起在 `get_ws_port` IPC 中返回给前端。
    pub ws_token: Arc<std::sync::OnceLock<String>>,
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

        // 下载进度广播通道（WS 服务器订阅后推送给前端）
        let (progress_tx, _) = tokio::sync::broadcast::channel(PROGRESS_CHANNEL_CAPACITY);

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
            progress_tx: Arc::new(progress_tx),
            ws_port: Arc::new(std::sync::OnceLock::new()),
            ws_token: Arc::new(std::sync::OnceLock::new()),
        }
    }
}
