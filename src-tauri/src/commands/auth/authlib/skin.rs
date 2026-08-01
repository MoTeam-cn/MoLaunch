//! authlib 皮肤/披风管理（查询 / 上传 / 删除）

// yggdrasil 皮肤管理命令（5 个）：统一从 `auth_storage.get_authlib_account`
// 取 access_token，调用 `authlib::client` 端点。token 过期返回 401 由前端提示重新登录。

use crate::error_util::log_err;
use crate::log_debug;
use crate::log_info;
use crate::minecraft::auth::authlib::{
    delete_cape, delete_skin, fetch_profile, parse_skin_cape_info, upload_cape, upload_skin,
    SkinCapeInfo,
};
use crate::state::AppState;

use super::helpers::{load_account_or_err, read_png_file};

/// 查询外置账号的皮肤/披风信息
///
/// 流程：
/// 1. 从 storage 读取账号（含 access_token）
/// 2. 调用 `fetch_profile` 拉取角色完整属性
/// 3. `parse_skin_cape_info` 解析 properties（textures + uploadableTextures）
///
/// 返回 `SkinCapeInfo`，前端据此显示皮肤/披风与动态启用上传按钮。
pub async fn authlib_get_skin_info(
    state: &AppState,
    server_url: String,
    uuid: String,
) -> Result<SkinCapeInfo, String> {
    log_debug!(
        "[Authlib] Get skin info: server={}, uuid={}",
        server_url,
        uuid
    );

    let account = load_account_or_err(state, &server_url, &uuid).await?;

    // fetch_profile 不需要 access_token（GET 公开端点），但保留 account 仅用于错误信息
    let profile = fetch_profile(&account.server_url, &account.uuid)
        .await
        .map_err(log_err("authlib_get_skin_info failed"))?;
    let info = parse_skin_cape_info(&profile);
    Ok(info)
}

/// 上传皮肤
///
/// `model` 取值：
/// - `"slim"` → Alex 模型（传 "slim"）
/// - `"default"` → Steve 模型（传空字符串）
///
/// 与 `save_custom_skin` / 微软 `upload_skin` 一致：传入本地文件路径，
/// 后端读取并校验 PNG 文件头与大小，避免前端引入 `@tauri-apps/plugin-fs` 依赖。
pub async fn authlib_upload_skin(
    state: &AppState,
    server_url: String,
    uuid: String,
    file_path: String,
    model: String,
) -> Result<(), String> {
    log_info!(
        "[Authlib] Upload skin: server={}, uuid={}, model={}, file={}",
        server_url,
        uuid,
        model,
        file_path
    );

    let account = load_account_or_err(state, &server_url, &uuid).await?;
    let png_bytes = read_png_file(&file_path).await?;
    // yggdrasil 规范：model 字段 "slim" 表示 Alex，空串表示 Steve
    let model_arg = if model == "slim" { "slim" } else { "" };
    upload_skin(
        &account.server_url,
        &account.access_token,
        &account.uuid,
        png_bytes,
        model_arg,
    )
    .await
    .map_err(log_err("authlib_upload_skin failed"))?;
    Ok(())
}

/// 删除皮肤
pub async fn authlib_delete_skin(
    state: &AppState,
    server_url: String,
    uuid: String,
) -> Result<(), String> {
    log_info!(
        "[Authlib] Delete skin: server={}, uuid={}",
        server_url,
        uuid
    );

    let account = load_account_or_err(state, &server_url, &uuid).await?;
    delete_skin(&account.server_url, &account.access_token, &account.uuid)
        .await
        .map_err(log_err("authlib_delete_skin failed"))?;
    Ok(())
}

/// 上传披风
///
/// 与 `authlib_upload_skin` 一致：传入本地文件路径，后端读取并校验。
pub async fn authlib_upload_cape(
    state: &AppState,
    server_url: String,
    uuid: String,
    file_path: String,
) -> Result<(), String> {
    log_info!(
        "[Authlib] Upload cape: server={}, uuid={}, file={}",
        server_url,
        uuid,
        file_path
    );

    let account = load_account_or_err(state, &server_url, &uuid).await?;
    let png_bytes = read_png_file(&file_path).await?;
    upload_cape(
        &account.server_url,
        &account.access_token,
        &account.uuid,
        png_bytes,
    )
    .await
    .map_err(log_err("authlib_upload_cape failed"))?;
    Ok(())
}

/// 删除披风
pub async fn authlib_delete_cape(
    state: &AppState,
    server_url: String,
    uuid: String,
) -> Result<(), String> {
    log_info!(
        "[Authlib] Delete cape: server={}, uuid={}",
        server_url,
        uuid
    );

    let account = load_account_or_err(state, &server_url, &uuid).await?;
    delete_cape(&account.server_url, &account.access_token, &account.uuid)
        .await
        .map_err(log_err("authlib_delete_cape failed"))?;
    Ok(())
}
