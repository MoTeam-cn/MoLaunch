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

/// MR 文件 env 字段：控制客户端/服务端是否下载
///
/// `client` 取值：
/// - `"required"`（默认）：必下载
/// - `"optional"`：可选，前端弹窗询问
/// - `"unsupported"`：跳过不下载
#[derive(Debug, Default, Deserialize)]
pub(super) struct MrFileEnv {
    #[serde(default)]
    pub(super) client: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct MrFile {
    pub(super) path: String,
    #[serde(default)]
    pub(super) downloads: Vec<String>,
    #[serde(default)]
    pub(super) file_size: u64,
    #[serde(default)]
    pub(super) env: MrFileEnv,
}

/// 校验 Modrinth 文件路径安全性
///
/// 防止路径穿越攻击：path 不能包含 `..`，不能是绝对路径，
/// 且最终完整路径必须在 instance_dir 下。
///
/// 注意：不 canonicalize 目标文件的父目录，因为下载前目标子目录
/// （如 resourcepacks/）可能尚未创建，canonicalize 会返回 Err 导致误判。
/// 改为 canonicalize instance_dir 后用 starts_with 做组件级校验，
/// 安全性由前面的 `..` / 绝对路径拦截保证。
fn validate_mr_path(path: &str, instance_dir: &std::path::Path) -> Result<std::path::PathBuf, String> {
    // 拒绝空路径
    if path.is_empty() {
        return Err("文件路径为空".to_string());
    }
    // 拒绝绝对路径（Windows 和 Unix）
    if path.starts_with('/') || path.starts_with('\\')
        || (path.len() >= 2 && path.as_bytes()[1] == b':')
    {
        return Err(format!("文件路径不能为绝对路径: {}", path));
    }
    // 拒绝包含 .. 的路径（防止穿越上级目录）
    let normalized = path.replace('\\', "/");
    for seg in normalized.split('/') {
        if seg == ".." {
            return Err(format!("文件路径不能包含 '..': {}", path));
        }
    }
    let full = instance_dir.join(path);
    // 最终校验：canonicalize instance_dir（已存在），join 后做组件级 starts_with 校验
    let instance_canonical = instance_dir.canonicalize().unwrap_or_default();
    let full_canonical = instance_canonical.join(path);
    if !full_canonical.starts_with(&instance_canonical) {
        return Err(format!("文件路径校验失败，越出实例目录: {}", path));
    }
    Ok(full)
}

/// 安装 MR 整合包依赖文件
///
/// 遍历 files[] 直接下载（path 相对于 instance 目录，如 mods/xxx.jar）。
/// - `env.client = "unsupported"` 的文件跳过
/// - `env.client = "optional"` 的文件由调用方决定是否下载（通过 `include_optional` 参数）
/// - 其他（含 None / "required"）正常下载
///
/// mods/ 目录下的 jar 文件按用户设置的 `community_filename_format` 重命名
/// （从 downloads URL 提取 project_id → 批量查询拿 slug → 查 mcmod 译名 → 应用格式）。
/// 非 mods/ 文件（resourcepacks/shaderpacks 等）保留原名（mcmod 数据库只覆盖 mod）。
pub(super) async fn install_mr_files(
    state: &AppState,
    mr_files: &[MrFile],
    instance_dir: &std::path::Path,
    stage_index: usize,
    include_optional: bool,
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

    // 4. 构造下载列表（跳过 unsupported，按 include_optional 决定 optional 文件）
    let mut download_list: Vec<(Vec<String>, String, u64)> = Vec::with_capacity(mr_files.len());
    let mut total_bytes: u64 = 0;
    let mut skipped_unsupported: usize = 0;
    let mut skipped_optional: usize = 0;
    for (i, f) in mr_files.iter().enumerate() {
        // env.client 处理
        let client_env = f.env.client.as_deref().unwrap_or("required");
        match client_env {
            "unsupported" => {
                skipped_unsupported += 1;
                continue;
            }
            "optional" if !include_optional => {
                skipped_optional += 1;
                continue;
            }
            _ => {}
        }

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

        // 路径穿越校验：防止恶意整合包写出 instance 目录
        let validated = validate_mr_path(&f.path, instance_dir)?;

        // 对 mods/ 路径下的文件应用 community_filename_format
        let target_path_str = if f.path.starts_with("mods/") {
            if let Some(parent) = validated.parent() {
                let orig_name = validated
                    .file_name()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_else(|| f.path.clone());
                let translated = file_translated.get(&i).cloned().flatten();
                let final_name = super::helpers::apply_filename_format(
                    &orig_name,
                    translated.as_deref(),
                    filename_format,
                );
                parent.join(&final_name).to_string_lossy().to_string()
            } else {
                validated.to_string_lossy().to_string()
            }
        } else {
            validated.to_string_lossy().to_string()
        };

        download_list.push((urls, target_path_str, f.file_size));
        total_bytes += f.file_size;
    }

    if skipped_unsupported > 0 {
        log_info!(
            "[Community] MR 跳过 {} 个 unsupported 文件",
            skipped_unsupported
        );
    }
    if skipped_optional > 0 {
        log_info!(
            "[Community] MR 跳过 {} 个 optional 文件（用户未选择下载）",
            skipped_optional
        );
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
        total_bytes,
    )
    .await?;

    log_info!("[Community] MR 文件下载完成");
    Ok(())
}
