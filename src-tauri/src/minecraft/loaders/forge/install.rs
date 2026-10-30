//! Forge 安装调度器 + Modern 安装（1.13+，injector 方式）

use std::path::Path;
use std::sync::Arc;

use crate::minecraft::download::config::DownloadManagerConfig;
use crate::minecraft::download::manager::DownloadManager;
use crate::minecraft::download::types::{DownloadStatus, DownloadTask};
use crate::minecraft::launcher_profiles;
use crate::minecraft::sources;
use crate::{log_info, log_warn};

use super::super::shared;
use super::legacy::install_legacy;

/// Install Forge
pub async fn install(
    mc_version: &str,
    forge_version: &str,
    game_dir: &Path,
    mirror_url: Option<&str>,
    progress_callback: Option<Arc<dyn Fn(f64) + Send + Sync>>,
    config: &DownloadManagerConfig,
) -> anyhow::Result<()> {
    if let Some(ref cb) = progress_callback {
        cb(0.0);
    }

    let file_name = format!("forge-{}-{}-installer.jar", mc_version, forge_version);
    let installer_url = sources::forge_installer_url(mc_version, forge_version);
    let temp_dir = crate::utils::cache_temp::ensure_task_temp_dir()?;
    let installer_path = temp_dir.join(&file_name);

    // 尝试获取文件 hash
    let hash_url = format!("{}.sha1", installer_url);
    let expected_hash = match crate::http::fetch_url(&hash_url).await {
        Ok(hash) => Some(hash.trim().to_string()),
        Err(_) => None,
    };

    // Download installer
    let urls = sources::build_replace_urls(
        &installer_url,
        mirror_url,
        sources::MAVEN_REPLACEMENTS,
        config.source_mode,
    );

    let manager = DownloadManager::from_config(config);
    let task = DownloadTask {
        id: "forge_installer".to_string(),
        urls,
        local_path: installer_path.to_string_lossy().to_string(),
        expected_size: 0,
        expected_hash,
    };

    let results = manager.download_batch(vec![task], None).await;
    if let Some(result) = results.first() {
        if result.status == DownloadStatus::Failed {
            let _ = std::fs::remove_file(&installer_path);
            return Err(anyhow::anyhow!("Failed to download Forge installer"));
        }
    }

    if let Some(ref cb) = progress_callback {
        cb(10.0);
    }

    // 根据 Forge 版本选择安装方式
    if super::super::forge_installer::needs_injector(forge_version, false) {
        install_modern(
            mc_version,
            forge_version,
            &installer_path,
            game_dir,
            progress_callback,
            config,
        )
        .await
    } else {
        install_legacy(
            mc_version,
            forge_version,
            &installer_path,
            game_dir,
            progress_callback,
        )
        .await
    }
}

/// Modern Forge installation (1.13+)
async fn install_modern(
    mc_version: &str,
    forge_version: &str,
    installer_path: &Path,
    game_dir: &Path,
    progress_callback: Option<Arc<dyn Fn(f64) + Send + Sync>>,
    config: &DownloadManagerConfig,
) -> anyhow::Result<()> {
    log_info!("[Forge] Installing {} for MC {}", forge_version, mc_version);

    let version_id = format!("{}-forge-{}", mc_version, forge_version);

    launcher_profiles::ensure_profiles_exist(game_dir).map_err(|e: String| anyhow::anyhow!(e))?;

    if let Some(ref cb) = progress_callback {
        cb(20.0);
    }

    // 下载 Mojang 映射文件
    if let Err(e) =
        shared::download_mojang_mappings(mc_version, game_dir, installer_path, config).await
    {
        log_warn!("[Forge] Failed to download mappings: {}", e);
    }

    if let Some(ref cb) = progress_callback {
        cb(30.0);
    }

    if let Some(ref cb) = progress_callback {
        cb(40.0);
    }

    if let Err(error) = shared::log_install_processors(installer_path, mc_version, game_dir) {
        log_warn!("[Install] 读不到 processor 清单: {}", error);
    }

    let java_path = shared::find_java_for_install(game_dir, mc_version)?;

    if let Some(ref cb) = progress_callback {
        cb(50.0);
    }

    // 注入器资源（与第三方启动器同一份 jar，走镜像加速）
    let (injector_path, wrapper_path) =
        super::super::forge_installer::extract_embedded_resources()?;

    // 安装器进度（0~1）映射到 50%~80% 区间，由上层写进阶段进度
    let installer_progress = progress_callback.clone().map(|cb| {
        let handler: super::super::forge_installer::ProgressHandler =
            std::sync::Arc::new(move |fraction: f64| cb(50.0 + fraction.clamp(0.0, 1.0) * 30.0));
        handler
    });

    let java_path_owned = java_path.to_string();
    let installer_path_str = installer_path.to_string_lossy().to_string();
    let injector_path_owned = injector_path.to_string();
    let wrapper_path_owned = wrapper_path.to_string();
    let game_dir_str = game_dir.to_string_lossy().to_string();

    // 主路径：注入器（与第三方启动器做法一致）
    let mut install_result = tokio::task::spawn_blocking({
        let (java, installer, injector, wrapper, dir, handler) = (
            java_path_owned.clone(),
            installer_path_str.clone(),
            injector_path_owned.clone(),
            wrapper_path_owned.clone(),
            game_dir_str.clone(),
            installer_progress.clone(),
        );
        move || {
            super::super::forge_installer::run_forge_installer(
                &java, &installer, &injector, &wrapper, &dir, false, handler,
            )
        }
    })
    .await
    .map_err(|e| anyhow::anyhow!("Forge installer task panicked: {}", e))?;

    // 兜底：注入器失败时（常见于镜像重定向超限），由启动器自己把 install_profile 里的库补齐再试一次
    let mut first_error_message: Option<String> = None;
    if let Some(first_error) = install_result.as_ref().err().map(ToString::to_string) {
        log_warn!(
            "[Forge] 安装器失败，改由启动器补全库后重试: {}",
            first_error
        );
        match shared::prefetch_install_libraries(installer_path, game_dir, config).await {
            Ok(count) if count > 0 => {
                log_info!("[Forge] 已补全 {} 个库，重试安装器", count);
                first_error_message = Some(first_error);
                install_result = tokio::task::spawn_blocking({
                    let (java, installer, injector, wrapper, dir, handler) = (
                        java_path_owned,
                        installer_path_str,
                        injector_path_owned,
                        wrapper_path_owned,
                        game_dir_str,
                        installer_progress,
                    );
                    move || {
                        super::super::forge_installer::run_forge_installer(
                            &java, &installer, &injector, &wrapper, &dir, false, handler,
                        )
                    }
                })
                .await
                .map_err(|e| anyhow::anyhow!("Forge installer task panicked: {}", e))?;
            }
            Ok(_) => log_warn!("[Forge] 没有需要补全的库，保留原始错误"),
            Err(e) => log_warn!("[Forge] 补全库失败，保留原始错误: {}", e),
        }
    }
    if let Err(error) = install_result {
        return Err(match first_error_message {
            Some(first) => anyhow::anyhow!("{}；补全库重试前的错误：{}", error, first),
            None => error,
        });
    }
    if let Some(ref cb) = progress_callback {
        cb(80.0);
    }

    // Find and copy the generated version JSON
    shared::copy_generated_version_json(game_dir, mc_version, &version_id, "forge");

    // 装完回头核对：注入器返回 0 不等于装好了，
    // processor 失败可能不影响退出码，产物却没生成。
    let installed_json = game_dir
        .join("versions")
        .join(&version_id)
        .join(format!("{}.json", version_id));
    if let Err(reason) = super::super::artifact_check::check_install(game_dir, &installed_json) {
        return Err(anyhow::anyhow!(reason));
    }

    // Copy MC JAR to Forge version folder
    shared::copy_mc_jar(game_dir, mc_version, &version_id);

    if let Some(ref cb) = progress_callback {
        cb(90.0);
    }

    log_info!("[Forge] Installed: {}", version_id);

    if let Some(ref cb) = progress_callback {
        cb(100.0);
    }

    Ok(())
}
