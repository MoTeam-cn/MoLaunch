//! OptiFine loader module

use std::sync::Arc;

use super::LoaderVersion;
use crate::minecraft::sources::{self, DownloadSourceMode};

/// List OptiFine versions
pub async fn list_versions(
    mirror_url: Option<&str>,
    source_mode: DownloadSourceMode,
) -> anyhow::Result<Vec<LoaderVersion>> {
    let urls = sources::build_urls(
        mirror_url,
        &format!("{}{}", sources::BMCLAPI_BASE, sources::BMCLAPI_OPTIFINE),
        sources::BMCLAPI_OPTIFINE,
        source_mode,
    );

    let content = sources::fetch_with_fallback(&urls).await?;

    if let Ok(json_array) = serde_json::from_str::<Vec<serde_json::Value>>(&content) {
        let mut versions: Vec<LoaderVersion> = json_array
            .iter()
            .filter_map(|v| {
                let mc_ver = v["mcversion"].as_str()?;
                let type_str = v["type"].as_str().unwrap_or("");
                let patch = v["patch"].as_str().unwrap_or("");

                let type_display = type_str
                    .replace("HD_U", "")
                    .replace("_", " ")
                    .trim()
                    .to_string();

                let display_name = if type_display.is_empty() {
                    format!("{} {}", mc_ver, patch)
                } else {
                    format!("{} {} {}", mc_ver, type_display, patch)
                };

                let is_preview =
                    patch.contains("pre") || patch.contains("alpha") || patch.contains("beta");

                Some(LoaderVersion {
                    version: display_name.trim().to_string(),
                    is_recommended: !is_preview,
                    release_time: None,
                })
            })
            .collect();

        versions.sort_by(|a, b| {
            if a.is_recommended != b.is_recommended {
                return b.is_recommended.cmp(&a.is_recommended);
            }
            compare_version(&b.version, &a.version)
        });

        return Ok(versions);
    }

    Ok(vec![])
}

/// Compare OptiFine versions
fn compare_version(a: &str, b: &str) -> std::cmp::Ordering {
    let a_parts = extract_parts(a);
    let b_parts = extract_parts(b);

    for (a_part, b_part) in a_parts.iter().zip(b_parts.iter()) {
        let a_num = a_part.parse::<u32>().ok();
        let b_num = b_part.parse::<u32>().ok();

        match (a_num, b_num) {
            (Some(a_n), Some(b_n)) => {
                let cmp = a_n.cmp(&b_n);
                if cmp != std::cmp::Ordering::Equal {
                    return cmp;
                }
            }
            _ => {
                let cmp = a_part.cmp(b_part);
                if cmp != std::cmp::Ordering::Equal {
                    return cmp;
                }
            }
        }
    }

    a_parts.len().cmp(&b_parts.len())
}

/// Extract version parts for comparison
fn extract_parts(version: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut is_digit = false;

    for c in version.chars() {
        if c.is_ascii_digit() {
            if !is_digit && !current.is_empty() {
                parts.push(current.clone());
                current.clear();
            }
            current.push(c);
            is_digit = true;
        } else if c.is_ascii_alphabetic() {
            if is_digit && !current.is_empty() {
                parts.push(current.clone());
                current.clear();
            }
            current.push(c.to_ascii_uppercase());
            is_digit = false;
        } else if (c == '.' || c == ' ' || c == '_' || c == '-') && !current.is_empty() {
            parts.push(current.clone());
            current.clear();
        }
    }

    if !current.is_empty() {
        parts.push(current);
    }

    parts
}

/// Install OptiFine (placeholder - requires manual installation)
pub async fn install(
    mc_version: &str,
    optifine_version: &str,
    progress_callback: Option<Arc<dyn Fn(f64) + Send + Sync>>,
    _source_mode: DownloadSourceMode,
) -> anyhow::Result<()> {
    if let Some(ref cb) = progress_callback {
        cb(0.0);
    }

    crate::log_info!(
        "[OptiFine] {} for MC {} - manual installation required",
        optifine_version,
        mc_version
    );

    if let Some(ref cb) = progress_callback {
        cb(100.0);
    }

    Ok(())
}
