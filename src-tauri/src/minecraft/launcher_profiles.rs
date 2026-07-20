//! Launcher Profiles 管理模块
//!
//! 管理 Minecraft 的 launcher_profiles.json 文件

use crate::{log_info, log_warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// 固定的占位 ID
const AUTH_ACCOUNT_ID: &str = "00000111112222233333444445555566";
const AUTH_PROFILE_ID: &str = "66666555554444433333222221111100";
const DEFAULT_CLIENT_TOKEN: &str = "23323323323323323323323323323333";

/// 登录类型
#[derive(Debug, Clone, PartialEq)]
pub enum LoginType {
    Legacy,    // 离线
    Nide,      // 统一通行证
    Auth,      // Authlib-Injector
    Microsoft, // 正版微软
}

/// 登录结果
#[derive(Debug, Clone)]
pub struct LoginResult {
    pub name: String,
    pub uuid: String,
    pub access_token: String,
    pub login_type: LoginType,
    pub client_token: String,
}

/// Launcher Profiles 根结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LauncherProfiles {
    pub profiles: HashMap<String, Profile>,
    #[serde(rename = "selectedProfile")]
    pub selected_profile: String,
    #[serde(rename = "clientToken")]
    pub client_token: String,
    #[serde(
        rename = "authenticationDatabase",
        skip_serializing_if = "Option::is_none"
    )]
    pub authentication_database: Option<HashMap<String, AuthAccount>>,
    #[serde(rename = "selectedUser", skip_serializing_if = "Option::is_none")]
    pub selected_user: Option<SelectedUser>,
}

/// Profile 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub icon: String,
    pub name: String,
    #[serde(rename = "lastVersionId")]
    pub last_version_id: String,
    #[serde(rename = "type")]
    pub profile_type: String,
    #[serde(rename = "lastUsed")]
    pub last_used: String,
    #[serde(rename = "gameDir", skip_serializing_if = "Option::is_none")]
    pub game_dir: Option<String>,
    #[serde(rename = "javaArgs", skip_serializing_if = "Option::is_none")]
    pub java_args: Option<String>,
    #[serde(rename = "javaDir", skip_serializing_if = "Option::is_none")]
    pub java_dir: Option<String>,
}

/// 认证账号
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthAccount {
    pub username: String,
    pub profiles: HashMap<String, AuthProfile>,
    #[serde(rename = "accessToken", skip_serializing_if = "Option::is_none")]
    pub access_token: Option<String>,
}

/// 认证 Profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthProfile {
    #[serde(rename = "displayName")]
    pub display_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub legacy: Option<bool>,
}

/// 选中的用户
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectedUser {
    pub account: String,
    pub profile: String,
}

impl LauncherProfiles {
    /// 创建默认的 launcher_profiles.json
    pub fn create_default() -> Self {
        let now = chrono::Utc::now()
            .format("%Y-%m-%dT%H:%M:%S.0000Z")
            .to_string();
        let mut profiles = HashMap::new();
        profiles.insert(
            "MoLaunch".to_string(),
            Profile {
                icon: "Grass".to_string(),
                name: "MoLaunch".to_string(),
                last_version_id: "latest-release".to_string(),
                profile_type: "latest-release".to_string(),
                last_used: now,
                game_dir: None,
                java_args: None,
                java_dir: None,
            },
        );

        LauncherProfiles {
            profiles,
            selected_profile: "MoLaunch".to_string(),
            client_token: DEFAULT_CLIENT_TOKEN.to_string(),
            authentication_database: None,
            selected_user: None,
        }
    }

    /// 从文件加载，如果不存在或解析失败则创建
    pub fn load_or_create(mc_folder: &Path) -> Result<Self, String> {
        let path = mc_folder.join("launcher_profiles.json");
        if path.exists() {
            let content = std::fs::read_to_string(&path)
                .map_err(|e| format!("读取 launcher_profiles.json 失败: {}", e))?;
            match serde_json::from_str::<Self>(&content) {
                Ok(profiles) => return Ok(profiles),
                Err(e) => {
                    log_warn!("[Profiles] 解析失败，备份原文件: {}", e);
                    let bak_path = path.with_extension("json.bak");
                    let _ = std::fs::rename(&path, &bak_path);
                }
            }
        }
        let profiles = Self::create_default();
        profiles.save(mc_folder)?;
        log_info!(
            "[Profiles] Created launcher_profiles.json in {}",
            mc_folder.display()
        );
        Ok(profiles)
    }

    /// 保存到文件
    pub fn save(&self, mc_folder: &Path) -> Result<(), String> {
        let path = mc_folder.join("launcher_profiles.json");
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("序列化 launcher_profiles.json 失败: {}", e))?;
        std::fs::write(&path, content)
            .map_err(|e| format!("写入 launcher_profiles.json 失败: {}", e))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(metadata) = std::fs::metadata(&path) {
                let mut perms = metadata.permissions();
                perms.set_mode(0o600);
                let _ = std::fs::set_permissions(&path, perms);
            }
        }
        Ok(())
    }

    /// 游戏启动时更新认证信息
    /// 只有 Microsoft 正版账号才会写入 authenticationDatabase
    pub fn update_for_launch(
        &mut self,
        mc_folder: &Path,
        login_result: &LoginResult,
    ) -> Result<(), String> {
        // 核心判断：非 Microsoft 账号不更新
        if login_result.login_type != LoginType::Microsoft {
            return Ok(());
        }

        log_info!(
            "[Profiles] Updating for Microsoft account: {}",
            login_result.name
        );

        // 构建认证数据库
        let mut auth_profiles = HashMap::new();
        auth_profiles.insert(
            AUTH_PROFILE_ID.to_string(),
            AuthProfile {
                display_name: login_result.name.clone(),
                legacy: None,
            },
        );

        let mut auth_db = HashMap::new();
        auth_db.insert(
            AUTH_ACCOUNT_ID.to_string(),
            AuthAccount {
                username: login_result.name.replace('"', "-"),
                profiles: auth_profiles,
                access_token: None,
            },
        );

        self.authentication_database = Some(auth_db.clone());
        self.client_token = login_result.client_token.clone();
        self.selected_user = Some(SelectedUser {
            account: AUTH_ACCOUNT_ID.to_string(),
            profile: AUTH_PROFILE_ID.to_string(),
        });

        // 保存（带重试逻辑）
        match self.save(mc_folder) {
            Ok(()) => Ok(()),
            Err(e) => {
                log_warn!("[Profiles] 保存失败，备份后重试: {}", e);
                let path = mc_folder.join("launcher_profiles.json");
                let bak_path = path.with_extension("json.bak");
                let _ = std::fs::rename(&path, &bak_path);
                *self = Self::create_default();
                self.save(mc_folder)?;
                // 重新写入认证数据
                self.authentication_database = Some(auth_db);
                self.client_token = login_result.client_token.clone();
                self.selected_user = Some(SelectedUser {
                    account: AUTH_ACCOUNT_ID.to_string(),
                    profile: AUTH_PROFILE_ID.to_string(),
                });
                self.save(mc_folder)?;
                Ok(())
            }
        }
    }
}

/// 确保 launcher_profiles.json 存在（用于 Forge/NeoForge 安装前）
pub fn ensure_profiles_exist(mc_folder: &Path) -> Result<(), String> {
    let _ = LauncherProfiles::load_or_create(mc_folder)?;
    Ok(())
}
