//! 下载设置相关命令

use crate::log_info;
use crate::state::AppState;
use tauri::State;

/// 校验镜像 URL，防止 SSRF（不引入 url crate，用字符串匹配）
fn validate_mirror_url(url: &str) -> Result<(), String> {
    // 必须是 https:// 或 http:// 开头
    if !url.starts_with("https://") && !url.starts_with("http://") {
        return Err("镜像 URL 必须以 http:// 或 https:// 开头".to_string());
    }
    // 去掉 scheme，提取主机部分
    let after_scheme = if let Some(rest) = url.strip_prefix("https://") {
        rest
    } else {
        url.strip_prefix("http://").unwrap_or(url)
    };
    // 处理 userinfo@host 情况，取最后一个 @ 之后的部分
    let host_part = after_scheme.split('@').last().unwrap_or(after_scheme);
    // 主机到第一个 / : ? # 之前结束
    let host_end = host_part
        .find(|c| c == '/' || c == ':' || c == '?' || c == '#')
        .unwrap_or(host_part.len());
    let host = &host_part[..host_end];
    // 去掉 IPv6 方括号
    let host = host.trim_start_matches('[').trim_end_matches(']');

    if host.is_empty() {
        return Err("镜像 URL 主机不能为空".to_string());
    }
    // 拒绝环回地址
    if host == "localhost" || host == "127.0.0.1" || host == "::1" {
        return Err("镜像 URL 不能指向环回地址".to_string());
    }
    // 拒绝链路本地地址
    if host.starts_with("169.254.") {
        return Err("镜像 URL 不能指向链路本地地址".to_string());
    }
    // 拒绝私有网段（基本检查）
    if host.starts_with("10.") || host.starts_with("192.168.") {
        return Err("镜像 URL 不能指向私有网络地址".to_string());
    }
    if host.starts_with("172.") {
        if let Some(second) = host.split('.').nth(1) {
            if let Ok(n) = second.parse::<u32>() {
                if n >= 16 && n <= 31 {
                    return Err("镜像 URL 不能指向私有网络地址".to_string());
                }
            }
        }
    }
    Ok(())
}

/// 设置镜像源
#[tauri::command]
pub async fn set_mirror_url(
    state: State<'_, AppState>,
    mirror_url: Option<String>,
    _skip_reinit: Option<bool>,
) -> Result<(), String> {
    // SSRF 防护：校验镜像 URL
    if let Some(ref url) = mirror_url {
        validate_mirror_url(url)?;
    }
    log_info!("Mirror URL changed to: {:?}", mirror_url);
    super::update_config(&state, |config| {
        config.mirror_url = mirror_url;
    })
    .await
}

/// 获取镜像源
#[tauri::command]
pub async fn get_mirror_url(state: State<'_, AppState>) -> Result<Option<String>, String> {
    let config = state.config.lock().await;
    Ok(config.mirror_url.clone())
}

/// 设置下载源模式
#[tauri::command]
pub async fn set_download_source(
    state: State<'_, AppState>,
    source: String,
    _skip_reinit: Option<bool>,
) -> Result<(), String> {
    let bmclapi = crate::minecraft::sources::BMCLAPI_BASE;

    log_info!("Download source changed to: {}", source);
    super::update_config(&state, |config| {
        match source.as_str() {
            "mirror" => {
                config.mirror_url_meta = Some(bmclapi.to_string());
                config.mirror_url_download = Some(bmclapi.to_string());
                config.mirror_url = Some(bmclapi.to_string());
                config.mirror_mode = 0;
            }
            "official" => {
                config.mirror_url_meta = None;
                config.mirror_url_download = None;
                config.mirror_url = None;
                config.mirror_mode = 0;
            }
            "smart" => {
                config.mirror_url_meta = None;
                config.mirror_url_download = None;
                config.mirror_url = None;
                config.mirror_mode = 1;
            }
            _ => {}
        }
        config.download_source = source;
    })
    .await
}

/// 获取下载源模式
#[tauri::command]
pub async fn get_download_source(state: State<'_, AppState>) -> Result<String, String> {
    let config = state.config.lock().await;
    Ok(config.download_source.clone())
}

/// 设置最大下载速度
#[tauri::command]
pub async fn set_max_download_speed(
    state: State<'_, AppState>,
    speed: u64,
    _skip_reinit: Option<bool>,
) -> Result<(), String> {
    log_info!("Max download speed changed to: {} bytes/sec", speed);
    super::update_config(&state, |config| {
        config.max_download_speed = speed;
    })
    .await
}

/// 获取最大下载速度
#[tauri::command]
pub async fn get_max_download_speed(state: State<'_, AppState>) -> Result<u64, String> {
    let config = state.config.lock().await;
    Ok(config.max_download_speed)
}

/// 设置下载线程数
#[tauri::command]
pub async fn set_max_download_threads(
    state: State<'_, AppState>,
    threads: u32,
) -> Result<(), String> {
    log_info!("Max download threads changed to: {}", threads);
    super::update_config(&state, |config| {
        config.max_download_threads = threads;
    })
    .await
}

/// 获取下载线程数
#[tauri::command]
pub async fn get_max_download_threads(state: State<'_, AppState>) -> Result<u32, String> {
    let config = state.config.lock().await;
    Ok(config.max_download_threads)
}

/// 设置分片数量
#[tauri::command]
pub async fn set_chunk_count(state: State<'_, AppState>, count: u32) -> Result<(), String> {
    log_info!("Chunk count changed to: {}", count);
    super::update_config(&state, |config| {
        config.chunk_count = count;
    })
    .await
}

/// 获取分片数量
#[tauri::command]
pub async fn get_chunk_count(state: State<'_, AppState>) -> Result<u32, String> {
    let config = state.config.lock().await;
    Ok(config.chunk_count)
}
