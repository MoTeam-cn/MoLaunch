//! 下载源模式 + URL 构建函数

use super::constants::BMCLAPI_BASE;

/// 下载源模式
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DownloadSourceMode {
    Official,
    Mirror,
    Smart,
}

impl DownloadSourceMode {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Self {
        match s {
            "official" => Self::Official,
            "mirror" => Self::Mirror,
            "smart" => Self::Smart,
            _ => Self::Smart,
        }
    }
}

/// 构建候选 URL 列表（根据下载源模式排序）
///
/// - mirror: 只用镜像源，失败直接报错
/// - official: 只用官方源，失败直接报错
/// - smart: 官方优先，失败回退 BMCLAPI
pub fn build_urls(
    mirror_url: Option<&str>,
    official_url: &str,
    bmclapi_path: &str,
    mode: DownloadSourceMode,
) -> Vec<String> {
    let bmclapi_url = format!("{}{}", BMCLAPI_BASE, bmclapi_path);

    match mode {
        DownloadSourceMode::Mirror => {
            // 只用镜像源：自定义镜像 -> BMCLAPI（无官方回退）
            let mut urls = Vec::new();
            if let Some(mirror) = mirror_url.filter(|m| !m.is_empty()) {
                urls.push(format!("{}{}", mirror.trim_end_matches('/'), bmclapi_path));
            }
            urls.push(bmclapi_url);
            urls
        }
        DownloadSourceMode::Official => {
            // 只用官方源，不回退
            vec![official_url.to_string()]
        }
        DownloadSourceMode::Smart => {
            // 官方优先，失败回退 BMCLAPI
            vec![official_url.to_string(), bmclapi_url]
        }
    }
}

/// 对 URL 应用替换规则
pub fn apply_replacements(url: &str, replacements: &[(&str, &str)]) -> String {
    let mut result = url.to_string();
    for (from, to) in replacements {
        result = result.replace(from, to);
    }
    result
}

/// 对已有 URL 做域名替换，生成候选列表
///
/// 根据模式决定是否包含替换版本：
/// - mirror: 只返回替换后的镜像 URL
/// - official: 只返回原始官方 URL
/// - smart: 官方原始 + BMCLAPI 替换（官方优先）
pub fn build_replace_urls(
    official_url: &str,
    mirror_url: Option<&str>,
    replacements: &[(&str, &str)],
    mode: DownloadSourceMode,
) -> Vec<String> {
    match mode {
        DownloadSourceMode::Mirror => {
            // 只用镜像源
            let mut urls = Vec::new();
            if let Some(mirror) = mirror_url.filter(|m| !m.is_empty()) {
                if let Ok(parsed) = reqwest::Url::parse(official_url) {
                    if let Some(path) = parsed.path().strip_prefix('/') {
                        urls.push(format!("{}/{}", mirror.trim_end_matches('/'), path));
                    }
                }
            }
            let bmclapi_url = apply_replacements(official_url, replacements);
            if !urls.contains(&bmclapi_url) {
                urls.push(bmclapi_url);
            }
            urls
        }
        DownloadSourceMode::Official => {
            // 只用官方源
            vec![official_url.to_string()]
        }
        DownloadSourceMode::Smart => {
            // 官方优先，失败回退 BMCLAPI
            let bmclapi_url = apply_replacements(official_url, replacements);
            vec![official_url.to_string(), bmclapi_url]
        }
    }
}
