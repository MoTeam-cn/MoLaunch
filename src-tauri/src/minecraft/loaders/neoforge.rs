//! NeoForge loader module

use crate::{log_info, log_warn};
use std::path::Path;
use std::sync::Arc;

use super::shared;
use super::LoaderVersion;
use crate::minecraft::download::config::DownloadManagerConfig;
use crate::minecraft::download::manager::DownloadManager;
use crate::minecraft::download::types::{DownloadStatus, DownloadTask};
use crate::minecraft::launcher_profiles;
use crate::minecraft::sources::{self, DownloadSourceMode};

/// List NeoForge versions
pub async fn list_versions(
    mc_version: &str,
    mirror_url: Option<&str>,
    source_mode: DownloadSourceMode,
) -> anyhow::Result<Vec<LoaderVersion>> {
    crate::log_separator!("NeoForge List");
    crate::log_info!("[NeoForge] Listing versions for MC {}", mc_version);

    let urls = sources::build_urls(
        mirror_url,
        sources::NEOFORGE_API,
        sources::BMCLAPI_NEOFORGE,
        source_mode,
    );
    crate::log_debug!("[NeoForge] 尝试源: {:?}", urls);

    let content = match sources::fetch_with_fallback(&urls).await {
        Ok(c) => {
            crate::log_debug!("[NeoForge] Response length: {} bytes", c.len());
            c
        }
        Err(e) => {
            crate::log_error!("[NeoForge] 所有源失败: {}", e);
            return Ok(vec![]);
        }
    };

    let json: serde_json::Value = match serde_json::from_str(&content) {
        Ok(j) => j,
        Err(e) => {
            crate::log_error!("[NeoForge] JSON parse failed: {}", e);
            return Ok(vec![]);
        }
    };

    let files_array = json["files"]
        .as_array()
        .or_else(|| json["versions"].as_array());

    let total = files_array.map(|a| a.len()).unwrap_or(0);
    crate::log_info!("[NeoForge] Total versions in API: {}", total);

    let mut versions = Vec::new();
    if let Some(files) = files_array {
        for file in files {
            let version_str = if let Some(name) = file["name"].as_str() {
                name
            } else if let Some(s) = file.as_str() {
                s
            } else {
                continue;
            };

            if is_compatible(version_str, mc_version) {
                let is_beta = version_str.contains("beta") || version_str.contains("alpha");
                versions.push(LoaderVersion {
                    version: version_str.to_string(),
                    is_recommended: !is_beta,
                    release_time: None,
                });
            }
        }
    }

    // 检查旧版格式
    let legacy_urls = sources::build_urls(
        mirror_url,
        sources::NEOFORGE_API_LEGACY,
        sources::BMCLAPI_NEOFORGE_LEGACY,
        source_mode,
    );
    if let Ok(legacy_content) = sources::fetch_with_fallback(&legacy_urls).await {
        if let Ok(legacy_json) = serde_json::from_str::<serde_json::Value>(&legacy_content) {
            let legacy_files = legacy_json["files"]
                .as_array()
                .or_else(|| legacy_json["versions"].as_array());
            if let Some(files) = legacy_files {
                for file in files {
                    let version_str = if let Some(name) = file["name"].as_str() {
                        name
                    } else if let Some(s) = file.as_str() {
                        s
                    } else {
                        continue;
                    };
                    let prefix = format!("{}-", mc_version);
                    if version_str.starts_with(&prefix) {
                        let loader_version =
                            version_str.strip_prefix(&prefix).unwrap_or(version_str);
                        versions.push(LoaderVersion {
                            version: loader_version.to_string(),
                            is_recommended: true,
                            release_time: None,
                        });
                    }
                }
            }
        }
    }

    versions.sort_by(|a, b| {
        let v_a = crate::utils::version::parse_number(&a.version);
        let v_b = crate::utils::version::parse_number(&b.version);
        v_b.cmp(&v_a)
    });

    crate::log_info!("[NeoForge] Final result: {} versions", versions.len());
    crate::log_separator!("NeoForge End");
    Ok(versions)
}

/// Check NeoForge version compatibility
fn is_compatible(neoforge_version: &str, mc_version: &str) -> bool {
    let mc_parts: Vec<&str> = mc_version.split('.').collect();
    if mc_parts.len() < 2 {
        return false;
    }

    let mc_major: u32 = mc_parts[0].parse().unwrap_or(0);
    let mc_minor: u32 = mc_parts[1].parse().unwrap_or(0);

    if neoforge_version.starts_with("0.") {
        return false;
    }

    let neoforge_parts: Vec<&str> = neoforge_version.split('.').collect();
    if neoforge_parts.len() < 2 {
        return false;
    }

    let neoforge_major: u32 = match neoforge_parts[0].parse() {
        Ok(v) => v,
        Err(_) => return false,
    };
    let neoforge_minor: u32 = match neoforge_parts[1].parse() {
        Ok(v) => v,
        Err(_) => return false,
    };

    if mc_major == 1 {
        neoforge_major == mc_minor
    } else {
        neoforge_major == mc_major && neoforge_minor == mc_minor
    }
}

/// Install NeoForge
pub async fn install(
    mc_version: &str,
    neoforge_version: &str,
    game_dir: &Path,
    mirror_url: Option<&str>,
    progress_callback: Option<Arc<dyn Fn(f64) + Send + Sync>>,
    config: &DownloadManagerConfig,
) -> anyhow::Result<()> {
    if let Some(ref cb) = progress_callback {
        cb(0.0);
    }

    log_info!(
        "[NeoForge] Installing {} for MC {}",
        neoforge_version,
        mc_version
    );

    let file_name = format!("neoforge-{}-installer.jar", neoforge_version);
    let installer_url = sources::neoforge_installer_url(neoforge_version);
    let temp_dir = crate::utils::cache_temp::ensure_task_temp_dir()?;
    let installer_path = temp_dir.join(&file_name);

    let hash_url = format!("{}.sha1", installer_url);
    let expected_hash = match crate::http::fetch_url(&hash_url).await {
        Ok(hash) => Some(hash.trim().to_string()),
        Err(_) => None,
    };

    let urls = sources::build_replace_urls(
        &installer_url,
        mirror_url,
        sources::MAVEN_REPLACEMENTS,
        config.source_mode,
    );

    let manager = DownloadManager::from_config(config);
    let task = DownloadTask {
        id: "neoforge_installer".to_string(),
        urls,
        local_path: installer_path.to_string_lossy().to_string(),
        expected_size: 0,
        expected_hash,
    };

    let results = manager.download_batch(vec![task], None).await;
    if let Some(result) = results.first() {
        if result.status == DownloadStatus::Failed {
            return Err(anyhow::anyhow!("Failed to download NeoForge installer"));
        }
    }

    if let Some(ref cb) = progress_callback {
        cb(10.0);
    }

    let version_id = format!("{}-neoforge-{}", mc_version, neoforge_version);

    launcher_profiles::ensure_profiles_exist(game_dir).map_err(|e: String| anyhow::anyhow!(e))?;

    if let Some(ref cb) = progress_callback {
        cb(20.0);
    }

    if let Err(e) =
        shared::download_mojang_mappings(mc_version, game_dir, &installer_path, config).await
    {
        log_warn!("[NeoForge] Failed to download mappings: {}", e);
    }

    if let Some(ref cb) = progress_callback {
        cb(30.0);
    }

    if let Some(ref cb) = progress_callback {
        cb(40.0);
    }

    if let Err(error) = shared::log_install_processors(&installer_path, mc_version, game_dir) {
        log_warn!("[Install] 读不到 processor 清单: {}", error);
    }

    let java_path = shared::find_java_for_install(game_dir, mc_version)?;

    if let Some(ref cb) = progress_callback {
        cb(50.0);
    }

    // 注入器资源（与第三方启动器同一份 jar，走镜像加速）
    let (injector_path, wrapper_path) = super::forge_installer::extract_embedded_resources()?;

    // 安装器进度（0~1）映射到 50%~80% 区间
    let installer_progress = progress_callback.clone().map(|cb| {
        let handler: super::forge_installer::ProgressHandler =
            std::sync::Arc::new(move |fraction: f64| cb(50.0 + fraction.clamp(0.0, 1.0) * 30.0));
        handler
    });

    // 执行器介入时把进度从 80% 推到 90%
    let processor_progress = |done: usize, total: usize| {
        if let Some(callback) = progress_callback.as_ref() {
            if total > 0 {
                callback(80.0 + (done as f64 / total as f64) * 10.0);
            }
        }
    };

    let java_path_str = java_path.to_string();
    let installer_path_str = installer_path.to_string_lossy().to_string();
    let game_dir_str = game_dir.to_string_lossy().to_string();

    // 主路径：注入器
    let mut install_result = super::forge_installer::run_forge_installer(
        &java_path_str,
        &installer_path_str,
        &injector_path,
        &wrapper_path,
        &game_dir_str,
        true,
        installer_progress.clone(),
    );

    // 兜底：注入器失败时由启动器补全库再试一次
    let mut first_error_message: Option<String> = None;
    if let Some(first_error) = install_result.as_ref().err().map(ToString::to_string) {
        log_warn!(
            "[NeoForge] 安装器失败，改由启动器补全库后重试: {}",
            first_error
        );
        match shared::prefetch_install_libraries(&installer_path, game_dir, config).await {
            Ok(count) if count > 0 => {
                log_info!("[NeoForge] 已补全 {} 个库，重试安装器", count);
                first_error_message = Some(first_error);
                install_result = super::forge_installer::run_forge_installer(
                    &java_path_str,
                    &installer_path_str,
                    &injector_path,
                    &wrapper_path,
                    &game_dir_str,
                    true,
                    installer_progress,
                );
            }
            Ok(_) => log_warn!("[NeoForge] 没有需要补全的库，保留原始错误"),
            Err(e) => log_warn!("[NeoForge] 补全库失败，保留原始错误: {}", e),
        }
    }
    if let Err(error) = install_result {
        let message = match first_error_message {
            Some(first) => format!("{}；补全库重试前的错误：{}", error, first),
            None => error.to_string(),
        };
        log_warn!(
            "[Install] 注入器失败，改由启动器执行 processors: {}",
            message
        );
        match super::processor_run::run_from_installer(
            &java_path,
            &installer_path,
            mc_version,
            game_dir,
            Some(&processor_progress),
        ) {
            Ok(ran) if ran > 0 => {
                log_info!("[Install] 自行执行 {} 个 processor，继续后续校验", ran)
            }
            Ok(_) => return Err(anyhow::anyhow!(message)),
            Err(runner_error) => {
                return Err(anyhow::anyhow!(
                    "{}；自行执行 processors 也失败：{}",
                    message,
                    runner_error
                ))
            }
        }
    }
    if let Some(ref cb) = progress_callback {
        cb(80.0);
    }

    shared::copy_generated_version_json(game_dir, mc_version, &version_id, "neoforge");

    // 装完回头核对：注入器返回 0 不等于装好了，
    // processor 失败可能不影响退出码，产物却没生成。
    let installed_json = game_dir
        .join("versions")
        .join(&version_id)
        .join(format!("{}.json", version_id));
    if let Err(reason) = super::artifact_check::check_install(game_dir, &installed_json) {
        // 注入器没把产物生成齐：用自己的执行器补一次
        log_warn!("[Install] 产物不齐，改由启动器执行 processors: {}", reason);
        match super::processor_run::run_from_installer(
            &java_path,
            &installer_path,
            mc_version,
            game_dir,
            Some(&processor_progress),
        ) {
            Ok(0) => return Err(anyhow::anyhow!(reason)),
            Ok(ran) => log_info!("[Install] 自行执行 {} 个 processor，重新校验", ran),
            Err(error) => {
                return Err(anyhow::anyhow!(
                    "{}；自行执行 processors 也失败：{}",
                    reason,
                    error
                ))
            }
        }
        if let Err(reason) = super::artifact_check::check_install(game_dir, &installed_json) {
            return Err(anyhow::anyhow!(reason));
        }
    }

    if let Some(ref cb) = progress_callback {
        cb(90.0);
    }

    log_info!("[NeoForge] Installed: {}", version_id);

    if let Some(ref cb) = progress_callback {
        cb(100.0);
    }

    Ok(())
}
