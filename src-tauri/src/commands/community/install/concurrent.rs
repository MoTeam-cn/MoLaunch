//! 社区资源下载安装 - 并发下载与 zip 操作
//!
//! 包含：
//! - `download_files_concurrent`：多文件并发下载，进度汇总到 download_state 的指定 stage
//! - `extract_overrides`：从整合包 zip 解压 overrides / client-overrides 到 instance 目录
//! - `detect_modpack_format`：检测整合包格式（CF manifest.json / MR modrinth.index.json）

use crate::log_info;
use crate::state::AppState;
use std::sync::Arc;

/// 并发下载多个文件，进度汇总到 download_state 的指定 stage
///
/// 统一走 DownloadManager：自动按文件大小走分片下载（>1MB/chunk 走 chunk::download_chunked）
/// 或普通下载（小文件直连），与 MC 本体/库/assets 走同一套下载基础设施。
/// 进度通过 `sync_stage_from_progress` 统一同步到 download_state（速度/字节累加由统一方法处理）。
pub(super) async fn download_files_concurrent(
    state: &AppState,
    stage_index: usize,
    files: &[(Vec<String>, String, u64)], // (urls, target_path, file_size)
    max_threads: usize,
    _precomputed_total: u64,
) -> Result<(), String> {
    use crate::minecraft::download::manager::DownloadManager;
    use crate::minecraft::download::types::DownloadTask;
    use crate::minecraft::sources::DownloadSourceMode;

    if files.is_empty() {
        let mut ds = state.download_state.lock().unwrap();
        ds.set_stage_bytes(stage_index, 1, 1);
        return Ok(());
    }

    // 构造 DownloadTask 列表
    let tasks: Vec<DownloadTask> = files
        .iter()
        .enumerate()
        .map(|(i, (urls, path, size))| DownloadTask {
            id: format!("modpack_{}", i),
            urls: urls.clone(),
            local_path: path.clone(),
            expected_size: *size as i64,
            expected_hash: None,
        })
        .collect();

    let total_count = files.len() as u64;

    // 进度回调：DownloadManager 已内置 300ms timer + 滑动窗口速度计算
    // 直接用 sync_stage_from_progress 统一同步，无需额外 timer / 原子计数器 / 速度计算
    let progress_state = state.download_state.clone();
    let progress_stage_index = stage_index;
    let progress_callback: Arc<
        dyn Fn(crate::minecraft::download::types::GlobalProgress) + Send + Sync,
    > = Arc::new(move |p| {
        let mut ds = progress_state.lock().unwrap();
        ds.sync_stage_from_progress(
            progress_stage_index,
            p.downloaded_bytes,
            p.total_bytes,
            p.completed_files,
            p.total_files,
            p.current_speed,
        );
    });

    // 用 DownloadManager 下载（自动分片 + 多线程 + 重试 + URL fallback）
    let config = state.config.lock().await;
    let chunk_count = config.download.chunk_count.max(1) as usize;
    drop(config);
    let manager = DownloadManager::new(max_threads, chunk_count, 0, DownloadSourceMode::Smart)
        .with_cancel_flag(state.download_cancel_flag.clone())
        .with_pause_flag(state.download_pause_flag.clone());
    let results = manager.download_batch(tasks, Some(progress_callback)).await;

    // 收集失败
    let mut errors: Vec<String> = Vec::new();
    for (i, r) in results.iter().enumerate() {
        if r.status != crate::minecraft::download::types::DownloadStatus::Completed
            && r.status != crate::minecraft::download::types::DownloadStatus::Skipped
        {
            let (urls, path, _) = &files[i];
            let err = r.error.clone().unwrap_or_else(|| format!("{:?}", r.status));
            log_info!("[Community] 下载失败: {} → {}", path, err);
            log_info!("[Community] 尝试过的 URL: {}", urls.join(" | "));
            errors.push(format!("{}: {}", urls.join(" | "), err));
        }
    }

    if !errors.is_empty() {
        log_info!("[Community] 共 {} 个文件下载失败：", errors.len());
        for (i, e) in errors.iter().enumerate() {
            log_info!("[Community] 失败 #{}: {}", i + 1, e);
        }
        return Err(format!(
            "部分文件下载失败 ({}/{}): 首个错误={}",
            errors.len(),
            total_count,
            errors[0]
        ));
    }

    Ok(())
}

/// 从 zip 解压 overrides 到 instance 目录
///
/// `prefixes` 为 overrides 前缀列表（已含 archive_base_folder 前缀和末尾 `/`），
/// 按格式决定：
/// - CurseForge/Modrinth：`["{base}overrides/", "{base}client-overrides/"]`
/// - HMCL：`["{base}minecraft/"]`
/// - MMC：`["{base}.minecraft/"]`
/// - MCBBS：`["{base}overrides/"]`
///
/// 靠前的前缀优先匹配；前缀被去掉后剩余路径作为 instance 目录下的相对路径。
/// client-overrides 之类的次要前缀允许覆盖 overrides 已写入的同名文件。
pub(super) fn extract_overrides(
    archive: &mut zip::ZipArchive<std::fs::File>,
    instance_dir: &std::path::Path,
    state: &AppState,
    prefixes: &[String],
    stage_index: usize,
) -> Result<(), String> {
    use std::io::Read;
    let mut count: usize = 0;
    let total = archive.len();

    if prefixes.is_empty() {
        log_info!("[Community] overrides 前缀为空，跳过解压");
        return Ok(());
    }

    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| format!("读取 zip 条目失败: {}", e))?;
        let name = entry.name().to_string();

        // 按前缀列表顺序匹配，命中第一个前缀就去掉
        let relative = prefixes
            .iter()
            .find_map(|p| name.strip_prefix(p.as_str()).map(|r| r));

        let relative = match relative {
            Some(r) => r,
            None => continue,
        };

        if relative.is_empty() || relative.ends_with('/') {
            continue;
        }

        let target = instance_dir.join(relative);
        if let Some(parent) = target.parent() {
            if !parent.exists() {
                crate::utils::fs::ensure_dir(parent)?;
            }
        }

        if entry.is_file() {
            let mut buf = Vec::new();
            entry
                .read_to_end(&mut buf)
                .map_err(|e| format!("读取文件失败: {}", e))?;
            std::fs::write(&target, &buf).map_err(|e| format!("写入文件失败: {}", e))?;
            count += 1;
        }

        // 每 10 个文件更新一次进度
        if count % 10 == 0 {
            let mut ds = state.download_state.lock().unwrap();
            ds.set_stage_bytes(stage_index, count as u64, total as u64);
        }
    }

    {
        let mut ds = state.download_state.lock().unwrap();
        ds.set_stage_bytes(stage_index, count as u64, total as u64);
    }
    log_info!(
        "[Community] overrides 解压完成 ({} 个文件，前缀: {:?})",
        count,
        prefixes
    );
    Ok(())
}

/// 检测到的整合包信息（detect_modpack_format 的返回值）
///
/// `archive_base_folder` 为整合包关键文件所在的层级前缀（如 `""` 或 `"subfolder/"`），
/// 用于后续构造 overrides 完整前缀。
pub(super) struct DetectedModpack {
    pub format: super::types::ModpackFormat,
    /// 关键文件所在层级前缀（如 `""` 或 `"subfolder/"`），已含末尾 `/`（根目录为空字符串）
    pub archive_base_folder: String,
    /// CF manifest.json 或 MCBBS manifest.json/mcbbs.packmeta 的原始内容
    pub manifest_content: Option<String>,
    /// MR modrinth.index.json 的原始内容
    pub index_content: Option<String>,
    /// HMCL modpack.json 的原始内容
    pub hmcl_content: Option<String>,
    /// MMC mmc-pack.json 的原始内容
    pub mmc_content: Option<String>,
}

/// 检测整合包格式
///
/// 识别优先级（参考 PCL2 ModModpack.vb ModpackInstall）：
/// 1. `mcbbs.packmeta` → Mcbbs
/// 2. `mmc-pack.json` → Mmc
/// 3. `modrinth.index.json` → Modrinth
/// 4. `manifest.json`：有 `addons` 字段 → Mcbbs，无 → Curseforge
/// 5. `modpack.json` → Hmcl
///
/// 第一遍扫描根目录关键文件，命中即返回；第二遍扫描一级子目录。
/// `archive_base_folder` 在根目录命中时为 `""`，子目录命中时为 `"子目录/"`。
pub(super) fn detect_modpack_format(
    archive: &mut zip::ZipArchive<std::fs::File>,
) -> Result<DetectedModpack, String> {
    // 收集所有条目名及其索引
    let entry_names: Vec<(usize, String)> = (0..archive.len())
        .map(|i| {
            let name = archive
                .by_index(i)
                .map(|e| e.name().to_string())
                .unwrap_or_default();
            (i, name)
        })
        .collect();

    // 按 PCL2 优先级顺序的关键文件名
    // manifest.json 需要进一步判断 addons 字段，特殊处理
    const PRIORITY: &[&str] = &[
        "mcbbs.packmeta",
        "mmc-pack.json",
        "modrinth.index.json",
        "manifest.json",
        "modpack.json",
    ];

    // 第一遍：扫描根目录（路径不含 /），按优先级顺序查找
    for key in PRIORITY {
        for &(i, ref name) in &entry_names {
            if name.contains('/') {
                continue;
            }
            if name == *key {
                if let Some(detected) = try_detect_at_root(archive, i, name, "")? {
                    return Ok(detected);
                }
            }
        }
    }

    // 第二遍：扫描一级子目录（路径形如 "subfolder/关键文件"），按优先级顺序查找
    for key in PRIORITY {
        for &(i, ref name) in &entry_names {
            let parts: Vec<&str> = name.split('/').collect();
            if parts.len() != 2 {
                continue;
            }
            if parts[1] == *key {
                let base = format!("{}/", parts[0]);
                if let Some(detected) = try_detect_at_root(archive, i, parts[1], &base)? {
                    return Ok(detected);
                }
            }
        }
    }

    Err("无法识别的整合包格式：未找到 manifest.json / modrinth.index.json / modpack.json / mmc-pack.json / mcbbs.packmeta".to_string())
}

/// 尝试在指定 base_folder 下识别关键文件
///
/// `entry_index` 为关键文件在 zip 中的索引，`entry_name` 为关键文件名（不含 base_folder 前缀）。
/// 命中返回 `Some(DetectedModpack)`，否则返回 `None`。
fn try_detect_at_root(
    archive: &mut zip::ZipArchive<std::fs::File>,
    entry_index: usize,
    entry_name: &str,
    base_folder: &str,
) -> Result<Option<DetectedModpack>, String> {
    use super::types::ModpackFormat;
    use std::io::Read;

    // 按 PCL2 优先级判断
    if entry_name == "mcbbs.packmeta" {
        let mut s = String::new();
        archive
            .by_index(entry_index)
            .map_err(|e| format!("读取 zip 条目失败: {}", e))?
            .read_to_string(&mut s)
            .map_err(|e| format!("读取 mcbbs.packmeta 失败: {}", e))?;
        log_info!("[Community] 检测到 MCBBS 整合包（mcbbs.packmeta）");
        return Ok(Some(DetectedModpack {
            format: ModpackFormat::Mcbbs,
            archive_base_folder: base_folder.to_string(),
            manifest_content: Some(s),
            index_content: None,
            hmcl_content: None,
            mmc_content: None,
        }));
    }

    if entry_name == "mmc-pack.json" {
        let mut s = String::new();
        archive
            .by_index(entry_index)
            .map_err(|e| format!("读取 zip 条目失败: {}", e))?
            .read_to_string(&mut s)
            .map_err(|e| format!("读取 mmc-pack.json 失败: {}", e))?;
        log_info!("[Community] 检测到 MMC 整合包（mmc-pack.json）");
        return Ok(Some(DetectedModpack {
            format: ModpackFormat::Mmc,
            archive_base_folder: base_folder.to_string(),
            manifest_content: None,
            index_content: None,
            hmcl_content: None,
            mmc_content: Some(s),
        }));
    }

    if entry_name == "modrinth.index.json" {
        let mut s = String::new();
        archive
            .by_index(entry_index)
            .map_err(|e| format!("读取 zip 条目失败: {}", e))?
            .read_to_string(&mut s)
            .map_err(|e| format!("读取 modrinth.index.json 失败: {}", e))?;
        log_info!("[Community] 检测到 Modrinth 整合包（modrinth.index.json）");
        return Ok(Some(DetectedModpack {
            format: ModpackFormat::Modrinth,
            archive_base_folder: base_folder.to_string(),
            manifest_content: None,
            index_content: Some(s),
            hmcl_content: None,
            mmc_content: None,
        }));
    }

    if entry_name == "manifest.json" {
        let mut s = String::new();
        archive
            .by_index(entry_index)
            .map_err(|e| format!("读取 zip 条目失败: {}", e))?
            .read_to_string(&mut s)
            .map_err(|e| format!("读取 manifest.json 失败: {}", e))?;
        // 判断是否有 addons 字段：有 → MCBBS，无 → CurseForge
        let has_addons = serde_json::from_str::<serde_json::Value>(&s)
            .ok()
            .and_then(|v| v.get("addons").map(|a| !a.is_null()))
            .unwrap_or(false);
        if has_addons {
            log_info!("[Community] 检测到 MCBBS 整合包（manifest.json 含 addons）");
            Ok(Some(DetectedModpack {
                format: ModpackFormat::Mcbbs,
                archive_base_folder: base_folder.to_string(),
                manifest_content: Some(s),
                index_content: None,
                hmcl_content: None,
                mmc_content: None,
            }))
        } else {
            log_info!("[Community] 检测到 CurseForge 整合包（manifest.json）");
            Ok(Some(DetectedModpack {
                format: ModpackFormat::Curseforge,
                archive_base_folder: base_folder.to_string(),
                manifest_content: Some(s),
                index_content: None,
                hmcl_content: None,
                mmc_content: None,
            }))
        }
    } else if entry_name == "modpack.json" {
        let mut s = String::new();
        archive
            .by_index(entry_index)
            .map_err(|e| format!("读取 zip 条目失败: {}", e))?
            .read_to_string(&mut s)
            .map_err(|e| format!("读取 modpack.json 失败: {}", e))?;
        log_info!("[Community] 检测到 HMCL 整合包（modpack.json）");
        Ok(Some(DetectedModpack {
            format: ModpackFormat::Hmcl,
            archive_base_folder: base_folder.to_string(),
            manifest_content: None,
            index_content: None,
            hmcl_content: Some(s),
            mmc_content: None,
        }))
    } else {
        Ok(None)
    }
}

/// 根据 format 和 archive_base_folder 构造 overrides 前缀列表
///
/// 每个前缀已含 `archive_base_folder` 前缀和末尾 `/`，供 `extract_overrides` 直接匹配。
pub(super) fn build_overrides_prefixes(
    format: super::types::ModpackFormat,
    base_folder: &str,
) -> Vec<String> {
    use super::types::ModpackFormat;
    let base = base_folder;
    match format {
        ModpackFormat::Curseforge | ModpackFormat::Modrinth => vec![
            format!("{}overrides/", base),
            format!("{}client-overrides/", base),
        ],
        ModpackFormat::Hmcl => vec![format!("{}minecraft/", base)],
        ModpackFormat::Mmc => vec![format!("{}.minecraft/", base)],
        ModpackFormat::Mcbbs => vec![format!("{}overrides/", base)],
    }
}
