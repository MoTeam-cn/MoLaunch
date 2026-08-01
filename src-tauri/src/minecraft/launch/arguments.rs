//! Launch arguments orchestration
//!
//! 编排 classpath / jvm_args / game_args 子模块，构建完整启动参数。

use crate::log_info;
use crate::minecraft::isolation::{self, IsolationMode};
use crate::minecraft::version::{setup::VersionSetup, state::VersionType};
use std::path::Path;

use super::classpath::build_classpath;
use super::game_args::build_game_args;
use super::jvm_args::build_jvm_args;
use super::{AuthInfo, LaunchArguments};

#[allow(clippy::too_many_arguments)]
/// Build launch arguments with isolation support
pub fn build_launch_arguments(
    game_dir: &Path,
    version_id: &str,
    java_path: &Path,
    auth_info: &AuthInfo,
    min_memory: u32,
    max_memory: u32,
    window_width: Option<u32>,
    window_height: Option<u32>,
    server_address: Option<&str>,
    server_port: Option<u32>,
    isolation_mode: u32,
    extra_jvm_args: &[String],
    extra_game_args: &[String],
    disable_jlw: bool,
    disable_lua: bool,
    custom_info: Option<&str>,
    game_language: Option<&str>,
) -> anyhow::Result<LaunchArguments> {
    let version_dir = game_dir.join("versions").join(version_id);
    let json_path = version_dir.join(format!("{}.json", version_id));

    if !json_path.exists() {
        return Err(anyhow::anyhow!("Version {} not found", version_id));
    }

    let json_content = std::fs::read_to_string(&json_path)?;
    let json: serde_json::Value = serde_json::from_str(&json_content)?;

    let main_class = json["mainClass"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("mainClass not found"))?
        .to_string();

    let classpath = build_classpath(game_dir, &json)?;
    let assets_dir = game_dir.join("assets").to_string_lossy().to_string();
    let asset_index = json["assetIndex"]["id"]
        .as_str()
        .or_else(|| json["assets"].as_str())
        .unwrap_or("legacy")
        .to_string();

    // 获取版本类型：优先从 setup.ini 读取，否则从 JSON 检测
    let version_type = match VersionSetup::load(&version_dir) {
        Ok(Some(setup)) => {
            log_info!(
                "Loaded version type from setup.ini: {:?}",
                setup.loader.version_type
            );
            setup.loader.version_type
        }
        _ => {
            let detected = VersionType::detect_from_json(version_id, &json);
            log_info!("Detected version type from JSON: {:?}", detected);
            detected
        }
    };

    // 计算隔离后的有效游戏目录
    // 注意：工作目录、环境变量 APPDATA 和崩溃分析路径必须使用此隔离目录
    let mode = IsolationMode::from_u32(isolation_mode);
    let effective_game_dir =
        isolation::get_effective_game_dir(game_dir, version_id, mode, version_type);

    // 确保隔离目录存在
    if effective_game_dir != game_dir {
        // 根据版本类型创建不同的目录结构
        let result = if version_type.is_modded() {
            isolation::ensure_modded_dirs(&effective_game_dir)
        } else {
            isolation::ensure_isolated_dirs(&effective_game_dir)
        };
        if let Err(e) = result {
            log_info!("Warning: Failed to create isolated dirs: {}", e);
        }
    }

    log_info!(
        "Game dir: {} -> effective: {} (isolation mode: {}, version type: {:?})",
        game_dir.display(),
        effective_game_dir.display(),
        isolation_mode,
        version_type
    );

    let jvm_args = build_jvm_args(
        game_dir,
        version_id,
        &classpath,
        min_memory,
        max_memory,
        java_path,
        auth_info,
        extra_jvm_args,
        &json,
        disable_jlw,
        disable_lua,
    )?;
    let game_args = build_game_args(
        &json,
        &effective_game_dir,
        version_id,
        &assets_dir,
        &asset_index,
        auth_info,
        window_width,
        window_height,
        server_address,
        server_port,
        extra_game_args,
        custom_info,
    )?;

    // 在 launch 前设置游戏语言（写入有效目录，适配隔离模式）
    // 仅当 game_language 配置非空且非 "none" 时才设置
    if let Some(lang) = game_language {
        if !lang.is_empty() && lang != "none" {
            // 获取真实 MC 版本号（用于决定语言代码大小写）
            // 优先从 setup.ini 读取 OriginalVersion，回退到 version.json 的 inheritsFrom/id
            let mc_version = crate::minecraft::version::setup::detect_version_and_loader(
                &version_dir,
                version_id,
            )
            .0;
            log_info!(
                "[Language] Resolved MC version for language case: {} (from version_id={})",
                mc_version,
                version_id
            );

            if let Err(e) = crate::minecraft::language::set_game_language(
                &effective_game_dir,
                version_id,
                &mc_version,
                lang,
            ) {
                log_info!("[Language] Failed to set game language: {}", e);
            }
        } else {
            log_info!(
                "[Language] Skipped (game_language={:?}, respecting user in-game choice)",
                lang
            );
        }
    } else {
        log_info!("[Language] Skipped (game_language=None)");
    }

    Ok(LaunchArguments {
        jvm_args,
        game_args,
        main_class,
        classpath,
        version_id: version_id.to_string(),
        game_dir: effective_game_dir.to_string_lossy().to_string(),
        assets_dir,
        asset_index,
        auth_info: auth_info.clone(),
    })
}
