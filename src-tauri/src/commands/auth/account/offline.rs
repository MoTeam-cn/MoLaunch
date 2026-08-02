//! 离线账号管理命令（列表/皮肤设置/自定义皮肤上传/删除/切换）
//! `save_custom_skin` 将用户选择的 PNG 复制到 `<app_data>/custom_skins/<uuid>.png`，
//! 并把 `custom:<path>|<variant>` 写入离线账号 skin 字段。包含 PNG 文件头校验和
//! 1MB 大小限制（比 Mojang 官方 24KB 宽松，因本地使用）。
//! 已聚合为 `meta_manager` IPC 入口，由 `utils::meta_manager::dispatch` 分发调用。

use crate::error_util::log_err;
use crate::log_info;
use crate::minecraft::auth::storage::StoredOfflineAccount;
use crate::state::{AppState, LocalAuthResult};

use super::OfflineAccountInfo;
use super::super::authlib::helpers::read_png_file;

/// 获取已存储的离线账号列表
pub async fn get_offline_accounts(state: &AppState) -> Result<Vec<OfflineAccountInfo>, String> {
    log_info!("[Startup][IPC] get_offline_accounts called");
    let persisted = state
        .auth_storage
        .load()
        .await
        .map_err(log_err("Failed to load auth storage"))?;
    Ok(persisted
        .offline_accounts
        .iter()
        .map(|a| OfflineAccountInfo {
            username: a.username.clone(),
            uuid: a.uuid.clone(),
            skin: a.skin.clone(),
        })
        .collect())
}

/// 设置离线账号的皮肤选择
pub async fn set_offline_skin(
    state: &AppState,
    uuid: String,
    skin: Option<String>,
) -> Result<(), String> {
    log_info!("Setting offline skin: uuid={}, skin={:?}", uuid, skin);
    state
        .auth_storage
        .set_offline_skin(&uuid, skin.as_deref())
        .await
        .map_err(log_err("Failed to set offline skin"))
}

/// 保存自定义皮肤文件并设置到离线账号
///
/// 将用户选择的 PNG 文件复制到 `<app_data>/custom_skins/<uuid>.png`，
/// 然后把 `custom:<path>|<variant>` 写入离线账号的 skin 字段。
pub async fn save_custom_skin(
    state: &AppState,
    uuid: String,
    file_path: String,
    variant: Option<String>,
) -> Result<String, String> {
    let variant = variant.unwrap_or_else(|| "classic".to_string());

    // 读取源文件并校验（PNG 文件头 + 1MB 大小限制，复用 authlib 的 read_png_file）
    let png_data = read_png_file(&file_path).await?;

    // 保存到 app data 目录
    let skin_dir = crate::storage::Storage::instance()
        .base_dir()
        .join("custom_skins");
    std::fs::create_dir_all(&skin_dir).map_err(|e| format!("创建皮肤目录失败: {}", e))?;

    let dest_path = skin_dir.join(format!("{}.png", uuid));
    std::fs::write(&dest_path, &png_data).map_err(|e| format!("保存皮肤文件失败: {}", e))?;

    // 构建 skin 字段：custom:/path|slim 或 custom:/path|classic
    let skin_value = format!("custom:{}|{}", dest_path.display(), variant);

    // 写入注册表
    state
        .auth_storage
        .set_offline_skin(&uuid, Some(&skin_value))
        .await
        .map_err(log_err("Failed to set offline skin"))?;

    log_info!(
        "Saved custom skin: uuid={}, file={}, variant={}",
        uuid,
        dest_path.display(),
        variant
    );

    Ok(skin_value)
}

/// 删除已存储的离线账号
pub async fn remove_offline_account(state: &AppState, uuid: String) -> Result<(), String> {
    log_info!("Removing offline account: {}", uuid);
    state
        .auth_storage
        .remove_offline_account(&uuid)
        .await
        .map_err(log_err("Failed to remove offline account"))
}

/// 切换到已存储的离线账号
pub async fn switch_offline_account(
    state: &AppState,
    uuid: String,
) -> Result<LocalAuthResult, String> {
    log_info!("Switching to offline account: {}", uuid);

    let account: StoredOfflineAccount = state
        .auth_storage
        .get_offline_account(&uuid)
        .await
        .map_err(log_err("Failed to get offline account"))?
        .ok_or_else(|| "离线账号不存在".to_string())?;

    let auth_result = LocalAuthResult {
        name: account.username.clone(),
        uuid: account.uuid.clone(),
        access_token: account.uuid.clone(),
        client_token: account.uuid.clone(),
        login_type: "Legacy".to_string(),
        profile_json: None,
        server_url: None,
        server_name: None,
    };

    // 更新当前用户（持久化）
    {
        let mut persisted = state
            .auth_storage
            .load()
            .await
            .map_err(log_err("Failed to load auth storage"))?;
        persisted.current_user = Some(crate::minecraft::auth::storage::CurrentUser {
            name: account.username.clone(),
            uuid: account.uuid.clone(),
            access_token: account.uuid.clone(),
            client_token: account.uuid.clone(),
            login_type: "Legacy".to_string(),
            profile_json: None,
            refresh_token: None,
            expires_at: None,
            server_url: None,
            server_name: None,
        });
        state
            .auth_storage
            .save(&persisted)
            .await
            .map_err(log_err("Failed to save auth storage"))?;
    }

    {
        let mut auth = state.auth.lock().await;
        auth.current_user = Some(auth_result.clone());
        auth.is_logged_in = true;
    }

    log_info!("Switched to offline account: {}", account.username);
    Ok(auth_result)
}
