//! Java 自动下载模块
//!
//! 从 Mojang 官方 Java Runtime 索引下载 Java（参考 PCL2 ModJava.vb:691-754）
//! 下载源：piston-meta.mojang.com（官方）/ bmclapi2.bangbang93.com（镜像）
//! 下载到 {APPDATA}\.minecraft\runtime\{component}\（与 PCL2 一致，跨游戏目录共享）

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::{AppHandle, Emitter};

use crate::{log_info, log_warn};
use crate::minecraft::sources::{build_replace_urls, DownloadSourceMode};
use crate::minecraft::utils::file_checker::compute_sha1_hex;

/// Mojang Java Runtime 索引 URL（官方）
const JAVA_RUNTIME_INDEX_OFFICIAL: &str =
    "https://piston-meta.mojang.com/v1/products/java-runtime/2ec0cc96c44e5a76b9c8b7c39df7210883d12871/all.json";

/// 文件下载域名替换：Mojang 官方域名 → BMCLAPI
const DOWNLOAD_DOMAIN_REPLACEMENTS: &[(&str, &str)] = &[
    ("https://piston-data.mojang.com", "https://bmclapi2.bangbang93.com"),
    ("https://piston-meta.mojang.com", "https://bmclapi2.bangbang93.com"),
];

/// Java 下载进度事件名
pub const JAVA_DOWNLOAD_PROGRESS_EVENT: &str = "java-download-progress";

/// Mojang all.json 中的单个 Java 条目
#[derive(Debug, Clone, Serialize, Deserialize)]
struct JavaRuntimeEntry {
    #[serde(rename = "manifest")]
    manifest: ManifestRef,
    version: VersionInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ManifestRef {
    url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct VersionInfo {
    name: String,
}

/// manifest.json 中的文件清单
#[derive(Debug, Clone, Serialize, Deserialize)]
struct RuntimeManifest {
    files: std::collections::HashMap<String, RuntimeFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RuntimeFile {
    #[serde(rename = "type")]
    file_type: String,
    downloads: Option<Downloads>,
    #[serde(default)]
    executable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Downloads {
    raw: DownloadInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DownloadInfo {
    url: String,
    size: u64,
    sha1: String,
}

/// 下载进度事件 payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JavaDownloadProgress {
    pub stage: String,
    pub current: usize,
    pub total: usize,
    pub bytes_downloaded: u64,
    pub bytes_total: u64,
    pub message: String,
}

/// 根据 Java 大版本号匹配 Mojang component
///
/// Mojang 的 all.json 按 component 名分类（如 java-runtime-gamma）
/// version.name 格式通常为 "{major}.{minor}.{patch}"，可用于匹配
fn match_component(
    all_json: &serde_json::Value,
    target_major: u32,
) -> Option<(String, JavaRuntimeEntry)> {
    let platform = if cfg!(target_arch = "aarch64") {
        "windows-arm64"
    } else {
        "windows-x64"
    };

    let platform_node = all_json.get(platform)?;
    let components = platform_node.as_object()?;

    // 按 target_major 匹配：优先精确匹配 version.name 的首段
    // PCL2 策略：先找精确 key，再模糊匹配 version.name 开头
    let target_str = target_major.to_string();

    // 1. 精确匹配 component key（如 "21"、"17"、"8"）
    if let Some(arr) = components.get(&target_str).and_then(|v| v.as_array()) {
        if let Some(first) = arr.first() {
            if let Ok(entry) = serde_json::from_value::<JavaRuntimeEntry>(first.clone()) {
                return Some((target_str.clone(), entry));
            }
        }
    }

    // 2. 模糊匹配 version.name 以 target_major 开头
    for (key, arr) in components {
        if let Some(arr) = arr.as_array() {
            for item in arr {
                if let Ok(entry) = serde_json::from_value::<JavaRuntimeEntry>(item.clone()) {
                    if entry.version.name.starts_with(&format!("{}.", target_str))
                        || entry.version.name == target_str
                    {
                        return Some((key.clone(), entry));
                    }
                }
            }
        }
    }

    // 3. 回退：按 component 名约定匹配
    let fallback_key = match target_major {
        21 => "java-runtime-gamma",
        17 => "java-runtime-alpha",
        8 => "java-runtime-legacy",
        _ => return None,
    };
    if let Some(arr) = components.get(fallback_key).and_then(|v| v.as_array()) {
        if let Some(first) = arr.first() {
            if let Ok(entry) = serde_json::from_value::<JavaRuntimeEntry>(first.clone()) {
                return Some((fallback_key.to_string(), entry));
            }
        }
    }

    None
}

/// 获取 Java Runtime 存储目录（{APPDATA}\.minecraft\runtime\{component}\）
///
/// 与 PCL2 一致，存到官启默认 .minecraft 目录下，跨游戏目录共享，不随 game_dir 删除而丢失。
fn get_runtime_dir(component: &str) -> Result<PathBuf, String> {
    let appdata = std::env::var("APPDATA")
        .map_err(|_| "无法获取 APPDATA 环境变量".to_string())?;
    Ok(PathBuf::from(appdata)
        .join(".minecraft")
        .join("runtime")
        .join(component))
}

/// 下载 Java Runtime
///
/// # 参数
/// - `target_major`: 目标 Java 大版本号（如 21、17、8）
/// - `mode`: 下载源模式（Official / Mirror / Smart）
/// - `mirror_url`: 自定义镜像源 URL（None 或空则用 BMCLAPI）
/// - `app`: Tauri AppHandle（用于推送进度事件）
///
/// # 返回
/// 下载的 Java 可执行文件路径（java.exe）
pub async fn download_java_runtime(
    target_major: u32,
    mode: DownloadSourceMode,
    mirror_url: Option<&str>,
    app: Option<&AppHandle>,
) -> Result<PathBuf, String> {
    let client = crate::http::get_client();

    // 阶段 1：拉取 all.json
    emit_progress(app, "fetching", 0, 1, 0, 0, "正在获取 Java 索引...");

    let index_urls = build_replace_urls(
        JAVA_RUNTIME_INDEX_OFFICIAL,
        mirror_url,
        DOWNLOAD_DOMAIN_REPLACEMENTS,
        mode,
    );
    let all_json_text = fetch_text_with_fallback(&client, &index_urls).await?;

    let all_json: serde_json::Value = serde_json::from_str(&all_json_text)
        .map_err(|e| format!("解析 Java 索引失败: {}", e))?;

    // 阶段 2：匹配 component
    emit_progress(app, "matching", 0, 1, 0, 0, "正在匹配 Java 版本...");

    let (component, entry) = match_component(&all_json, target_major).ok_or_else(|| {
        format!(
            "未找到适配 Java {} 的 Mojang Runtime（platform: windows-x64）",
            target_major
        )
    })?;

    log_info!(
        "[JavaDownload] Matched component: {} (version: {})",
        component,
        entry.version.name
    );

    // 阶段 3：拉取 manifest
    emit_progress(app, "manifest", 0, 1, 0, 0, "正在获取文件清单...");

    let manifest_urls = build_replace_urls(
        &entry.manifest.url,
        mirror_url,
        DOWNLOAD_DOMAIN_REPLACEMENTS,
        mode,
    );
    let manifest_text = fetch_text_with_fallback(&client, &manifest_urls).await?;

    let manifest: RuntimeManifest = serde_json::from_str(&manifest_text)
        .map_err(|e| format!("解析文件清单失败: {}", e))?;

    // 过滤出需要下载的文件（有 downloads.raw 的）
    let files_to_download: Vec<(String, RuntimeFile)> = manifest
        .files
        .iter()
        .filter(|(_, f)| f.downloads.is_some())
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();

    let total_files = files_to_download.len();
    log_info!("[JavaDownload] {} files to download", total_files);

    // 阶段 4：下载文件
    let runtime_dir = get_runtime_dir(&component)?;
    let total_bytes: u64 = files_to_download
        .iter()
        .map(|(_, f)| f.downloads.as_ref().unwrap().raw.size)
        .sum();

    emit_progress(
        app,
        "downloading",
        0,
        total_files,
        0,
        total_bytes,
        &format!("正在下载 Java {} (共 {} 个文件)...", target_major, total_files),
    );

    let mut downloaded_bytes: u64 = 0;
    let mut completed: usize = 0;

    for (path_str, file) in &files_to_download {
        let download_info = &file.downloads.as_ref().unwrap().raw;
        let local_path = runtime_dir.join(path_str);

        // 校验路径穿越：manifest 来自远程，必须确保最终路径仍在 runtime_dir 内
        // 1. 拒绝显式包含 ".." 的路径
        if path_str.contains("..") {
            return Err(format!(
                "Path traversal detected in manifest path: {}",
                path_str
            ));
        }
        // 2. canonicalize 校验最终路径父目录仍位于 runtime_dir 内
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

        // 跳过已存在且校验通过的文件（断点续传）
        if local_path.exists() {
            if let Ok(meta) = std::fs::metadata(&local_path) {
                if meta.len() == download_info.size {
                    // 尺寸匹配后再做 SHA1 校验，避免攻击者预先放置任意内容绕过
                    let sha1_ok = if download_info.sha1.is_empty() {
                        true
                    } else {
                        match std::fs::read(&local_path) {
                            Ok(existing_bytes) => {
                                verify_bytes_sha1(
                                    &existing_bytes,
                                    &download_info.sha1,
                                    path_str,
                                )
                                .is_ok()
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
                    };
                    if sha1_ok {
                        completed += 1;
                        downloaded_bytes += download_info.size;
                        emit_progress(
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
                    // SHA1 不匹配，继续下载覆盖
                }
            }
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
            match client.get(url).send().await {
                Ok(resp) => {
                    if resp.status().is_success() {
                        match resp.bytes().await {
                            Ok(bytes) => {
                                if bytes.len() as u64 == download_info.size {
                                    // 写入前校验 SHA1，防止镜像源返回篡改内容
                                    verify_bytes_sha1(
                                        &bytes,
                                        &download_info.sha1,
                                        path_str,
                                    )?;
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
            let _ = std::fs::remove_dir_all(&runtime_dir);
            return Err(format!(
                "下载文件失败: {} - {}",
                path_str, download_err
            ));
        }

        // 设置可执行权限（Unix）
        if file.executable {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ = std::fs::set_permissions(
                    &local_path,
                    std::fs::Permissions::from_mode(0o755),
                );
            }
        }

        completed += 1;
        downloaded_bytes += download_info.size;

        emit_progress(
            app,
            "downloading",
            completed,
            total_files,
            downloaded_bytes,
            total_bytes,
            &format!("{}/{}: {}", completed, total_files, path_str),
        );
    }

    // 阶段 5：验证下载的 Java
    emit_progress(app, "verifying", 0, 1, 0, 0, "正在验证 Java...");

    let java_exe = find_java_exe(&runtime_dir)?;
    if let Some(ref handle) = app {
        let _ = handle.emit(
            JAVA_DOWNLOAD_PROGRESS_EVENT,
            JavaDownloadProgress {
                stage: "verifying".to_string(),
                current: 0,
                total: 1,
                bytes_downloaded: 0,
                bytes_total: 0,
                message: format!("验证 Java: {}", java_exe.display()),
            },
        );
    }

    // 调用 detect_java 验证
    match crate::minecraft::java::detect_java(&java_exe) {
        Ok(runtime) => {
            log_info!(
                "[JavaDownload] Verified: Java {} ({})",
                runtime.version,
                java_exe.display()
            );
        }
        Err(e) => {
            log_info!("[JavaDownload] Java verification failed: {}", e);
            // 不阻断，仍然返回路径
        }
    }

    // 完成
    emit_progress(app, "done", total_files, total_files, total_bytes, total_bytes, "下载完成");

    Ok(java_exe)
}

/// 在 runtime 目录中查找 java.exe
fn find_java_exe(runtime_dir: &std::path::Path) -> Result<PathBuf, String> {
    // 常见路径：runtime/{component}/windows-x64/{component}/bin/java.exe
    let candidates = [
        runtime_dir.join("bin").join("java.exe"),
        runtime_dir.join("windows-x64").join(runtime_dir.file_name().unwrap_or_default()).join("bin").join("java.exe"),
    ];

    for c in &candidates {
        if c.exists() {
            return Ok(c.clone());
        }
    }

    // 递归查找 java.exe
    fn find_recursive(dir: &std::path::Path) -> Option<PathBuf> {
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

/// 带回退的文本获取（依次尝试 URL 列表）
async fn fetch_text_with_fallback(
    client: &reqwest::Client,
    urls: &[String],
) -> Result<String, String> {
    let mut last_err = String::new();
    for url in urls {
        match client.get(url).send().await {
            Ok(resp) if resp.status().is_success() => {
                match resp.text().await {
                    Ok(text) => return Ok(text),
                    Err(e) => last_err = format!("读取失败: {}", e),
                }
            }
            Ok(resp) => last_err = format!("HTTP {}", resp.status()),
            Err(e) => last_err = format!("请求失败: {}", e),
        }
    }
    Err(format!("所有源均失败: {}", last_err))
}

/// 推送进度事件
fn emit_progress(
    app: Option<&AppHandle>,
    stage: &str,
    current: usize,
    total: usize,
    bytes_downloaded: u64,
    bytes_total: u64,
    message: &str,
) {
    if let Some(handle) = app {
        let _ = handle.emit(
            JAVA_DOWNLOAD_PROGRESS_EVENT,
            JavaDownloadProgress {
                stage: stage.to_string(),
                current,
                total,
                bytes_downloaded,
                bytes_total,
                message: message.to_string(),
            },
        );
    }
}

/// 校验字节的 SHA1，`expected_sha1` 为空则跳过（返回 Ok）
fn verify_bytes_sha1(
    bytes: &[u8],
    expected_sha1: &str,
    path_str: &str,
) -> Result<(), String> {
    if expected_sha1.is_empty() {
        return Ok(());
    }
    let computed = compute_sha1_hex(bytes);
    if computed.to_lowercase() != expected_sha1.to_lowercase() {
        log_warn!(
            "[JavaDownload] SHA1 mismatch for {}: expected {}, got {}",
            path_str,
            expected_sha1,
            computed
        );
        return Err(format!("SHA1 verification failed for {}", path_str));
    }
    log_info!("[JavaDownload] SHA1 verified for {}", path_str);
    Ok(())
}
