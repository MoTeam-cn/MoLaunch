//! 版本个性化设置读写

use crate::minecraft::version::setup::{PersonalizationUpdate, VersionSetup};
use crate::state::AppState;
use crate::{log_error, log_info};
use serde::{Deserialize, Serialize};
use tauri::State;

use super::sanitize_version_id;
use super::list::version_type_to_string;

/// 版本个性化信息（返回给前端）
///
/// 注意：与 `PersonalizationUpdate` 保持一致，使用 camelCase 序列化，
/// 前端无需 snakeMap 转换即可直接访问（如 `p.windowTitle`）。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionPersonalization {
    pub logo: String,
    pub custom_info: String,
    pub display_type: i32,
    pub is_star: bool,
    pub indie_type: i32,
    pub version_type: String,
    pub original_version: String,
    /// 游戏窗口标题（空=跟随全局）
    pub window_title: String,
    /// 自动进入服务器（"IP:Port"，空=不自动进入）
    pub server_enter: String,
    /// 额外 JVM 参数（空=跟随全局）
    pub advance_jvm_args: String,
    /// 额外游戏参数（空=跟随全局）
    pub advance_game_args: String,
    /// 启动前执行命令（空=跟随全局）
    pub advance_run_cmd: String,
    /// 版本独立 Java 路径（仅 JavaMode="custom" 时生效）
    pub java_path: String,
    /// Java 选择模式：空/auto=自动选择, "auto_version"=自动选择指定版本范围, "folder"=使用版本文件夹中的 Java, "custom"=使用指定的 Java
    pub java_mode: String,
    /// 自动选择时的最小 Java 主版本（仅 auto_version 模式生效，0=不限）
    pub java_version_min: u32,
    /// 自动选择时的最大 Java 主版本（仅 auto_version 模式生效，0=不限）
    pub java_version_max: u32,
    /// 内存模式（空=跟随全局, "auto"=自动, "custom"=自定义）
    pub memory_mode: String,
    /// 版本独立最小内存（MB，仅 custom 模式生效）
    pub min_memory: u32,
    /// 版本独立最大内存（MB，仅 custom 模式生效）
    pub max_memory: u32,
    // ===== 高级选项开关 =====
    /// 禁止更新 Mod
    pub advance_disable_mod_update: bool,
    /// 忽略 Java 兼容性警告
    pub advance_ignore_java_warning: bool,
    /// 关闭文件校验
    pub advance_disable_assets_verify: bool,
    /// 禁用 Java Launch Wrapper
    pub advance_disable_jlw: bool,
    /// 禁用 LWJGL Unsafe Agent
    pub advance_disable_lua: bool,
}

/// 获取版本个性化设置
#[tauri::command]
pub async fn get_version_personalization(
    state: State<'_, AppState>,
    version_id: String,
) -> Result<VersionPersonalization, String> {
    sanitize_version_id(&version_id)?;

    let game_dir = crate::state::resolve_game_dir_from_state(&state).await;

    let version_dir = game_dir.join("versions").join(&version_id);
    let setup = VersionSetup::load_or_create(&version_dir, &version_id);

    Ok(VersionPersonalization {
        logo: setup.logo.unwrap_or_default(),
        custom_info: setup.custom_info.unwrap_or_default(),
        display_type: setup.display_type.unwrap_or(0),
        is_star: setup.is_star.unwrap_or(false),
        indie_type: setup.indie_type.unwrap_or(0),
        version_type: version_type_to_string(&setup.version_type),
        original_version: setup.original_version,
        window_title: setup.window_title.unwrap_or_default(),
        server_enter: setup.server_enter.unwrap_or_default(),
        advance_jvm_args: setup.advance_jvm_args.unwrap_or_default(),
        advance_game_args: setup.advance_game_args.unwrap_or_default(),
        advance_run_cmd: setup.advance_run_cmd.unwrap_or_default(),
        java_path: setup.java_path.unwrap_or_default(),
        java_mode: setup.java_mode.unwrap_or_default(),
        java_version_min: setup.java_version_min.unwrap_or(0),
        java_version_max: setup.java_version_max.unwrap_or(0),
        memory_mode: setup.memory_mode.unwrap_or_default(),
        min_memory: setup.min_memory.unwrap_or(0),
        max_memory: setup.max_memory.unwrap_or(0),
        advance_disable_mod_update: setup.advance_disable_mod_update.unwrap_or(false),
        advance_ignore_java_warning: setup.advance_ignore_java_warning.unwrap_or(false),
        advance_disable_assets_verify: setup.advance_disable_assets_verify.unwrap_or(false),
        advance_disable_jlw: setup.advance_disable_jlw.unwrap_or(false),
        advance_disable_lua: setup.advance_disable_lua.unwrap_or(false),
    })
}

/// 更新版本个性化字段（传 null/undefined 的字段不会被修改）
#[tauri::command]
pub async fn update_version_personalization(
    state: State<'_, AppState>,
    version_id: String,
    update: PersonalizationUpdate,
) -> Result<(), String> {
    sanitize_version_id(&version_id)?;
    log_info!("Updating personalization for version: {}", version_id);

    let game_dir = crate::state::resolve_game_dir_from_state(&state).await;

    let version_dir = game_dir.join("versions").join(&version_id);
    VersionSetup::update_personalization(&version_dir, &update).map_err(|e| {
        log_error!("Failed to update personalization: {}", e);
        e.to_string()
    })?;

    log_info!("Personalization updated for version: {}", version_id);
    Ok(())
}
