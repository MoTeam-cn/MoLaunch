//! frpc 二进制下载与管理
//!
//! 从 `install.rs` 拆分，职责：
//! - 系统默认厂商 frpc：从 apiServer `/v1/frp/manifest` 接口获取最新版本下载 URL，
//!   下载 ZIP 并提取 frpc 二进制（替代早期 GitHub Releases 直链下载）
//! - 外部厂商 frpc：按 `manifest.binary.distribution` 处理
//!   - bundled：仅校验文件存在（厂商包自带）
//!   - url：从配置 URL 下载（HTTPS + 域名白名单 + SHA256 校验 + 可选解压）
//!
//! 依赖 `provider.rs` 的路径函数、manifest 读取等（`super::provider::*`）。
//! apiServer 调用复用 `crate::minecraft::online::client::OnlineClient` 与
//! `crate::utils::online_manager::load_creds_with_auto_refresh`，与信令 action 风格一致。

use super::provider::{
    frpc_path, get_frpc_path_for_provider, is_external_frpc_ready, is_frpc_ready,
    read_frpc_version, read_provider_manifest, system_default_dir, write_frpc_version,
    SYSTEM_DEFAULT_ID,
};
use super::{ensure_dir, providers_root, ProviderManifest};
use crate::log_info;
use crate::minecraft::online::client::OnlineClient;
use crate::minecraft::online::frp::{FrpManifest, FrpManifestQuery};
use crate::state::AppState;
use std::path::Path;

// ============================================================
// frpc 二进制管理
// ============================================================

/// 下载 frpc 二进制
///
/// `provider_id` 为 None 或 `system-default` 时走系统默认厂商下载逻辑
/// （从 apiServer `/v1/frp/manifest` 获取 URL）。
/// 外部厂商根据 manifest.binary.distribution 处理：
/// - bundled: 仅校验文件存在（厂商包自带 frpc）
/// - url: 从配置的 URL 下载（HTTPS + 域名白名单 + SHA256 校验）
pub async fn ensure_frpc(
    state: &AppState,
    provider_id: Option<String>,
) -> Result<String, String> {
    let pid = provider_id.unwrap_or_else(|| SYSTEM_DEFAULT_ID.to_string());
    if pid == SYSTEM_DEFAULT_ID {
        return ensure_system_default_frpc(state).await;
    }
    let manifest = read_provider_manifest(&pid)?;
    if is_external_frpc_ready(&pid, &manifest) {
        let path = get_frpc_path_for_provider(&pid)?;
        return Ok(format!("frpc 已就绪: {}", path.display()));
    }
    // frpc 未就绪：bundled 无法补下，仅 url 可下载
    match manifest.binary.distribution.as_str() {
        "bundled" => Err(format!(
            "厂商 {} 的 frpc 二进制缺失，请重新安装厂商包",
            pid
        )),
        "url" => ensure_external_frpc(&pid, &manifest).await,
        other => Err(format!(
            "厂商 {} 使用不支持的分发方式: {}",
            pid, other
        )),
    }
}

/// 系统默认厂商 frpc 下载
///
/// 流程：
/// 1. 已就绪 → 直接返回
/// 2. 调用 apiServer `GET /v1/frp/manifest` 查询最新版本 + 下载 URL
///    - `data=None`（已是最新）但本地无 frpc：报错（apiServer 未提供旧版本下载）
///    - `data=Some(manifest)`：使用 manifest.url 下载
/// 3. 下载 ZIP → 提取 frpc 二进制 → 写入 frpc_path()
///
/// 失败时不保留半成品文件，避免下次误判为就绪。
async fn ensure_system_default_frpc(state: &AppState) -> Result<String, String> {
    if is_frpc_ready() {
        return Ok(format!("frpc 已就绪: {}", frpc_path().display()));
    }

    let dir = system_default_dir();
    ensure_dir(&dir)?;

    // 1. 加载设备凭证 + 创建 OnlineClient
    let creds = crate::utils::online_manager::load_creds_with_auto_refresh(state)
        .await
        .map_err(|e| format!("加载设备凭证失败: {}", e))?;
    let base_url = {
        let config = state.config.lock().await;
        config.online.api_server_url.clone()
    };
    let client = OnlineClient::new(&base_url);

    // 2. 构造 manifest 查询参数（apiServer 风格的 platform/arch）
    let (api_platform, api_arch) = api_server_platform_arch()?;
    // 本地已安装则上报真实版本（从 frpc_version.txt 读取），未安装则传 "0.0.0"
    // 强制 apiServer 返回最新版本下载链接
    // 注：apiServer 校验版本号格式（语义化版本），空字符串会返回 code=1001 错误，
    // 必须传 "0.0.0" 这种合法格式表示"查询最新版本"
    let current_version = read_frpc_version().unwrap_or_else(|| "0.0.0".to_string());
    let query = FrpManifestQuery {
        component: "client".to_string(),
        target: api_platform.to_string(),
        arch: api_arch.to_string(),
        current_version: current_version.clone(),
    };

    log_info!(
        "[Frp] 查询 apiServer frp manifest (current_version={})",
        if current_version.is_empty() { "(未安装)" } else { &current_version }
    );

    // 3. 调用 apiServer 获取 manifest
    let result = client
        .frp_get_manifest(&creds, &query)
        .await
        .map_err(|e| format!("查询 frp manifest 失败: {}", e))?;

    if result.code != 1 {
        return Err(format!(
            "查询 frp manifest 业务失败 [code={}]: {}",
            result.code, result.msg
        ));
    }

    let manifest: FrpManifest = result.data.ok_or_else(|| {
        "apiServer 未返回 frp 下载链接（本地已是最新但 frpc 二进制缺失，请删除 frpc_version.txt 后重试）".to_string()
    })?;

    log_info!(
        "[Frp] apiServer 返回最新 frpc 版本: {} (发布于 {})",
        manifest.version,
        manifest.pub_date
    );

    // 4. 通过 DownloadSession 下载 ZIP（复用项目下载基础设施，支持进度/暂停/取消）
    //
    // 参照 `commands/tools/download.rs` 的 `download_file` 模式：
    // - DownloadSession::start_grouped 初始化 stages + flag + manager
    // - 构造 DownloadTask，download_batch 执行下载
    // - 下载到临时 ZIP 文件，提取 frpc 后删除
    let zip_url = manifest.url.clone();
    let zip_path = dir.join("frpc_download.zip");
    log_info!("[Frp] 开始下载 frpc: {} -> {}", zip_url, zip_path.display());

    let session = crate::minecraft::download::DownloadSession::start_grouped(
        state,
        "frpc 下载",
        vec![("frpc 二进制", 1.0)],
    )
    .await;
    {
        let mut ds = state.download_state.lock().unwrap();
        ds.version_name = format!("frpc v{}", manifest.version);
    }

    let task = crate::minecraft::download::types::DownloadTask {
        id: "frpc_client".to_string(),
        urls: vec![zip_url.clone()],
        local_path: zip_path.to_string_lossy().to_string(),
        expected_size: 0,
        expected_hash: None,
    };

    let progress_callback = session.make_progress_callback(state, 0);
    let results = session
        .manager()
        .download_batch(vec![task], Some(progress_callback))
        .await;

    let result = results.first().ok_or("frpc 下载结果为空")?;
    use crate::minecraft::download::types::DownloadStatus;
    if result.status != DownloadStatus::Completed && result.status != DownloadStatus::Skipped {
        let err = result
            .error
            .clone()
            .unwrap_or_else(|| "未知错误".to_string());
        session.mark_failed(state, 1);
        // 清理半成品 ZIP
        let _ = std::fs::remove_file(&zip_path);
        return Err(format!("下载 frpc 失败: {}", err));
    }

    let zip_bytes = std::fs::read(&zip_path)
        .map_err(|e| {
            session.mark_failed(state, 1);
            format!("读取已下载 frpc ZIP 失败: {}", e)
        })?;

    log_info!("[Frp] frpc ZIP 下载完成，大小: {} 字节", zip_bytes.len());

    // 5. 从 ZIP 提取 frpc 二进制
    //
    // 兼容两种打包格式：
    // - GitHub Releases：`frp_<version>_<platform>_<arch>/frpc.exe`
    // - apiServer 分发：`frp_client_<version>_<platform>_<arch>/frpc.exe` 或扁平 `frpc.exe`
    //
    // 提取策略：在 ZIP 中查找路径以 `/frpc.exe` 或 `/frpc` 结尾的条目（顶层目录任意），
    // 或直接为 `frpc.exe` / `frpc` 的根级条目。选择路径最短的匹配（优先顶层）。
    let target_path = frpc_path();
    extract_frpc_from_zip(&zip_bytes, &target_path)?;

    // 6. 校验文件大小（防止下载截断/损坏）
    let metadata = std::fs::metadata(&target_path)
        .map_err(|e| format!("frpc 文件元数据读取失败: {}", e))?;
    if metadata.len() < 1024 {
        std::fs::remove_file(&target_path).ok();
        return Err("frpc 下载文件过小，可能已损坏".to_string());
    }

    // 7. 写入版本元数据文件（供 list_providers 展示 + 下次 manifest 查询使用）
    write_frpc_version(&manifest.version);
    log_info!(
        "[Frp] frpc 下载完成: {} (version={})",
        target_path.display(),
        manifest.version
    );
    Ok(format!("frpc 下载完成: {}", target_path.display()))
}

/// 外部厂商 frpc 下载（distribution=url）
///
/// 校验 URL HTTPS + 域名白名单 + SHA256（如有）。
/// 下载完成后若 archive=true，则解压到厂商目录。
async fn ensure_external_frpc(
    provider_id: &str,
    manifest: &ProviderManifest,
) -> Result<String, String> {
    let dl = manifest
        .binary
        .download
        .as_ref()
        .ok_or_else(|| format!("厂商 {} 缺少 binary.download 配置", provider_id))?;

    validate_download_url(&dl.url, &dl.allowed_domains)?;

    let provider_dir = providers_root().join(provider_id);
    let target_path = provider_dir.join(&dl.target_path);
    if let Some(parent) = target_path.parent() {
        ensure_dir(parent)?;
    }

    log_info!("[Frp] 开始下载外部厂商 frpc: {} ({})", provider_id, dl.url);

    // 构造禁止自动重定向的 client，手动校验重定向域名（防止重定向到非白名单域名）
    // 对应设计文档 §7.7 frpc 二进制下载安全
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|e| format!("构造 HTTP 客户端失败: {}", e))?;

    let mut current_url = dl.url.clone();
    let mut redirects = 0u32;
    const MAX_REDIRECTS: u32 = 5;
    let response = loop {
        let resp = client
            .get(&current_url)
            .send()
            .await
            .map_err(|e| format!("下载失败: {}", e))?;

        // 3xx 重定向：手动校验 Location 域名是否在白名单内
        if resp.status().is_redirection() {
            redirects += 1;
            if redirects > MAX_REDIRECTS {
                return Err("重定向次数超过限制".to_string());
            }
            let location = resp
                .headers()
                .get(reqwest::header::LOCATION)
                .and_then(|v| v.to_str().ok())
                .ok_or_else(|| "重定向响应缺少 Location 头".to_string())?;
            let base = reqwest::Url::parse(&current_url)
                .map_err(|e| format!("解析 URL 失败: {}", e))?;
            let next_url = base
                .join(location)
                .map_err(|e| format!("解析重定向 URL 失败: {}", e))?;
            let host = next_url.host_str().unwrap_or("");
            if !dl.allowed_domains.iter().any(|d| host_matches(host, d)) {
                return Err(format!("重定向域名 {} 不在白名单中", host));
            }
            log_info!("[Frp] 重定向到白名单域名: {}", next_url);
            current_url = next_url.to_string();
            continue;
        }
        break resp;
    };

    if !response.status().is_success() {
        return Err(format!("下载失败: HTTP {}", response.status()));
    }
    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("读取下载内容失败: {}", e))?;

    if let Some(ref expected_sha) = dl.sha256 {
        let actual = compute_sha256(&bytes);
        if actual != *expected_sha {
            return Err(format!(
                "SHA256 校验失败：期望 {}，实际 {}",
                expected_sha, actual
            ));
        }
    }

    std::fs::write(&target_path, &bytes)
        .map_err(|e| format!("写入文件失败: {}", e))?;

    if dl.archive {
        extract_archive(&target_path, &provider_dir)?;
    }

    log_info!("[Frp] 外部厂商 frpc 下载完成: {}", target_path.display());
    Ok(format!("frpc 下载完成: {}", target_path.display()))
}

// ============================================================
// 下载辅助函数
// ============================================================

/// 校验下载 URL：必须 HTTPS + 域名在白名单中
fn validate_download_url(url: &str, allowed_domains: &[String]) -> Result<(), String> {
    if !url.starts_with("https://") {
        return Err("下载 URL 必须使用 HTTPS".to_string());
    }
    let rest = &url[8..];
    let host_end = rest.find(|c| c == '/' || c == ':').unwrap_or(rest.len());
    let host = &rest[..host_end];
    if !allowed_domains.iter().any(|d| host_matches(host, d)) {
        return Err(format!("下载域名 {} 不在白名单中", host));
    }
    Ok(())
}

/// 域名白名单匹配，支持 `*.example.com` 一级通配符
fn host_matches(host: &str, pattern: &str) -> bool {
    if let Some(rest) = pattern.strip_prefix("*.") {
        host.ends_with(rest) && host.len() > rest.len()
    } else {
        host == pattern
    }
}

/// 计算 SHA256（十六进制小写）
fn compute_sha256(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

/// 从 ZIP 字节流提取 frpc 二进制到目标路径
///
/// 跨平台自探测：翻遍 ZIP 所有层级目录，匹配 basename 为 `frpc` / `frpc.exe`
/// 的非目录条目，按以下优先级选择最终条目：
/// 1. 当前平台首选名优先（Windows=frpc.exe，macOS/Linux=frpc），
///    兼容 apiServer 按平台返回 ZIP 的常态；
/// 2. 路径短优先（顶层目录 > 子目录），避免命中 `*/utils/frpc.exe` 等辅助文件。
///
/// 不会提取 LICENSE / frpc.toml / frpc.ini 等附加文件——basename 必须精确等于
/// `frpc` 或 `frpc.exe`，其他文件名一律跳过。
///
/// 兼容多种打包格式：
/// - GitHub Releases：`frp_<version>_<platform>_<arch>/frpc.exe`
/// - apiServer 分发：`frp_client_<version>_<platform>_<arch>/frpc.exe`
/// - 扁平打包：`frpc.exe` 直接位于 ZIP 根
/// - 任意嵌套层级：`some/deep/dir/frpc.exe`
fn extract_frpc_from_zip(zip_bytes: &[u8], target_path: &Path) -> Result<(), String> {
    let cursor = std::io::Cursor::new(&zip_bytes[..]);
    let mut archive = zip::ZipArchive::new(cursor)
        .map_err(|e| format!("解析 frpc ZIP 失败: {}", e))?;

    let preferred = frpc_filename(); // 当前平台期望名

    // 收集所有匹配条目：(索引, 路径, 是否为当前平台首选名)
    let mut candidates: Vec<(usize, String, bool)> = Vec::new();
    for i in 0..archive.len() {
        let file = archive
            .by_index(i)
            .map_err(|e| format!("读取 ZIP 条目失败: {}", e))?;
        let name = file.name().to_string();
        // 跳过目录条目
        if name.ends_with('/') {
            continue;
        }
        // 取路径最后一段作为文件名
        let basename = name.rsplit('/').next().unwrap_or(&name);
        // 精确匹配 frpc 或 frpc.exe（排除 frpc.toml / frpc.ini / frpc_full.ini 等）
        if basename == "frpc" || basename == "frpc.exe" {
            let is_preferred = basename == preferred;
            candidates.push((i, name, is_preferred));
        }
    }

    if candidates.is_empty() {
        return Err("ZIP 中未找到 frpc 二进制（期望文件名 frpc 或 frpc.exe）".to_string());
    }

    // 排序：首选名优先（is_preferred=true 排前），其次路径短优先（浅层目录）
    candidates.sort_by(|a, b| match b.2.cmp(&a.2) {
        std::cmp::Ordering::Equal => a.1.len().cmp(&b.1.len()),
        other => other,
    });
    let (best_idx, best_name, _) = &candidates[0];

    log_info!(
        "[Frp] 从 ZIP 提取: {}（共 {} 个候选，当前平台期望: {}）",
        best_name,
        candidates.len(),
        preferred
    );

    let mut file = archive
        .by_index(*best_idx)
        .map_err(|e| format!("读取 ZIP 条目失败: {}", e))?;

    if let Some(parent) = target_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("创建 frpc 目录失败: {}", e))?;
    }
    let mut out = std::fs::File::create(target_path)
        .map_err(|e| format!("创建 frpc 文件失败: {}", e))?;
    std::io::copy(&mut file, &mut out)
        .map_err(|e| format!("写入 frpc 文件失败: {}", e))?;

    Ok(())
}

/// 解压归档文件到目标目录（Zip Slip 防护）
fn extract_archive(archive_path: &Path, dst: &Path) -> Result<(), String> {
    let file = std::fs::File::open(archive_path)
        .map_err(|e| format!("打开归档失败: {}", e))?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| format!("解析归档失败: {}", e))?;
    let canonical_dst = dst.canonicalize().unwrap_or_else(|_| dst.to_path_buf());
    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| format!("读取 ZIP 条目失败: {}", e))?;
        let name = entry.name().to_string();
        if name.ends_with('/') {
            std::fs::create_dir_all(dst.join(&name))
                .map_err(|e| format!("创建目录失败: {}", e))?;
            continue;
        }
        let file_path = dst.join(&name);
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("创建父目录失败: {}", e))?;
            let canonical_parent = parent
                .canonicalize()
                .map_err(|e| format!("canonicalize 失败: {}", e))?;
            if !canonical_parent.starts_with(&canonical_dst) {
                return Err(format!("Zip Slip 检测: {}", name));
            }
        }
        let mut out = std::fs::File::create(&file_path)
            .map_err(|e| format!("创建文件失败: {}", e))?;
        std::io::copy(&mut entry, &mut out)
            .map_err(|e| format!("写入文件失败: {}", e))?;
    }
    Ok(())
}

// ============================================================
// 平台/架构标识
// ============================================================

/// apiServer 端期望的平台/架构字符串（用于 `/v1/frp/manifest` 查询参数）
///
/// - 平台：`windows` / `macos` / `linux`
/// - 架构：`x86_64` / `aarch64` / `i686` / `armv7`
pub(super) fn api_server_platform_arch() -> Result<(&'static str, &'static str), String> {
    let platform = if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else {
        return Err("不支持的操作系统".to_string());
    };

    let arch = if cfg!(target_arch = "x86_64") {
        "x86_64"
    } else if cfg!(target_arch = "x86") {
        "i686"
    } else if cfg!(target_arch = "aarch64") {
        "aarch64"
    } else if cfg!(target_arch = "arm") {
        "armv7"
    } else {
        return Err("不支持的 CPU 架构".to_string());
    };

    Ok((platform, arch))
}

/// 请求 apiServer 获取最新 frpc 版本号（不下载文件）
///
/// 用于 `list_providers` 在本地未安装 frpc 时显示云端最新版本号。
/// 查询参数 `current_version=0.0.0` 强制 apiServer 返回最新版本信息
/// （apiServer 校验语义化版本格式，空字符串会返回 code=1001 错误）。
///
/// 失败时返回 Err，调用方回退显示"未安装"。
pub(super) async fn fetch_latest_frpc_version(state: &AppState) -> Result<String, String> {
    let creds = crate::utils::online_manager::load_creds_with_auto_refresh(state)
        .await
        .map_err(|e| format!("加载设备凭证失败: {}", e))?;
    let base_url = {
        let config = state.config.lock().await;
        config.online.api_server_url.clone()
    };
    let client = OnlineClient::new(&base_url);

    let (api_platform, api_arch) = api_server_platform_arch()?;
    let query = FrpManifestQuery {
        component: "client".to_string(),
        target: api_platform.to_string(),
        arch: api_arch.to_string(),
        current_version: "0.0.0".to_string(),
    };

    let result = client
        .frp_get_manifest(&creds, &query)
        .await
        .map_err(|e| format!("查询 frp manifest 失败: {}", e))?;

    if result.code != 1 {
        return Err(format!(
            "查询 frp manifest 业务失败 [code={}]: {}",
            result.code, result.msg
        ));
    }

    result
        .data
        .map(|m| m.version)
        .ok_or_else(|| "apiServer 未返回版本信息".to_string())
}

/// frpc 二进制文件名（含扩展名）
fn frpc_filename() -> String {
    #[cfg(target_os = "windows")]
    {
        "frpc.exe".to_string()
    }
    #[cfg(not(target_os = "windows"))]
    {
        "frpc".to_string()
    }
}
