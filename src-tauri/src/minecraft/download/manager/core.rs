//! DownloadManager 主实现：批量下载编排（限速 / URL 重排 / 进度跟踪）

use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::sync::Mutex as StdMutex;
use tauri::AppHandle;
use tauri::Emitter;

use super::super::super::sources::DownloadSourceMode;
use super::super::config::DownloadManagerConfig;
use super::super::types::{ContentValidator, GlobalProgress};
use crate::state::AppState;

/// 下载面板显隐事件（前端监听此事件控制浮动下载面板显示/隐藏）
pub const PANEL_STATE_EVENT: &str = "download-panel-state";

/// 下载管理器
pub struct DownloadManager {
    pub(crate) client: reqwest::Client,
    pub(crate) max_threads: usize,
    pub(crate) chunk_count: usize,
    pub(crate) speed_limit: u64,
    pub(crate) source_mode: DownloadSourceMode,
    pub(crate) progress: Arc<StdMutex<GlobalProgress>>,
    /// 取消信号（可选，由外部传入）
    pub(crate) cancel_flag: Option<Arc<std::sync::atomic::AtomicBool>>,
    /// 暂停信号（可选，由外部传入）
    pub(crate) pause_flag: Option<Arc<std::sync::atomic::AtomicBool>>,
    /// 应用句柄（非 None 且非 silent 时下载开始/结束 emit 面板事件）
    pub(crate) app_handle: Option<AppHandle>,
    /// 静默下载（不 emit 面板事件，供 Java 下载 / 程序更新 / 启动补全等后台任务）
    pub(crate) silent: bool,
    /// 保持调用方传入的 URL 顺序（跳过 reorder_urls，供镜像优先+官方保底等场景）
    pub(crate) preserve_order: bool,
    /// 内容校验器（可选）：大小校验通过后执行，失败视为该下载源无效（删除文件回退下一 URL）
    pub(crate) content_validator: Option<ContentValidator>,
    /// 共享批次计数（来自 AppState，协调并发批次的面板显隐）
    pub(crate) active_batches: Option<Arc<std::sync::atomic::AtomicUsize>>,
}

impl DownloadManager {
    pub fn new(
        max_threads: usize,
        chunk_count: usize,
        speed_limit: u64,
        source_mode: DownloadSourceMode,
    ) -> Self {
        let client = crate::http::get_client();

        Self {
            client,
            max_threads,
            chunk_count,
            speed_limit,
            source_mode,
            progress: Arc::new(StdMutex::new(GlobalProgress::default())),
            cancel_flag: None,
            pause_flag: None,
            app_handle: None,
            silent: false,
            preserve_order: false,
            content_validator: None,
            active_batches: None,
        }
    }

    /// 从 DownloadManagerConfig 构造（统一参数来源，避免硬编码）
    pub fn from_config(config: &DownloadManagerConfig) -> Self {
        let client = match config.user_agent.as_deref() {
            Some(ua) if !ua.is_empty() => crate::http::build_client_with_user_agent(ua, None),
            _ => crate::http::get_client(),
        };
        let mut manager = Self::new(
            config.max_threads,
            config.chunk_count,
            config.speed_limit,
            config.source_mode,
        );
        manager.client = client;
        manager.app_handle = config.app_handle.clone();
        manager.silent = config.silent;
        manager.active_batches = config.panel_counter.clone();
        manager
    }

    /// 从 AppState 提取下载配置并构造（统一收敛 3 处重复的 lock/extract/drop）
    ///
    /// `app_handle` / `panel_counter` 由 `DownloadManagerConfig::from_state` 自动填充
    pub async fn from_state(state: &AppState) -> Self {
        Self::from_config(&DownloadManagerConfig::from_state(state).await)
    }

    /// 设置取消信号（用于支持前端取消下载）
    pub fn with_cancel_flag(mut self, flag: Arc<std::sync::atomic::AtomicBool>) -> Self {
        self.cancel_flag = Some(flag);
        self
    }

    /// 设置暂停信号（用于支持前端暂停/恢复下载）
    pub fn with_pause_flag(mut self, flag: Arc<std::sync::atomic::AtomicBool>) -> Self {
        self.pause_flag = Some(flag);
        self
    }

    /// 设置静默模式（不 emit 面板显隐事件）
    ///
    /// 供后台任务使用：Java 下载 / 程序更新 / 启动时文件补全等场景
    /// 不应打扰用户弹出下载面板。
    pub fn with_silent(mut self, silent: bool) -> Self {
        self.silent = silent;
        self
    }

    /// 保持调用方传入的 URL 顺序（跳过 reorder_urls）
    ///
    /// 供"镜像优先 + 官方保底"等调用方已排好序的场景使用，
    /// 避免按用户下载源模式过滤/重排破坏意图。
    pub fn with_preserve_order(mut self, preserve: bool) -> Self {
        self.preserve_order = preserve;
        self
    }

    /// 设置内容校验器：大小校验通过后执行，失败视为该下载源无效（删除文件回退下一 URL）
    ///
    /// 供"镜像优先 + 官方保底"等场景使用：镜像返回 HTML/挑战页等非目标内容
    /// （HTTP 200 且长度匹配，大小校验无法识别）时自动剔除该源，回退官方保底。
    pub fn with_content_validator(mut self, validator: ContentValidator) -> Self {
        self.content_validator = Some(validator);
        self
    }

    /// 接入共享批次计数（协调并发批次的面板显隐）
    ///
    /// 多个 DownloadManager 实例共享同一个 `AppState.panel_active_count`，
    /// 首个批次开始 emit 显示、最后批次结束 emit 隐藏，避免并发下载时面板提前消失。
    pub fn with_panel_counter(mut self, counter: Arc<std::sync::atomic::AtomicUsize>) -> Self {
        self.active_batches = Some(counter);
        self
    }

    /// 通知前端下载面板显隐（silent 或缺少 AppHandle 时静默跳过）
    fn notify_panel(&self, visible: bool) {
        if self.silent {
            return;
        }
        let Some(app) = &self.app_handle else {
            return;
        };
        let _ = app.emit(PANEL_STATE_EVENT, serde_json::json!({ "visible": visible }));
    }

    /// 持有面板显示（多批次串行操作的顶层调用方使用）
    ///
    /// 与 `release_panel` 配对：批次循环外持有一次后，内部各 `download_batch`
    /// 的增减只在共享计数上叠加，阶段切换时计数不再瞬间归零，
    /// 避免 emit `visible:false` 导致前端下载面板闪烁/误跳转。
    /// silent 或缺少 AppHandle 时为无操作。
    pub fn hold_panel(&self) {
        if self.silent || self.app_handle.is_none() {
            return;
        }
        let first = match &self.active_batches {
            Some(c) => c.fetch_add(1, Ordering::SeqCst) == 0,
            None => true,
        };
        if first {
            self.notify_panel(true);
        }
    }

    /// 释放面板显示（与 `hold_panel` 配对，操作结束时调用）
    pub fn release_panel(&self) {
        if self.silent || self.app_handle.is_none() {
            return;
        }
        let last = match &self.active_batches {
            Some(c) => c.fetch_sub(1, Ordering::SeqCst) == 1,
            None => true,
        };
        if last {
            self.notify_panel(false);
        }
    }

    /// 获取当前源模式（用于构造 URL）
    pub fn source_mode(&self) -> DownloadSourceMode {
        self.source_mode
    }

    /// 获取当前进度
    pub async fn get_progress(&self) -> GlobalProgress {
        self.progress.lock().unwrap().clone()
    }
}

#[cfg(test)]
#[path = "../manager_tests.rs"]
mod tests;
