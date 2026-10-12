//! 检查更新：构造 URL、发起请求、解析 manifest、版本比较
//!
//! 走**需鉴权**的 `GET /v1/updates/manifest/raw`：JWT 提供 `device_pk`，
//! 服务端据此做按设备灰度。**不再有 GitHub Releases 回退**——灰度与发布控制
//! 都在服务端，回退等于让未注册或被限制的客户端绕开它们。

use serde_json::Value;
use tauri::AppHandle;

use super::UpdateInfo;
use crate::api_paths::UPDATES_CHECK;
use crate::log_info;
use crate::state::AppState;

/// 更新检查的请求超时
const UPDATE_CHECK_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

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
/// **鉴权**：先取有效设备凭证（过期自动续期），以 `Authorization: Bearer` 调用
/// `/v1/updates/manifest/raw`。未注册或续期失败即无法检查更新——这是刻意的，
/// 灰度与发布控制都在服务端，绕过鉴权就等于绕过它们。
///
/// **base_url**：从 `AppConfig.online.api_server_url` 读取（继承联机设置），不硬编码。
/// macOS/Linux 下载安装仍转发到官方 plugin（`download_and_install_unix`）。
pub async fn check_update(state: &AppState, _app: &AppHandle) -> Result<UpdateInfo, String> {
    let current_version = env!("CARGO_PKG_VERSION");

    // 1. 服务端地址与设备凭证（凭证过期时由 load_creds_with_auto_refresh 自动续期）
    let base_url = crate::commands::online::manager::read_api_server_url(state).await;
    let creds = crate::commands::online::manager::load_creds_with_auto_refresh(state)
        .await
        .map_err(|e| format!("检查更新失败：{e}"))?;
    if creds.device_token.is_empty() {
        return Err("检查更新失败：本地未持有设备凭证，请先在「设置 → 联机」连接云端".to_string());
    }

    // 2. 通道：由当前版本后缀推导（alpha/beta/stable），开发者模式可覆盖分支用于调试
    let channel = {
        let branch = crate::commands::system::developer::get_update_branch();
        if branch == "auto" {
            crate::utils::client_type::channel_name(current_version).to_string()
        } else {
            branch
        }
    };

    let target = platform_target();
    let arch = std::env::consts::ARCH;
    let path = UPDATES_CHECK
        .replace("{{target}}", target)
        .replace("{{arch}}", arch)
        .replace("{{current_version}}", current_version)
        .replace("{{channel}}", &channel);
    let url = format!("{}{}", base_url.trim_end_matches('/'), path);

    log_info!(
        "[Updater] 检查更新（需鉴权）: {}",
        crate::utils::net::sanitize_url_for_log(&url)
    );

    // 3. 发起请求：JWT 是灰度与发布控制的载体，必须携带
    let client = crate::http::get_client();
    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", creds.device_token))
        .timeout(UPDATE_CHECK_TIMEOUT)
        .send()
        .await
        .map_err(|e| {
            if crate::http::is_tls_cert_error(&e) {
                // TLS 证书异常疑似中间人，直接上报
                "检查更新失败：检测到中间人攻击，已自动断开链接".to_string()
            } else if e.is_timeout() {
                format!(
                    "检查更新失败：请求超时（{} 秒）",
                    UPDATE_CHECK_TIMEOUT.as_secs()
                )
            } else {
                format!("检查更新失败：网络错误: {e}")
            }
        })?;

    // 204/304 = 无更新
    if response.status() == 204 || response.status() == 304 {
        log_info!("[Updater] 服务器返回 {}（无可用更新）", response.status());
        return Ok(UpdateInfo::default());
    }

    if !response.status().is_success() {
        let status = response.status();
        return Err(match status.as_u16() {
            401 | 403 => {
                "检查更新失败：设备凭证无效或已失效，请在「设置 → 联机」重新连接云端".to_string()
            }
            _ => format!("检查更新失败: HTTP {status}"),
        });
    }

    let json: Value = response
        .json()
        .await
        .map_err(|e| format!("检查更新失败：解析更新信息失败: {e}"))?;

    // UnifiedResponse 包装：{ code, msg, data }
    let code = json.get("code").and_then(|v| v.as_i64()).unwrap_or(0);
    if code != 1 {
        log_info!("[Updater] 服务器返回 code={}（无可用更新）", code);
        return Ok(UpdateInfo::default());
    }
    // /v1/updates/manifest/raw 的 data 直接就是清单（不像旧 /v3 端点包一层 manifest）
    let Some(manifest) = json.get("data").filter(|v| !v.is_null()) else {
        log_info!("[Updater] 服务器返回 data=null（无可用更新）");
        return Ok(UpdateInfo::default());
    };

    let version = manifest
        .get("version")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    if version.is_empty() {
        log_info!("[Updater] manifest 无 version 字段（无可用更新）");
        return Ok(UpdateInfo::default());
    }

    // 版本比较：manifest 版本必须大于当前版本才算有更新
    if !is_version_newer(version, current_version) {
        log_info!("[Updater] 当前版本 {} 已是最新", current_version);
        return Ok(UpdateInfo::default());
    }

    log_info!("[Updater] 发现新版本: {} -> {}", current_version, version);

    let download_url = manifest
        .get("url")
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
        notes: manifest
            .get("notes")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        force_update: manifest
            .get("force_update")
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
        download_url,
        signature: manifest
            .get("signature")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
    })
}
