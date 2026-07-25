//! 联网检查：Modrinth hash + CurseForge fingerprint
//!
//! 参考 PCL2 PageInstanceExport.xaml.vb 的"联网获取文件信息"步骤。
//! 对扫描到的 mod 文件并发查询 Modrinth 和 CurseForge，获取下载地址。
//! 获取到地址的文件不直接打包进 overrides，而是写入 modrinth.index.json。

use std::collections::HashMap;
use std::path::Path;

use tauri::AppHandle;

use crate::log_info;
use crate::log_warn;
use crate::minecraft::community::curseforge;
use crate::minecraft::community::modrinth;
use crate::minecraft::community::preload::{
    compute_curseforge_fingerprint, compute_modrinth_sha1,
};
use crate::minecraft::community::types::FileDownloadInfo;

use super::scan::is_mod_like_file;
use super::types::{ExportFileInfo, ModDownloadInfo};

/// 对文件列表中的 mod 文件进行联网检查
///
/// 返回 `(获取到下载地址的 mod 信息列表)`。
/// 未获取到地址的文件将继续打包进 overrides。
pub async fn check_mod_files_online(
    _app: &AppHandle,
    files: &[ExportFileInfo],
) -> Result<Vec<ModDownloadInfo>, String> {
    // 1. 筛选需要联网检查的文件
    let mod_files: Vec<&ExportFileInfo> = files
        .iter()
        .filter(|f| is_mod_like_file(&f.relative_path))
        .collect();

    if mod_files.is_empty() {
        log_info!("[Export] 没有需要联网检查的 mod 文件");
        return Ok(Vec::new());
    }

    log_info!(
        "[Export] 开始联网检查 {} 个 mod 文件",
        mod_files.len()
    );

    // 2. 计算 hash（同步遍历，hash 计算内部读取文件）
    let mut mr_hashes: Vec<(String, String)> = Vec::new(); // (sha1, relative_path)
    let mut cf_hashes: Vec<(u32, String)> = Vec::new(); // (fingerprint, relative_path)

    for f in &mod_files {
        let mr_sha1 = compute_modrinth_sha1(&f.abs_path).unwrap_or_default();
        let cf_fp = compute_curseforge_fingerprint(&f.abs_path).unwrap_or(0);
        if !mr_sha1.is_empty() {
            mr_hashes.push((mr_sha1, f.relative_path.clone()));
        }
        if cf_fp != 0 {
            cf_hashes.push((cf_fp, f.relative_path.clone()));
        }
    }

    // 3. 并发查询 Modrinth + CurseForge（直接获取下载地址）
    let mr_sha1_list: Vec<String> = mr_hashes.iter().map(|(h, _)| h.clone()).collect();
    let cf_fp_list: Vec<u32> = cf_hashes.iter().map(|(h, _)| *h).collect();

    let (mr_result, cf_result) = tokio::join!(
        query_modrinth(mr_sha1_list),
        query_curseforge(cf_fp_list),
    );

    // 4. 合并结果：以文件相对路径为键，收集所有下载地址 + hash
    let mut info_map: HashMap<String, ModDownloadInfo> = HashMap::new();

    if let Ok(mr_map) = mr_result {
        log_info!("[Export] Modrinth 返回 {} 个结果", mr_map.len());
        merge_mr_results(&mr_map, &mr_hashes, &mut info_map);
    } else if let Err(e) = mr_result {
        log_warn!("[Export] Modrinth 查询失败: {}", e);
    }

    if let Ok(cf_map) = cf_result {
        log_info!("[Export] CurseForge 返回 {} 个结果", cf_map.len());
        merge_cf_results(&cf_map, &cf_hashes, &mut info_map, &mod_files);
    } else if let Err(e) = cf_result {
        log_warn!("[Export] CurseForge 查询失败: {}", e);
    }

    // 5. 输出结果（按文件扫描顺序排序，方便日志阅读）
    let mut result: Vec<ModDownloadInfo> = mod_files
        .iter()
        .filter_map(|f| info_map.get(&f.relative_path).cloned())
        .collect();

    // CF 不返回 sha1/sha512，对缺 hash 的条目本地补算
    for info in &mut result {
        if info.sha1.is_empty() || info.sha512.is_empty() {
            if let Some(f) = mod_files.iter().find(|f| f.relative_path == info.relative_path) {
                if info.sha1.is_empty() {
                    if let Ok(h) = compute_modrinth_sha1(&f.abs_path) {
                        info.sha1 = h;
                    }
                }
                if info.sha512.is_empty() {
                    if let Ok(h) = compute_sha512(&f.abs_path) {
                        info.sha512 = h;
                    }
                }
            }
        }
    }

    log_info!(
        "[Export] 联网检查汇总：{} 个 mod 获取到下载地址",
        result.len()
    );

    Ok(result)
}

/// 合并 Modrinth 查询结果到 info_map
fn merge_mr_results(
    mr_map: &HashMap<String, FileDownloadInfo>,
    mr_hashes: &[(String, String)],
    info_map: &mut HashMap<String, ModDownloadInfo>,
) {
    for (sha1, path) in mr_hashes {
        if let Some(info) = mr_map.get(sha1) {
            let entry = info_map.entry(path.clone()).or_insert_with(|| ModDownloadInfo {
                relative_path: path.clone(),
                sha1: String::new(),
                sha512: String::new(),
                downloads: Vec::new(),
                file_size: info.file_size,
                project_id: None,
                file_id: None,
            });
            // MR 返回的 sha1/sha512 直接使用
            if !info.sha1.is_empty() {
                entry.sha1 = info.sha1.clone();
            }
            if let Some(s512) = &info.sha512 {
                if !s512.is_empty() {
                    entry.sha512 = s512.clone();
                }
            }
            entry.downloads.push(info.download_url.clone());
        }
    }
}

/// 合并 CurseForge 查询结果到 info_map
///
/// CF 不返回 sha1/sha512，留空待后续本地补算。
/// CF 返回 project_id 和 file_id，用于导出 CF 格式整合包时写入 manifest.files[]。
fn merge_cf_results(
    cf_map: &HashMap<u32, FileDownloadInfo>,
    cf_hashes: &[(u32, String)],
    info_map: &mut HashMap<String, ModDownloadInfo>,
    mod_files: &[&ExportFileInfo],
) {
    for (fp, path) in cf_hashes {
        if let Some(info) = cf_map.get(fp) {
            let entry = info_map.entry(path.clone()).or_insert_with(|| ModDownloadInfo {
                relative_path: path.clone(),
                sha1: String::new(),
                sha512: String::new(),
                downloads: Vec::new(),
                file_size: info.file_size,
                project_id: info.project_id,
                file_id: info.file_id,
            });
            entry.downloads.push(info.download_url.clone());
            // CF 返回 project_id/file_id 直接使用（MR 未设置时也填充，CF 优先级低）
            if entry.project_id.is_none() {
                entry.project_id = info.project_id;
            }
            if entry.file_id.is_none() {
                entry.file_id = info.file_id;
            }
            // file_size 用本地文件大小兜底（CF 偶发返回 0）
            if entry.file_size == 0 {
                if let Some(f) = mod_files.iter().find(|f| f.relative_path == *path) {
                    entry.file_size = f.size;
                }
            }
        }
    }
}

async fn query_modrinth(
    sha1s: Vec<String>,
) -> Result<HashMap<String, FileDownloadInfo>, String> {
    if sha1s.is_empty() {
        return Ok(HashMap::new());
    }
    modrinth::version_files_search_with_downloads(sha1s).await
}

async fn query_curseforge(
    fingerprints: Vec<u32>,
) -> Result<HashMap<u32, FileDownloadInfo>, String> {
    if fingerprints.is_empty() {
        return Ok(HashMap::new());
    }
    curseforge::fingerprint_search_with_downloads(fingerprints).await
}

/// 计算文件 SHA512 hash（hex 编码）
fn compute_sha512(path: &Path) -> Result<String, String> {
    use sha2::{Digest, Sha512};
    let bytes = std::fs::read(path).map_err(|e| format!("读取文件失败: {}", e))?;
    let mut hasher = Sha512::new();
    hasher.update(&bytes);
    Ok(hex::encode(hasher.finalize()))
}
