//! 微软 / 离线账号高层操作（独立 impl 块）

use super::super::super::microsoft::MicrosoftLoginResult;
use super::super::types::{CurrentUser, StoredMsAccount, StoredOfflineAccount};
use super::super::AuthStorage;

impl AuthStorage {
    /// 保存微软登录结果并设为当前用户
    pub async fn save_ms_login(&self, result: &MicrosoftLoginResult) -> Result<(), String> {
        let mut state = self.load().await.unwrap_or_default();

        // 更新或添加到微软账号列表
        let account = StoredMsAccount::from(result);
        if let Some(existing) = state
            .ms_accounts
            .iter_mut()
            .find(|a| a.uuid == account.uuid)
        {
            *existing = account.clone();
        } else {
            state.ms_accounts.push(account.clone());
        }

        // 设为当前用户
        state.current_user = Some(CurrentUser {
            name: result.username.clone(),
            uuid: result.uuid.clone(),
            access_token: result.access_token.clone(),
            client_token: String::new(),
            login_type: "Microsoft".to_string(),
            profile_json: Some(result.profile_json.clone()),
            refresh_token: Some(result.refresh_token.clone()),
            expires_at: Some(result.expires_at),
            server_url: None,
            server_name: None,
        });

        self.save(&state).await
    }

    /// 保存离线登录并设为当前用户
    ///
    /// 同时把账号添加到离线账号列表（UUID 去重），保留已有的 skin 选择。
    pub async fn save_offline_login(&self, username: &str, uuid: &str) -> Result<(), String> {
        let mut state = self.load().await.unwrap_or_default();

        // 添加到离线账号列表（UUID 去重，保留已有的 skin 选择）
        let account = StoredOfflineAccount {
            username: username.to_string(),
            uuid: uuid.to_string(),
            skin: state
                .offline_accounts
                .iter()
                .find(|a| a.uuid == uuid)
                .and_then(|a| a.skin.clone()),
        };
        if let Some(existing) = state
            .offline_accounts
            .iter_mut()
            .find(|a| a.uuid == account.uuid)
        {
            *existing = account.clone();
        } else {
            state.offline_accounts.push(account);
        }

        state.current_user = Some(CurrentUser {
            name: username.to_string(),
            uuid: uuid.to_string(),
            access_token: uuid.to_string(),
            client_token: uuid.to_string(),
            login_type: "Legacy".to_string(),
            profile_json: None,
            refresh_token: None,
            expires_at: None,
            server_url: None,
            server_name: None,
        });

        self.save(&state).await
    }

    /// 设置离线账号的皮肤选择
    pub async fn set_offline_skin(&self, uuid: &str, skin: Option<&str>) -> Result<(), String> {
        let mut state = self.load().await.unwrap_or_default();
        if let Some(account) = state.offline_accounts.iter_mut().find(|a| a.uuid == uuid) {
            account.skin = skin.map(|s| s.to_string());
            self.save(&state).await
        } else {
            Err("离线账号不存在".to_string())
        }
    }

    /// 删除指定离线账号
    pub async fn remove_offline_account(&self, uuid: &str) -> Result<(), String> {
        let mut state = self.load().await.unwrap_or_default();
        state.offline_accounts.retain(|a| a.uuid != uuid);

        // 如果删除的是当前用户，也清除当前用户
        if let Some(ref current) = state.current_user {
            if current.uuid == uuid {
                state.current_user = None;
            }
        }

        self.save(&state).await
    }

    /// 获取指定离线账号
    pub async fn get_offline_account(
        &self,
        uuid: &str,
    ) -> Result<Option<StoredOfflineAccount>, String> {
        let state = self.load().await?;
        Ok(state.offline_accounts.into_iter().find(|a| a.uuid == uuid))
    }

    /// 清除当前用户（登出）
    pub async fn clear_current_user(&self) -> Result<(), String> {
        let mut state = self.load().await.unwrap_or_default();
        state.current_user = None;
        self.save(&state).await
    }

    /// 删除指定微软账号
    pub async fn remove_ms_account(&self, uuid: &str) -> Result<(), String> {
        let mut state = self.load().await.unwrap_or_default();
        state.ms_accounts.retain(|a| a.uuid != uuid);

        // 如果删除的是当前用户，也清除当前用户
        if let Some(ref current) = state.current_user {
            if current.uuid == uuid {
                state.current_user = None;
            }
        }

        self.save(&state).await
    }

    /// 获取指定微软账号（用于刷新）
    pub async fn get_ms_account(&self, uuid: &str) -> Result<Option<StoredMsAccount>, String> {
        let state = self.load().await?;
        Ok(state.ms_accounts.into_iter().find(|a| a.uuid == uuid))
    }

    /// 获取当前微软账号的 refresh_token（用于静默刷新）
    pub async fn get_current_refresh_token(&self) -> Result<Option<String>, String> {
        let state = self.load().await?;
        match state.current_user {
            Some(ref user) if user.login_type == "Microsoft" => Ok(user.refresh_token.clone()),
            _ => Ok(None),
        }
    }

    /// 更新微软账号的 Token（刷新后调用）
    pub async fn update_ms_token(
        &self,
        uuid: &str,
        access_token: &str,
        refresh_token: &str,
        expires_at: u64,
    ) -> Result<(), String> {
        let mut state = self.load().await.unwrap_or_default();

        // 更新账号列表
        for account in state.ms_accounts.iter_mut() {
            if account.uuid == uuid {
                account.access_token = access_token.to_string();
                account.refresh_token = refresh_token.to_string();
                account.expires_at = expires_at;
                break;
            }
        }

        // 更新当前用户
        if let Some(ref mut current) = state.current_user {
            if current.uuid == uuid {
                current.access_token = access_token.to_string();
                current.refresh_token = Some(refresh_token.to_string());
                current.expires_at = Some(expires_at);
            }
        }

        self.save(&state).await
    }
}