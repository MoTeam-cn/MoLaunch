//! 系统默认厂商 frpc 下载：从 apiServer `/v1/frp/manifest` 获取最新版本下载 URL，
//! 下载 ZIP 提取 frpc 二进制（替代早期 GitHub Releases 直链）。
//! 依赖 `provider.rs` 路径/manifest 函数；apiServer 调用复用 `OnlineClient` 与 `load_creds_with_auto_refresh`。

use super::super::ensure_dir;
use super::super::provider::{
    frpc_path, is_frpc_ready, read_frpc_version, system_default_dir, write_frpc_version,
};
use super::{api_server_platform_arch, archive};
use crate::log_info;
use crate::minecraft::online::client::OnlineClient;
use crate::minecraft::online::frp::{FrpManifest, FrpManifestQuery};
use crate::state::AppState;

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
pub(super) async fn ensure_system_default_frpc(state: &AppState) -> Result<String, String> {
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
        if current_version.is_empty() {
            "(未安装)"
        } else {
            &current_version
        }
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

    let zip_bytes = std::fs::read(&zip_path).map_err(|e| {
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
    archive::extract_frpc_from_zip(&zip_bytes, &target_path)?;

    // 提取成功后删除临时 ZIP 文件（注释承诺"提取 frpc 后删除"，原实现遗漏）
    let _ = std::fs::remove_file(&zip_path);

    // 6. 校验文件大小（防止下载截断/损坏）
    let metadata =
        std::fs::metadata(&target_path).map_err(|e| format!("frpc 文件元数据读取失败: {}", e))?;
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

/// 请求 apiServer 获取最新 frpc 版本号（不下载文件）
///
/// 用于 `list_providers` 在本地未安装 frpc 时显示云端最新版本号。
/// 查询参数 `current_version=0.0.0` 强制 apiServer 返回最新版本信息
/// （apiServer 校验语义化版本格式，空字符串会返回 code=1001 错误）。
///
/// 失败时返回 Err，调用方回退显示"未安装"。
pub async fn fetch_latest_frpc_version(state: &AppState) -> Result<String, String> {
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
