//! DownloadSession：下载会话编排层
//! 封装"reset_stages + flag 重置 + manager 构造 + callback 工厂"组合，消除 5 处调用点重复代码。
//! `start_grouped` 用于顶层入口（完整初始化），`attach` 用于子流程接入（仅构造 manager）。

use std::sync::Arc;

use crate::state::{AppState, DownloadStage};

use super::manager::DownloadManager;
use super::types::GlobalProgress;

/// 下载会话编排层
///
/// 不持有 stages / flag 的所有权（这些都在 `AppState` 中），
/// 仅持有 `DownloadManager` 实例和 group_name 元数据。
/// 调用方通过 `make_progress_callback` 工厂构造统一回调，
/// 通过 `manager()` 执行 `download_batch`。
pub struct DownloadSession {
    manager: DownloadManager,
    #[allow(dead_code)]
    group_name: String,
}

impl DownloadSession {
    /// 启动一个分组下载会话（顶层入口）
    ///
    /// 自动完成：
    /// 1. `reset_stages` 注册 stages（全部归属 `group_name`）
    /// 2. 重置 cancel/pause flag（防止上次任务残留状态影响新任务）
    /// 3. 从 config 构造 `DownloadManager` 并接入 flag
    ///
    /// `silent`：静默下载（不 emit 面板显隐事件），供后台任务使用；
    /// 用户主动下载（MC 本体 / 资源 / 整合包）传 `false`。
    ///
    /// 用于独立入口：`download_resource` / `download_resource_to_path` / `download_file`（外部）。
    pub async fn start_grouped(
        state: &AppState,
        group_name: &str,
        stages: Vec<(&str, f64)>,
        silent: bool,
    ) -> Self {
        let manager = DownloadManager::from_state(state).await;
        Self::start_grouped_with_manager(state, group_name, stages, manager, silent).await
    }

    /// 启动一个分组下载会话（使用外部传入的 `DownloadManager`）
    ///
    /// 与 [`start_grouped`] 相同，但允许调用方传入自定义构造的 manager
    /// （如外部下载工具按任务覆盖 UA / 线程数 / 分片数 / 限速）。
    pub async fn start_grouped_with_manager(
        state: &AppState,
        group_name: &str,
        stages: Vec<(&str, f64)>,
        manager: DownloadManager,
        silent: bool,
    ) -> Self {
        // 1. 注册 stages
        {
            let stages: Vec<DownloadStage> = stages
                .into_iter()
                .map(|(name, weight)| DownloadStage::new_grouped(name, weight, group_name))
                .collect();
            let mut ds = state.download_state.lock().unwrap();
            ds.reset_stages(stages);
        }

        // 2. 重置 flag
        state
            .download_cancel_flag
            .store(false, std::sync::atomic::Ordering::Relaxed);
        state
            .download_pause_flag
            .store(false, std::sync::atomic::Ordering::Relaxed);

        // 3. 接入 flag + 静默 + 共享批次计数
        let manager = manager
            .with_cancel_flag(state.download_cancel_flag.clone())
            .with_pause_flag(state.download_pause_flag.clone())
            .with_silent(silent)
            .with_panel_counter(state.panel_active_count.clone());

        // 会话级持有面板：阶段切换时共享计数不会瞬间归零 emit `visible:false`
        // 导致前端下载面板闪烁/误跳转（由 Drop 统一释放）
        manager.hold_panel();

        Self {
            manager,
            group_name: group_name.to_string(),
        }
    }

    /// 子流程接入（仅构造 manager）
    ///
    /// 用于父函数已 `reset_stages` + flag 重置的场景：
    /// - `download_modpack_archive`（`install_modpack` 已初始化）
    /// - `download_files_concurrent`（`install_modpack` 已初始化）
    ///
    /// **不**重置 stages / flag，避免覆盖父会话状态。
    /// 与顶层会话一致持有面板（父会话已持有时计数叠加，互不干扰）。
    pub async fn attach(state: &AppState, silent: bool) -> Self {
        let manager = DownloadManager::from_state(state)
            .await
            .with_cancel_flag(state.download_cancel_flag.clone())
            .with_pause_flag(state.download_pause_flag.clone())
            .with_silent(silent);
        manager.hold_panel();
        Self {
            manager,
            group_name: String::new(),
        }
    }

    /// 构造进度回调（统一 `sync_stage_from_progress` + `broadcast_current`）
    ///
    /// `stage_index`：回调写入哪个 stage（由调用方根据 stages 注册顺序决定）。
    ///
    /// 消除 5 处 callback 闭包复制：resource×2 / tools×1 / modpack_stages / concurrent。
    pub fn make_progress_callback(
        &self,
        state: &AppState,
        stage_index: usize,
    ) -> Arc<dyn Fn(GlobalProgress) + Send + Sync> {
        let cb_state = state.download_state.clone();
        let state_for_cb = state.clone();
        Arc::new(move |p: GlobalProgress| {
            {
                let mut ds = cb_state.lock().unwrap();
                ds.sync_stage_from_progress(
                    stage_index,
                    p.downloaded_bytes,
                    p.total_bytes,
                    p.completed_files,
                    p.total_files,
                    p.current_speed,
                );
            }
            // 广播进度到 WS（确保所有下载路径都能推送）
            crate::commands::version::download::broadcast_current(&state_for_cb);
        })
    }

    /// 引用内部 manager，调用方用它执行 `download_batch`
    pub fn manager(&self) -> &DownloadManager {
        &self.manager
    }

    /// 标记整体完成（所有 Loading 阶段标记为 Finished）
    pub fn mark_complete(&self, state: &AppState) {
        let mut ds = state.download_state.lock().unwrap();
        ds.mark_complete();
    }

    /// 标记整体失败
    pub fn mark_failed(&self, state: &AppState, error_code: i32) {
        let mut ds = state.download_state.lock().unwrap();
        ds.mark_failed(error_code);
    }
}

impl Drop for DownloadSession {
    fn drop(&mut self) {
        // 释放会话持有的面板显示（与构造时的 hold_panel 配对）
        self.manager.release_panel();
    }
}
