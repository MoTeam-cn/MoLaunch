//! 厂商管理：内置系统默认厂商 + 外部厂商列表 + 启用/禁用 + 路径辅助
//! 内置系统默认厂商（`system-default`）的 frpc 首次使用时从 apiServer `/v1/frp/manifest`
//! 获取最新版本下载 URL（见 `binary.rs`）。外部厂商存放于 `<base_dir>/providers/<provider_id>/`，
//! 包含 manifest.json 描述厂商元信息、frpc 分发方式（bundled/url）和认证配置。
//! 厂商启用状态持久化到 `<base_dir>/frp/providers.json`；安装/卸载见 [`super::install`]，frpc 下载见 [`super::binary`]。

use super::{ensure_dir, providers_root, providers_state_path, validate_provider_id, ProviderInfo, ProviderManifest};
use crate::log_info;
use crate::state::AppState;
use std::collections::HashMap;
use std::path::PathBuf;

/// 系统默认厂商 ID
pub const SYSTEM_DEFAULT_ID: &str = "system-default";

// ============================================================
// 路径辅助
// ============================================================

/// 系统默认厂商目录（`<base_dir>/providers/system-default/`）
pub(super) fn system_default_dir() -> PathBuf {
    providers_root().join(SYSTEM_DEFAULT_ID)
}

/// frpc 二进制路径（系统默认厂商）
///
/// Windows: `<base_dir>/providers/system-default/frpc.exe`
/// macOS/Linux: `<base_dir>/providers/system-default/frpc`
pub(super) fn frpc_path() -> PathBuf {
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
pub(super) fn read_frpc_version() -> Option<String> {
    let path = frpc_version_path();
    let v = std::fs::read_to_string(&path).ok()?;
    let v = v.trim().to_string();
    if v.is_empty() { None } else { Some(v) }
}

/// 写入 frpc 版本到元数据文件（下载成功后调用）
pub(super) fn write_frpc_version(version: &str) {
    let path = frpc_version_path();
    if let Some(parent) = path.parent() {
        if std::fs::create_dir_all(parent).is_ok() {
            let _ = std::fs::write(&path, version);
        }
    }
}

/// 获取指定厂商的 frpc 二进制路径
///
/// 系统默认厂商返回 `frpc_path()`。
/// 外部厂商根据 manifest.binary.distribution 返回：
/// - bundled: `<providers_root>/<id>/<binary.path>`
/// - url: `<providers_root>/<id>/<binary.download.target_path>`
pub fn get_frpc_path_for_provider(provider_id: &str) -> Result<PathBuf, String> {
    if provider_id == SYSTEM_DEFAULT_ID {
        return Ok(frpc_path());
    }
    let manifest = read_provider_manifest(provider_id)?;
    let dir = providers_root().join(provider_id);
    match manifest.binary.distribution.as_str() {
        "bundled" => {
            let rel = manifest
                .binary
                .path
                .ok_or_else(|| format!("厂商 {} 的 manifest 缺少 binary.path", provider_id))?;
            Ok(dir.join(rel))
        }
        "url" => {
            let dl = manifest
                .binary
                .download
                .ok_or_else(|| format!("厂商 {} 的 manifest 缺少 binary.download", provider_id))?;
            Ok(dir.join(dl.target_path))
        }
        other => Err(format!(
            "厂商 {} 使用不支持的分发方式: {}",
            provider_id, other
        )),
    }
}

/// frpc 二进制是否就绪（系统默认厂商）
pub(super) fn is_frpc_ready() -> bool {
    frpc_path().exists()
}

/// 判断外部厂商的 frpc 是否就绪
pub(super) fn is_external_frpc_ready(provider_id: &str, manifest: &ProviderManifest) -> bool {
    let dir = providers_root().join(provider_id);
    match manifest.binary.distribution.as_str() {
        "bundled" => {
            if let Some(ref rel_path) = manifest.binary.path {
                dir.join(rel_path).exists()
            } else {
                false
            }
        }
        "url" => {
            if let Some(ref dl) = manifest.binary.download {
                dir.join(&dl.target_path).exists()
            } else {
                false
            }
        }
        _ => false,
    }
}

// ============================================================
// 厂商启用状态持久化
// ============================================================

/// 读取厂商启用状态（`<base_dir>/frp/providers.json`）
///
/// 文件不存在或损坏时返回空 HashMap（所有外部厂商默认启用）。
pub(super) fn read_providers_state() -> HashMap<String, bool> {
    let path = providers_state_path();
    if !path.exists() {
        return HashMap::new();
    }
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return HashMap::new(),
    };
    if content.trim().is_empty() {
        return HashMap::new();
    }
    serde_json::from_str(&content).unwrap_or_else(|_| HashMap::new())
}

/// 写入厂商启用状态
pub(super) fn write_providers_state(state: &HashMap<String, bool>) -> Result<(), String> {
    let path = providers_state_path();
    if let Some(parent) = path.parent() {
        ensure_dir(parent)?;
    }
    let content = serde_json::to_string_pretty(state)
        .map_err(|e| format!("序列化厂商状态失败: {}", e))?;
    std::fs::write(&path, content).map_err(|e| format!("写入厂商状态失败: {}", e))
}

// ============================================================
// 厂商清单读取
// ============================================================

/// 读取外部厂商的 manifest.json
pub(super) fn read_provider_manifest(provider_id: &str) -> Result<ProviderManifest, String> {
    validate_provider_id(provider_id)?;
    let manifest_path = providers_root().join(provider_id).join("manifest.json");
    if !manifest_path.exists() {
        return Err(format!("厂商 manifest 不存在: {}", manifest_path.display()));
    }
    let content = std::fs::read_to_string(&manifest_path)
        .map_err(|e| format!("读取 manifest 失败: {}", e))?;
    let manifest: ProviderManifest = serde_json::from_str(&content)
        .map_err(|e| format!("解析 manifest 失败: {}", e))?;
    if manifest.id != provider_id {
        return Err(format!(
            "manifest.id ({}) 与目录名 ({}) 不一致",
            manifest.id, provider_id
        ));
    }
    Ok(manifest)
}

// ============================================================
// 厂商列表
// ============================================================

/// 列出所有厂商（内置 + 外部）
///
/// 内置系统默认厂商始终返回。外部厂商扫描 `<base_dir>/providers/` 读 manifest.json，
/// 损坏或 id 不匹配的跳过。系统默认厂商版本：已安装读 `frpc_version.txt`；
/// 未安装请求 apiServer `GET /v1/frp/manifest`（传 `0.0.0`）取最新号，失败回退"未安装"。
/// 注：apiServer 校验版本格式，空串返回 code=1001，须传 `0.0.0` 表示查最新。
pub async fn list_providers(state: &AppState) -> Result<Vec<ProviderInfo>, String> {
    let mut providers = Vec::new();
    let state_map = read_providers_state();

    // 内置：系统默认
    let frpc_ready = is_frpc_ready();
    // 版本号：本地已安装用真实版本，未安装请求 apiServer 获取最新版本
    let version = if frpc_ready {
        read_frpc_version().unwrap_or_else(|| "未知".to_string())
    } else {
        super::binary::fetch_latest_frpc_version(state)
            .await
            .unwrap_or_else(|e| {
                log_info!("[Frp] 获取最新 frpc 版本失败，回退显示'未安装': {}", e);
                "未安装".to_string()
            })
    };
    providers.push(ProviderInfo {
        id: SYSTEM_DEFAULT_ID.to_string(),
        name: "系统默认".to_string(),
        description: "Frp 原版，仅支持 frpc + 配置文件启动".to_string(),
        version,
        author: "MoTeam".to_string(),
        builtin: true,
        auth_type: "none".to_string(),
        frpc_ready,
        enabled: true,
        distribution: "system".to_string(),
        homepage: None,
    });

    // 外部：扫描 providers/ 目录
    let root = providers_root();
    if root.exists() {
        let entries = std::fs::read_dir(&root)
            .map_err(|e| format!("读取厂商目录失败: {}", e))?;
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let manifest_path = path.join("manifest.json");
            if !manifest_path.exists() {
                continue;
            }

            let id = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();
            if id.is_empty() || id == SYSTEM_DEFAULT_ID {
                continue;
            }

            let content = match std::fs::read_to_string(&manifest_path) {
                Ok(c) => c,
                Err(_) => continue,
            };
            let manifest: ProviderManifest = match serde_json::from_str(&content) {
                Ok(m) => m,
                Err(_) => continue,
            };

            if manifest.id != id {
                continue;
            }

            let frpc_ready = is_external_frpc_ready(&id, &manifest);
            let enabled = state_map.get(&id).copied().unwrap_or(true);

            providers.push(ProviderInfo {
                id: manifest.id,
                name: manifest.name,
                description: manifest.description,
                version: manifest.version,
                author: manifest.author,
                builtin: false,
                auth_type: manifest.auth.auth_type,
                frpc_ready,
                enabled,
                distribution: manifest.binary.distribution,
                homepage: manifest.homepage,
            });
        }
    }

    Ok(providers)
}

// ============================================================
// 启用 / 禁用
// ============================================================

/// 启用厂商
pub async fn enable_provider(provider_id: String) -> Result<(), String> {
    let mut state = read_providers_state();
    state.insert(provider_id.clone(), true);
    write_providers_state(&state)?;
    log_info!("[Frp] 厂商已启用: {}", provider_id);
    Ok(())
}

/// 禁用厂商（不允许禁用系统默认厂商）
pub async fn disable_provider(provider_id: String) -> Result<(), String> {
    if provider_id == SYSTEM_DEFAULT_ID {
        return Err("不能禁用系统默认厂商".to_string());
    }
    let mut state = read_providers_state();
    state.insert(provider_id.clone(), false);
    write_providers_state(&state)?;
    log_info!("[Frp] 厂商已禁用: {}", provider_id);
    Ok(())
}
