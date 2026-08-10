//! Legacy Forge installation (1.7.10 ~ 1.12.2)
//!
//! 以 `install_profile.json` 为入口，两种格式：方式 2（1.7.10-，含 `install` 字段）；方式 1（1.8~1.12.2，含 `json` 字段并解压 maven/）

use std::path::Path;
use std::sync::Arc;

use crate::{log_info, log_warn};

use super::super::shared;

/// Legacy Forge installation (1.12.2 and below)
pub(super) async fn install_legacy(
    mc_version: &str,
    forge_version: &str,
    installer_path: &Path,
    game_dir: &Path,
    progress_callback: Option<Arc<dyn Fn(f64) + Send + Sync>>,
) -> anyhow::Result<()> {
    use std::io::Read;

    log_info!(
        "[Forge] Legacy 安装 {} for MC {}",
        forge_version,
        mc_version
    );

    let version_id = format!("{}-forge-{}", mc_version, forge_version);
    let version_dir = game_dir.join("versions").join(&version_id);
    std::fs::create_dir_all(&version_dir)?;

    let installer_file = std::fs::File::open(installer_path)?;
    let mut zip = zip::ZipArchive::new(installer_file)?;

    let profile_json: serde_json::Value = {
        let mut entry = zip.by_name("install_profile.json")?;
        let mut content = String::new();
        entry.read_to_string(&mut content)?;
        serde_json::from_str(&content)?
    };

    if let Some(ref cb) = progress_callback {
        cb(30.0);
    }

    if profile_json.get("install").is_some() {
        // Legacy 方式 2（1.7.10 及更早）
        log_info!("[Forge] Legacy 方式 2: {}", forge_version);
        let install = &profile_json["install"];

        let file_path = install["filePath"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("install.filePath not found"))?;
        let lib_path = install["path"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("install.path not found"))?;

        let jar_dest = shared::maven_path_to_local(lib_path, game_dir);
        if let Some(parent) = jar_dest.parent() {
            std::fs::create_dir_all(parent)?;
        }

        {
            let mut entry = zip.by_name(file_path)?;
            let mut buf = Vec::new();
            entry.read_to_end(&mut buf)?;
            std::fs::write(&jar_dest, buf)?;
        }
        log_info!("[Forge] 提取 JAR: {} -> {}", file_path, jar_dest.display());

        if let Some(ref cb) = progress_callback {
            cb(60.0);
        }

        let version_info = profile_json
            .get("versionInfo")
            .ok_or_else(|| anyhow::anyhow!("versionInfo not found"))?;
        let mut version_json = version_info.clone();

        version_json["id"] = serde_json::Value::String(version_id.clone());
        if version_json.get("inheritsFrom").is_none() {
            version_json["inheritsFrom"] = serde_json::Value::String(mc_version.to_string());
        }

        let json_path = version_dir.join(format!("{}.json", version_id));
        std::fs::write(&json_path, serde_json::to_string_pretty(&version_json)?)?;
        log_info!("[Forge] 写入版本 JSON: {}", json_path.display());
    } else {
        // Legacy 方式 1（1.8 ~ 1.12.2）
        log_info!("[Forge] Legacy 方式 1: {}", forge_version);

        let json_entry_name = profile_json["json"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("install_profile.json 中缺少 json 字段"))?
            .trim_start_matches('/');

        let mut version_json: serde_json::Value = {
            let mut entry = zip.by_name(json_entry_name)?;
            let mut content = String::new();
            entry.read_to_string(&mut content)?;
            serde_json::from_str(&content)?
        };

        version_json["id"] = serde_json::Value::String(version_id.clone());

        let json_path = version_dir.join(format!("{}.json", version_id));
        std::fs::write(&json_path, serde_json::to_string_pretty(&version_json)?)?;
        log_info!("[Forge] 写入版本 JSON: {}", json_path.display());

        if let Some(ref cb) = progress_callback {
            cb(50.0);
        }

        // 解压 maven/ 文件夹到 libraries/
        let maven_dest = game_dir.join("libraries");
        let maven_entries: Vec<String> = zip
            .file_names()
            .filter(|name| name.starts_with("maven/"))
            .map(|s| s.to_string())
            .collect();

        let mut extracted_count = 0;
        for entry_name in &maven_entries {
            let relative_path = entry_name.strip_prefix("maven/").unwrap_or(entry_name);
            if relative_path.is_empty() {
                continue;
            }
            // Zip Slip 防护：段级校验拒绝 `..` 与绝对路径（复用 utils::path，
            // 不用 canonicalize：Windows 上已存在的基目录返回 `\\?\` 前缀，未解压目标降级为普通路径，比较必然失败）
            if crate::utils::path::ensure_safe_relative_path(relative_path).is_err() {
                crate::log_warn!("[Forge] Skip path traversal entry: {}", entry_name);
                continue;
            }
            let dest_path = maven_dest.join(relative_path);

            let mut entry = zip.by_name(entry_name)?;
            if entry.is_dir() {
                std::fs::create_dir_all(&dest_path)?;
            } else {
                if let Some(parent) = dest_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                let mut buf = Vec::new();
                entry.read_to_end(&mut buf)?;
                std::fs::write(&dest_path, buf)?;
                extracted_count += 1;
            }
        }
        log_info!(
            "[Forge] 解压 maven/ 到 libraries/: {} 个文件",
            extracted_count
        );

        // 复制原版 JAR 到 Forge 版本目录（Legacy 方式 1 需要）
        let mc_jar = game_dir
            .join("versions")
            .join(mc_version)
            .join(format!("{}.jar", mc_version));
        let forge_jar = version_dir.join(format!("{}.jar", version_id));
        if mc_jar.exists() && !forge_jar.exists() {
            if let Err(e) = std::fs::copy(&mc_jar, &forge_jar) {
                log_warn!("[Forge] Failed to copy MC JAR: {}", e);
            } else {
                log_info!(
                    "[Forge] Copied MC JAR: {} -> {}",
                    mc_jar.display(),
                    forge_jar.display()
                );
            }
        }
    }

    if let Some(ref cb) = progress_callback {
        cb(90.0);
    }

    log_info!("[Forge] Legacy 安装完成: {}", version_id);
    Ok(())
}
