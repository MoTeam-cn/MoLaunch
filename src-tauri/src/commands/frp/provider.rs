//! 厂商管理：内置与外部厂商的配置、状态及路径辅助
// 子模块（平级文件，`#[path]` 指向 provider.rs 同级目录）
#[path = "provider_external.rs"]
mod provider_external;
#[path = "provider_system.rs"]
mod provider_system;

use super::binary::fetch_latest_frpc_version;
use super::{providers_root, ProviderInfo, ProviderManifest};
use crate::log_info;
use crate::state::AppState;
use std::path::PathBuf;

// 子模块符号 re-export（保持 `crate::commands::frp::provider::xxx` 引用可用）
pub(super) use provider_external::{
    is_external_frpc_ready, read_icon_as_data_url, read_provider_manifest, read_providers_state,
    resolve_auth_type, resolve_device_code_config, resolve_oauth2_config, write_providers_state,
};
pub(crate) use provider_system::{
    current_platform_key, frpc_path, frpc_platform_skip, is_frpc_ready, read_frpc_version,
    read_provider_frpc_version, resolve_download_config, system_default_dir, write_frpc_version,
    write_provider_frpc_version, SYSTEM_DEFAULT_ID,
};

/// 获取指定厂商的 frpc 二进制路径
///
/// 系统默认厂商返回 `frpc_path()`。
/// 外部厂商根据 manifest.binary.distribution 返回：
/// - bundled: 优先按平台从 `binary.paths` 查找，回退到 `binary.path`，拼接为 `<providers_root>/<id>/<rel>`
/// - url: `<providers_root>/<id>/<binary.download.target_path>`
pub fn get_frpc_path_for_provider(provider_id: &str) -> Result<PathBuf, String> {
    if provider_id == SYSTEM_DEFAULT_ID {
        return Ok(frpc_path());
    }
    let manifest = read_provider_manifest(provider_id)?;
    let dir = providers_root().join(provider_id);
    match manifest.binary.distribution.as_str() {
        "bundled" => {
            let rel = provider_system::resolve_bundled_path(&manifest.binary).ok_or_else(|| {
                format!(
                    "厂商 {} 的 manifest 缺少 binary.path 和 binary.paths（当前平台 {} 无匹配）",
                    provider_id,
                    current_platform_key()
                )
            })?;
            Ok(dir.join(rel))
        }
        "url" => {
            let dl = manifest
                .binary
                .download
                .ok_or_else(|| format!("厂商 {} 的 manifest 缺少 binary.download", provider_id))?;
            let (_, target_path) = resolve_download_config(&dl);
            Ok(dir.join(target_path))
        }
        other => Err(format!(
            "厂商 {} 使用不支持的分发方式: {}",
            provider_id, other
        )),
    }
}

// 厂商列表
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
        fetch_latest_frpc_version(state).await.unwrap_or_else(|e| {
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
        icon: None,
    });

    // 外部：扫描 providers/ 目录
    let root = providers_root();
    if root.exists() {
        let entries = std::fs::read_dir(&root).map_err(|e| format!("读取厂商目录失败: {}", e))?;
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
            let auth_type = resolve_auth_type(&id, &manifest);
            let icon = manifest
                .icon
                .as_ref()
                .and_then(|icon_rel| read_icon_as_data_url(&id, icon_rel));

            providers.push(ProviderInfo {
                id: manifest.id,
                name: manifest.name,
                description: manifest.description,
                version: manifest.version,
                author: manifest.author,
                builtin: false,
                auth_type,
                frpc_ready,
                enabled,
                distribution: manifest.binary.distribution,
                homepage: manifest.homepage,
                icon,
            });
        }
    }

    Ok(providers)
}

// 启用 / 禁用
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
