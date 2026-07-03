//! Natives extraction module

use std::path::{Path, PathBuf};

/// Extract natives files to target directory
pub fn extract_natives(
    natives_entries: &[super::super::version::libraries::LibEntry],
    game_dir: &Path,
    version_id: &str,
) -> anyhow::Result<()> {
    let target_dir = game_dir
        .join("versions")
        .join(version_id)
        .join(format!("{}-natives", version_id));

    std::fs::create_dir_all(&target_dir)?;

    let mut extracted_files: Vec<String> = Vec::new();

    for entry in natives_entries {
        if !entry.is_natives {
            continue;
        }

        let jar_path = Path::new(&entry.local_path);
        if !jar_path.exists() {
            log::warn!("Natives JAR not found: {}", entry.local_path);
            continue;
        }

        let file = std::fs::File::open(jar_path)?;
        let archive = match zip::ZipArchive::new(file) {
            Ok(a) => a,
            Err(e) => {
                log::error!("Cannot open natives JAR: {} - {}", entry.local_path, e);
                let _ = std::fs::remove_file(jar_path);
                continue;
            }
        };

        extract_dlls_from_zip(archive, &target_dir, &mut extracted_files)?;
    }

    cleanup_natives_dir(&target_dir, &extracted_files)?;

    Ok(())
}

fn extract_dlls_from_zip(
    mut archive: zip::ZipArchive<std::fs::File>,
    target_dir: &Path,
    extracted_files: &mut Vec<String>,
) -> anyhow::Result<()> {
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let entry_name = entry.name().to_string();

        let ext = Path::new(&entry_name)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        if !["dll", "so", "dylib"].contains(&ext) {
            continue;
        }

        if entry_name.starts_with("META-INF/") {
            continue;
        }

        let file_name = Path::new(&entry_name)
            .file_name()
            .and_then(|f| f.to_str())
            .unwrap_or(&entry_name);

        let target_path = target_dir.join(file_name);
        extracted_files.push(target_path.to_string_lossy().to_string());

        if target_path.exists() {
            let existing_size = std::fs::metadata(&target_path)?.len();
            let entry_size = entry.size();

            if existing_size == entry_size {
                log::debug!("Skipping existing natives: {}", file_name);
                continue;
            }

            log::info!("Replacing natives: {}", file_name);
            std::fs::remove_file(&target_path)?;
        }

        let mut target_file = std::fs::File::create(&target_path)?;
        std::io::copy(&mut entry, &mut target_file)?;

        log::info!("Extracted natives: {}", file_name);
    }

    Ok(())
}

fn cleanup_natives_dir(target_dir: &Path, extracted_files: &[String]) -> anyhow::Result<()> {
    if !target_dir.exists() {
        return Ok(());
    }

    for entry in std::fs::read_dir(target_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            let path_str = path.to_string_lossy().to_string();
            if !extracted_files.contains(&path_str) {
                log::info!("Removing extra natives: {}", path.display());
                let _ = std::fs::remove_file(&path);
            }
        }
    }

    Ok(())
}

/// Get natives directory path
pub fn get_natives_dir(game_dir: &Path, version_id: &str) -> PathBuf {
    game_dir
        .join("versions")
        .join(version_id)
        .join(format!("{}-natives", version_id))
}
