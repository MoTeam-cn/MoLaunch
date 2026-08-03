//! 服务器元数据获取与 authlib-injector.jar 下载/校验/预取

use super::types::{AuthlibInjectorMeta, YggdrasilError};
use super::{join_url, parse_error};
use crate::minecraft::auth::authlib::types::ServerMetadata;
use crate::minecraft::sources::{
    authlib_injector_meta_url_mirror, authlib_injector_meta_url_official,
};

/// authlib-injector.jar 在缓存目录的相对路径
///
/// 与 `launch/jvm_args.rs::add_authlib_args` 中常量保持一致。
const AUTHLIB_INJECTOR_JAR_REL: &str = "launch/authlib-injector.jar";

/// GET / 获取服务器元数据
///
/// 用于：
/// 1. 登录页显示服务器名、注册链接
/// 2. 启动游戏时生成 `-Dauthlibinjector.yggdrasil.prefetched` 参数
pub async fn fetch_server_metadata(server_url: &str) -> Result<ServerMetadata, YggdrasilError> {
    let url = join_url(server_url, "");
    // 通过 http.rs 统一入口发起 GET，保留状态码以便区分 200 与错误响应
    let (status, text) =
        crate::http::get_text_with_status(&url)
            .await
            .map_err(|e| YggdrasilError {
                status: 0,
                message: e.to_string(),
                is_network: true,
            })?;
    if status != 200 {
        return Err(parse_error(status, text));
    }
    serde_json::from_str::<ServerMetadata>(&text).map_err(|e| YggdrasilError {
        status: 0,
        message: format!("解析服务器元数据失败: {}", e),
        is_network: false,
    })
}

/// 下载 authlib-injector.jar 元数据
///
/// 从官方源 `authlib-injector.yushi.moe` 获取最新版本元数据，
/// 包含 `download_url` 和 `checksums.sha256`。
/// BMCLAPI 作为镜像备用源。
///
/// URL 常量统一在 `minecraft::sources` 模块定义，请求走 `crate::http` 模块。
pub async fn fetch_authlib_injector_meta() -> Result<AuthlibInjectorMeta, String> {
    let primary = authlib_injector_meta_url_official();
    let mirror = authlib_injector_meta_url_mirror();

    let text = match crate::http::fetch_url(&primary).await {
        Ok(t) => t,
        Err(e) => {
            crate::log_info!("[Authlib] 官方源失败，尝试 BMCLAPI 镜像: {}", e);
            crate::http::fetch_url(&mirror)
                .await
                .map_err(|e| format!("获取 authlib-injector 元数据失败: {}", e))?
        }
    };

    let json: serde_json::Value = serde_json::from_str(&text)
        .map_err(|e| format!("解析 authlib-injector 元数据失败: {}", e))?;

    let download_url = json["download_url"]
        .as_str()
        .ok_or("元数据缺少 download_url")?
        .to_string();
    let sha256 = json["checksums"]["sha256"]
        .as_str()
        .ok_or("元数据缺少 checksums.sha256")?
        .to_string();

    Ok(AuthlibInjectorMeta {
        download_url,
        sha256,
    })
}

/// 确保 authlib-injector.jar 已下载到缓存目录（启动游戏前调用，仅当 `auth_info.server_url` 有值时）
///
/// 流程：缓存命中直接返回 → 未命中则 fetch_authlib_injector_meta 获取 URL 和 sha256 → 下载
/// 二进制 → 校验 sha256 → 写入缓存。同时可选预取服务器元数据（base64 缓存供
/// `-Dauthlibinjector.yggdrasil.prefetched` 参数使用）。
/// 失败时不阻塞启动（返回 Err 由调用方决定，无外置登录也能进游戏，仅角色/皮肤异常）。
/// 阶段 5 改造：下载从 `http::fetch_bytes` 改为 `DownloadManager::download_batch`（限速/fallback/进度）；
/// sha256 校验手动实现（DownloadManager 用 sha1，与 authlib 的 sha256 不兼容）。
pub async fn ensure_authlib_injector_jar(
    server_url: Option<&str>,
    manager: &crate::minecraft::download::DownloadManager,
) -> Result<std::path::PathBuf, String> {
    // 1. 缓存命中
    if crate::utils::cache::exists(AUTHLIB_INJECTOR_JAR_REL) {
        crate::log_debug!("[Authlib] authlib-injector.jar 缓存命中");
        if let Some(url) = server_url {
            prefetch_metadata_if_missing(url).await;
        }
        return Ok(crate::utils::cache::path(AUTHLIB_INJECTOR_JAR_REL));
    }

    // 2. 获取元数据
    let meta = fetch_authlib_injector_meta().await?;
    crate::log_info!(
        "[Authlib] 准备下载 authlib-injector.jar: url={}, sha256={}",
        meta.download_url,
        &meta.sha256[..8]
    );

    // 3. 通过 DownloadManager 下载到缓存路径（统一限速/URL fallback/进度推送）
    use crate::minecraft::download::types::{DownloadStatus, DownloadTask};
    let target_path = crate::utils::cache::path(AUTHLIB_INJECTOR_JAR_REL);
    // 确保父目录存在（DownloadManager 取消下载时需创建父目录，避免 os error 2）
    if let Some(parent) = target_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let task = DownloadTask {
        id: "authlib_injector".to_string(),
        urls: vec![meta.download_url.clone()],
        local_path: target_path.to_string_lossy().to_string(),
        expected_size: 0,    // 不校验 size，下载后手动 sha256 校验
        expected_hash: None, // 不校验 hash，下载后手动 sha256 校验（sha256 与 DownloadManager 的 sha1 不兼容）
    };
    let results = manager.download_batch(vec![task], None).await;
    let result = results.first().ok_or("下载结果为空")?;
    if result.status != DownloadStatus::Completed && result.status != DownloadStatus::Skipped {
        let err = result
            .error
            .clone()
            .unwrap_or_else(|| "未知错误".to_string());
        return Err(format!("下载 authlib-injector.jar 失败: {}", err));
    }

    // 4. 读取下载的文件 + 校验 sha256
    let bytes = tokio::fs::read(&target_path)
        .await
        .map_err(|e| format!("读取下载的 authlib-injector.jar 失败: {}", e))?;
    let actual_sha = crate::utils::hash::sha256_hex(&bytes);
    if actual_sha != meta.sha256 {
        // 校验失败：删除损坏的文件
        let _ = std::fs::remove_file(&target_path);
        return Err(format!(
            "authlib-injector.jar sha256 校验失败：期望 {}，实际 {}",
            meta.sha256,
            &actual_sha[..actual_sha.len().min(8)]
        ));
    }

    crate::log_info!(
        "[Authlib] authlib-injector.jar 下载完成 (sha256={})",
        &meta.sha256[..8]
    );

    // 5. 预取服务器元数据
    if let Some(url) = server_url {
        prefetch_metadata_if_missing(url).await;
    }

    Ok(target_path)
}

/// 预取服务器元数据并缓存（base64 编码），若对应 host 的缓存已存在则跳过
///
/// 缓存路径：`launch/authlib-prefetched-<host>.txt`
/// 失败时仅打印警告，不阻塞启动（authlib-injector 会在游戏运行时自行拉取）
async fn prefetch_metadata_if_missing(server_url: &str) {
    let host = match extract_host_for_cache(server_url) {
        Some(h) => h,
        None => return,
    };
    let rel = format!("launch/authlib-prefetched-{}.txt", host);
    if crate::utils::cache::exists(&rel) {
        return; // 已缓存
    }

    match fetch_server_metadata(server_url).await {
        Ok(meta) => {
            // 序列化为 JSON 后 base64 编码（authlib-injector 规范要求）
            let json = match serde_json::to_string(&meta) {
                Ok(s) => s,
                Err(e) => {
                    crate::log_warn!("[Authlib] 序列化服务器元数据失败: {}", e);
                    return;
                }
            };
            let b64 = base64_encode(json.as_bytes());
            if let Err(e) = crate::utils::cache::write(&rel, &b64) {
                crate::log_warn!("[Authlib] 缓存服务器元数据失败: {}", e);
            }
        }
        Err(e) => {
            crate::log_warn!(
                "[Authlib] 预取服务器元数据失败（游戏运行时将自行拉取）: {}",
                e
            );
        }
    }
}

/// base64 标准编码（不含换行）
fn base64_encode(data: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(data)
}

/// 从 server_url 提取 host，用作缓存文件名的安全标识
///
/// 仅保留字母、数字、点、连字符，其他字符替换为 `_`。
/// 与 `launch/jvm_args.rs::extract_host` 保持一致实现。
fn extract_host_for_cache(server_url: &str) -> Option<String> {
    let after_scheme = server_url
        .strip_prefix("https://")
        .or_else(|| server_url.strip_prefix("http://"))
        .unwrap_or(server_url);
    let host_part = after_scheme.split('/').next()?;
    let sanitized: String = host_part
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '.' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect();
    if sanitized.is_empty() {
        None
    } else {
        Some(sanitized)
    }
}
