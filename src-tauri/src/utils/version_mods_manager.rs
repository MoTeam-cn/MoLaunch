//! 版本 Mod 管理统一分发逻辑（version_mods_manager 的工具实现）
//!
//! 使用 `utils::dispatcher::Dispatcher` 注册式分发。
//! 11 个 version::mods action 在 `once_cell::sync::Lazy` 初始化时注册到 DISPATCHER。
//!
//! 命令清单（11 个，按子模块分组）：
//! - list.rs（2 个）：
//!   - `is_version_modable`：判断版本是否可安装 Mod
//!   - `list_mods`：列出版本 mods 目录中的 mod（同步阶段，只枚举文件）
//! - manage.rs（2 个）：
//!   - `toggle_mod`：启用/禁用 mod（重命名扩展名）
//!   - `delete_mod`：删除 mod 文件
//! - install.rs（4 个）：
//!   - `install_mod`：从外部文件安装 mod（复制到 mods 目录）
//!   - `open_mods_dir`：打开 mods 目录
//!   - `reveal_mod_file`：在资源管理器中选中 mod 文件
//!   - `get_version_mods_dir`：获取 mods 目录路径（不打开）
//! - update.rs（1 个，阶段 4 新增）：
//!   - `update_mod`：原子化更新 mod（下载新版本 + 删旧版本）
//! - watcher.rs（2 个）：
//!   - `watch_mods_dir`：监听 mods 目录变化（需要 AppHandle emit 事件）
//!   - `unwatch_mods_dir`：停止监听（无参数无 state）
//!
//! 注意：
//! - 大部分 action 需要 `AppState`，handler 内用 `&state` 调用子模块函数
//! - `watch_mods_dir` 额外需要 `AppHandle`（用于 emit `mods-dir-changed` 事件）
//! - `unwatch_mods_dir` 无参数无 state，handler 内用 `_state, _app, _params` 全忽略

use once_cell::sync::Lazy;
use serde::Deserialize;
use tauri::AppHandle;

use crate::commands::version::mods::{install, list, manage, update, watcher};
use crate::handler;
use crate::state::AppState;
use crate::utils::dispatcher::{ActionRequest, Dispatcher};

// ============================================================
// 各 action 的强类型参数
// ============================================================

/// 仅需 versionId 的 action 参数（is_version_modable / list_mods / open_mods_dir
/// / get_version_mods_dir / watch_mods_dir 共 5 个）
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VersionIdParams {
    version_id: String,
}

/// toggle_mod 参数
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ToggleModParams {
    version_id: String,
    file_name: String,
    enable: bool,
}

/// delete_mod 参数
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DeleteModParams {
    version_id: String,
    file_name: String,
}

/// install_mod 参数
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct InstallModParams {
    version_id: String,
    source_path: String,
}

/// reveal_mod_file 参数
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RevealModFileParams {
    version_id: String,
    file_name: String,
}

/// update_mod 参数（阶段 4 新增）
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateModParams {
    version_id: String,
    old_file_name: String,
    download_url: String,
    new_file_name: String,
    expected_size: i64,
}

// ============================================================
// Dispatcher 注册
// ============================================================

static DISPATCHER: Lazy<Dispatcher> = Lazy::new(|| {
    let mut d = Dispatcher::new();

    // === list.rs（2 个） ===
    d.register("is_version_modable", handler!(state, _app, params, {
        let p: VersionIdParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let r = list::is_version_modable(&state, p.version_id).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    d.register("list_mods", handler!(state, _app, params, {
        let p: VersionIdParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let r = list::list_mods(&state, p.version_id).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    // === manage.rs（2 个） ===
    d.register("toggle_mod", handler!(state, _app, params, {
        let p: ToggleModParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let r = manage::toggle_mod(&state, p.version_id, p.file_name, p.enable).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    d.register("delete_mod", handler!(state, _app, params, {
        let p: DeleteModParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        manage::delete_mod(&state, p.version_id, p.file_name).await?;
        serde_json::to_value(()).map_err(|e| e.to_string())
    }));

    // === install.rs（4 个） ===
    d.register("install_mod", handler!(state, _app, params, {
        let p: InstallModParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        install::install_mod(&state, p.version_id, p.source_path).await?;
        serde_json::to_value(()).map_err(|e| e.to_string())
    }));

    d.register("open_mods_dir", handler!(state, _app, params, {
        let p: VersionIdParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        install::open_mods_dir(&state, p.version_id).await?;
        serde_json::to_value(()).map_err(|e| e.to_string())
    }));

    d.register("reveal_mod_file", handler!(state, _app, params, {
        let p: RevealModFileParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        install::reveal_mod_file(&state, p.version_id, p.file_name).await?;
        serde_json::to_value(()).map_err(|e| e.to_string())
    }));

    d.register("get_version_mods_dir", handler!(state, _app, params, {
        let p: VersionIdParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let r = install::get_version_mods_dir(&state, p.version_id).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    // === update.rs（1 个，阶段 4 新增） ===
    d.register("update_mod", handler!(state, _app, params, {
        let p: UpdateModParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        update::update_mod(
            &state,
            p.version_id,
            p.old_file_name,
            p.download_url,
            p.new_file_name,
            p.expected_size,
        )
        .await?;
        serde_json::to_value(()).map_err(|e| e.to_string())
    }));

    // === watcher.rs（2 个） ===
    d.register("watch_mods_dir", handler!(state, app, params, {
        let p: VersionIdParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        watcher::watch_mods_dir(&state, &app, p.version_id).await?;
        serde_json::to_value(()).map_err(|e| e.to_string())
    }));

    d.register("unwatch_mods_dir", handler!(_state, _app, _params, {
        watcher::unwatch_mods_dir().await?;
        serde_json::to_value(()).map_err(|e| e.to_string())
    }));

    d
});

/// 分发入口
pub async fn dispatch(
    state: AppState,
    app: AppHandle,
    req: ActionRequest,
) -> Result<serde_json::Value, String> {
    DISPATCHER.dispatch(state, app, req).await
}
