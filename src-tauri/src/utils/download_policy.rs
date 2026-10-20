//! 官网下发的下载策略清单（GET https://molaunch.moiu.cn/download-policy.json）
//!
//! 启动后后台拉取一次并缓存到应用数据目录；拉取失败沿用上次缓存，
//! 无缓存时按「不限制」处理（fail-open），不阻塞启动。

use std::path::PathBuf;
use std::time::Duration;

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::{log_info, log_warn};

/// 清单地址（由官网仓库 Molaunch-Web 维护）
const POLICY_URL: &str = "https://molaunch.moiu.cn/download-policy.json";
/// 拉取超时（与协议清单同一量级）
const POLICY_TIMEOUT: Duration = Duration::from_secs(5);
/// 本地缓存文件名（应用数据目录）
const CACHE_FILE: &str = "download-policy.json";

/// 地区检测配置
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct RegionCheckPolicy {
    /// 是否检测地区
    pub enabled: bool,
    /// 受限地区（ISO 3166-1 alpha-2，空数组 = 不限制）
    pub blocked_regions: Vec<String>,
}

/// 下载限制配置
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct DownloadRulesPolicy {
    /// 命中受限地区时是否禁用 MC 下载
    pub disabled_for_blocked_regions: bool,
    /// 是否允许开发者模式临时绕过（缺失按 false）
    pub allow_developer_mode_bypass: bool,
}

/// 提示文案（与官网弹窗同一份）
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PolicyNotice {
    /// 提示标题
    pub title: String,
    /// 提示正文
    pub text: String,
    /// 确认按钮文案
    pub confirm_text: String,
}

/// 下载策略清单
///
/// 所有字段带 `#[serde(default)]` 且忽略未知字段：官网后续加字段不会导致解析失败。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct DownloadPolicy {
    /// 总开关：false 时忽略全部限制
    pub enabled: bool,
    /// 清单版本号（仅日志与缓存比对用）
    pub version: u64,
    /// 最后更新日期（仅日志用）
    pub updated_at: String,
    /// 地区检测配置
    pub region_check: RegionCheckPolicy,
    /// 下载限制配置
    pub download: DownloadRulesPolicy,
    /// 提示文案
    pub notice: PolicyNotice,
}

impl DownloadPolicy {
    /// 归一化后的受限地区列表（大写、去空白、去重、丢弃非法值）
    pub fn blocked_regions(&self) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        for raw in &self.region_check.blocked_regions {
            let code = raw.trim().to_ascii_uppercase();
            if code.len() != 2 || !code.chars().all(|c| c.is_ascii_alphabetic()) {
                continue;
            }
            if !out.contains(&code) {
                out.push(code);
            }
        }
        out
    }

    /// 是否需要检测地区（总开关 + 地区检测开关 + 受限地区非空）
    pub fn region_check_active(&self) -> bool {
        self.enabled && self.region_check.enabled && !self.blocked_regions().is_empty()
    }

    /// 是否命中「需要禁用下载」的受限地区
    pub fn blocks_region(&self, country_code: &str) -> bool {
        if !self.region_check_active() || !self.download.disabled_for_blocked_regions {
            return false;
        }
        let code = country_code.trim().to_ascii_uppercase();
        self.blocked_regions().iter().any(|r| r == &code)
    }
}

/// 进程内策略缓存（None = 尚未加载）
static CACHED: Lazy<Mutex<Option<DownloadPolicy>>> = Lazy::new(|| Mutex::new(None));

/// 本地缓存文件路径（应用数据目录）
fn cache_path() -> PathBuf {
    crate::storage::Storage::instance()
        .base_dir()
        .join(CACHE_FILE)
}

/// 当前生效的策略：内存 → 磁盘缓存 → 默认「不限制」
pub async fn current() -> DownloadPolicy {
    {
        let guard = CACHED.lock().await;
        if let Some(policy) = guard.as_ref() {
            return policy.clone();
        }
    }
    let loaded = load_cache();
    let mut guard = CACHED.lock().await;
    if let Some(policy) = guard.as_ref() {
        return policy.clone();
    }
    let policy = loaded.unwrap_or_default();
    *guard = Some(policy.clone());
    policy
}

/// 读取磁盘缓存（失败返回 None，按「不限制」处理）
fn load_cache() -> Option<DownloadPolicy> {
    let path = cache_path();
    let bytes = std::fs::read(&path).ok()?;
    match serde_json::from_slice::<DownloadPolicy>(&bytes) {
        Ok(policy) => {
            log_info!(
                "[DownloadPolicy] 读取缓存: version={}, updatedAt={}",
                policy.version,
                policy.updated_at
            );
            Some(policy)
        }
        Err(e) => {
            log_warn!("[DownloadPolicy] 缓存解析失败，按不限制处理: {}", e);
            None
        }
    }
}

/// 写入磁盘缓存（失败只告警，不影响本次生效）
fn save_cache(policy: &DownloadPolicy) {
    let path = cache_path();
    let bytes = match serde_json::to_vec_pretty(policy) {
        Ok(bytes) => bytes,
        Err(e) => {
            log_warn!("[DownloadPolicy] 缓存序列化失败: {}", e);
            return;
        }
    };
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    if let Err(e) = std::fs::write(&path, bytes) {
        log_warn!("[DownloadPolicy] 缓存写入失败: {}", e);
    }
}

/// 载入缓存后再拉取（供 region::prefetch 启动预热串行使用）
pub async fn refresh_with_cache() {
    let _ = current().await;
    refresh().await;
}

/// 请求官网清单并更新缓存；任何失败都保留现有策略（不覆盖）
pub async fn refresh() -> Option<DownloadPolicy> {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let url = format!("{POLICY_URL}?t={ts}");

    let response = match crate::http::get_client()
        .get(&url)
        .timeout(POLICY_TIMEOUT)
        .send()
        .await
    {
        Ok(response) => response,
        Err(e) => {
            log_warn!("[DownloadPolicy] 拉取失败，沿用缓存: {}", e);
            return None;
        }
    };

    if !response.status().is_success() {
        log_warn!(
            "[DownloadPolicy] 官网返回 HTTP {}，沿用缓存",
            response.status()
        );
        return None;
    }

    let policy = match response.json::<DownloadPolicy>().await {
        Ok(policy) => policy,
        Err(e) => {
            log_warn!("[DownloadPolicy] 响应解析失败，沿用缓存: {}", e);
            return None;
        }
    };

    log_info!(
        "[DownloadPolicy] 已更新: version={}, updatedAt={}, enabled={}, blocked={:?}",
        policy.version,
        policy.updated_at,
        policy.enabled,
        policy.blocked_regions()
    );
    save_cache(&policy);
    let mut guard = CACHED.lock().await;
    *guard = Some(policy.clone());
    Some(policy)
}

#[cfg(test)]
#[path = "download_policy_test.rs"]
mod download_policy_test;
