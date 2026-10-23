//! 检查更新：构造 URL、发起请求、解析 manifest、版本比较
//!
//! 主通道走**需鉴权**的 `GET /v1/updates/manifest/raw`：JWT 提供 `device_pk`，
//! 服务端据此做按设备灰度。开发者中心可把检查更新源切到 GitHub 清单
//! （`latest.json` 静态资产，见 `github` 模块），或设为 auto 在云端失败时降级。

use serde_json::Value;
use tauri::AppHandle;

use super::UpdateInfo;
use crate::api_paths::UPDATES_CHECK;
use crate::log_info;
use crate::log_warn;
use crate::state::AppState;

/// 更新检查的请求超时
const UPDATE_CHECK_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

/// 当前平台标识（用于 updater endpoint 的 target 参数）
///
/// 服务端仅接受 `windows` / `macos` / `linux`，架构信息由 `arch` 查询参数单独传递。
fn platform_target() -> &'static str {
    std::env::consts::OS
}
/// 判断 manifest 版本是否高于 current 版本
///
/// 支持 pre-release 段（rc/beta/alpha 等），如 0.3.5-rc7 > 0.3.5-rc6。
/// **渠道词与其后的数字会先归一**：`rc.4` 与 `rc4` 等价、`beta.2` 与 `beta2` 等价，
/// 数字按数值比较（`rc.10 > rc.9`）。历史版本号带点与不带点两种写法并存
/// （`v0.3.8-rc1` 与 `v0.3.8-rc.3`），不归一会让 `rc2` 被判为高于 `rc.4`。
pub(super) fn is_version_newer(manifest: &str, current: &str) -> bool {
    use std::cmp::Ordering;

    /// 预发布标识符段：渠道词 + 可选数字
    ///
    /// `rc4` 与 `rc.4` 都归一为 `("rc", Some(4))`，从而落在同一把尺子上。
    struct PreSegment {
        word: String,
        num: Option<u64>,
    }

    /// 预发布字符串切段并归一；纯数字段并入前一个尚无数字的段
    fn parse_pre(raw: &str) -> Option<Vec<PreSegment>> {
        let mut out: Vec<PreSegment> = Vec::new();
        for seg in raw.split(['.', '-']).filter(|s| !s.is_empty()) {
            if !seg.bytes().all(|b| b.is_ascii_alphanumeric()) {
                return None;
            }
            if seg.bytes().all(|b| b.is_ascii_digit()) {
                let value: u64 = seg.parse().ok()?;
                match out.last_mut() {
                    Some(last) if last.num.is_none() => last.num = Some(value),
                    _ => out.push(PreSegment {
                        word: String::new(),
                        num: Some(value),
                    }),
                }
                continue;
            }
            let word_len = seg.trim_end_matches(|c: char| c.is_ascii_digit()).len();
            let (word, digits) = seg.split_at(word_len);
            out.push(PreSegment {
                word: word.to_ascii_lowercase(),
                num: digits.parse().ok(),
            });
        }
        Some(out)
    }

    /// 逐段比较：先比词，再比数字；无数字视为小于有数字
    fn cmp_pre(left: &[PreSegment], right: &[PreSegment]) -> Ordering {
        for (a, b) in left.iter().zip(right) {
            let ordering = a.word.cmp(&b.word).then_with(|| match (a.num, b.num) {
                (Some(x), Some(y)) => x.cmp(&y),
                (None, Some(_)) => Ordering::Less,
                (Some(_), None) => Ordering::Greater,
                (None, None) => Ordering::Equal,
            });
            if ordering != Ordering::Equal {
                return ordering;
            }
        }
        left.len().cmp(&right.len())
    }

    fn parse(s: &str) -> Option<(u64, u64, u64, Vec<PreSegment>)> {
        let trimmed = s.trim();
        let without_v = trimmed.strip_prefix('v').unwrap_or(trimmed);
        let without_build = without_v
            .split_once('+')
            .map_or(without_v, |(head, _)| head);
        let (core, pre) = without_build
            .split_once('-')
            .map_or((without_build, ""), |(c, p)| (c, p));
        let mut parts = core.split('.');
        let major = parts.next()?.parse().ok()?;
        let minor = parts.next()?.parse().ok()?;
        let patch = parts.next()?.parse().ok()?;
        let pre = if pre.is_empty() {
            Vec::new()
        } else {
            parse_pre(pre)?
        };
        Some((major, minor, patch, pre))
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

/// 检查更新（统一入口）
///
/// 更新源由开发者中心「检查更新源」决定：
/// - `api`（默认）：云端需鉴权清单，服务端按设备灰度；
/// - `github`：Release 里的静态 latest.json，不需要设备凭证；
/// - `auto`：云端优先，失败（含未注册、网络异常）降级到 GitHub 清单。
pub async fn check_update(state: &AppState, _app: &AppHandle) -> Result<UpdateInfo, String> {
    let current_version = env!("CARGO_PKG_VERSION");
    let channel = resolve_channel(current_version);
    match crate::commands::system::developer::get_update_source().as_str() {
        "github" => super::github::check_github(&channel, current_version).await,
        "auto" => match check_api(state, current_version, &channel).await {
            Ok(info) => Ok(info),
            Err(err) => {
                log_warn!("[Updater] 云端更新通道失败，降级 GitHub 清单: {}", err);
                super::github::check_github(&channel, current_version)
                    .await
                    .map_err(|gh| format!("{}；GitHub 备用源亦失败：{}", err, gh))
            }
        },
        _ => check_api(state, current_version, &channel).await,
    }
}

/// 更新通道：由当前版本后缀推导（alpha/beta/stable），开发者模式可覆盖用于调试
fn resolve_channel(current_version: &str) -> String {
    let branch = crate::commands::system::developer::get_update_branch();
    if branch == "auto" {
        crate::utils::client_type::channel_name(current_version).to_string()
    } else {
        branch
    }
}

/// 云端通道检查（需设备 JWT，服务端按设备灰度）
async fn check_api(
    state: &AppState,
    current_version: &str,
    channel: &str,
) -> Result<UpdateInfo, String> {
    // 1. 服务端地址与设备凭证（凭证过期时自动续期）
    let base_url = crate::commands::online::manager::read_api_server_url(state).await;
    // 凭证缺失时先补一次静默注册：首次启动的注册可能尚未完成或曾经失败，
    // 这里按需补齐，避免用户首次点「检查更新」就撞上「未注册」。
    let creds = crate::commands::online::manager::ensure_credentials(state)
        .await
        .map_err(|e| format!("检查更新失败：{}", e))?;
    if creds.device_token.is_empty() {
        return Err("检查更新失败：设备凭证为空，请稍后重试".to_string());
    }
    let target = platform_target();
    let arch = std::env::consts::ARCH;
    let path = UPDATES_CHECK
        .replace("{{target}}", target)
        .replace("{{arch}}", arch)
        .replace("{{current_version}}", current_version)
        .replace("{{channel}}", channel);
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

#[cfg(test)]
#[path = "check_test.rs"]
mod check_test;
