//! 检查更新：构造 URL、发起请求、解析 manifest、版本比较

use serde_json::Value;
use tauri::AppHandle;

use super::UpdateInfo;
use crate::api_paths::UPDATES_MANIFEST_RAW;
use crate::state::AppState;

/// 当前平台标识（用于 updater endpoint 的 target 参数）
///
/// 服务端仅接受 `windows` / `macos` / `linux`，架构信息由 `arch` 查询参数单独传递。
fn platform_target() -> &'static str {
    std::env::consts::OS
}

/// 语义化版本比较：manifest_version 高于 current_version 返回 true
/// 支持 pre-release 段（rc/beta/alpha 等），如 0.3.5-rc7 > 0.3.5-rc6
fn is_version_newer(manifest: &str, current: &str) -> bool {
    use std::cmp::Ordering;

    /// pre-release 标识符段：纯数字按数值，字母段按前缀+数字尾比较
    enum PrePart {
        Num(u64),
        Word(String, Option<u64>),
    }

    impl PrePart {
        fn new(s: &str) -> Self {
            if let Ok(n) = s.parse() {
                return Self::Num(n);
            }
            let digits_at = s.trim_end_matches(|c: char| c.is_ascii_digit()).len();
            Self::Word(s[..digits_at].to_string(), s[digits_at..].parse().ok())
        }

        fn cmp(&self, other: &Self) -> Ordering {
            match (self, other) {
                (Self::Num(a), Self::Num(b)) => a.cmp(b),
                (Self::Num(_), Self::Word(..)) => Ordering::Less,
                (Self::Word(..), Self::Num(_)) => Ordering::Greater,
                (Self::Word(a, an), Self::Word(b, bn)) => a.cmp(b).then_with(|| an.cmp(bn)),
            }
        }
    }

    fn parse(s: &str) -> Option<(u64, u64, u64, Vec<PrePart>)> {
        let s = s.trim_start_matches('v');
        let (core, pre) = s.split_once('-').map_or((s, ""), |(c, p)| (c, p));
        let mut parts = core.split('.');
        let major = parts.next()?.parse().ok()?;
        let minor = parts.next()?.parse().ok()?;
        let patch = parts.next()?.parse().ok()?;
        let pre = if pre.is_empty() {
            Vec::new()
        } else {
            pre.split('.').map(PrePart::new).collect()
        };
        Some((major, minor, patch, pre))
    }

    fn cmp_pre(a: &[PrePart], b: &[PrePart]) -> Ordering {
        for (x, y) in a.iter().zip(b) {
            let o = x.cmp(y);
            if o != Ordering::Equal {
                return o;
            }
        }
        a.len().cmp(&b.len())
    }

    match (parse(manifest), parse(current)) {
        (Some(m), Some(c)) => {
            let ord = m.0.cmp(&c.0).then(m.1.cmp(&c.1)).then(m.2.cmp(&c.2));
            let ord = if ord == Ordering::Equal {
                match (m.3.is_empty(), c.3.is_empty()) {
                    (true, true) => Ordering::Equal,
                    (true, false) => Ordering::Greater,
                    (false, true) => Ordering::Less,
                    (false, false) => cmp_pre(&m.3, &c.3),
                }
            } else {
                ord
            };
            ord == Ordering::Greater
        }
        _ => manifest != current,
    }
}

/// 检查更新（所有平台统一入口）
///
/// 使用 `crate::http::get_client()` 发起请求（走用户配置的代理），
/// 手动解析 manifest JSON 并比较版本，不依赖 tauri-plugin-updater 的内部 HTTP 客户端。
///
/// **base_url**：从 `AppConfig.online.api_server_url` 读取（继承联机设置），不再硬编码。
/// **鉴权**：v1 raw 端点需要 JWT，尝试加载设备 JWT 携带 `Authorization: Bearer` 头；
/// 未注册设备时无 auth 请求（服务端 raw 端点对未鉴权请求返回空 manifest）。
/// macOS/Linux 下载安装仍转发到官方 plugin（`download_and_install_unix`，走 v3 端点）。
pub async fn check_update(state: &AppState, _app: &AppHandle) -> Result<UpdateInfo, String> {
    let current_version = env!("CARGO_PKG_VERSION");
    let target = platform_target();
    let arch = std::env::consts::ARCH;

    // 1. 从配置读取 api_server_url（短锁，立即 clone 释放）
    let base_url = {
        let config = state.config.lock().await;
        config.online.api_server_url.clone()
    };

    // 2. 构建 URL（base_url + 路径模板替换，模板见 crate::api_paths::UPDATES_MANIFEST_RAW）
    // channel 由当前版本后缀自动推导（alpha/beta/stable）；开发者模式可覆盖分支用于调试
    let channel = {
        let branch = crate::commands::system::developer::get_update_branch();
        if branch == "auto" {
            crate::utils::client_type::channel_name(current_version).to_string()
        } else {
            branch
        }
    };
    let path = UPDATES_MANIFEST_RAW
        .replace("{{target}}", target)
        .replace("{{arch}}", arch)
        .replace("{{current_version}}", current_version)
        .replace("{{channel}}", &channel);
    let url = format!("{}{}", base_url.trim_end_matches('/'), path);

    // 3. 尝试加载设备 JWT（未注册时忽略，无 auth 请求）
    let jwt = crate::commands::online::manager::load_creds_with_auto_refresh(state)
        .await
        .ok()
        .map(|creds| creds.device_token);

    log::info!(
        "[Updater] 检查更新: {} (auth: {})",
        crate::utils::net::sanitize_url_for_log(&url),
        jwt.is_some()
    );

    // 4. 构建请求（有 JWT 则携带 Authorization 头）
    let client = crate::http::get_client();
    let mut req_builder = client.get(&url);
    if let Some(ref token) = jwt {
        req_builder = req_builder.header("Authorization", format!("Bearer {}", token));
    }
    let response = req_builder.send().await.map_err(|e| {
        if crate::http::is_tls_cert_error(&e) {
            "检测到中间人攻击，已自动断开链接".to_string()
        } else {
            format!("检查更新失败: {e}")
        }
    })?;

    // 204/304 = 无更新
    if response.status() == 204 || response.status() == 304 {
        log::info!("[Updater] 服务器返回 {}（无可用更新）", response.status());
        return Ok(UpdateInfo::default());
    }

    if !response.status().is_success() {
        return Err(format!("检查更新失败: HTTP {}", response.status()));
    }

    let json: Value = response
        .json()
        .await
        .map_err(|e| format!("解析更新信息失败: {e}"))?;

    // UnifiedResponse 包装：{ code, msg, data }（v1 raw/manifest 端点统一格式）
    let code = json.get("code").and_then(|v| v.as_i64()).unwrap_or(0);
    if code != 1 {
        log::info!("[Updater] 服务器返回 code={}（无可用更新）", code);
        return Ok(UpdateInfo::default());
    }
    let data = json.get("data");

    let version = data
        .and_then(|d| d.get("version"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    if version.is_empty() {
        log::info!("[Updater] manifest 无 version 字段（无可用更新）");
        return Ok(UpdateInfo::default());
    }

    // 版本比较：manifest 版本必须大于当前版本才算有更新
    if !is_version_newer(version, current_version) {
        log::info!("[Updater] 当前版本 {} 已是最新", current_version);
        return Ok(UpdateInfo::default());
    }

    log::info!("[Updater] 发现新版本: {} -> {}", current_version, version);

    let download_url = data
        .and_then(|d| d.get("url"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    // 域名白名单校验（防 manifest 被篡改后指向任意地址）
    if !download_url.is_empty() {
        crate::utils::net::validate_updater_download_url(&download_url)?;
    }

    Ok(UpdateInfo {
        available: true,
        version: version.to_string(),
        notes: data
            .and_then(|d| d.get("notes"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        force_update: data
            .and_then(|d| d.get("force_update"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
        download_url,
        signature: data
            .and_then(|d| d.get("signature"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
    })
}

#[cfg(test)]
#[path = "check_test.rs"]
mod tests;
