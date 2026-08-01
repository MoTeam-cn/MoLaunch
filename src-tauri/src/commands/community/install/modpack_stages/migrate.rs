//! MMC instance.cfg / MCBBS launchInfo 配置迁移到版本 setup.ini

use crate::log_info;

use super::super::types::{ModpackFormat, ModpackInfo};

/// 将 MMC instance.cfg / MCBBS launchInfo 配置迁移到版本 setup.ini
///
/// 必须在 extract_overrides 之后调用（MMC iconKey 复制依赖 overrides 已解压）。
/// 字段映射：MMC PreLaunchCommand→advance_run_cmd、JoinServer→server_enter、
/// iconKey→logo、JvmArgs→advance_jvm_args；MCBBS javaArgument→advance_jvm_args、
/// launchArgument→advance_game_args。所有格式强制写入 indie_type=1 开启版本隔离。
pub fn migrate_modpack_config(
    info: &ModpackInfo,
    instance_dir: &std::path::Path,
    instance_name: &str,
) -> Result<(), String> {
    use crate::minecraft::version::setup::{PersonalizationUpdate, VersionSetup};

    // 强制开启版本隔离（所有格式都写）
    let mut update = PersonalizationUpdate {
        indie_type: Some(1),
        ..Default::default()
    };

    match info.format {
        ModpackFormat::Mmc => {
            let Some(cfg_content) = &info.mmc_cfg_content else {
                VersionSetup::update_personalization(instance_dir, &update)
                    .map_err(|e| format!("写入版本 setup.ini 失败: {}", e))?;
                return Ok(());
            };
            let cfg = super::super::mmc::parse_instance_cfg(cfg_content);

            // PreLaunchCommand（仅 OverrideCommands=true 时迁移）
            if cfg.override_commands {
                if let Some(cmd) = &cfg.pre_launch_command {
                    let replaced = super::super::mmc::substitute_pre_launch_vars(
                        cmd,
                        instance_dir,
                        instance_name,
                    );
                    log_info!("[Community] MMC 迁移 PreLaunchCommand: {}", replaced);
                    update.advance_run_cmd = Some(replaced);
                }
            }

            // JoinServerOnLaunchAddress（仅 JoinServerOnLaunch=true 时迁移）
            if cfg.join_server_on_launch {
                if let Some(addr) = &cfg.join_server_address {
                    log_info!("[Community] MMC 迁移 JoinServer: {}", addr);
                    update.server_enter = Some(addr.clone());
                }
            }

            // IgnoreJavaCompatibility
            if cfg.ignore_java_compatibility {
                log_info!("[Community] MMC 迁移 IgnoreJavaCompatibility=true");
                update.advance_ignore_java_warning = Some(true);
            }

            // iconKey：复制 {iconKey}.png 到 MoLaunch/Logo.png
            if let Some(icon_key) = &cfg.icon_key {
                let src_png = instance_dir.join(format!("{}.png", icon_key));
                if src_png.exists() {
                    let logo_dir = instance_dir.join("MoLaunch");
                    std::fs::create_dir_all(&logo_dir)
                        .map_err(|e| format!("创建 MoLaunch 目录失败: {}", e))?;
                    let logo_path = logo_dir.join("Logo.png");
                    if std::fs::copy(&src_png, &logo_path).is_ok() {
                        log_info!(
                            "[Community] MMC 复制图标: {} → {}",
                            src_png.display(),
                            logo_path.display()
                        );
                        update.logo = Some("MoLaunch\\Logo.png".to_string());
                    }
                } else {
                    log_info!(
                        "[Community] MMC iconKey 指定的图标不存在: {}",
                        src_png.display()
                    );
                }
            }

            // JvmArgs（简化：无论 OverrideJavaArgs 都直接覆盖版本独立 JVM 参数）
            if let Some(jvm_args) = &cfg.jvm_args {
                log_info!(
                    "[Community] MMC 迁移 JvmArgs (override={}): {}",
                    cfg.override_java_args,
                    jvm_args
                );
                update.advance_jvm_args = Some(jvm_args.clone());
            }
        }
        ModpackFormat::Mcbbs => {
            if let Some(manifest) = &info.mcbbs_manifest {
                if let Some(launch_info) = &manifest.launch_info {
                    if let Some(java_args) = &launch_info.java_argument {
                        if !java_args.is_empty() {
                            let joined = java_args.join(" ");
                            log_info!("[Community] MCBBS 迁移 javaArgument: {}", joined);
                            update.advance_jvm_args = Some(joined);
                        }
                    }
                    if let Some(launch_args) = &launch_info.launch_argument {
                        if !launch_args.is_empty() {
                            let joined = launch_args.join(" ");
                            log_info!("[Community] MCBBS 迁移 launchArgument: {}", joined);
                            update.advance_game_args = Some(joined);
                        }
                    }
                }
            }
        }
        _ => {}
    }

    VersionSetup::update_personalization(instance_dir, &update)
        .map_err(|e| format!("写入版本 setup.ini 失败: {}", e))?;
    log_info!(
        "[Community] 配置迁移完成: instance={} format={:?} (indie_type=1)",
        instance_name,
        info.format
    );

    Ok(())
}
