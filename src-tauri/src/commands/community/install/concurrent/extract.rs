//! overrides 解压与前缀构造

use crate::log_info;
use crate::state::AppState;

/// 从 zip 解压 overrides 到 instance 目录
///
/// `prefixes` 为 overrides 前缀列表（已含 archive_base_folder 前缀和末尾 `/`），
/// 靠前的前缀优先匹配；前缀被去掉后剩余路径作为 instance 目录下的相对路径。
/// 解压最多重试 5 次，每次失败线性退避，失败不清空目标目录采用覆盖写。
pub fn extract_overrides(
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
        let relative = prefixes.iter().find_map(|p| name.strip_prefix(p.as_str()));

        let relative = match relative {
            Some(r) => r,
            None => continue,
        };

        if relative.is_empty() || relative.ends_with('/') {
            continue;
        }

        crate::utils::path::ensure_safe_relative_path(&relative)
            .map_err(|e| format!("overrides 条目路径非法: {} ({})", name, e))?;

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
        if count.is_multiple_of(10) {
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

/// 根据 format 和 archive_base_folder 构造 overrides 前缀列表
///
/// 每个前缀已含 `archive_base_folder` 前缀和末尾 `/`。CurseForge 支持 manifest.overrides
/// 字段自定义覆写目录名（默认 "overrides"，可为 "." 或 "./" 表示根目录）。
pub fn build_overrides_prefixes(
    format: super::super::types::ModpackFormat,
    base_folder: &str,
    overrides_name: Option<&str>,
) -> Vec<String> {
    use super::super::types::ModpackFormat;
    let base = base_folder;
    match format {
        ModpackFormat::Curseforge => {
            let ov = overrides_name.unwrap_or("overrides");
            let prefix = if ov == "." || ov == "./" {
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
        ModpackFormat::Compress => vec![base.to_string()],
    }
}
