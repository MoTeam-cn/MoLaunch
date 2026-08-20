//! 系统默认厂商 frpc 下载：从 GitHub API（fatedier/frp releases）获取最新版本，
//! 镜像竞速优先 + 官方保底下载压缩包，提取 frpc 二进制（Windows zip / macOS·Linux tar.gz）。
//! 无需登录联机账号，版本随上游自动同步（复用 easytier 的 GitHub 下载公共组件）。
//! 依赖 `provider.rs` 路径/版本函数；下载编排复用 DownloadSession（支持进度/暂停/取消）。

use super::super::ensure_dir;
use super::super::provider::{frpc_path, is_frpc_ready, system_default_dir, write_frpc_version};
use super::archive;
use crate::log_info;
use crate::state::AppState;
use crate::utils::github_download::{build_proxy_url, fetch_latest_release};

/// frp 官方 GitHub 仓库（版本随上游自动同步）
const FRP_REPO: &str = "fatedier/frp";

/// 构造 frp 官方 release 资产名（`frp_{version}_{os}_{arch}.{ext}`）
///
/// os：frp 用 `darwin` 表示 macOS（区别于 easytier 的 `macos`）；
/// arch：`amd64` / `arm64` / `386` / `arm`；
/// 格式：Windows 为 zip，macOS/Linux 为 tar.gz。
/// 返回 (资产名, 扩展名)。
fn frp_asset_name(version: &str) -> Result<(String, &'static str), String> {
    let os = if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "darwin"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else {
        return Err("不支持的操作系统".to_string());
    };
    let arch = if cfg!(target_arch = "x86_64") {
        "amd64"
    } else if cfg!(target_arch = "x86") {
        "386"
    } else if cfg!(target_arch = "aarch64") {
        "arm64"
    } else if cfg!(target_arch = "arm") {
        "arm"
    } else {
        return Err("不支持的 CPU 架构".to_string());
    };
    let ext = if cfg!(target_os = "windows") {
        "zip"
    } else {
        "tar.gz"
    };
    Ok((format!("frp_{version}_{os}_{arch}.{ext}"), ext))
}

/// 系统默认厂商 frpc 下载
///
/// 流程：
/// 1. 已就绪 → 直接返回
/// 2. GitHub API 双源（主 `api.github.com` / 备选 `github-api.mocdn.net`）查询最新版本
/// 3. 构造资产名与下载 URL：镜像竞速选最快（`github_proxies` 由前端启动时测速筛选），官方保底
/// 4. 下载压缩包 → 按平台提取 frpc 二进制（zip / tar.gz）→ 写入 frpc_path()
///
/// 失败时不保留半成品文件，避免下次误判为就绪。
pub(super) async fn ensure_system_default_frpc(state: &AppState) -> Result<String, String> {
    if is_frpc_ready() {
        return Ok(format!("frpc 已就绪: {}", frpc_path().display()));
    }

    let dir = system_default_dir();
    ensure_dir(&dir)?;

    // 1. GitHub API 查询最新版本（无需登录，主源失败自动回退备选源）
    let version = fetch_latest_release(&crate::http::get_client(), FRP_REPO)
        .await
        .map_err(|e| format!("查询 frp 最新版本失败: {}", e))?;
    let (asset, ext) = frp_asset_name(&version)?;
    log_info!("[Frp] GitHub 返回最新 frpc 版本: {}", version);

    // 2. 构造下载 URL：镜像竞速选最快（github_proxies 由前端启动时测速筛选），官方保底
    let mut urls: Vec<String> = Vec::new();
    let proxies = state.github_proxies.lock().await.clone();
    if !proxies.is_empty() {
        let candidates: Vec<String> = proxies
            .iter()
            .map(|p| build_proxy_url(p, FRP_REPO, &version, &asset))
            .collect();
        crate::log_debug!("[Frp] 镜像竞速候选: {:?}", candidates);
        if let Ok(fastest) = crate::utils::probe::pick_fastest(&candidates, None).await {
            urls.push(fastest);
        }
    }
    urls.push(format!(
        "https://github.com/{FRP_REPO}/releases/download/v{version}/{asset}"
    ));

    // 3. 通过 DownloadSession 下载压缩包（复用项目下载基础设施，支持进度/暂停/取消）
    //
    // 参照 `commands/tools/download.rs` 的 `download_file` 模式：
    // - DownloadSession::start_grouped 初始化 stages + flag + manager
    // - 构造 DownloadTask（urls 顺序回退：镜像失败自动切官方），download_batch 执行下载
    // - 下载到临时压缩包，提取 frpc 后删除
    //
    // silent=true：frpc 属后台组件补全（同 Java / 更新程序），
    // 前端 ProviderList 按钮有独立 loading 状态，不弹下载面板
    let archive_path = dir.join(format!("frpc_download.{ext}"));
    log_info!(
        "[Frp] 开始下载 frpc: {} -> {}",
        urls[0],
        archive_path.display()
    );

    let session = crate::minecraft::download::DownloadSession::start_grouped(
        state,
        "frpc 下载",
        vec![("frpc 二进制", 1.0)],
        true,
    )
    .await;
    {
        let mut ds = state.download_state.lock().unwrap();
        ds.version_name = format!("frpc v{}", version);
    }

    let task = crate::minecraft::download::types::DownloadTask {
        id: "frpc_client".to_string(),
        urls,
        local_path: archive_path.to_string_lossy().to_string(),
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
        // 清理半成品压缩包
        let _ = std::fs::remove_file(&archive_path);
        return Err(format!("下载 frpc 失败: {}", err));
    }

    let bytes = std::fs::read(&archive_path).map_err(|e| {
        session.mark_failed(state, 1);
        format!("读取已下载 frpc 压缩包失败: {}", e)
    })?;
    log_info!("[Frp] frpc 压缩包下载完成，大小: {} 字节", bytes.len());

    // 4. 按平台提取 frpc 二进制（Windows zip / macOS·Linux tar.gz）
    //
    // 兼容两种打包格式：
    // - GitHub Releases：`frp_<version>_<platform>_<arch>/frpc`（或 `frpc.exe`）
    //
    // 提取策略：在归档中查找路径以 `/frpc` 或 `/frpc.exe` 结尾的条目（顶层目录任意），
    // 或直接为 `frpc` / `frpc.exe` 的根级条目。选择路径最短的匹配（优先顶层）。
    let target_path = frpc_path();
    if ext == "zip" {
        archive::extract_frpc_from_zip(&bytes, &target_path)?;
    } else {
        archive::extract_frpc_from_tar_gz(&bytes, &target_path)?;
    }

    // Unix 下 tar.gz 提取不保留执行位，需补 +x
    #[cfg(unix)]
    crate::minecraft::system::shell::make_executable(&target_path);

    // 提取成功后删除临时压缩包
    let _ = std::fs::remove_file(&archive_path);

    // 标记整体完成：start_grouped 已 reset_stages 置 is_active=true，
    // 若不 mark_complete 会残留 is_active，导致重启后 isDownloading 恢复误判下载面板
    session.mark_complete(state);

    // 5. 校验文件大小（防止下载截断/损坏）
    let metadata =
        std::fs::metadata(&target_path).map_err(|e| format!("frpc 文件元数据读取失败: {}", e))?;
    if metadata.len() < 1024 {
        std::fs::remove_file(&target_path).ok();
        return Err("frpc 下载文件过小，可能已损坏".to_string());
    }

    // 6. 写入版本元数据文件（供 list_providers 展示）
    write_frpc_version(&version);
    log_info!(
        "[Frp] frpc 下载完成: {} (version={})",
        target_path.display(),
        version
    );
    Ok(format!("frpc 下载完成: {}", target_path.display()))
}

/// 查询最新 frpc 版本号（不下载文件）
///
/// 用于 `list_providers` 在本地未安装 frpc 时显示云端最新版本号。
/// 无需登录，GitHub API 双源查询（主源失败自动回退备选源）；失败时调用方回退显示"未安装"。
pub async fn fetch_latest_frpc_version() -> Result<String, String> {
    fetch_latest_release(&crate::http::get_client(), FRP_REPO).await
}
