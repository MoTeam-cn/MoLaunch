//! 应用全局状态（AppState）

use crate::minecraft::online::scaffolding::easytier::{EasyTier, PortForwardRule};
use crate::minecraft::online::scaffolding::server::ScaffoldingServer;
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
    /// easytier 虚拟网络进程（房主与房客共用，加入房间时创建，退出时停止）
    ///
    /// 房主固定虚拟 IP `10.144.144.1`（联机中心锚点），房客走 `--dhcp` 自动分配。
    /// `easytier_join` 创建并替换，`easytier_stop` 停止并置 None。
    pub easytier: Arc<TokioMutex<Option<EasyTier>>>,
    /// 房客 no-tun 用户态端口转发规则（`scaffolding_client_probe` 创建，`easytier_stop` 清理）
    ///
    /// no-tun 模式不创建虚拟网卡，房客经本地 port-forward 进出虚拟网络：
    /// 联机中心 TCP 转发（探测通道）+ MC TCP/UDP 转发（进服通道）。
    /// 记录用于端口变更时重建规则；停止 easytier 进程即隐式清除，此处仅同步记录。
    pub client_port_forwards: Arc<TokioMutex<Vec<PortForwardRule>>>,
    /// 房主网络凭据（network_name, network_secret），供监视循环按 MC 端口变化重建白名单
    ///
    /// `scaffolding_host_start` 写入，`scaffolding_host_stop` / 自动关房时清空。
    pub host_network_cred: Arc<TokioMutex<Option<(String, String)>>>,
    /// 联机中心 TCP 服务（仅房主，监听虚拟 IP，解析 §2.3 大端序帧）
    ///
    /// `scaffolding_host_start` 创建并替换，`scaffolding_host_stop` 停止并置 None。
    pub scaffolding_server: Arc<TokioMutex<Option<ScaffoldingServer>>>,
    /// 联机中心后台监视任务句柄（房主，每 5s 扫描 MC 端口 + 30s 自动关房）
    ///
    /// `scaffolding_host_start` spawn 并存储，`scaffolding_host_stop` 与自动关房时 abort 并置 None。
    pub scaffolding_host_watch: Arc<TokioMutex<Option<tokio::task::AbortHandle>>>,
    /// 房主手动指定的 MC 端口（最高权重：自动探测不覆盖；None 为自动模式）
    ///
    /// 由 `scaffolding_host_set_mc_port` 写入，`scaffolding_host_start`/`scaffolding_host_stop` 复位。
    pub manual_mc_port: Arc<TokioMutex<Option<u16>>>,
    /// MC 局域网服务器伪装（加入方本地伪装 LAN 服务器，多人游戏界面直接发现房主房间）
    ///
    /// 加入方进入房间后由 `lan_fake_server_start` 创建并替换（纯 UDP 广播，
    /// 进服流量由 port-forward 承担），退出房间时 `lan_fake_server_stop` 关闭并置 None。
    pub lan_fake_server: Arc<TokioMutex<Option<crate::commands::online::manager::LanFakeServer>>>,
    /// 应用句柄（Tauri setup 钩子中注入）
    ///
    /// 供后台任务/进度回调向前端 emit 事件（如 `download-progress`）。
    /// 在 setup 之前不会被使用（下载只能经 IPC 触发，IPC 在 setup 后可用）。
    pub app_handle: Arc<std::sync::OnceLock<tauri::AppHandle>>,
    /// 非静默进行中的下载批次计数（多个 DownloadManager 实例共享，
    /// 用于协调下载面板显隐：首个批次开始显示、最后批次结束隐藏）
    pub panel_active_count: Arc<std::sync::atomic::AtomicUsize>,
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
            easytier: Arc::new(TokioMutex::new(None)),
            client_port_forwards: Arc::new(TokioMutex::new(Vec::new())),
            host_network_cred: Arc::new(TokioMutex::new(None)),
            scaffolding_server: Arc::new(TokioMutex::new(None)),
            scaffolding_host_watch: Arc::new(TokioMutex::new(None)),
            manual_mc_port: Arc::new(TokioMutex::new(None)),
            lan_fake_server: Arc::new(TokioMutex::new(None)),
            app_handle: Arc::new(std::sync::OnceLock::new()),
            panel_active_count: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
        }
    }
}
