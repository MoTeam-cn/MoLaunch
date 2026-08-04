//! 内置系统默认厂商数据与 frpc 路径辅助
//! 系统默认厂商（`system-default`）的 frpc 首次使用时从 apiServer `/v1/frp/manifest`
//! 获取最新版本下载 URL（见 `crate::commands::frp::binary`）。

use super::super::{providers_root, BinaryConfig};
use std::path::PathBuf;

/// 系统默认厂商 ID
pub(crate) const SYSTEM_DEFAULT_ID: &str = "system-default";

// 路径辅助
/// 系统默认厂商目录（`<base_dir>/providers/system-default/`）
pub(crate) fn system_default_dir() -> PathBuf {
    providers_root().join(SYSTEM_DEFAULT_ID)
}

/// frpc 二进制路径（系统默认厂商）
///
/// Windows: `<base_dir>/providers/system-default/frpc.exe`
/// macOS/Linux: `<base_dir>/providers/system-default/frpc`
pub(crate) fn frpc_path() -> PathBuf {
    let dir = system_default_dir();
    #[cfg(target_os = "windows")]
    {
        dir.join("frpc.exe")
    }
    #[cfg(not(target_os = "windows"))]
    {
        dir.join("frpc")
    }
}

/// frpc 版本元数据文件路径（`<system_default_dir>/frpc_version.txt`）
///
/// 由 `ensure_system_default_frpc` 下载成功后写入 `manifest.version`，
/// 供 `list_providers` 展示真实版本与 `ensure_system_default_frpc` 下次 manifest 查询使用。
pub(super) fn frpc_version_path() -> PathBuf {
    system_default_dir().join("frpc_version.txt")
}

/// 读取本地 frpc 版本（从 `frpc_version.txt`）
///
/// 返回 `None` 表示版本文件缺失（旧版安装或首次安装前）。
pub(crate) fn read_frpc_version() -> Option<String> {
    let path = frpc_version_path();
    let v = std::fs::read_to_string(&path).ok()?;
    let v = v.trim().to_string();
    if v.is_empty() {
        None
    } else {
        Some(v)
    }
}

/// 写入 frpc 版本到元数据文件（下载成功后调用）
pub(crate) fn write_frpc_version(version: &str) {
    let path = frpc_version_path();
    if let Some(parent) = path.parent() {
        if std::fs::create_dir_all(parent).is_ok() {
            let _ = std::fs::write(&path, version);
        }
    }
}

/// 读取厂商目录下记录的 frpc 版本
///
/// 文件位于 `<providers_root>/<id>/frpc_version.txt`。
pub(crate) fn read_provider_frpc_version(provider_id: &str) -> Option<String> {
    let path = providers_root().join(provider_id).join("frpc_version.txt");
    let v = std::fs::read_to_string(&path).ok()?;
    let v = v.trim().to_string();
    if v.is_empty() {
        None
    } else {
        Some(v)
    }
}

/// 写入厂商目录下记录的 frpc 版本（安装/更新/下载成功后调用）
pub(crate) fn write_provider_frpc_version(provider_id: &str, version: &str) {
    let path = providers_root().join(provider_id).join("frpc_version.txt");
    if let Some(parent) = path.parent() {
        if std::fs::create_dir_all(parent).is_ok() {
            let _ = std::fs::write(&path, version);
        }
    }
}

/// frpc 二进制是否就绪（系统默认厂商）
pub(crate) fn is_frpc_ready() -> bool {
    frpc_path().exists()
}

/// 获取当前平台的 key（`{os}_{arch}` 格式）
///
/// os: windows / macos / linux
/// arch: amd64 (x86_64) / arm64 (aarch64) / 386 (x86)
pub(crate) fn current_platform_key() -> String {
    let os = std::env::consts::OS;
    let arch = match std::env::consts::ARCH {
        "x86_64" => "amd64",
        "aarch64" => "arm64",
        "x86" => "386",
        other => other,
    };
    format!("{}_{}", os, arch)
}

/// 从 BinaryConfig 解析 bundled 模式下的 frpc 相对路径
///
/// 优先按当前平台从 paths 中查找，回退到 path 字段。
pub(crate) fn resolve_bundled_path(binary: &BinaryConfig) -> Option<String> {
    let platform_key = current_platform_key();
    if let Some(ref paths) = binary.paths {
        if let Some(p) = paths.get(&platform_key) {
            return Some(p.clone());
        }
    }
    binary.path.clone()
}

/// 解析 url 分发模式下当前平台的下载 URL 与目标相对路径
///
/// 返回 `(url, target_path)`：
/// - url：优先 urls[当前平台]，回退 download.url
/// - target_path：优先 target_paths[当前平台]，回退 download.target_path
pub(crate) fn resolve_download_config(
    download: &crate::commands::frp::DownloadConfig,
) -> (String, String) {
    let platform_key = current_platform_key();
    let url = if let Some(ref urls) = download.urls {
        urls.get(&platform_key)
            .cloned()
            .unwrap_or_else(|| download.url.clone())
    } else {
        download.url.clone()
    };
    let target_path = if let Some(ref paths) = download.target_paths {
        paths
            .get(&platform_key)
            .cloned()
            .unwrap_or_else(|| download.target_path.clone())
    } else {
        download.target_path.clone()
    };
    (url, target_path)
}

/// 计算安装时应跳过的「其他平台」frpc 相对路径集合
///
/// 厂商包（bundled 或 url 下载的 ZIP）常打包全部平台的 frpc，安装时只需
/// 当前平台对应的二进制，其余平台的 frpc 不复制/解压，避免冗余。
///
/// - bundled：`binary.paths` 所有平台路径中排除当前平台
/// - url：`download.target_paths` 所有平台路径中排除当前平台
/// - 当前平台无法解析（无映射且无兜底）或厂商无平台映射 → 返回空集合（不过滤，
///   保持旧行为，避免误删文件）
///
/// 返回 `(应跳过的相对路径集合, 当前平台 frpc 相对路径)`。
pub(crate) fn frpc_platform_skip(
    binary: &BinaryConfig,
) -> (std::collections::HashSet<String>, Option<String>) {
    use std::collections::HashSet;

    let platform_key = current_platform_key();
    let mut all_paths: HashSet<String> = HashSet::new();
    let mut current: Option<String> = None;

    match binary.distribution.as_str() {
        "bundled" => {
            if let Some(ref paths) = binary.paths {
                all_paths.extend(paths.values().cloned());
                current = paths.get(&platform_key).cloned();
            } else if let Some(ref p) = binary.path {
                current = Some(p.clone());
            }
        }
        "url" => {
            if let Some(ref dl) = binary.download {
                if let Some(ref target_paths) = dl.target_paths {
                    all_paths.extend(target_paths.values().cloned());
                    current = target_paths.get(&platform_key).cloned();
                } else {
                    current = Some(dl.target_path.clone());
                }
            }
        }
        _ => {}
    }

    let skip: HashSet<String> = match &current {
        Some(cur) => all_paths.into_iter().filter(|p| p != cur).collect(),
        // 当前平台无映射：无法判断哪个文件属于当前平台，保守不过滤
        None => HashSet::new(),
    };
    (skip, current)
}

#[cfg(test)]
#[path = "provider_system_tests.rs"]
mod tests;
