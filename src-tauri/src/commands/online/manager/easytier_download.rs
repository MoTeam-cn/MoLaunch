//! easytier 内核下载与安装实现（zip 下载 / 解压 / 安装目录管理）
//!
//! 与 `easytier_install.rs` 拆分（单文件 ≤350 行约束）：本文件只含安装实现，
//! IPC 注册 / 状态查询 / 进度事件入口保持在父模块。

use std::io::Read;
use std::path::Path;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;

use crate::log_info;
use crate::minecraft::download::types::{DownloadStatus, DownloadTask, GlobalProgress};
use crate::state::AppState;
use crate::utils::github_download::{build_proxy_url, pick_fastest};

use super::easytier_install::{
    asset_name, emit_progress, fetch_latest_release, install_dir, EASYTIER_REPO, VERSION_FILE,
};
#[cfg(unix)]
use super::easytier_install::{cli_name, core_name};

/// 解压 zip 到目标目录（剥离共享顶层目录 + Zip Slip 防护）
fn extract_zip_safely(zip_path: &Path, dst: &Path) -> Result<(), String> {
    let file = std::fs::File::open(zip_path).map_err(|e| format!("打开 ZIP 失败: {e}"))?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("解析 ZIP 失败: {e}"))?;
    // 确定共享顶层前缀（所有条目同一根目录时剥离，扁平包不剥离）
    let mut roots = std::collections::HashSet::new();
    let mut flat = false;
    for name in archive.file_names() {
        if name.contains('/') {
            if let Some(root) = name.split('/').next().filter(|v| !v.is_empty()) {
                roots.insert(root.to_string());
            }
        } else if !name.is_empty() {
            flat = true;
        }
    }
    let prefix = if flat || roots.len() != 1 {
        String::new()
    } else {
        format!("{}/", roots.into_iter().next().unwrap())
    };
    let canonical_dst = dst.canonicalize().unwrap_or_else(|_| dst.to_path_buf());
    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| format!("读取 ZIP 条目失败: {e}"))?;
        let name = entry.name().to_string();
        let rel = name.strip_prefix(&prefix).unwrap_or(&name);
        if rel.is_empty() {
            continue;
        }
        let path = dst.join(rel);
        if rel.ends_with('/') {
            std::fs::create_dir_all(&path).map_err(|e| format!("创建目录失败: {e}"))?;
            continue;
        }
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("创建父目录失败: {e}"))?;
            if !parent
                .canonicalize()
                .map_err(|e| format!("canonicalize 失败: {e}"))?
                .starts_with(&canonical_dst)
            {
                return Err(format!("Zip Slip 检测: {rel}"));
            }
        }
        let mut out = std::fs::File::create(&path).map_err(|e| format!("创建文件失败: {e}"))?;
        std::io::copy(&mut entry, &mut out).map_err(|e| format!("写入文件失败: {e}"))?;
    }
    Ok(())
}

/// 递归移动目录内容（临时目录 → 安装目录）
fn move_dir_contents(src: &Path, dst: &Path) -> Result<(), String> {
    std::fs::create_dir_all(dst).map_err(|e| format!("创建目录失败: {e}"))?;
    for entry in std::fs::read_dir(src)
        .map_err(|e| format!("读取目录失败: {e}"))?
        .flatten()
    {
        let p = entry.path();
        let target = dst.join(entry.file_name());
        if p.is_dir() {
            move_dir_contents(&p, &target)?;
        } else {
            std::fs::rename(&p, &target).map_err(|e| format!("移动文件失败: {e}"))?;
        }
    }
    Ok(())
}

/// 探测下载源文件大小（HEAD 请求，失败返回 0 走单流兜底）
///
/// 注意：reqwest 对 HEAD 响应 `content_length()` 一律返回 0（HEAD 无 body），
/// 需直接读 Content-Length 响应头才能拿到真实大小。
async fn probe_zip_size(client: &reqwest::Client, url: &str) -> u64 {
    match client
        .head(url)
        .timeout(Duration::from_secs(10))
        .send()
        .await
    {
        Ok(r) if r.status().is_success() => r
            .headers()
            .get(reqwest::header::CONTENT_LENGTH)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(0),
        _ => 0,
    }
}

/// 校验 zip 魔数（PK\x03\x04 文件头 / PK\x05\x06 空归档 / PK\x07\x08 分卷）
///
/// 镜像可能返回 HTML/挑战页等非 zip 内容（HTTP 200 且长度匹配，大小校验无法识别），
/// 魔数校验失败时下载链自动剔除该源回退官方保底，避免解压阶段才报 EOCD 错误。
fn is_zip_file(path: &Path) -> bool {
    let mut buf = [0u8; 4];
    let mut file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return false,
    };
    if file.read_exact(&mut buf).is_err() {
        return false;
    }
    buf.starts_with(b"PK")
        && matches!(buf[2], 0x03 | 0x05 | 0x07)
        && matches!(buf[3], 0x04 | 0x06 | 0x08)
}

/// 下载并安装指定版本（下载 → 解压 → version.txt → 执行权限）
pub(super) async fn install_version(
    state: &AppState,
    app: &tauri::AppHandle,
    version: &str,
) -> Result<(), String> {
    // Windows 下无法覆盖运行中的 exe：正在组网时拒绝重装，提示先退出
    if state.easytier.lock().await.is_some() {
        return Err("easytier 正在组网运行中，请先退出联机网络再更新内核".to_string());
    }
    let client = crate::http::get_client();
    let dir = install_dir()?;
    let asset = asset_name(version);
    let zip_path = std::env::temp_dir().join(format!("molaunch-easytier-{asset}"));
    let _ = std::fs::remove_file(&zip_path);

    emit_progress(app, "download", 5, &format!("下载 easytier v{version}"));
    let proxies = state.github_proxies.lock().await.clone();
    crate::log_debug!(
        "[EasyTier] 下载镜像源: {:?}",
        proxies
            .iter()
            .map(|p| (&p.name, &p.proxy_type))
            .collect::<Vec<_>>()
    );
    // 候选 URL：镜像优先（竞速选最快镜像），官方保底
    let mut urls: Vec<String> = Vec::new();
    if !proxies.is_empty() {
        let candidates: Vec<String> = proxies
            .iter()
            .map(|p| build_proxy_url(p, EASYTIER_REPO, version, &asset))
            .collect();
        crate::log_debug!("[EasyTier] 镜像竞速候选: {candidates:?}");
        if let Ok(fastest) = pick_fastest(&candidates).await {
            urls.push(fastest);
        }
    }
    urls.push(format!(
        "https://github.com/{EASYTIER_REPO}/releases/download/v{version}/{asset}"
    ));

    // 探测大小（分片下载需要 expected_size > 0；探测失败走单流兜底）
    let expected_size = probe_zip_size(&client, &urls[0]).await;
    let task = DownloadTask {
        id: format!("easytier-{version}"),
        urls,
        local_path: zip_path.to_string_lossy().to_string(),
        expected_size: expected_size as i64,
        expected_hash: None,
    };

    // 下载进度：5%→80% 按字节映射，仅在百分比变化时推送（避免逐 chunk 刷屏）
    let last_pct = std::sync::atomic::AtomicU8::new(5);
    let app2 = app.clone();
    let progress_cb: Arc<dyn Fn(GlobalProgress) + Send + Sync> = Arc::new(move |p| {
        let pct = if p.total_bytes > 0 {
            5 + (p.downloaded_bytes.saturating_mul(75) / p.total_bytes) as u8
        } else {
            5
        };
        if pct > last_pct.load(Ordering::Relaxed) {
            last_pct.store(pct, Ordering::Relaxed);
            emit_progress(&app2, "download", pct, &format!("下载中 {pct}%"));
        }
    });
    let manager = crate::minecraft::download::DownloadManager::from_state(state)
        .await
        .with_silent(true)
        .with_preserve_order(true)
        // 镜像可能返回 HTML/挑战页等非 zip 内容（大小校验无法识别），魔数校验失败自动回退官方
        .with_content_validator(Arc::new(|p| {
            if is_zip_file(p) {
                Ok(())
            } else {
                Err("下载内容不是有效的 ZIP 文件（镜像可能返回了错误页面）".to_string())
            }
        }));
    let results = manager.download_batch(vec![task], Some(progress_cb)).await;
    let result = results
        .into_iter()
        .next()
        .ok_or_else(|| "下载失败：无结果".to_string())?;
    if result.status != DownloadStatus::Completed {
        return Err(result.error.unwrap_or_else(|| "下载失败".to_string()));
    }
    // 下载完成：强制推进到 80%（分片下载实际字节与 HEAD 探测值可能有偏差，
    // 字节映射可能停在 80 以下，需收尾补发避免进度条停留中途）
    emit_progress(app, "download", 80, "下载完成");
    emit_progress(app, "extract", 85, "解压安装");
    let extract_dir =
        std::env::temp_dir().join(format!("molaunch-easytier-extract-{}", std::process::id()));
    if extract_dir.exists() {
        let _ = std::fs::remove_dir_all(&extract_dir);
    }
    std::fs::create_dir_all(&extract_dir).map_err(|e| format!("创建临时目录失败: {e}"))?;
    extract_zip_safely(&zip_path, &extract_dir)?;
    let _ = std::fs::remove_file(&zip_path);

    // 清空安装目录旧文件（防残留旧版本），再移动新文件
    if dir.exists() {
        for entry in std::fs::read_dir(&dir)
            .map_err(|e| format!("读取安装目录失败: {e}"))?
            .flatten()
        {
            let p = entry.path();
            if p.is_dir() {
                let _ = std::fs::remove_dir_all(&p);
            } else {
                let _ = std::fs::remove_file(&p);
            }
        }
    }
    move_dir_contents(&extract_dir, &dir)?;
    let _ = std::fs::remove_dir_all(&extract_dir);

    // Unix 补执行权限
    #[cfg(unix)]
    {
        crate::minecraft::system::shell::make_executable(&dir.join(core_name()));
        crate::minecraft::system::shell::make_executable(&dir.join(cli_name()));
    }

    std::fs::write(dir.join(VERSION_FILE), version).map_err(|e| format!("写版本标记失败: {e}"))?;
    emit_progress(app, "done", 100, &format!("easytier v{version} 安装完成"));
    log_info!("[EasyTier] 已安装 v{version} 到 {}", dir.display());
    Ok(())
}

/// 下载安装最新版（`easytier_install` / `easytier_update` 共用）
pub(super) async fn install_latest(state: &AppState, app: &tauri::AppHandle) -> Result<(), String> {
    let version = fetch_latest_release().await?;
    install_version(state, app, &version).await
}
