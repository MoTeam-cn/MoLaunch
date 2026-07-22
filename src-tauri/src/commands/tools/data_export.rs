//! 启动器数据导出（zip 打包）
//!
//! 将启动器的核心数据（配置 / 版本列表 / 账号）打包为 zip 文件，
//! 便于用户备份或迁移。账号敏感信息会脱敏处理：
//! - 微软账号：不导出 `expires_at` / `is_expired`（token 状态字段）
//! - 离线账号：不导出 `skin`（避免大段 base64 占用空间）
//!
//! 导出内容按 `ExportLauncherDataParams` 的开关字段控制：
//! - `include_config`：导出 `config.json`（AppConfig 序列化）
//! - `include_versions`：导出 `versions.json`（已安装版本列表 + 类型）
//! - `include_accounts`：导出 `accounts.json`（脱敏后的账号列表）

use std::io::Write;
use std::path::PathBuf;

use serde::Serialize;

use crate::commands::version::list::detect_version_type_from_dir;
use crate::error_util::log_err;
use crate::log_info;
use crate::log_warn;
use crate::minecraft::auth::storage::PersistedAuthState;
use crate::minecraft::version::scan as version_scan;
use crate::state::AppState;
use crate::state::resolve_game_dir;

use super::types::{ExportLauncherDataParams, ExportResult};

/// 脱敏后的微软账号信息（不导出 expires_at / is_expired）
#[derive(Debug, Serialize)]
struct ExportedMsAccount {
    username: String,
    uuid: String,
}

/// 脱敏后的离线账号信息（不导出 skin 字段，避免大段 base64）
#[derive(Debug, Serialize)]
struct ExportedOfflineAccount {
    username: String,
    uuid: String,
}

/// 账号导出 payload（accounts.json 内容）
#[derive(Debug, Serialize)]
struct ExportedAccounts {
    /// 当前登录用户名（脱敏：仅保留 name / login_type，不含 token）
    current_user: Option<ExportedCurrentUser>,
    ms_accounts: Vec<ExportedMsAccount>,
    offline_accounts: Vec<ExportedOfflineAccount>,
}

#[derive(Debug, Serialize)]
struct ExportedCurrentUser {
    name: String,
    uuid: String,
    login_type: String,
}

/// 导出版本条目
#[derive(Debug, Serialize)]
struct ExportedVersion {
    id: String,
    version_type: String,
}

/// 启动器数据导出：打包 config / versions / accounts 为 zip
///
/// 输出 zip 内文件：
/// - `config.json`：AppConfig 序列化（include_config=true）
/// - `versions.json`：已安装版本列表 + 类型（include_versions=true）
/// - `accounts.json`：脱敏账号列表（include_accounts=true）
pub async fn export_launcher_data(
    state: &AppState,
    params: ExportLauncherDataParams,
) -> Result<serde_json::Value, String> {
    if params.output_path.trim().is_empty() {
        return Err("output_path 不能为空".to_string());
    }

    let output_path = PathBuf::from(&params.output_path);
    // 确保父目录存在
    if let Some(parent) = output_path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("创建输出目录失败: {}", e))?;
        }
    }

    log_info!(
        "[DataExport] 开始导出: output={}, config={}, versions={}, accounts={}",
        params.output_path,
        params.include_config,
        params.include_versions,
        params.include_accounts
    );

    let mut exported_items: Vec<String> = Vec::new();

    // 准备 zip 文件
    let file = std::fs::File::create(&output_path)
        .map_err(|e| format!("创建输出文件失败: {}", e))?;
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default();

    // 1. 导出 config.json
    if params.include_config {
        let config = state.config.lock().await.clone();
        let json = serde_json::to_string_pretty(&config)
            .map_err(|e| format!("序列化 config 失败: {}", e))?;
        zip.start_file("config.json", options)
            .map_err(|e| format!("写入 config.json 失败: {}", e))?;
        zip.write_all(json.as_bytes())
            .map_err(|e| format!("写入 config.json 失败: {}", e))?;
        exported_items.push("config".to_string());
        log_info!("[DataExport] 已添加 config.json");
    }

    // 2. 导出 versions.json
    if params.include_versions {
        let versions = export_versions_list(state).await;
        let json = serde_json::to_string_pretty(&versions)
            .map_err(|e| format!("序列化 versions 失败: {}", e))?;
        zip.start_file("versions.json", options)
            .map_err(|e| format!("写入 versions.json 失败: {}", e))?;
        zip.write_all(json.as_bytes())
            .map_err(|e| format!("写入 versions.json 失败: {}", e))?;
        exported_items.push("versions".to_string());
        log_info!("[DataExport] 已添加 versions.json ({} 条)", versions.len());
    }

    // 3. 导出 accounts.json（脱敏）
    if params.include_accounts {
        let accounts = export_accounts(state).await?;
        let json = serde_json::to_string_pretty(&accounts)
            .map_err(|e| format!("序列化 accounts 失败: {}", e))?;
        zip.start_file("accounts.json", options)
            .map_err(|e| format!("写入 accounts.json 失败: {}", e))?;
        zip.write_all(json.as_bytes())
            .map_err(|e| format!("写入 accounts.json 失败: {}", e))?;
        exported_items.push("accounts".to_string());
        log_info!(
            "[DataExport] 已添加 accounts.json (ms={}, offline={})",
            accounts.ms_accounts.len(),
            accounts.offline_accounts.len()
        );
    }

    zip.finish()
        .map_err(|e| format!("完成 zip 写入失败: {}", e))?;

    let file_size = std::fs::metadata(&output_path)
        .map(|m| m.len())
        .unwrap_or(0);

    log_info!(
        "[DataExport] 导出完成: {} ({} bytes, items={:?})",
        params.output_path,
        file_size,
        exported_items
    );

    let result = ExportResult {
        success: true,
        file_path: params.output_path,
        file_size,
        exported_items,
    };
    serde_json::to_value(&result).map_err(|e| e.to_string())
}

/// 收集已安装版本列表 + 类型
async fn export_versions_list(state: &AppState) -> Vec<ExportedVersion> {
    let game_dir = {
        let config = state.config.lock().await;
        resolve_game_dir(&config.game_dir)
    };

    let versions = version_scan::scan_installed_versions(&game_dir);
    if versions.is_empty() {
        return Vec::new();
    }

    let mut out = Vec::with_capacity(versions.len());
    for v in versions {
        let version_type = detect_version_type_from_dir(&game_dir, &v.id);
        let type_str = match version_type {
            crate::minecraft::version::state::VersionType::Release => "release",
            crate::minecraft::version::state::VersionType::Snapshot => "snapshot",
            crate::minecraft::version::state::VersionType::Old => "old",
            crate::minecraft::version::state::VersionType::Fool => "fool",
            crate::minecraft::version::state::VersionType::Forge => "forge",
            crate::minecraft::version::state::VersionType::NeoForge => "neoforge",
            crate::minecraft::version::state::VersionType::Fabric => "fabric",
            crate::minecraft::version::state::VersionType::Quilt => "quilt",
            crate::minecraft::version::state::VersionType::OptiFine => "optifine",
            crate::minecraft::version::state::VersionType::LiteLoader => "liteloader",
            crate::minecraft::version::state::VersionType::Unknown => "unknown",
        }
        .to_string();
        out.push(ExportedVersion {
            id: v.id,
            version_type: type_str,
        });
    }
    out
}

/// 从 auth_storage 读取账号并脱敏
async fn export_accounts(state: &AppState) -> Result<ExportedAccounts, String> {
    let persisted: PersistedAuthState = state
        .auth_storage
        .load()
        .await
        .map_err(log_err("加载 auth_storage 失败"))?;

    let current_user = persisted.current_user.as_ref().map(|u| ExportedCurrentUser {
        name: u.name.clone(),
        uuid: u.uuid.clone(),
        login_type: u.login_type.clone(),
    });

    let ms_accounts = persisted
        .ms_accounts
        .iter()
        .map(|a| ExportedMsAccount {
            username: a.username.clone(),
            uuid: a.uuid.clone(),
        })
        .collect();

    let offline_accounts = persisted
        .offline_accounts
        .iter()
        .map(|a| ExportedOfflineAccount {
            username: a.username.clone(),
            uuid: a.uuid.clone(),
        })
        .collect();

    // 软提示：当前用户为空时记录日志（不影响导出）
    if current_user.is_none() {
        log_warn!("[DataExport] 当前未登录用户，accounts.json 的 current_user 将为 null");
    }

    Ok(ExportedAccounts {
        current_user,
        ms_accounts,
        offline_accounts,
    })
}
