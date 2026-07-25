//! Mod 详情预加载命令
//!
//! 对接 `minecraft::community::preload` 模块，前端在 `list_mods` 完成后调用本命令，
//! 后台异步并发从 CF/MR 批量查询每个 mod 的 ResourceProject，通过 Tauri event 推送。
//!
//! 列表加载完成后立即启动后台联网补全。
//!
//! ## 取消机制
//!
//! `preload_mods_detail_cmd` 内部 `tokio::spawn` 后台 task 持续 emit 事件，
//! 如果前端 ModTab 卸载时 task 仍在跑，emit 会打到已注销的 listener callback
//! 触发 `Couldn't find callback id xxx` 警告。这里用 `AbortHandle` 全局保存当前 task：
//! - 新任务启动前 abort 旧任务（避免多个 task 并发 emit）
//! - 前端 ModTab 卸载时调用 `cancel_preload_mods_detail_cmd` abort 当前 task
//!
//! 注：已聚合为 `version_install_manager` IPC 入口，本函数由
//! `utils::version_install_manager::dispatch` 反序列化参数后调用。

use std::sync::{Mutex, OnceLock};

use tauri::AppHandle;
use tokio::task::AbortHandle;

use crate::error_util::log_err;
use crate::minecraft::community::preload::{preload_mods_detail, PreloadModInput};
use crate::state::AppState;

use super::mods::get_mods_dir;
use super::sanitize_version_id;

/// 全局当前预加载 task 的 AbortHandle（同一时间只有一个预加载 task）
///
/// 用 `OnceLock<Mutex<Option<AbortHandle>>>` 而非 `static` 直接初始化，
/// 因为 `AbortHandle` 不是 `const` 构造的。`OnceLock` 提供线程安全的延迟初始化。
static CURRENT_PRELOAD: OnceLock<Mutex<Option<AbortHandle>>> = OnceLock::new();

fn current_preload() -> &'static Mutex<Option<AbortHandle>> {
    CURRENT_PRELOAD.get_or_init(|| Mutex::new(None))
}

/// 取消当前正在运行的预加载 task（如果有）
///
/// 前端 ModTab 组件卸载时调用，避免后台 task 继续 emit 给已注销的 listener。
/// abort 是异步的：task 会在下一个 await 点终止，不会立即停止当前同步 emit。
fn abort_current_preload() {
    let mut guard = current_preload()
        .lock()
        .expect("[Preload] abort handle mutex poisoned");
    if let Some(handle) = guard.take() {
        handle.abort();
        crate::log_debug!("[Preload] 已取消上一个预加载 task");
    }
}

/// 触发 mod 详情预加载
///
/// 前端调用后立即返回（不阻塞），后台异步：
/// 1. 读取持久化缓存（6h TTL）
/// 2. 未命中的 mod 计算 MurmurHash2 + SHA1
/// 3. 并发批量查询 CF + MR
/// 4. 每查到一个 emit `mods-preload-update` 事件（payload: `{ file_name, project }`）
///
/// 前端监听该事件，按 `file_name` 匹配更新对应 mod 的 `project` 字段。
///
/// 如果调用时已有预加载 task 在跑，会先 abort 旧 task 再启动新的。
pub async fn preload_mods_detail_cmd(
    app: &AppHandle,
    state: &AppState,
    version_id: String,
) -> Result<(), String> {
    sanitize_version_id(&version_id)?;

    // 获取 mods 目录
    let mods_dir: std::path::PathBuf = get_mods_dir(&state, &version_id).await?;
    if !mods_dir.exists() {
        return Ok(()); // 没 mods 目录，无需预加载
    }

    // 扫描 mods 目录，构建预加载输入
    let mut inputs: Vec<PreloadModInput> = Vec::new();
    let entries = std::fs::read_dir(&mods_dir).map_err(log_err("Failed to read mods directory"))?;
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let file_name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };
        let lower = file_name.to_lowercase();
        // 只处理 jar/litemod 及其禁用变体（与 list_mods 保持一致）
        let is_mod = lower.ends_with(".jar")
            || lower.ends_with(".litemod")
            || lower.ends_with(".jar.disabled")
            || lower.ends_with(".jar.old")
            || lower.ends_with(".litemod.disabled")
            || lower.ends_with(".litemod.old");
        if !is_mod {
            continue;
        }
        inputs.push(PreloadModInput {
            file_name,
            path: path.to_string_lossy().to_string(),
        });
    }

    if inputs.is_empty() {
        return Ok(());
    }

    crate::log_info!(
        "[Preload Cmd] 启动预加载：{} 个 mod（version={}）",
        inputs.len(),
        version_id
    );

    // 取消上一个预加载 task（避免多个 task 并发 emit 同一事件）
    abort_current_preload();

    // 后台异步执行，不阻塞命令返回
    let app_clone = app.clone();
    let join_handle = tokio::spawn(async move {
        preload_mods_detail(app_clone, version_id, inputs).await;
    });

    // 保存 AbortHandle 供后续取消
    {
        let mut guard = current_preload()
            .lock()
            .expect("[Preload] abort handle mutex poisoned");
        *guard = Some(join_handle.abort_handle());
    }

    Ok(())
}

/// 取消当前预加载 task（ModTab 组件卸载时调用）
///
/// 前端在 ModTab onUnmounted 中调用本命令，abort 后台 spawn 的预加载 task，
/// 避免 task 继续 emit `mods-preload-update` 给已注销的前端 listener。
pub async fn cancel_preload_mods_detail_cmd() -> Result<(), String> {
    abort_current_preload();
    Ok(())
}
