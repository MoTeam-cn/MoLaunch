//! DownloadManager 构造配置
//!
//! 从 `AppConfig.download` 提取 DownloadManager 所需字段，收敛调用层重复的 lock/extract 套件。

use tauri::AppHandle;

use crate::minecraft::sources::DownloadSourceMode;
use crate::state::AppState;

/// DownloadManager 构造配置
///
/// 仅包含 DownloadManager::new 所需的字段，不含 mirror_url / meta_source 等
/// 其他下载配置（那些由调用方自行从 config 读取）。
pub struct DownloadManagerConfig {
    /// 文件级并发数（同时下载多少个文件，由 Semaphore 控制）
    pub max_threads: usize,
    /// 单文件分片数（大文件拆几片，0/1 = 单流）
    pub chunk_count: usize,
    /// 全局限速 bytes/sec（0 = 不限速）
    pub speed_limit: u64,
    /// 下载源模式（Official / Mirror / Smart）
    pub source_mode: DownloadSourceMode,
    /// 自定义 User-Agent（None 使用默认 UA）
    pub user_agent: Option<String>,
    /// 应用句柄（下载开始/结束 emit `download-panel-state` 控制前端面板显隐）
    pub app_handle: Option<AppHandle>,
    /// 静默下载（不 emit 面板事件，供 Java 下载 / 程序更新 / 启动补全等后台任务）
    pub silent: bool,
}

impl DownloadManagerConfig {
    /// 从 AppState 提取下载配置
    ///
    /// 统一收敛 3 处重复的 `state.config.lock().await` + 字段提取逻辑。
    /// 读取 `config.download.source`（文件下载源），不是 `meta_source`（元数据源）。
    pub async fn from_state(state: &AppState) -> Self {
        let config = state.config.lock().await;
        Self {
            max_threads: config.download.max_threads as usize,
            // max(1) 保持与 resource.rs / concurrent.rs / tools/download.rs 历史行为一致
            // （chunk_count=0 在 chunk/mod.rs 中虽被 `<= 1` 提前 return 保护，但 max(1) 更防御性）
            chunk_count: config.download.chunk_count.max(1) as usize,
            speed_limit: config.download.max_speed,
            source_mode: DownloadSourceMode::from_str(&config.download.source),
            user_agent: None,
            app_handle: state.app_handle.get().cloned(),
            silent: false,
        }
    }

    /// 从 AppState 提取下载配置（使用 meta_source 而非 source）
    ///
    /// 阶段 6 新增：加载器 installer 历史用 `meta_source`（元数据源）构造 DownloadManager，
    /// 本方法保持该行为，同时让用户设置的 `max_threads`/`chunk_count`/`speed_limit` 对 installer 生效。
    ///
    /// 与 `from_state` 的唯一区别：`source_mode` 读 `config.download.meta_source`（元数据源），
    /// 而非 `config.download.source`（文件下载源）。
    pub async fn from_state_for_meta(state: &AppState) -> Self {
        let config = state.config.lock().await;
        Self {
            max_threads: config.download.max_threads as usize,
            chunk_count: config.download.chunk_count.max(1) as usize,
            speed_limit: config.download.max_speed,
            source_mode: DownloadSourceMode::from_str(&config.download.meta_source),
            user_agent: None,
            app_handle: state.app_handle.get().cloned(),
            silent: false,
        }
    }

    /// 标记静默下载（不 emit 面板显隐事件）
    ///
    /// 供后台任务使用：Java 下载 / 程序更新 / 启动时文件补全等场景
    /// 不应打扰用户弹出下载面板。
    pub fn with_silent(mut self, silent: bool) -> Self {
        self.silent = silent;
        self
    }

    /// 应用外部下载的可选覆盖参数（线程数 / 分片数 / 限速 / UA）
    ///
    /// 仅覆盖 `Some` 的字段，`None` 保持原有值。供外部下载工具按任务级覆盖全局配置。
    pub fn apply_overrides(
        &mut self,
        max_threads: Option<u32>,
        chunk_count: Option<u32>,
        max_speed: Option<u64>,
        user_agent: Option<String>,
    ) {
        if let Some(v) = max_threads {
            if v > 0 {
                self.max_threads = v as usize;
            }
        }
        if let Some(v) = chunk_count {
            self.chunk_count = v.max(1) as usize;
        }
        if let Some(v) = max_speed {
            self.speed_limit = v;
        }
        if let Some(ua) = user_agent {
            let trimmed = ua.trim().to_string();
            self.user_agent = if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            };
        }
    }
}
