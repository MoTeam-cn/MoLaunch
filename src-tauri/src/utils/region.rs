//! 下载限制判定：地区检测（机器环境层 + 网络出口层）+ 官网下发的策略清单。
//!
//! 判定顺序：策略总开关 → 地区检测开关 → 受限地区列表 → 网络出口国家。
//! 策略由 `utils::download_policy` 从官网拉取（拉取失败按不限制处理，fail-open）；
//! 网络出口判定失败按「受限」处理（fail-closed），结果进程内缓存并在启动时预热。

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use super::download_policy::{self, DownloadPolicy, PolicyNotice};
use crate::{log_info, log_warn};

/// 策略未下发提示文案时的兜底（后端拒绝下载时使用）
pub const FALLBACK_NOTICE_TEXT: &str =
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
    /// 网络出口是否位于中国大陆
    pub is_china_mainland_network: bool,
    /// 网络出口国家代码（未知为 None）
    pub country_code: Option<String>,
    /// 最终是否禁用下载（已扣除开发者模式临时绕过）
    pub is_restricted: bool,
    /// 当前策略的受限地区列表（归一化后）
    pub blocked_regions: Vec<String>,
    /// 官网下发的提示文案
    pub notice: PolicyNotice,
    /// 策略是否允许开发者模式绕过
    pub allow_developer_mode_bypass: bool,
    /// 启动器当前是否处于开发者模式
    pub developer_mode: bool,
    /// 是否已通过开发者模式临时关闭限制（仅本次运行有效）
    pub bypassed: bool,
}

/// 网络层判定结果（进程内缓存）
#[derive(Debug, Clone)]
struct NetworkVerdict {
    country_code: Option<String>,
}

/// 机器层判定（进程内只算一次，纯同步，不发起网络请求）
static MACHINE_IS_CHINA_MAINLAND: Lazy<bool> = Lazy::new(check_machine_is_china_mainland);

/// 网络层判定缓存（None 表示尚未判定）
static NETWORK_VERDICT: Lazy<tokio::sync::Mutex<Option<NetworkVerdict>>> =
    Lazy::new(|| tokio::sync::Mutex::new(None));

/// 开发者模式临时关闭限制（仅内存，重启后按清单重新生效）
static BYPASS: AtomicBool = AtomicBool::new(false);

/// 机器环境是否位于中国大陆
pub fn is_china_mainland_machine() -> bool {
    *MACHINE_IS_CHINA_MAINLAND
}

/// 完整地区判定（含网络层；不需要检测地区时不发起任何网络请求）
pub async fn detect() -> RegionInfo {
    let policy = download_policy::current().await;
    let blocked_regions = policy.blocked_regions();
    let developer_mode = developer_mode_enabled();
    let allow_bypass = policy.download.allow_developer_mode_bypass;
    let bypassed = allow_bypass && BYPASS.load(Ordering::Relaxed);
    let machine = is_china_mainland_machine();

    // 总开关关闭 / 地区检测关闭 / 受限地区为空 → 不检测、不禁用
    if !policy.region_check_active() {
        return RegionInfo {
            is_china_mainland_machine: machine,
            is_china_mainland_network: false,
            country_code: None,
            is_restricted: false,
            blocked_regions,
            notice: policy.notice,
            allow_developer_mode_bypass: allow_bypass,
            developer_mode,
            bypassed,
        };
    }

    // 机器层快路径：受限地区只有 CN 且本机不是中国大陆环境 → 直接放行（不发请求）
    if only_china_blocked(&blocked_regions) && !machine {
        return RegionInfo {
            is_china_mainland_machine: false,
            is_china_mainland_network: false,
            country_code: None,
            is_restricted: false,
            blocked_regions,
            notice: policy.notice,
            allow_developer_mode_bypass: allow_bypass,
            developer_mode,
            bypassed,
        };
    }

    let verdict = network_verdict().await;
    let country_code = verdict.country_code;
    let in_blocked_region = restricted_by_country(&policy, country_code.as_deref());
    let restricted = in_blocked_region && policy.download.disabled_for_blocked_regions && !bypassed;
    let is_china_network = country_code
        .as_deref()
        .map(|code| code.eq_ignore_ascii_case("CN"))
        .unwrap_or(false);

    RegionInfo {
        is_china_mainland_machine: machine,
        is_china_mainland_network: is_china_network,
        country_code,
        is_restricted: restricted,
        blocked_regions,
        notice: policy.notice,
        allow_developer_mode_bypass: allow_bypass,
        developer_mode,
        bypassed,
    }
}

/// 下载类操作守卫：命中限制时返回官网下发的提示文案
pub async fn ensure_download_allowed() -> Result<(), String> {
    let info = detect().await;
    if !info.is_restricted {
        return Ok(());
    }
    if info.notice.text.trim().is_empty() {
        Err(FALLBACK_NOTICE_TEXT.to_string())
    } else {
        Err(info.notice.text)
    }
}

/// 开发者模式临时关闭/恢复下载限制
///
/// 仅当策略允许（`allowDeveloperModeBypass`）且启动器处于开发者模式时可用；
/// 状态只存内存，重启后按清单重新生效。
pub async fn set_bypass(enabled: bool) -> Result<RegionInfo, String> {
    let policy = download_policy::current().await;
    if enabled {
        if !policy.download.allow_developer_mode_bypass {
            return Err("当前下载策略不允许开发者模式绕过限制".to_string());
        }
        if !developer_mode_enabled() {
            return Err("需要先开启开发者模式才能临时关闭下载限制".to_string());
        }
    }
    BYPASS.store(enabled, Ordering::Relaxed);
    log_info!(
        "[Region] 开发者模式临时{}下载限制",
        if enabled { "关闭" } else { "恢复" }
    );
    Ok(detect().await)
}

/// 启动预热：先拉取策略，再按策略判定网络出口，避免首次打开下载页阻塞
pub fn prefetch() {
    tauri::async_runtime::spawn(async {
        download_policy::refresh_with_cache().await;
        let info = detect().await;
        log_info!(
            "[Region] 启动预判完成: country={:?}, restricted={}, blocked={:?}",
            info.country_code,
            info.is_restricted,
            info.blocked_regions
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

/// 请求网络出口地区；失败时 country 为 None（由判定方按 fail-closed 处理）
async fn fetch_network_verdict() -> NetworkVerdict {
    log_info!("[Region] 正在判定网络出口地区");
    let client = match reqwest::Client::builder().timeout(NETWORK_TIMEOUT).build() {
        Ok(client) => client,
        Err(e) => {
            log_warn!("[Region] 判定客户端创建失败: {}", e);
            return NetworkVerdict { country_code: None };
        }
    };

    let response = match client.get(IPINFO_URL).send().await {
        Ok(response) => response,
        Err(e) => {
            log_warn!("[Region] 网络出口判定失败: {}", e);
            return NetworkVerdict { country_code: None };
        }
    };

    if !response.status().is_success() {
        log_warn!(
            "[Region] 网络出口判定返回状态 {}，按受限处理",
            response.status()
        );
        return NetworkVerdict { country_code: None };
    }

    let body: IpInfoResponse = match response.json().await {
        Ok(body) => body,
        Err(e) => {
            log_warn!("[Region] 网络出口响应解析失败: {}", e);
            return NetworkVerdict { country_code: None };
        }
    };

    let country_code = body
        .country
        .as_deref()
        .map(|code| code.trim().to_uppercase())
        .filter(|code| !code.is_empty());
    log_info!("[Region] 网络出口判定完成: country={:?}", country_code);
    NetworkVerdict { country_code }
}

/// 受限地区是否只有中国大陆（此时可用机器层短路，避免非中国大陆机器发起 IP 请求）
fn only_china_blocked(blocked_regions: &[String]) -> bool {
    !blocked_regions.is_empty()
        && blocked_regions
            .iter()
            .all(|region| region.eq_ignore_ascii_case("CN"))
}

/// 出口国家是否落在受限地区列表内；拿不到国家按受限处理（fail-closed）
fn restricted_by_country(policy: &DownloadPolicy, country_code: Option<&str>) -> bool {
    match country_code {
        Some(code) => policy.blocks_region(code),
        None => true,
    }
}

/// 启动器是否处于开发者模式（注册表 DeveloperMode）
fn developer_mode_enabled() -> bool {
    crate::commands::system::developer::is_developer_mode_enabled()
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
