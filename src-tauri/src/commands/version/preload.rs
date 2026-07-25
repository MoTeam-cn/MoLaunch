//! Mod 详情预加载命令
//!
//! 对接 `minecraft::community::preload` 模块，前端在 `list_mods` 完成后调用本命令，
//! 后台异步并发从 CF/MR 批量查询每个 mod 的 ResourceProject，通过 Tauri event 推送。
//!
//! 列表加载完成后立即启动后台联网补全。
//!
//! 注：已聚合为 `version_install_manager` IPC 入口，本函数由
//! `utils::version_install_manager::dispatch` 反序列化参数后调用。

use tauri::AppHandle;

use crate::error_util::log_err;
use crate::minecraft::community::preload::{preload_mods_detail, PreloadModInput};
use crate::state::AppState;

use super::mods::get_mods_dir;
use super::sanitize_version_id;

/// 触发 mod 详情预加载
///
/// 前端调用后立即返回（不阻塞），后台异步：
/// 1. 读取持久化缓存（6h TTL）
/// 2. 未命中的 mod 计算 MurmurHash2 + SHA1
/// 3. 并发批量查询 CF + MR
/// 4. 每查到一个 emit `mods-preload-update` 事件（payload: `{ file_name, project }`）
///
/// 前端监听该事件，按 `file_name` 匹配更新对应 mod 的 `project` 字段。
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

    // 后台异步执行，不阻塞命令返回
    let app_clone = app.clone();
    tokio::spawn(async move {
        preload_mods_detail(app_clone, version_id, inputs).await;
    });

    Ok(())
}
