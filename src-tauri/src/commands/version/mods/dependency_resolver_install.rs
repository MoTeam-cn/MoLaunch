//! Mod + 前置依赖批量安装
//!
//! 由 `dependency_resolver` 拆分而来，承接 `install_mod_with_dependencies`
//! 与安装结果类型 `InstallResult`。复用父模块的类型与下载能力。

use std::path::PathBuf;

use serde::Serialize;

use super::ResolvedDependency;
use crate::commands::version::mods::helpers::get_mods_dir;
use crate::minecraft::community::types::ResourceVersion;
use crate::minecraft::download::types::{DownloadStatus, DownloadTask};
use crate::minecraft::download::DownloadSession;
use crate::state::AppState;

/// 安装结果
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallResult {
    /// 成功安装的文件数（含跳过已存在的）
    pub installed_count: u32,
    /// 失败的文件数
    pub failed_count: u32,
    /// 失败详情（文件名：错误原因）
    pub failures: Vec<String>,
}

/// 批量安装主 mod + 用户勾选的前置 mod
///
/// 流程：
/// 1. 解析下载目录（优先 version_id 获取 mods 目录，其次 target_dir 参数）
/// 2. 构造下载任务列表（主 mod + 前置的 suggested_version）
/// 3. 启动 DownloadSession（1 个 stage "下载 Mod 及前置"）并发下载
/// 4. 统计成功/失败，返回 InstallResult
///
/// suggested_version 为 None 或 download_url 为空的版本记为失败。
pub async fn install_mod_with_dependencies(
    state: &AppState,
    version_id: Option<&str>,
    target_dir: Option<&str>,
    main_version: &ResourceVersion,
    deps: &[ResolvedDependency],
) -> Result<InstallResult, String> {
    let mods_dir = match (version_id, target_dir) {
        (Some(vid), _) => get_mods_dir(state, vid).await?,
        (None, Some(dir)) => PathBuf::from(dir),
        (None, None) => {
            return Err("必须提供 version_id 或 target_dir 之一".to_string());
        }
    };
    if !mods_dir.exists() {
        std::fs::create_dir_all(&mods_dir).map_err(|e| format!("创建 mods 目录失败: {}", e))?;
    }

    let mut tasks: Vec<DownloadTask> = Vec::new();
    let mut task_names: Vec<String> = Vec::new();
    let mut failures: Vec<String> = Vec::new();

    // 主 mod
    if main_version.download_url.is_empty() {
        failures.push(format!("{}（主 mod 下载地址为空）", main_version.file_name));
    } else {
        let path = mods_dir.join(&main_version.file_name);
        tasks.push(DownloadTask {
            id: format!("mod_main_{}", main_version.file_name),
            urls: crate::minecraft::sources::cdn_urls(&main_version.download_url),
            local_path: path.to_string_lossy().to_string(),
            expected_size: main_version.size as i64,
            expected_hash: main_version.hash.clone(),
        });
        task_names.push(main_version.file_name.clone());
    }

    // 前置 mod
    for dep in deps {
        let Some(ref sv) = dep.suggested_version else {
            failures.push(format!("{}（未找到兼容版本）", dep.project.raw_name));
            continue;
        };
        if sv.download_url.is_empty() {
            failures.push(format!("{}（下载地址为空）", sv.file_name));
            continue;
        }
        let path = mods_dir.join(&sv.file_name);
        tasks.push(DownloadTask {
            id: format!("mod_dep_{}", sv.file_name),
            urls: crate::minecraft::sources::cdn_urls(&sv.download_url),
            local_path: path.to_string_lossy().to_string(),
            expected_size: sv.size as i64,
            expected_hash: sv.hash.clone(),
        });
        task_names.push(sv.file_name.clone());
    }

    let total = tasks.len();
    if total == 0 {
        return Ok(InstallResult {
            installed_count: 0,
            failed_count: failures.len() as u32,
            failures,
        });
    }

    crate::log_info!(
        "[Mods] 开始安装 Mod + 前置：共 {} 个文件（主 mod + {} 个前置）",
        total,
        total - 1
    );

    // 启动 DownloadSession（1 个 stage）
    let session = DownloadSession::start_grouped(
        state,
        "Mod 及前置",
        vec![("下载 Mod 及前置", 100.0)],
        false,
    )
    .await;
    {
        let mut ds = state.download_state.lock().unwrap();
        ds.version_name = main_version.file_name.clone();
    }

    let progress_callback = session.make_progress_callback(state, 0);
    let results = session
        .manager()
        .download_batch(tasks, Some(progress_callback))
        .await;

    let mut installed_count = 0u32;
    for (name, result) in task_names.iter().zip(results.iter()) {
        if result.status == DownloadStatus::Completed || result.status == DownloadStatus::Skipped {
            installed_count += 1;
        } else {
            let err = result
                .error
                .clone()
                .unwrap_or_else(|| "未知错误".to_string());
            failures.push(format!("{}：{}", name, err));
        }
    }

    let failed_count = total as u32 - installed_count;

    if installed_count > 0 {
        session.mark_complete(state);
    } else {
        session.mark_failed(state, 1);
    }

    crate::log_info!(
        "[Mods] Mod + 前置安装完成：成功 {} / {}，失败 {}",
        installed_count,
        total,
        failed_count
    );

    Ok(InstallResult {
        installed_count,
        failed_count,
        failures,
    })
}
