//! 文件系统操作与文件下载（阶段 4：runtime 目录定位 + 路径穿越校验 + 断点续传 + 下载循环）

use std::path::{Path, PathBuf};
use tauri::AppHandle;

use crate::log_warn;
use crate::minecraft::sources::{build_replace_urls, DownloadSourceMode};

use super::constants::DOWNLOAD_DOMAIN_REPLACEMENTS;
use super::progress;
use super::types::{DownloadInfo, RuntimeFile, RuntimeManifest};
use super::verify::verify_bytes_sha1;

/// 从 manifest 中过滤出需要下载的文件（有 downloads.raw 的）
pub fn filter_downloadable_files(manifest: &RuntimeManifest) -> Vec<(String, RuntimeFile)> {
    manifest
        .files
        .iter()
        .filter(|(_, f)| f.downloads.is_some())
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect()
}

/// 计算待下载文件的总字节数
pub fn total_bytes(files_to_download: &[(String, RuntimeFile)]) -> u64 {
    files_to_download
        .iter()
        .map(|(_, f)| f.downloads.as_ref().unwrap().raw.size)
        .sum()
}

/// 获取 Java Runtime 存储目录（{APPDATA}\.minecraft\runtime\{component}\）
///
/// 存到官启默认 .minecraft 目录下，跨游戏目录共享，不随 game_dir 删除而丢失。
pub fn get_runtime_dir(component: &str) -> Result<PathBuf, String> {
    crate::utils::cache_app::runtime_dir(component)
}

/// 在 runtime 目录中查找 java.exe
pub fn find_java_exe(runtime_dir: &Path) -> Result<PathBuf, String> {
    // 常见路径：runtime/{component}/windows-x64/{component}/bin/java.exe
    let candidates = [
        runtime_dir.join("bin").join("java.exe"),
        runtime_dir
            .join("windows-x64")
            .join(runtime_dir.file_name().unwrap_or_default())
            .join("bin")
            .join("java.exe"),
    ];

    for c in &candidates {
        if c.exists() {
            return Ok(c.clone());
        }
    }

    // 递归查找 java.exe
    fn find_recursive(dir: &Path) -> Option<PathBuf> {
        if dir.is_dir() {
            for entry in std::fs::read_dir(dir).ok()?.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(found) = find_recursive(&path) {
                        return Some(found);
                    }
                } else if path.file_name().map(|n| n == "java.exe").unwrap_or(false) {
                    return Some(path);
                }
            }
        }
        None
    }

    find_recursive(runtime_dir)
        .ok_or_else(|| format!("在 {} 中未找到 java.exe", runtime_dir.display()))
}

/// 校验路径穿越：manifest 来自远程，必须确保最终路径仍在 runtime_dir 内
///
/// 1. 拒绝显式包含 ".." 的路径
/// 2. canonicalize 校验最终路径父目录仍位于 runtime_dir 内
fn validate_path_traversal(
    path_str: &str,
    local_path: &Path,
    runtime_dir: &Path,
) -> Result<(), String> {
    if path_str.contains("..") {
        return Err(format!(
            "Path traversal detected in manifest path: {}",
            path_str
        ));
    }
    let canonical_base = runtime_dir
        .canonicalize()
        .unwrap_or_else(|_| runtime_dir.to_path_buf());
    if let Some(parent) = local_path.parent() {
        if let Ok(canonical_parent) = parent.canonicalize() {
            if !canonical_parent.starts_with(&canonical_base) {
                return Err(format!(
                    "Path traversal detected: {} is outside runtime dir",
                    path_str
                ));
            }
        }
    }
    Ok(())
}

/// 检查已存在文件是否可跳过（断点续传）
///
/// 尺寸匹配后再做 SHA1 校验，避免攻击者预先放置任意内容绕过。
/// 返回 true 表示可跳过下载。
fn should_skip_existing(local_path: &Path, download_info: &DownloadInfo, path_str: &str) -> bool {
    if !local_path.exists() {
        return false;
    }
    let Ok(meta) = std::fs::metadata(local_path) else {
        return false;
    };
    if meta.len() != download_info.size {
        return false;
    }
    if download_info.sha1.is_empty() {
        return true;
    }
    match std::fs::read(local_path) {
        Ok(existing_bytes) => {
            verify_bytes_sha1(&existing_bytes, &download_info.sha1, path_str).is_ok()
        }
        Err(e) => {
            log_warn!(
                "[JavaDownload] 读取已存在文件失败，重新下载: {}: {}",
                path_str,
                e
            );
            false
        }
    }
}

/// 下载所有文件（阶段 4）
///
/// 遍历 `files_to_download`，逐个下载并校验；支持断点续传与多源回退。
/// 任意文件下载失败时清理整个 runtime 目录并返回 Err。
pub async fn download_all_files(
    client: &reqwest::Client,
    files_to_download: &[(String, RuntimeFile)],
    runtime_dir: &Path,
    mirror_url: Option<&str>,
    mode: DownloadSourceMode,
    app: Option<&AppHandle>,
) -> Result<(), String> {
    let total_files = files_to_download.len();
    let total_bytes: u64 = total_bytes(files_to_download);

    let mut downloaded_bytes: u64 = 0;
    let mut completed: usize = 0;

    for (path_str, file) in files_to_download {
        let download_info = &file.downloads.as_ref().unwrap().raw;
        let local_path = runtime_dir.join(path_str);

        validate_path_traversal(path_str, &local_path, runtime_dir)?;

        // 跳过已存在且校验通过的文件（断点续传）
        if should_skip_existing(&local_path, download_info, path_str) {
            completed += 1;
            downloaded_bytes += download_info.size;
            progress::emit(
                app,
                "downloading",
                completed,
                total_files,
                downloaded_bytes,
                total_bytes,
                &format!("已跳过: {}", path_str),
            );
            continue;
        }

        // 创建父目录
        if let Some(parent) = local_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("创建目录失败: {}: {}", parent.display(), e))?;
        }

        // 构建候选 URL 列表（根据下载源模式）
        let urls = build_replace_urls(
            &download_info.url,
            mirror_url,
            DOWNLOAD_DOMAIN_REPLACEMENTS,
            mode,
        );

        let mut download_err = String::new();
        let mut success = false;

        for url in &urls {
            // Java 运行时文件约 50-100MB，覆盖全局 30s timeout 为 5 分钟
            // 避免慢速网络下大文件下载被误杀
            match client
                .get(url)
                .timeout(std::time::Duration::from_secs(300))
                .send()
                .await
            {
                Ok(resp) => {
                    if resp.status().is_success() {
                        match resp.bytes().await {
                            Ok(bytes) => {
                                if bytes.len() as u64 == download_info.size {
                                    // 写入前校验 SHA1，防止镜像源返回篡改内容
                                    verify_bytes_sha1(&bytes, &download_info.sha1, path_str)?;
                                    std::fs::write(&local_path, &bytes).map_err(|e| {
                                        format!("写入文件失败: {}: {}", local_path.display(), e)
                                    })?;
                                    success = true;
                                    break;
                                } else {
                                    download_err = format!(
                                        "尺寸不匹配: 期望 {} 实际 {}",
                                        download_info.size,
                                        bytes.len()
                                    );
                                }
                            }
                            Err(e) => download_err = format!("读取响应失败: {}", e),
                        }
                    } else {
                        download_err = format!("HTTP {}", resp.status());
                    }
                }
                Err(e) => download_err = format!("请求失败: {}", e),
            }
        }

        if !success {
            // 清理整个 runtime 目录
            let _ = std::fs::remove_dir_all(runtime_dir);
            return Err(format!("下载文件失败: {} - {}", path_str, download_err));
        }

        // 设置可执行权限（Unix）
        if file.executable {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ =
                    std::fs::set_permissions(&local_path, std::fs::Permissions::from_mode(0o755));
            }
        }

        completed += 1;
        downloaded_bytes += download_info.size;

        progress::emit(
            app,
            "downloading",
            completed,
            total_files,
            downloaded_bytes,
            total_bytes,
            &format!("{}/{}: {}", completed, total_files, path_str),
        );
    }

    Ok(())
}
