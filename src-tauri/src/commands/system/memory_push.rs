//! 内存信息推送：前端订阅/退订控制后端定时 emit
//!
//! 页面挂载时 `memory_subscribe`（计数 +1），卸载时 `memory_unsubscribe`（计数 -1）；
//! 计数 0→1 启动 1s 推送 task，归零后 task 下个 tick 检测到无订阅者即退出，
//! 退订侧再 abort 兜底，保证无页面打开时后端零开销。

use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

use tauri::{AppHandle, Emitter};

use crate::minecraft::system::get_system_memory;
use crate::state::AppState;

/// 内存推送事件名（前端 useMemoryVisualizer 监听同一事件）
pub const MEMORY_PUSH_EVENT: &str = "memory-info";
/// 推送间隔（与原有前端 1s 轮询节奏一致）
const PUSH_INTERVAL: Duration = Duration::from_secs(1);

/// 内存推送订阅状态
#[derive(Default)]
pub struct MemoryPushState {
    /// 订阅计数（每个打开的内存页面 +1，与 unsubscribe 严格配对）
    pub subscribers: AtomicU32,
}

/// 订阅内存推送（页面挂载时调用）：计数 0→1 时启动 1s 推送 task
pub async fn memory_subscribe(state: &AppState, app: &AppHandle) -> Result<(), String> {
    let push = state.memory_push.clone();
    let prev = push.subscribers.fetch_add(1, Ordering::SeqCst);
    if prev == 0 {
        // 归零后立即重新订阅的竞态窗口内 task 可能尚未退出，复用已有句柄
        let mut guard = state.memory_push_task.lock().await;
        if guard.is_none() {
            let app = app.clone();
            let push = push.clone();
            *guard = Some(
                tokio::spawn(async move {
                    let mut interval = tokio::time::interval(PUSH_INTERVAL);
                    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
                    loop {
                        interval.tick().await;
                        // 无订阅者则停止推送
                        if push.subscribers.load(Ordering::SeqCst) == 0 {
                            break;
                        }
                        let mem = get_system_memory();
                        let _ = app.emit(MEMORY_PUSH_EVENT, &mem);
                    }
                })
                .abort_handle(),
            );
        }
    }
    Ok(())
}

/// 退订内存推送（页面卸载时调用）：计数归零时 abort 推送 task
pub async fn memory_unsubscribe(state: &AppState) -> Result<(), String> {
    let push = state.memory_push.clone();
    // 前端严格配对调用（subscribe/unsubscribe 成对），此处直接 fetch_sub
    let prev = push.subscribers.fetch_sub(1, Ordering::SeqCst);
    if prev == 1 {
        let mut guard = state.memory_push_task.lock().await;
        if let Some(h) = guard.as_ref() {
            h.abort();
        }
        *guard = None;
    }
    Ok(())
}
