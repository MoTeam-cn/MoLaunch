//! 资源包/光影详情预加载命令
//! `list_packs` 完成后后台异步从 CF/MR 批量查询每个包的 ResourceProject，通过
//! `packs-preload-update` 事件推送。用 `AbortHandle` 全局保存当前 task，
//! 前端 PackTab 卸载时调 `cancel_preload_packs_detail_cmd` abort 当前 task。

use std::sync::{Mutex, OnceLock};

use tauri::AppHandle;
use tokio::task::AbortHandle;

use crate::error_util::log_err;
use crate::minecraft::community::preload::{preload_packs_detail, PreloadModInput};
use crate::minecraft::community::types::ResourceType;
use crate::state::AppState;

use super::helpers::resolve_packs_dir;
use super::super::sanitize_version_id;
use super::types::PackKind;

/// 全局当前预加载 task 的 AbortHandle（同一时间只有一个 packs 预加载 task）
static CURRENT_PRELOAD: OnceLock<Mutex<Option<AbortHandle>>> = OnceLock::new();

fn current_preload() -> &'static Mutex<Option<AbortHandle>> {
    CURRENT_PRELOAD.get_or_init(|| Mutex::new(None))
}

fn abort_current_preload() {
    let mut guard = current_preload()
        .lock()
        .expect("[PackPreload] abort handle mutex poisoned");
    if let Some(handle) = guard.take() {
        handle.abort();
    }
}

/// 触发资源包/光影详情预加载
pub async fn preload_packs_detail_cmd(
    app: &AppHandle,
    state: &AppState,
    version_id: String,
    kind: PackKind,
) -> Result<(), String> {
    sanitize_version_id(&version_id)?;

    let packs_dir = resolve_packs_dir(state, &version_id, kind).await?;
    if !packs_dir.exists() {
        return Ok(()); // 没有目录，无需预加载
    }

    // 扫描内容目录，构建预加载输入（zip 及其禁用变体，与 list_packs 保持一致）
    let mut inputs: Vec<PreloadModInput> = Vec::new();
    let entries =
        std::fs::read_dir(&packs_dir).map_err(log_err("Failed to read packs directory"))?;
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
        let is_pack = lower.ends_with(".zip")
            || lower.ends_with(".zip.disabled")
            || lower.ends_with(".zip.old");
        if !is_pack {
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
        "[PackPreload] 启动预加载：{} 个包（kind={:?}, version={}）",
        inputs.len(),
        kind,
        version_id
    );

    let resource_type = match kind {
        PackKind::Resourcepack => ResourceType::ResourcePack,
        PackKind::Shader => ResourceType::Shader,
    };

    // 取消上一个预加载 task（避免多个 task 并发 emit 同一事件）
    abort_current_preload();

    let app_clone = app.clone();
    let join_handle = tokio::spawn(async move {
        preload_packs_detail(app_clone, version_id, resource_type, inputs).await;
    });

    {
        let mut guard = current_preload()
            .lock()
            .expect("[PackPreload] abort handle mutex poisoned");
        *guard = Some(join_handle.abort_handle());
    }

    Ok(())
}

/// 取消当前预加载 task（PackTab 组件卸载时调用）
pub async fn cancel_preload_packs_detail_cmd() -> Result<(), String> {
    abort_current_preload();
    Ok(())
}
