//! 社区资源下载安装 - Modrinth 整合包处理
//!
//! 包含 MR modrinth.index.json 数据结构与 install_mr_files 安装流程。
//! install_mr_files 流程：遍历 files[] 直接下载（path 相对于 instance 目录）→
//! mods/ 目录下的 jar 文件按 community_filename_format 重命名
//! （从 downloads URL 提取 project_id → 批量查询拿 slug → 查 mcmod 译名 → 应用格式）。
//! 非 mods/ 文件（resourcepacks/shaderpacks 等）保留原名（mcmod 数据库只覆盖 mod）。

use crate::log_info;
use crate::state::AppState;
use serde::Deserialize;

/// MR modrinth.index.json 结构
#[derive(Debug, Deserialize)]
pub(super) struct MrIndex {
    #[serde(default)]
    pub(super) dependencies: std::collections::HashMap<String, String>,
    #[serde(default)]
    pub(super) files: Vec<MrFile>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct MrFile {
    pub(super) path: String,
    #[serde(default)]
    pub(super) downloads: Vec<String>,
    #[serde(default)]
    pub(super) file_size: u64,
}

/// 安装 MR 整合包依赖文件
///
/// 遍历 files[] 直接下载（path 相对于 instance 目录，如 mods/xxx.jar）。
/// mods/ 目录下的 jar 文件按用户设置的 `community_filename_format` 重命名
/// （从 downloads URL 提取 project_id → 批量查询拿 slug → 查 mcmod 译名 → 应用格式）。
/// 非 mods/ 文件（resourcepacks/shaderpacks 等）保留原名（mcmod 数据库只覆盖 mod）。
pub(super) async fn install_mr_files(
    state: &AppState,
    mr_files: &[MrFile],
    instance_dir: &std::path::Path,
    max_threads: usize,
    stage_index: usize,
) -> Result<(), String> {
    if mr_files.is_empty() {
        log_info!("[Community] MR index 无依赖文件");
        return Ok(());
    }

    // 1. 从所有文件的 downloads URL 提取 project_id（仅对 mods/ 路径下的文件）
    let mut file_project_ids: std::collections::HashMap<usize, String> =
        std::collections::HashMap::new();
    let mut all_project_ids: Vec<String> = Vec::new();
    for (i, f) in mr_files.iter().enumerate() {
        // 仅对 mods/ 目录下的文件应用命名规范（resourcepacks/shaders 等保留原名）
        if !f.path.starts_with("mods/") {
            continue;
        }
        if let Some(first_url) = f.downloads.first() {
            if let Some(pid) = super::helpers::extract_mr_project_id(first_url) {
                file_project_ids.insert(i, pid.clone());
                all_project_ids.push(pid);
            }
        }
    }

    // 2. 批量查询 project info 拿 slug，再查 mcmod 译名
    let slug_map =
        crate::minecraft::community::modrinth::batch_get_project_slugs(&all_project_ids).await;
    let mut file_translated: std::collections::HashMap<usize, Option<String>> =
        std::collections::HashMap::new();
    for (i, pid) in &file_project_ids {
        let slug = slug_map.get(pid);
        let translated = slug
            .and_then(|s| crate::minecraft::community::mcmod::lookup_mr(s).map(|n| n.to_string()));
        file_translated.insert(*i, translated);
    }

    // 3. 读取用户设置的文件名格式
    let filename_format = state.config.lock().await.community.filename_format;

    // 4. 构造下载列表
    let mut download_list: Vec<(Vec<String>, String, u64)> = Vec::with_capacity(mr_files.len());
    let mut total_bytes: u64 = 0;
    for (i, f) in mr_files.iter().enumerate() {
        if f.downloads.is_empty() {
            log_info!("[Community] MR 文件无下载 URL，跳过: {}", f.path);
            continue;
        }
        // Modrinth 的 downloads 是数组，对每个 URL 根据 source 策略扩展镜像 fallback
        let mut urls: Vec<String> = Vec::new();
        for u in &f.downloads {
            if u.is_empty() {
                continue;
            }
            for candidate in crate::minecraft::sources::cdn_urls(u) {
                if !urls.contains(&candidate) {
                    urls.push(candidate);
                }
            }
        }
        if urls.is_empty() {
            log_info!("[Community] MR 文件所有 URL 为空，跳过: {}", f.path);
            continue;
        }

        // 对 mods/ 路径下的文件应用 community_filename_format
        let target_path_str = if f.path.starts_with("mods/") {
            if let Some(parent) = std::path::Path::new(&f.path).parent() {
                let orig_name = std::path::Path::new(&f.path)
                    .file_name()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_else(|| f.path.clone());
                let translated = file_translated.get(&i).cloned().flatten();
                let final_name = super::helpers::apply_filename_format(
                    &orig_name,
                    translated.as_deref(),
                    filename_format,
                );
                instance_dir
                    .join(parent)
                    .join(&final_name)
                    .to_string_lossy()
                    .to_string()
            } else {
                instance_dir.join(&f.path).to_string_lossy().to_string()
            }
        } else {
            instance_dir.join(&f.path).to_string_lossy().to_string()
        };

        download_list.push((urls, target_path_str, f.file_size));
        total_bytes += f.file_size;
    }

    log_info!(
        "[Community] MR 下载 {} 个文件，总大小 {}",
        download_list.len(),
        crate::utils::format::bytes(total_bytes)
    );

    super::concurrent::download_files_concurrent(
        state,
        stage_index,
        &download_list,
        max_threads,
        total_bytes,
    )
    .await?;

    log_info!("[Community] MR 文件下载完成");
    Ok(())
}
