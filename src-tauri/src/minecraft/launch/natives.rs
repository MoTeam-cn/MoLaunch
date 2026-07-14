//! Natives extraction module

use crate::{log_info, log_warn, log_error, log_debug};
use std::path::{Path, PathBuf};

/// Compute SHA1 hex digest of a file's contents.
fn compute_file_sha1(path: &Path) -> anyhow::Result<String> {
    use sha1::Digest;
    let bytes = std::fs::read(path)?;
    let mut hasher = sha1::Sha1::new();
    hasher.update(&bytes);
    Ok(hex::encode(hasher.finalize()))
}

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
            log_warn!("Natives JAR not found: {}", entry.local_path);
            continue;
        }

        // JAR 级别 SHA1 校验（CWE-494/CWE-345）：
        // 若 LibEntry 提供了版本 JSON 中记录的预期 SHA1，先校验 JAR 文件完整性。
        // 匹配才解压；不匹配则跳过该 JAR，防止恶意替换的 JAR 被解压。
        if let Some(ref expected_sha1) = entry.sha1 {
            if !expected_sha1.is_empty() {
                match compute_file_sha1(jar_path) {
                    Ok(actual_sha1) => {
                        if actual_sha1.eq_ignore_ascii_case(expected_sha1) {
                            log_info!(
                                "[Natives] JAR SHA1 verified: {} (sha1={})",
                                entry.local_path,
                                actual_sha1
                            );
                        } else {
                            log_warn!(
                                "[Natives] JAR SHA1 mismatch for {}: expected={}, actual={} — skipping extraction",
                                entry.local_path,
                                expected_sha1,
                                actual_sha1
                            );
                            continue;
                        }
                    }
                    Err(e) => {
                        log_warn!(
                            "[Natives] Failed to compute SHA1 for {}: {} — proceeding without JAR verification",
                            entry.local_path,
                            e
                        );
                    }
                }
            } else {
                log_warn!(
                    "[Natives] Empty expected SHA1 for JAR {}, skipping JAR verification",
                    entry.local_path
                );
            }
        } else {
            log_warn!(
                "[Natives] No expected SHA1 for JAR {}, skipping JAR verification",
                entry.local_path
            );
        }

        let file = std::fs::File::open(jar_path)?;
        let archive = match zip::ZipArchive::new(file) {
            Ok(a) => a,
            Err(e) => {
                log_error!("Cannot open natives JAR: {} - {}", entry.local_path, e);
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
    use sha1::Digest;
    use std::io::Read;

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

        // 读取条目内容到缓冲区，用于 SHA1 校验和写入
        let mut buffer = Vec::new();
        entry.read_to_end(&mut buffer)?;
        let entry_size = buffer.len() as u64;

        // 计算条目内容的 SHA1（用于与已存在文件做双重校验，防止相同大小但内容不同的恶意 DLL 替换）
        let mut hasher = sha1::Sha1::new();
        hasher.update(&buffer);
        let entry_sha1 = hex::encode(hasher.finalize());

        if target_path.exists() {
            let existing_size = std::fs::metadata(&target_path)?.len();

            if existing_size == entry_size {
                // 大小匹配，再做 SHA1 双重校验（CWE-494/CWE-345）
                // 攻击者可制作相同大小但内容不同的恶意 DLL，仅靠大小判断无法防御
                let existing_bytes = std::fs::read(&target_path)?;
                let mut hasher = sha1::Sha1::new();
                hasher.update(&existing_bytes);
                let existing_sha1 = hex::encode(hasher.finalize());

                if existing_sha1 == entry_sha1 {
                    log_debug!(
                        "Skipping existing natives (size+sha1 verified): {} (sha1={})",
                        file_name,
                        existing_sha1
                    );
                    continue;
                }

                log_warn!(
                    "Natives size matches but SHA1 differs, replacing: {} (existing={}, expected={})",
                    file_name,
                    existing_sha1,
                    entry_sha1
                );
            } else {
                log_info!("Replacing natives (size differs): {}", file_name);
            }

            std::fs::remove_file(&target_path)?;
        }

        std::fs::write(&target_path, &buffer)?;

        log_info!(
            "Extracted natives: {} (size={}, sha1={})",
            file_name,
            entry_size,
            entry_sha1
        );
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
                log_info!("Removing extra natives: {}", path.display());
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
