//! DownloadManager 构造配置
//!
//! 从 `AppConfig.download` 提取 DownloadManager 所需的 4 个字段，
//! 消灭调用层重复的 `state.config.lock() → extract → drop` 套件。
//! 现有 3 处重复（download.rs / install/mod.rs / manage.rs）统一收敛到 `from_state`。

use crate::minecraft::sources::DownloadSourceMode;
use crate::state::AppState;

/// DownloadManager 构造配置
///
/// 仅包含 DownloadManager::new 所需的 4 个字段，不含 mirror_url / meta_source 等
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
            chunk_count: config.download.chunk_count as usize,
            speed_limit: config.download.max_speed,
            source_mode: DownloadSourceMode::from_str(&config.download.source),
        }
    }
}
