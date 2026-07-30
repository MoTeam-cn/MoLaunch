//! 版本 Mod 管理统一分发逻辑（version_mods_manager 的工具实现）
//!
//! 使用 `utils::dispatcher::Dispatcher` 注册式分发，11 个 version::mods action 在
//! `once_cell::sync::Lazy` 初始化时注册到 DISPATCHER。
//! `watch_mods_dir` 额外需要 `AppHandle`（emit `mods-dir-changed` 事件）。

use once_cell::sync::Lazy;
use serde::Deserialize;
use tauri::AppHandle;

use crate::commands::version::mods::dependency_resolver::{
    check_mod_dependencies, install_mod_with_dependencies, DependencyCheckResult, InstallResult,
    ResolvedDependency,
};
use crate::commands::version::mods::{install, list, manage, update, watcher};
use crate::handler;
use crate::minecraft::community::types::{Platform, ResourceVersion};
use crate::state::AppState;
use crate::utils::dispatcher::{ActionRequest, Dispatcher};

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

/// check_mod_dependencies 参数（前置 mod 检查）
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CheckModDependenciesParams {
    /// 版本 ID（版本管理场景必填，Community 场景可空）
    version_id: Option<String>,
    /// 自定义 mods 目录（Community 场景无 version_id 时使用，可选；为空则跳过已安装扫描）
    mods_dir: Option<String>,
    /// 平台名："CurseForge" 或 "Modrinth"
    platform: String,
    /// 用户选中的资源版本（含 dependencies 字段）
    mod_version: ResourceVersion,
    /// 目标游戏版本（如 "1.20.1"）
    game_version: String,
    /// 目标加载器 flags（1=Forge, 4=Fabric, 16=NeoForge, 8=Quilt）
    mod_loader: u32,
}

/// install_mod_with_dependencies 参数（主 mod + 前置批量安装）
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct InstallModWithDepsParams {
    /// 版本 ID（版本管理场景必填，Community 场景可空）
    version_id: Option<String>,
    /// 自定义下载目录（Community 场景无 version_id 时必填）
    target_dir: Option<String>,
    /// 主 mod 版本（用户在详情页选中的版本）
    main_version: ResourceVersion,
    /// 用户勾选要安装的前置（含 suggested_version）
    deps: Vec<ResolvedDependency>,
}

/// 解析平台名字符串到 Platform 枚举
fn parse_platform(s: &str) -> Result<Platform, String> {
    match s.to_lowercase().as_str() {
        "curseforge" | "cf" => Ok(Platform::CurseForge),
        "modrinth" | "mr" => Ok(Platform::Modrinth),
        _ => Err(format!("不支持的平台: {}", s)),
    }
}

static DISPATCHER: Lazy<Dispatcher> = Lazy::new(|| {
    let mut d = Dispatcher::new();

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

    d.register("check_mod_dependencies", handler!(state, _app, params, {
        let p: CheckModDependenciesParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let platform = parse_platform(&p.platform)?;
        let result: DependencyCheckResult = check_mod_dependencies(
            &state,
            p.version_id.as_deref(),
            p.mods_dir.as_deref(),
            platform,
            &p.mod_version,
            &p.game_version,
            p.mod_loader,
        )
        .await?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    }));

    d.register("install_mod_with_dependencies", handler!(state, _app, params, {
        let p: InstallModWithDepsParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let result: InstallResult = install_mod_with_dependencies(
            &state,
            p.version_id.as_deref(),
            p.target_dir.as_deref(),
            &p.main_version,
            &p.deps,
        )
        .await?;
        serde_json::to_value(result).map_err(|e| e.to_string())
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
