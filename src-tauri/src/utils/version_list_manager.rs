//! 版本列表/文件夹/管理/个性化命令的统一分发逻辑（version_list_manager 的工具实现）
//!
//! 使用 `utils::dispatcher::Dispatcher` 注册式分发。
//! 19 个 action 在 `once_cell::sync::Lazy` 初始化时注册到 DISPATCHER，
//! 聚合原 `version::list`（8 个）+ `version::folder`（5 个）+ `version::manage`（4 个）
//! + `version::personalization`（2 个）共 19 个独立 IPC 命令。
//!
//! 命令清单（19 个，按子模块分组）：
//! - list.rs（8 个）：
//!   - `list_versions`：获取版本列表
//!   - `list_installed_versions`：获取已安装版本 ID 列表
//!   - `list_installed_versions_with_type`：获取已安装版本列表（含类型/图标）
//!   - `uninstall_version`：卸载版本
//!   - `get_version_effective_dir`：获取版本有效游戏目录（考虑隔离）
//!   - `get_version_game_version`：获取版本对应的 MC 游戏版本号
//!   - `get_version_loader_info`：获取版本加载器信息（类型 + 版本号，联机大厅阶段 1 新增）
//!   - `read_local_modpack_meta`：读取本地整合包元数据（联机大厅阶段 3 新增）
//!   - `check_local_modpack`：校验本地是否已安装指定整合包（联机大厅阶段 4 新增）
//! - folder.rs（5 个）：
//!   - `list_mc_folders`：列出所有 MC 文件夹
//!   - `add_mc_folder`：添加 MC 文件夹（自动去重）
//!   - `remove_mc_folder`：移除 MC 文件夹
//!   - `switch_mc_folder`：切换当前 MC 文件夹
//!   - `rename_mc_folder`：重命名 MC 文件夹
//! - manage.rs（4 个）：
//!   - `fix_version_files`：补全版本文件（需要 AppHandle emit `version-fix-progress` 事件）
//!   - `rename_version`：重命名版本
//!   - `get_selected_version`：获取上次选中的版本
//!   - `set_selected_version`：保存当前选中的版本
//! - personalization.rs（2 个）：
//!   - `get_version_personalization`：获取版本个性化设置
//!   - `update_version_personalization`：更新版本个性化字段
//!
//! 注意：所有子模块函数已去掉 `#[tauri::command]` 标注，参数签名改为 `&AppState` / `&AppHandle`，
//! 由本 dispatcher 反序列化 params 后调用。`fix_version_files` 需要 `AppHandle`（用于 emit 事件）。

use once_cell::sync::Lazy;
use serde::Deserialize;
use tauri::AppHandle;

use crate::commands::version::{folder, list, manage, personalization};
use crate::handler;
use crate::minecraft::version::setup::PersonalizationUpdate;
use crate::state::AppState;
use crate::utils::dispatcher::{ActionRequest, Dispatcher};

// ============================================================
// 各 action 的强类型参数
// ============================================================

/// 仅需 versionId 的 action 参数（uninstall_version / get_version_effective_dir
/// / get_version_game_version / get_version_personalization 共 4 个）
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VersionIdParams {
    version_id: String,
}

/// check_local_modpack 参数（联机大厅阶段 4 新增）
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CheckLocalModpackParams {
    manifest_hash: Option<String>,
    source: String,
    project_id: String,
    file_id: String,
}

/// add_mc_folder 参数
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AddMcFolderParams {
    name: String,
    path: String,
}

/// remove_mc_folder / switch_mc_folder 共用参数（仅需 path）
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct McFolderPathParams {
    path: String,
}

/// rename_mc_folder 参数
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RenameMcFolderParams {
    path: String,
    new_name: String,
}

/// rename_version 参数
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RenameVersionParams {
    version_id: String,
    new_name: String,
}

/// set_selected_version 参数（version_id 可为 null）
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SetSelectedVersionParams {
    version_id: Option<String>,
}

/// update_version_personalization 参数
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdatePersonalizationParams {
    version_id: String,
    update: PersonalizationUpdate,
}

// ============================================================
// Dispatcher 注册
// ============================================================

static DISPATCHER: Lazy<Dispatcher> = Lazy::new(|| {
    let mut d = Dispatcher::new();

    // === list.rs（6 个） ===
    d.register("list_versions", handler!(state, _app, _params, {
        let r = list::list_versions(&state).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    d.register("list_installed_versions", handler!(state, _app, _params, {
        let r = list::list_installed_versions(&state).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    d.register("list_installed_versions_with_type", handler!(state, _app, _params, {
        let r = list::list_installed_versions_with_type(&state).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    d.register("uninstall_version", handler!(state, _app, params, {
        let p: VersionIdParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        list::uninstall_version(&state, p.version_id).await?;
        serde_json::to_value(()).map_err(|e| e.to_string())
    }));

    d.register("get_version_effective_dir", handler!(state, _app, params, {
        let p: VersionIdParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let r = list::get_version_effective_dir(&state, p.version_id).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    d.register("get_version_game_version", handler!(state, _app, params, {
        let p: VersionIdParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let r = list::get_version_game_version(&state, p.version_id).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    d.register("get_version_loader_info", handler!(state, _app, params, {
        let p: VersionIdParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let (loader_type, loader_version) = list::get_version_loader_info(&state, p.version_id).await?;
        serde_json::to_value(serde_json::json!({
            "loaderType": loader_type,
            "loaderVersion": loader_version,
        })).map_err(|e| e.to_string())
    }));

    d.register("read_local_modpack_meta", handler!(state, _app, params, {
        let p: VersionIdParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let meta = list::read_local_modpack_meta(&state, p.version_id).await?;
        serde_json::to_value(meta).map_err(|e| e.to_string())
    }));

    d.register("check_local_modpack", handler!(state, _app, params, {
        let p: CheckLocalModpackParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let result = list::check_local_modpack(&state, p.manifest_hash, p.source, p.project_id, p.file_id).await?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    }));

    // === folder.rs（5 个） ===
    d.register("list_mc_folders", handler!(state, _app, _params, {
        let r = folder::list_mc_folders(&state).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    d.register("add_mc_folder", handler!(state, _app, params, {
        let p: AddMcFolderParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let r = folder::add_mc_folder(&state, p.name, p.path).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    d.register("remove_mc_folder", handler!(state, _app, params, {
        let p: McFolderPathParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let r = folder::remove_mc_folder(&state, p.path).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    d.register("switch_mc_folder", handler!(state, _app, params, {
        let p: McFolderPathParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let r = folder::switch_mc_folder(&state, p.path).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    d.register("rename_mc_folder", handler!(state, _app, params, {
        let p: RenameMcFolderParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let r = folder::rename_mc_folder(&state, p.path, p.new_name).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    // === manage.rs（4 个） ===
    d.register("fix_version_files", handler!(state, app, params, {
        let p: VersionIdParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        manage::fix_version_files(&state, &app, p.version_id).await?;
        serde_json::to_value(()).map_err(|e| e.to_string())
    }));

    d.register("rename_version", handler!(state, _app, params, {
        let p: RenameVersionParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        manage::rename_version(&state, p.version_id, p.new_name).await?;
        serde_json::to_value(()).map_err(|e| e.to_string())
    }));

    d.register("get_selected_version", handler!(state, _app, _params, {
        let r = manage::get_selected_version(&state).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    d.register("set_selected_version", handler!(state, _app, params, {
        let p: SetSelectedVersionParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        manage::set_selected_version(&state, p.version_id).await?;
        serde_json::to_value(()).map_err(|e| e.to_string())
    }));

    // === personalization.rs（2 个） ===
    d.register("get_version_personalization", handler!(state, _app, params, {
        let p: VersionIdParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let r = personalization::get_version_personalization(&state, p.version_id).await?;
        serde_json::to_value(r).map_err(|e| e.to_string())
    }));

    d.register("update_version_personalization", handler!(state, _app, params, {
        let p: UpdatePersonalizationParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        personalization::update_version_personalization(&state, p.version_id, p.update).await?;
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
