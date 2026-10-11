//! 检查更新：构造 URL、发起请求、解析 manifest、版本比较

use serde_json::Value;
use tauri::AppHandle;

use super::UpdateInfo;
use crate::api_paths::UPDATES_CHECK;
use crate::state::AppState;

/// GitHub Releases 仓库（备用更新源）
const GITHUB_REPO: &str = "MoTeam-cn/MoLaunch";

/// 更新检查端点路径（参与签名，必须与 api-server 一致）
const UPDATE_CHECK_PATH: &str = "/v3/updates/check";

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
/// **更新源**：由 `AppConfig.update_source` 决定
/// - `api`（默认）：MoLaunch 云端 `/v3/updates/check`（启动器自用端点，签名校验，
///   **无需设备注册与 JWT**，因此不会触发设备 token 续期）。
/// - `github`：GitHub Releases `releases/latest`，无需任何鉴权。
///
/// 注意：`/v3/updates/manifest` 是 Tauri updater 插件专用端点
/// （见 tauri.conf.json 的 plugins.updater.endpoints），启动器不得复用。
///
/// **base_url**：云端源从 `AppConfig.online.api_server_url` 读取（继承联机设置），不再硬编码。
/// macOS/Linux 下载安装仍转发到官方 plugin（`download_and_install_unix`）。
pub async fn check_update(state: &AppState, _app: &AppHandle) -> Result<UpdateInfo, String> {
    let current_version = env!("CARGO_PKG_VERSION");

    // 更新源分流：github 走 Releases，其余一律走云端业务端点
    let update_source = {
        let config = state.config.lock().await;
        config.update_source.clone()
    };
    if update_source == "github" {
        return check_update_github(current_version).await;
    }

    let target = platform_target();
    let arch = std::env::consts::ARCH;

    // 1. 从配置读取 api_server_url（短锁，立即 clone 释放）
    let base_url = {
        let config = state.config.lock().await;
        config.online.api_server_url.clone()
    };

    // 2. 构建 URL（base_url + 路径模板替换 + 签名参数，模板见 crate::api_paths::UPDATES_CHECK）
    // channel 由当前版本后缀自动推导（alpha/beta/stable）；开发者模式可覆盖分支用于调试
    let channel = {
        let branch = crate::commands::system::developer::get_update_branch();
        if branch == "auto" {
            crate::utils::client_type::channel_name(current_version).to_string()
        } else {
            branch
        }
    };
    let path = UPDATES_CHECK
        .replace("{{target}}", target)
        .replace("{{arch}}", arch)
        .replace("{{current_version}}", current_version)
        .replace("{{channel}}", &channel);

    // 3. 签名参数（内嵌 K0 + 本地落盘的 hint；无密钥时不带签名，服务端 soft 策略照常返回清单）
    //    本端点不加载任何设备凭证，因此检查更新不会触发设备 token 续期
    let (sign_query, sign_lx) = crate::utils::update_sign::build_sign_query(
        UPDATE_CHECK_PATH,
        target,
        arch,
        current_version,
        &channel,
    );
    let url = format!("{}{}{}", base_url.trim_end_matches('/'), path, sign_query);

    log::info!(
        "[Updater] 检查更新（签名: {}）: {}",
        sign_lx.as_deref().unwrap_or("none"),
        crate::utils::net::sanitize_url_for_log(&url)
    );

    // 4. 构建请求（公开端点，不携带任何凭证）
    let client = crate::http::get_client();
    let response = client.get(&url).send().await.map_err(|e| {
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

    // UnifiedResponse 包装：{ code, msg, data }
    let data = match json.get("data") {
        Some(data) => {
            let code = json.get("code").and_then(|v| v.as_i64()).unwrap_or(0);
            if code != 1 {
                log::info!("[Updater] 服务器返回 code={}（无可用更新）", code);
                return Ok(UpdateInfo::default());
            }
            data
        }
        None => &json,
    };

    // 签名有效的响应会回带比本地更新的密钥，落盘供下次请求使用（轮换自愈）
    if let Some(hint) = data.get("sign_hint") {
        crate::utils::update_sign::save_hint(hint);
    }

    // /v3/updates/check 把清单包在 data.manifest 下；兼容 data 直接就是清单的形态
    let manifest = data.get("manifest").unwrap_or(data);

    let version = manifest
        .get("version")
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

/// GitHub Releases 更新检查（备用更新源，无需任何鉴权）
///
/// 取 `releases/latest`：版本号取 tag（去掉前缀 v），更新日志取 release body，
/// 下载地址按当前平台/架构从 release 资源中挑选。
/// 注意：GitHub 不提供强制更新与灰度信息，`force_update` 恒为 false、无签名。
async fn check_update_github(current_version: &str) -> Result<UpdateInfo, String> {
    let url = format!(
        "https://api.github.com/repos/{}/releases/latest",
        GITHUB_REPO
    );
    let client = crate::http::get_client();
    let response = client
        .get(&url)
        .header("Accept", "application/vnd.github+json")
        .header("User-Agent", "MoLaunch-Updater")
        .send()
        .await
        .map_err(|e| format!("检查更新失败: {e}"))?;

    if !response.status().is_success() {
        return Err(format!(
            "检查更新失败: HTTP {}（GitHub）",
            response.status()
        ));
    }

    let json: Value = response
        .json()
        .await
        .map_err(|e| format!("解析更新信息失败: {e}"))?;

    let version = json
        .get("tag_name")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim_start_matches('v');
    if version.is_empty() {
        return Err("GitHub Release 缺少 tag_name".to_string());
    }
    if !is_version_newer(version, current_version) {
        log::info!("[Updater] 当前版本 {} 已是最新（GitHub）", current_version);
        return Ok(UpdateInfo::default());
    }

    let download_url = pick_github_asset(&json).ok_or_else(|| {
        "GitHub Release 中未找到匹配当前平台的安装包，请前往 Releases 页手动下载".to_string()
    })?;
    crate::utils::net::validate_github_download_url(&download_url)?;

    log::info!(
        "[Updater] 发现新版本（GitHub）: {} -> {}",
        current_version,
        version
    );

    Ok(UpdateInfo {
        available: true,
        version: version.to_string(),
        notes: json
            .get("body")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        force_update: false,
        download_url,
        signature: String::new(),
    })
}

/// 从 GitHub Release 的 assets 中挑选当前平台/架构的安装包
///
/// - Windows：便携版 `MoLaunch_<ver>_x64.exe`（排除 `-setup.exe` 安装版，自更新需替换主程序本体）
/// - macOS：`*.app.tar.gz`
/// - Linux：`*.AppImage`
///
/// 未匹配到对应资源时返回 None（宁可报错让用户手动下载，也不下载错误文件）。
fn pick_github_asset(release: &Value) -> Option<String> {
    let assets = release.get("assets")?.as_array()?;
    let suffixes: &[&str] = if cfg!(target_os = "windows") {
        &["_x64.exe", "_x86_64.exe", "_arm64.exe", "_x86.exe"]
    } else if cfg!(target_os = "macos") {
        &[".app.tar.gz"]
    } else {
        &[".AppImage"]
    };

    for asset in assets {
        let name = asset.get("name").and_then(|v| v.as_str()).unwrap_or("");
        let url = asset
            .get("browser_download_url")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        if name.is_empty() || url.is_empty() || name.ends_with("-setup.exe") {
            continue;
        }
        if suffixes.iter().any(|s| name.ends_with(s)) {
            return Some(url.to_string());
        }
    }
    None
}

#[cfg(test)]
#[path = "check_test.rs"]
mod tests;
