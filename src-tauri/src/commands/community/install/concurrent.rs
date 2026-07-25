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
    if prefixes.is_empty() {
        log_info!("[Community] overrides 前缀为空，跳过解压");
        return Ok(());
    }

    // 解压重试：最多 5 次，每次失败后线性退避（N * 2 秒）
    // 失败后不清空目标目录，采用覆盖写策略
    const MAX_ATTEMPTS: usize = 5;
    let total = archive.len();
    let mut last_err: Option<String> = None;

    for attempt in 1..=MAX_ATTEMPTS {
        match extract_overrides_once(archive, instance_dir, state, prefixes, stage_index, total) {
            Ok(count) => {
                if attempt > 1 {
                    log_info!(
                        "[Community] overrides 第 {} 次重试解压成功 ({} 个文件)",
                        attempt,
                        count
                    );
                }
                log_info!(
                    "[Community] overrides 解压完成 ({} 个文件，前缀: {:?})",
                    count,
                    prefixes
                );
                return Ok(());
            }
            Err(e) => {
                last_err = Some(e.clone());
                if attempt < MAX_ATTEMPTS {
                    let backoff_secs = attempt as u64 * 2;
                    crate::log_warn!(
                        "[Community] overrides 解压第 {} 次失败: {}，{} 秒后重试",
                        attempt,
                        e,
                        backoff_secs
                    );
                    std::thread::sleep(std::time::Duration::from_secs(backoff_secs));
                } else {
                    crate::log_warn!(
                        "[Community] overrides 解压 {} 次均失败: {}",
                        MAX_ATTEMPTS,
                        e
                    );
                }
            }
        }
    }

    Err(last_err.unwrap_or_else(|| "overrides 解压失败（未知原因）".to_string()))
}

/// 单次 overrides 解压（不带重试）
fn extract_overrides_once(
    archive: &mut zip::ZipArchive<std::fs::File>,
    instance_dir: &std::path::Path,
    state: &AppState,
    prefixes: &[String],
    stage_index: usize,
    total: usize,
) -> Result<usize, String> {
    use std::io::Read;
    let mut count: usize = 0;

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
    Ok(count)
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
    /// MMC instance.cfg 的原始内容（仅 MMC 格式有值，用于配置迁移）
    pub mmc_cfg_content: Option<String>,
    /// LauncherPack 内层整合包在 zip 中的完整路径（如 `modpack.zip` 或 `subfolder/modpack.mrpack`）
    pub launcher_inner_path: Option<String>,
}

/// 检测整合包格式
///
/// 识别优先级：
/// 1. `mcbbs.packmeta` → Mcbbs
/// 2. `mmc-pack.json` → Mmc
/// 3. `modrinth.index.json` → Modrinth
/// 4. `manifest.json`：有 `addons` 字段 → Mcbbs，无 → Curseforge
/// 5. `modpack.json` → Hmcl
/// 6. 根目录/一级子目录含 `modpack.zip` 或 `modpack.mrpack` → LauncherPack
/// 7. 其他 → Compress（普通压缩包）
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

    // 按优先级顺序的关键文件名
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

    // 第三遍：扫描根目录/一级子目录的 `modpack.zip` / `modpack.mrpack` → LauncherPack
    // 带启动器整合包：外层 zip 内嵌一个真正的整合包，需提取后递归安装
    for &(i, ref name) in &entry_names {
        let base = if let Some(stripped) = name.strip_prefix("modpack.zip") {
            if stripped.is_empty() {
                ""
            } else {
                continue;
            }
        } else if let Some(stripped) = name.strip_prefix("modpack.mrpack") {
            if stripped.is_empty() {
                ""
            } else {
                continue;
            }
        } else {
            continue;
        };
        let _ = i;
        log_info!("[Community] 检测到带启动器整合包（内嵌 {}）", name);
        return Ok(DetectedModpack {
            format: super::types::ModpackFormat::LauncherPack,
            archive_base_folder: base.to_string(),
            manifest_content: None,
            index_content: None,
            hmcl_content: None,
            mmc_content: None,
            mmc_cfg_content: None,
            launcher_inner_path: Some(name.clone()),
        });
    }
    for &(i, ref name) in &entry_names {
        let parts: Vec<&str> = name.split('/').collect();
        if parts.len() != 2 {
            continue;
        }
        if parts[1] == "modpack.zip" || parts[1] == "modpack.mrpack" {
            let _ = i;
            let base = format!("{}/", parts[0]);
            log_info!("[Community] 检测到带启动器整合包（内嵌 {}）", name);
            return Ok(DetectedModpack {
                format: super::types::ModpackFormat::LauncherPack,
                archive_base_folder: base,
                manifest_content: None,
                index_content: None,
                hmcl_content: None,
                mmc_content: None,
                mmc_cfg_content: None,
                launcher_inner_path: Some(name.clone()),
            });
        }
    }

    // 第四遍：Compress 兜底，扫描 `.minecraft/` 目录前缀
    // 普通压缩包：用户直接压缩 .minecraft 目录，无关键 manifest 文件
    let mut minecraft_prefix: Option<String> = None;
    for &(_, ref name) in &entry_names {
        // 命中 `.minecraft/` 在根目录或一级子目录
        if let Some(rest) = name.strip_prefix(".minecraft/") {
            if !rest.is_empty() {
                minecraft_prefix = Some(".minecraft/".to_string());
                break;
            }
        }
        if let Some(rest) = name.strip_prefix("/.minecraft/") {
            if !rest.is_empty() {
                minecraft_prefix = Some("/.minecraft/".to_string());
                break;
            }
        }
        // 一级子目录形式：`subfolder/.minecraft/...`
        let parts: Vec<&str> = name.split('/').collect();
        if parts.len() >= 2 && parts[0] != ".minecraft" {
            // 检测形如 `subfolder/.minecraft/...`
            if parts.len() >= 3 && parts[1] == ".minecraft" {
                let prefix = format!("{}/.minecraft/", parts[0]);
                minecraft_prefix = Some(prefix);
                break;
            }
        }
    }
    if let Some(prefix) = minecraft_prefix {
        log_info!(
            "[Community] 检测到普通压缩包整合包（.minecraft 前缀: {}）",
            prefix
        );
        return Ok(DetectedModpack {
            format: super::types::ModpackFormat::Compress,
            archive_base_folder: prefix,
            manifest_content: None,
            index_content: None,
            hmcl_content: None,
            mmc_content: None,
            mmc_cfg_content: None,
            launcher_inner_path: None,
        });
    }

    Err("无法识别的整合包格式：未找到 manifest.json / modrinth.index.json / modpack.json / mmc-pack.json / mcbbs.packmeta / modpack.zip / .minecraft/ 目录".to_string())
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

    // 按优先级判断
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
            mmc_cfg_content: None,
            launcher_inner_path: None,
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

        // 顺带读取同 base_folder 下的 instance.cfg（用于配置迁移：PreLaunchCommand/JvmArgs 等）
        let cfg_path = format!("{}instance.cfg", base_folder);
        let mut mmc_cfg_content: Option<String> = None;
        for i in 0..archive.len() {
            let Ok(mut entry) = archive.by_index(i) else {
                continue;
            };
            if entry.name() == cfg_path {
                let mut cfg = String::new();
                if entry.read_to_string(&mut cfg).is_ok() {
                    mmc_cfg_content = Some(cfg);
                }
                break;
            }
        }
        if mmc_cfg_content.is_some() {
            log_info!("[Community] MMC instance.cfg 已加载，将迁移配置到 setup.ini");
        }

        return Ok(Some(DetectedModpack {
            format: ModpackFormat::Mmc,
            archive_base_folder: base_folder.to_string(),
            manifest_content: None,
            index_content: None,
            hmcl_content: None,
            mmc_content: Some(s),
            mmc_cfg_content,
            launcher_inner_path: None,
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
            mmc_cfg_content: None,
            launcher_inner_path: None,
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
                mmc_cfg_content: None,
            launcher_inner_path: None,
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
                mmc_cfg_content: None,
            launcher_inner_path: None,
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
            mmc_cfg_content: None,
            launcher_inner_path: None,
        }))
    } else {
        Ok(None)
    }
}

/// 根据 format 和 archive_base_folder 构造 overrides 前缀列表
///
/// 每个前缀已含 `archive_base_folder` 前缀和末尾 `/`，供 `extract_overrides` 直接匹配。
/// CurseForge 支持 manifest.overrides 字段自定义覆写目录名（默认 "overrides"，
/// 可为 "." 或 "./" 表示根目录）。
pub(super) fn build_overrides_prefixes(
    format: super::types::ModpackFormat,
    base_folder: &str,
    overrides_name: Option<&str>,
) -> Vec<String> {
    use super::types::ModpackFormat;
    let base = base_folder;
    match format {
        ModpackFormat::Curseforge => {
            // manifest.overrides 字段：默认 "overrides"，可为 "." / "./" 表示根目录
            let ov = overrides_name.unwrap_or("overrides");
            let prefix = if ov == "." || ov == "./" {
                // 根目录：前缀为 base_folder 本身（匹配所有文件）
                base.to_string()
            } else {
                format!("{}{}/", base, ov)
            };
            vec![prefix, format!("{}client-overrides/", base)]
        }
        ModpackFormat::Modrinth => vec![
            format!("{}overrides/", base),
            format!("{}client-overrides/", base),
        ],
        ModpackFormat::Hmcl => vec![format!("{}minecraft/", base)],
        ModpackFormat::Mmc => vec![format!("{}.minecraft/", base)],
        ModpackFormat::Mcbbs => vec![format!("{}overrides/", base)],
        // LauncherPack 不会走到这里：上层 install 流程会先解压内层整合包再递归调用安装
        ModpackFormat::LauncherPack => Vec::new(),
        // Compress：archive_base_folder 已是 `.minecraft/` 前缀，直接作为 overrides 前缀
        // 前缀本身已含末尾 `/`，extract_overrides 会去掉该前缀将内容解压到 instance 目录
        ModpackFormat::Compress => vec![base.to_string()],
    }
}
