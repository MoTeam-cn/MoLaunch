//! authlib-injector 外置登录命令（yggdrasil 协议）
//!
//! 命令清单：
//! - `authlib_fetch_server_meta`     获取服务器元数据（登录页展示服务器名/注册链接）
//! - `authlib_login`                 账号密码登录（返回 Success 或 NeedSelect）
//! - `authlib_select_profile`        多角色场景下选定 profile 完成登录
//! - `authlib_refresh`               切换账号时验证/刷新 token（三步降级）
//! - `get_authlib_accounts`          已保存账号列表
//! - `remove_authlib_account`        删除指定账号
//! - `switch_authlib_account`        切换到已保存账号
//! - `authlib_get_skin_info`         查询外置账号皮肤/披风信息（含 uploadableTextures）
//! - `authlib_upload_skin`           上传皮肤 PNG（multipart/form-data）
//! - `authlib_delete_skin`           删除皮肤
//! - `authlib_upload_cape`           上传披风 PNG
//! - `authlib_delete_cape`           删除披风
//!
//! 设计要点：
//! - 多角色（available_profiles > 1）：首次登录返回 `NeedSelect`，前端弹窗选择后调用
//!   `authlib_select_profile` 用 refresh 指定 selected_profile 完成登录。
//! - 切换已保存账号：调用 `authlib_refresh` 走 validate → refresh → 用密码重登 的三步降级，
//!   任何一步成功即返回 `LocalAuthResult`，全部失败则返回错误。
//! - 服务器元数据缓存：前端登录页输入 server_url 后实时拉取，用于显示服务器名和注册链接。
//! - 皮肤命令统一从 `state.auth_storage.get_authlib_account` 取 access_token，
//!   调用 `authlib::client` 的 5 个 yggdrasil 皮肤端点。本次实现不处理 token 过期自动刷新，
//!   如果用户报告 401 后再补 refresh 逻辑（保持最小修改）。
//!
//! 注：原 `#[tauri::command]` 标注已移除，函数改为接收 `&AppState`，
//! 由 `commands::auth::meta_manager` 统一 IPC 入口通过
//! `utils::meta_manager::dispatch` 分发调用。

use serde::Serialize;

use crate::error_util::log_err;
use crate::{log_debug, log_info};
use crate::log_warn;
use crate::minecraft::auth::authlib::{
    delete_cape, delete_skin, fetch_profile, fetch_server_metadata, login_with_cached_token,
    login_with_password, parse_skin_cape_info, refresh_with_profile, upload_cape, upload_skin,
    LoginOutcome, Profile, ServerMetadata, SkinCapeInfo,
};
use crate::minecraft::auth::storage::StoredAuthlibAccount;
use crate::state::{AppState, LocalAuthResult};

// ============================================================
// 前端可见的数据类型
// ============================================================

/// authlib 登录结果
///
/// - `Success`：单角色或服务器已选定角色，含可直接使用的 `LocalAuthResult`
/// - `NeedSelect`：多角色且无 selected_profile，前端需弹窗让用户选择
///
/// 安全说明：`NeedSelect` 中的 `access_token` / `client_token` 标记 `#[serde(skip)]`，
/// 不会序列化到 IPC 返回前端。前端选定 profile 后调用 `authlib_select_profile`，
/// 后端从 `state.authlib_pending`（内存暂存）取出 token 完成刷新，不依赖前端回传。
#[derive(Debug, Serialize)]
#[serde(tag = "status")]
pub enum AuthlibLoginResult {
    #[serde(rename = "success")]
    Success { user: LocalAuthResult },
    #[serde(rename = "need_select")]
    NeedSelect {
        #[serde(skip)]
        access_token: String,
        #[serde(skip)]
        client_token: String,
        available_profiles: Vec<Profile>,
    },
}

/// 已保存的 authlib 账号信息（前端列表展示用）
#[derive(Debug, Clone, Serialize)]
pub struct AuthlibAccountInfo {
    /// 登录账号（邮箱或用户名）
    pub username: String,
    /// 选中的角色 UUID
    pub uuid: String,
    /// 选中的角色名
    pub player_name: String,
    /// yggdrasil API 根地址
    pub server_url: String,
    /// 服务器显示名
    pub server_name: String,
}

/// 服务器元数据（前端登录页展示用）
#[derive(Debug, Serialize)]
pub struct AuthlibServerMeta {
    /// 服务器名（从 meta.serverName 提取）
    pub server_name: String,
    /// 注册链接（从 meta.links.register 提取）
    pub register_url: Option<String>,
    /// 主页链接（从 meta.links.homepage 提取）
    pub homepage_url: Option<String>,
}

impl From<ServerMetadata> for AuthlibServerMeta {
    fn from(meta: ServerMetadata) -> Self {
        Self {
            server_name: meta.server_name(),
            register_url: meta.register_url(),
            homepage_url: meta.homepage_url(),
        }
    }
}

// ============================================================
// 命令实现
// ============================================================

/// 获取 yggdrasil 服务器元数据
///
/// 前端登录页输入 server_url 后调用，用于显示服务器名/注册链接。
/// 失败时返回错误（前端提示用户检查地址或网络）。
pub async fn authlib_fetch_server_meta(
    server_url: String,
) -> Result<AuthlibServerMeta, String> {
    log_info!("[Authlib] Fetching server metadata: {}", server_url);
    let meta = fetch_server_metadata(&server_url)
        .await
        .map_err(|e| e.to_string())?;
    Ok(AuthlibServerMeta::from(meta))
}

/// 账号密码登录
///
/// 流程：
/// 1. 调用 `login_with_password` 走 yggdrasil `/authserver/authenticate`
/// 2. 单角色或服务器已选定 → 返回 `Success`，前端拿到 `LocalAuthResult` 直接登录
/// 3. 多角色 → 返回 `NeedSelect`，前端弹窗让用户选择，再调用 `authlib_select_profile`
///
/// `password` 会随账号一起持久化（加密），用于 token 失效后自动重新登录。
pub async fn authlib_login(
    state: &AppState,
    server_url: String,
    username: String,
    password: String,
) -> Result<AuthlibLoginResult, String> {
    log_info!(
        "[Authlib] Login attempt: server={}, user={}",
        server_url,
        username
    );

    // 拉取服务器元数据（用于服务器显示名）
    let server_name = match fetch_server_metadata(&server_url).await {
        Ok(meta) => meta.server_name(),
        Err(e) => {
            log_warn!("[Authlib] 获取服务器元数据失败，使用占位名: {}", e);
            "未知服务器".to_string()
        }
    };

    let outcome = login_with_password(&server_url, &username, &password)
        .await
        .map_err(log_err("authlib login failed"))?;

    match outcome {
        LoginOutcome::Success(_) => {
            // 单角色，直接持久化
            let current = state
                .auth_storage
                .save_authlib_login(&server_url, &server_name, &username, &password, &outcome)
                .await
                .map_err(log_err("Failed to persist authlib login"))?;

            let user = LocalAuthResult {
                name: current.name,
                uuid: current.uuid,
                access_token: current.access_token,
                client_token: current.client_token,
                login_type: "AuthlibInjector".to_string(),
                profile_json: None,
                server_url: current.server_url,
                server_name: current.server_name,
            };

            // 同步内存状态
            {
                let mut auth_state = state.auth.lock().await;
                auth_state.current_user = Some(user.clone());
                auth_state.is_logged_in = true;
            }

            log_info!("[Authlib] Login success: {}", user.name);
            Ok(AuthlibLoginResult::Success { user })
        }
        LoginOutcome::NeedSelect {
            access_token,
            client_token,
            available_profiles,
        } => {
            // 多角色：不持久化，等前端选定后调用 authlib_select_profile
            // 但需要把 username/password/server_url 暂存到内存，select_profile 时取出
            log_info!(
                "[Authlib] Multi-profile detected, need select: count={}",
                available_profiles.len()
            );
            let mut pending = state.authlib_pending.lock().await;
            *pending = Some(PendingAuthlibLogin {
                server_url: server_url.clone(),
                server_name: server_name.clone(),
                username: username.clone(),
                password: password.clone(),
                access_token: access_token.clone(),
                client_token: client_token.clone(),
            });
            Ok(AuthlibLoginResult::NeedSelect {
                access_token,
                client_token,
                available_profiles,
            })
        }
    }
}

/// 多角色场景下选定 profile 完成登录
///
/// 前端拿到 `NeedSelect` 后弹窗让用户选择 profile，选定后调用此命令。
/// 内部调用 yggdrasil `/authserver/refresh` 指定 selected_profile，
/// 成功后持久化账号并设为当前用户。
pub async fn authlib_select_profile(
    state: &AppState,
    profile: Profile,
) -> Result<LocalAuthResult, String> {
    log_info!("[Authlib] Selecting profile: id={}, name={}", profile.id, profile.name);

    let pending = {
        let mut pending_lock = state.authlib_pending.lock().await;
        pending_lock
            .take()
            .ok_or_else(|| "没有待处理的 authlib 登录，请重新登录".to_string())?
    };

    let resp = refresh_with_profile(
        &pending.server_url,
        &pending.access_token,
        &pending.client_token,
        profile.clone(),
    )
    .await
    .map_err(log_err("authlib select_profile failed"))?;

    // 构造 LoginOutcome::Success 让 save_authlib_login 处理持久化
    let outcome = LoginOutcome::Success(resp);
    let current = state
        .auth_storage
        .save_authlib_login(
            &pending.server_url,
            &pending.server_name,
            &pending.username,
            &pending.password,
            &outcome,
        )
        .await
        .map_err(log_err("Failed to persist authlib login after select"))?;

    let user = LocalAuthResult {
        name: current.name,
        uuid: current.uuid,
        access_token: current.access_token,
        client_token: current.client_token,
        login_type: "AuthlibInjector".to_string(),
        profile_json: None,
        server_url: current.server_url,
        server_name: current.server_name,
    };

    {
        let mut auth_state = state.auth.lock().await;
        auth_state.current_user = Some(user.clone());
        auth_state.is_logged_in = true;
    }

    log_info!("[Authlib] Profile selected, login success: {}", user.name);
    Ok(user)
}

/// 切换到已保存的 authlib 账号（三步降级：validate → refresh → 用密码重登）
///
/// 调用方仅传 `server_url` + `uuid`，内部从持久化存储读取账号信息。
/// 任何一步成功即返回 `LocalAuthResult`，全部失败则返回错误。
pub async fn switch_authlib_account(
    state: &AppState,
    server_url: String,
    uuid: String,
) -> Result<LocalAuthResult, String> {
    log_info!("[Authlib] Switching account: server={}, uuid={}", server_url, uuid);

    let account = state
        .auth_storage
        .get_authlib_account(&server_url, &uuid)
        .await
        .map_err(log_err("Failed to load authlib account"))?
        .ok_or_else(|| "账号不存在".to_string())?;

    let cached_profile = Profile {
        id: account.uuid.clone(),
        name: account.player_name.clone(),
    };

    // 三步降级
    let outcome = login_with_cached_token(
        &account.server_url,
        &account.access_token,
        &account.client_token,
        Some(&cached_profile),
    )
    .await;

    let (access_token, client_token) = match outcome {
        Ok(LoginOutcome::Success(resp)) => {
            // 验证或刷新成功，token 可能已更新
            let new_access = resp.access_token.clone();
            let new_client = resp.client_token.clone();
            // 如果 token 变化，更新持久化存储
            if new_access != account.access_token || new_client != account.client_token {
                if let Err(e) = state
                    .auth_storage
                    .update_authlib_token(&server_url, &uuid, &new_access, &new_client)
                    .await
                {
                    log_warn!("[Authlib] 更新 token 失败: {}", e);
                }
            }
            (new_access, new_client)
        }
        Ok(LoginOutcome::NeedSelect { .. }) => {
            // 已有缓存 profile 但服务器要求重选，理论不应发生，回退到密码重登
            log_warn!("[Authlib] validate/refresh 返回 NeedSelect，回退到密码登录");
            authlib_relogin_with_password(state, &account).await?
        }
        Err(e) if e.is_network => {
            return Err(format!("网络错误，无法切换账号: {}", e));
        }
        Err(_) => {
            // token 完全失效，用密码重新登录
            log_info!("[Authlib] token 完全失效，用密码重新登录");
            authlib_relogin_with_password(state, &account).await?
        }
    };

    // 更新当前用户
    let current = state
        .auth_storage
        .switch_authlib_account(&server_url, &uuid)
        .await
        .map_err(log_err("Failed to switch authlib account"))?
        .ok_or_else(|| "切换账号失败：账号不存在".to_string())?;

    // 用最新的 token 覆盖（switch_authlib_account 用的是持久化的 token，
    // 但我们刚刚可能通过密码重登拿到了新 token，需要覆盖）
    let user = LocalAuthResult {
        name: current.name,
        uuid: current.uuid,
        access_token,
        client_token,
        login_type: "AuthlibInjector".to_string(),
        profile_json: None,
        server_url: current.server_url,
        server_name: current.server_name,
    };

    {
        let mut auth_state = state.auth.lock().await;
        auth_state.current_user = Some(user.clone());
        auth_state.is_logged_in = true;
    }

    log_info!("[Authlib] Switched to account: {}", user.name);
    Ok(user)
}

/// 用账号密码重新登录（token 完全失效时的兜底）
///
/// 登录成功后更新持久化的 token，并返回新的 access_token + client_token。
async fn authlib_relogin_with_password(
    state: &AppState,
    account: &StoredAuthlibAccount,
) -> Result<(String, String), String> {
    let outcome = login_with_password(&account.server_url, &account.username, &account.password)
        .await
        .map_err(log_err("authlib password relogin failed"))?;

    match outcome {
        LoginOutcome::Success(resp) => {
            let new_access = resp.access_token.clone();
            let new_client = resp.client_token.clone();
            // 检查 selected_profile 是否与缓存一致，不一致则警告（不强制更新）
            if let Some(ref profile) = resp.selected_profile {
                if profile.id != account.uuid {
                    log_warn!(
                        "[Authlib] 重新登录后角色变化: old={}, new={}",
                        account.uuid,
                        profile.id
                    );
                }
            }
            // 更新持久化的 token
            if let Err(e) = state
                .auth_storage
                .update_authlib_token(
                    &account.server_url,
                    &account.uuid,
                    &new_access,
                    &new_client,
                )
                .await
            {
                log_warn!("[Authlib] 更新 token 失败: {}", e);
            }
            Ok((new_access, new_client))
        }
        LoginOutcome::NeedSelect { .. } => {
            Err("账号密码登录后需要重新选择角色，请重新登录".to_string())
        }
    }
}

/// 获取已保存的 authlib 账号列表
pub async fn get_authlib_accounts(
    state: &AppState,
) -> Result<Vec<AuthlibAccountInfo>, String> {
    let persisted = state
        .auth_storage
        .load()
        .await
        .map_err(log_err("Failed to load auth storage"))?;
    Ok(persisted
        .authlib_accounts
        .iter()
        .map(|a| AuthlibAccountInfo {
            username: a.username.clone(),
            uuid: a.uuid.clone(),
            player_name: a.player_name.clone(),
            server_url: a.server_url.clone(),
            server_name: a.server_name.clone(),
        })
        .collect())
}

/// 删除指定 authlib 账号
pub async fn remove_authlib_account(
    state: &AppState,
    server_url: String,
    uuid: String,
) -> Result<(), String> {
    log_info!("[Authlib] Removing account: server={}, uuid={}", server_url, uuid);
    state
        .auth_storage
        .remove_authlib_account(&server_url, &uuid)
        .await
        .map_err(log_err("Failed to remove authlib account"))
}

// ============================================================
// yggdrasil 皮肤管理命令（5 个，参考 yggdrasil-api-analysis.md 4.3/4.4 节）
//
// 统一从 `auth_storage.get_authlib_account` 取 access_token，
// 调用 `authlib::client` 的对应端点。token 过期返回 401 时由前端提示用户重新登录。
// ============================================================

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
    log_debug!("[Authlib] Get skin info: server={}, uuid={}", server_url, uuid);

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
    upload_skin(&account.server_url, &account.access_token, &account.uuid, png_bytes, model_arg)
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
    log_info!("[Authlib] Delete skin: server={}, uuid={}", server_url, uuid);

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
    log_info!("[Authlib] Delete cape: server={}, uuid={}", server_url, uuid);

    let account = load_account_or_err(state, &server_url, &uuid).await?;
    delete_cape(&account.server_url, &account.access_token, &account.uuid)
        .await
        .map_err(log_err("authlib_delete_cape failed"))?;
    Ok(())
}

/// 从持久化存储读取 authlib 账号
///
/// 失败返回友好错误信息（前端可直接 toast 显示）。
/// 多处复用：5 个皮肤命令都需要先取账号再调用 client。
async fn load_account_or_err(
    state: &AppState,
    server_url: &str,
    uuid: &str,
) -> Result<StoredAuthlibAccount, String> {
    state
        .auth_storage
        .get_authlib_account(server_url, uuid)
        .await
        .map_err(log_err("Failed to load authlib account"))?
        .ok_or_else(|| "authlib 账号不存在".to_string())
}

/// 读取 PNG 文件并校验文件头与大小
///
/// 与 `commands::auth::account::offline::save_custom_skin` 中的校验保持一致：
/// - PNG 文件头：`89 50 4E 47 0D`（5 字节）
/// - 大小限制：1MB（yggdrasil 服务端通常自行限制，这里仅做客户端预校验）
///
/// 在 `spawn_blocking` 中执行 `std::fs::read`，避免阻塞异步运行时
/// （与微软 `upload_skin` 命令的处理方式一致）。
async fn read_png_file(file_path: &str) -> Result<Vec<u8>, String> {
    let path = file_path.to_string();
    tokio::task::spawn_blocking(move || {
        let bytes = std::fs::read(&path).map_err(|e| format!("读取皮肤文件失败: {}", e))?;
        if bytes.len() < 8 || bytes[0..5] != [0x89, 0x50, 0x4E, 0x47, 0x0D] {
            return Err("文件不是有效的 PNG 格式".to_string());
        }
        if bytes.len() > 1024 * 1024 {
            return Err("皮肤文件过大（超过 1MB）".to_string());
        }
        Ok(bytes)
    })
    .await
    .map_err(|e| format!("读取文件任务失败: {}", e))?
}

// ============================================================
// AppState 扩展：暂存多角色登录上下文
// ============================================================

/// 多角色登录的待处理上下文
///
/// `authlib_login` 返回 `NeedSelect` 时暂存到 AppState，
/// 前端选定 profile 后 `authlib_select_profile` 取出使用。
#[derive(Debug, Clone)]
pub struct PendingAuthlibLogin {
    pub server_url: String,
    pub server_name: String,
    pub username: String,
    pub password: String,
    pub access_token: String,
    pub client_token: String,
}
