//! 中国大陆地区判定：机器环境层 + 网络出口层。
//!
//! 机器层同步判定（UTC+8 时区 + 系统区域/GeoID），命中后再按网络出口国家确认；
//! 网络层请求失败按"受限"处理（fail-closed），结果进程内缓存并在启动时预热。

use std::time::Duration;

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use crate::{log_info, log_warn};

/// 中国大陆下载限制提示（后端拒绝与前端展示共用同一句文案）
pub const RESTRICTED_MESSAGE: &str =
    "受最近某事件影响，MoLaunch 启动器暂不向中国用户提供 MC 下载功能，恕我无能为力... qwq...";

/// 中国大陆 UTC 偏移（秒）
const CHINA_UTC_OFFSET_SECONDS: i32 = 8 * 3600;
/// 网络出口地区判定接口（ipinfo.io，未带 token 时按调用方出口 IP 返回）
const IPINFO_URL: &str = "https://ipinfo.io/json";
/// 网络判定超时：超时按受限处理，避免下载页长时间等待
const NETWORK_TIMEOUT: Duration = Duration::from_secs(5);
/// Windows 地理位置表中的中国大陆 ID（GEOCLASS_NATION）
#[cfg(target_os = "windows")]
const WINDOWS_CHINA_GEO_NATION: u32 = 45;

/// 地区判定结果（IPC：system_manager 的 get_region_info）
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RegionInfo {
    /// 机器环境是否位于中国大陆（时区 + 系统区域）
    pub is_china_mainland_machine: bool,
    /// 网络出口是否位于中国大陆（仅机器层命中时判定）
    pub is_china_mainland_network: bool,
    /// 网络出口国家代码（未知为 None）
    pub country_code: Option<String>,
    /// 是否命中下载限制（机器层与网络层同时命中）
    pub is_restricted: bool,
}

/// 网络层判定结果（进程内缓存）
#[derive(Debug, Clone)]
struct NetworkVerdict {
    restricted: bool,
    country_code: Option<String>,
}

/// 机器层判定（进程内只算一次，纯同步，不发起网络请求）
static MACHINE_IS_CHINA_MAINLAND: Lazy<bool> = Lazy::new(check_machine_is_china_mainland);

/// 网络层判定缓存（None 表示尚未判定）
static NETWORK_VERDICT: Lazy<tokio::sync::Mutex<Option<NetworkVerdict>>> =
    Lazy::new(|| tokio::sync::Mutex::new(None));

/// 机器环境是否位于中国大陆
pub fn is_china_mainland_machine() -> bool {
    *MACHINE_IS_CHINA_MAINLAND
}

/// 完整地区判定（含网络层；非中国大陆机器直接短路，不发网络请求）
pub async fn detect() -> RegionInfo {
    if !is_china_mainland_machine() {
        return RegionInfo {
            is_china_mainland_machine: false,
            is_china_mainland_network: false,
            country_code: None,
            is_restricted: false,
        };
    }

    let verdict = network_verdict().await;
    RegionInfo {
        is_china_mainland_machine: true,
        is_china_mainland_network: verdict.restricted,
        country_code: verdict.country_code,
        is_restricted: verdict.restricted,
    }
}

/// 下载类操作守卫：命中限制时返回统一错误文案
pub async fn ensure_download_allowed() -> Result<(), String> {
    if detect().await.is_restricted {
        Err(RESTRICTED_MESSAGE.to_string())
    } else {
        Ok(())
    }
}

/// 启动预热：中国大陆机器在后台先判定网络出口，避免首次打开下载页阻塞
pub fn prefetch() {
    if !is_china_mainland_machine() {
        return;
    }
    tauri::async_runtime::spawn(async {
        let info = detect().await;
        log_info!(
            "[Region] 启动预判完成: country={:?}, restricted={}",
            info.country_code,
            info.is_restricted
        );
    });
}

/// 读取网络层判定（命中缓存则不再请求）
async fn network_verdict() -> NetworkVerdict {
    let mut guard = NETWORK_VERDICT.lock().await;
    if let Some(cached) = guard.as_ref() {
        return cached.clone();
    }
    let verdict = fetch_network_verdict().await;
    *guard = Some(verdict.clone());
    verdict
}

/// ipinfo.io 响应（仅取判定所需字段）
#[derive(Debug, Deserialize)]
struct IpInfoResponse {
    #[serde(default)]
    country: Option<String>,
}

/// 请求网络出口地区；任何失败都按受限处理（fail-closed）
async fn fetch_network_verdict() -> NetworkVerdict {
    log_info!("[Region] 正在判定网络出口地区");
    let client = match reqwest::Client::builder().timeout(NETWORK_TIMEOUT).build() {
        Ok(client) => client,
        Err(e) => {
            log_warn!("[Region] 判定客户端创建失败，按受限处理: {}", e);
            return restricted_verdict();
        }
    };

    let response = match client.get(IPINFO_URL).send().await {
        Ok(response) => response,
        Err(e) => {
            log_warn!("[Region] 网络出口判定失败，按受限处理: {}", e);
            return restricted_verdict();
        }
    };

    // 限流（429）等非成功状态拿不到国家信息，按受限处理
    if !response.status().is_success() {
        log_warn!(
            "[Region] 网络出口判定返回状态 {}，按受限处理",
            response.status()
        );
        return restricted_verdict();
    }

    let body: IpInfoResponse = match response.json().await {
        Ok(body) => body,
        Err(e) => {
            log_warn!("[Region] 网络出口响应解析失败，按受限处理: {}", e);
            return restricted_verdict();
        }
    };

    let country_code = body
        .country
        .as_deref()
        .map(|code| code.trim().to_uppercase())
        .filter(|code| !code.is_empty());
    let restricted = restricted_from_body(country_code.as_deref());
    log_info!(
        "[Region] 网络出口判定完成: country={:?}, restricted={}",
        country_code,
        restricted
    );

    NetworkVerdict {
        restricted,
        country_code,
    }
}

/// 受限兜底结果（网络判定不可用）
fn restricted_verdict() -> NetworkVerdict {
    NetworkVerdict {
        restricted: true,
        country_code: None,
    }
}

/// 按接口返回值判定是否受限：只有明确拿到非 CN 国家才放行，缺失一律按受限处理
fn restricted_from_body(country_code: Option<&str>) -> bool {
    match country_code {
        Some(code) => code.eq_ignore_ascii_case("CN"),
        None => true,
    }
}

/// 机器层判定：UTC+8 时区 + 系统区域为中国大陆
fn check_machine_is_china_mainland() -> bool {
    let offset_seconds = chrono::Local::now().offset().local_minus_utc();
    if offset_seconds != CHINA_UTC_OFFSET_SECONDS {
        return false;
    }
    let is_china = is_china_region();
    log_info!(
        "[Region] 机器环境判定: offset={}s, china_region={}",
        offset_seconds,
        is_china
    );
    is_china
}

/// Windows：注册表区域名（zh-CN）或地理位置 ID（45 = 中国）
#[cfg(target_os = "windows")]
fn is_china_region() -> bool {
    let locale_is_china = read_windows_locale_name()
        .map(|locale| locale.trim().to_ascii_uppercase().ends_with("-CN"))
        .unwrap_or(false);
    locale_is_china || read_windows_geo_nation() == Some(WINDOWS_CHINA_GEO_NATION)
}

/// Windows：读取系统区域名（如 zh-CN）
#[cfg(target_os = "windows")]
fn read_windows_locale_name() -> Option<String> {
    use winreg::enums::{HKEY_CURRENT_USER, KEY_READ};
    use winreg::RegKey;

    let key = RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey_with_flags("Control Panel\\International", KEY_READ)
        .ok()?;
    key.get_value::<String, _>("LocaleName").ok()
}

/// Windows：读取地理位置 ID（注册表中 Nation 通常存为字符串，统一按字符串解析）
#[cfg(target_os = "windows")]
fn read_windows_geo_nation() -> Option<u32> {
    use winreg::enums::{HKEY_CURRENT_USER, KEY_READ};
    use winreg::RegKey;

    let key = RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey_with_flags("Control Panel\\International\\Geo", KEY_READ)
        .ok()?;
    key.get_value::<String, _>("Nation")
        .ok()?
        .trim()
        .parse::<u32>()
        .ok()
}

/// Unix：TZ 为中国时区，或本地化语言为中国大陆简体
#[cfg(not(target_os = "windows"))]
fn is_china_region() -> bool {
    if std::env::var("TZ")
        .map(|tz| tz.trim() == "Asia/Shanghai")
        .unwrap_or(false)
    {
        return true;
    }
    ["LC_ALL", "LC_MESSAGES", "LANG"].iter().any(|name| {
        std::env::var(name)
            .map(|value| value.contains("zh_CN"))
            .unwrap_or(false)
    })
}

#[cfg(test)]
#[path = "region_test.rs"]
mod region_test;
