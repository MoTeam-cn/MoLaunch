//! Packs 图标提取（zip 内 pack.png → icon.png → preview.png；文件夹读同名文件）
//! 限量读取防 zip 炸弹。

use std::io::Read;
use std::path::Path;

use base64::Engine;

const CANDIDATES: &[&str] = &["pack.png", "icon.png", "preview.png"];
const MAX_ICON_SIZE: u64 = 2 * 1024 * 1024;

/// 从 zip 或文件夹中提取包图标（原始字节），无则返回 None
pub(crate) fn extract_pack_icon(path: &Path) -> Option<Vec<u8>> {
    if path.is_dir() {
        for name in CANDIDATES {
            if let Ok(data) = std::fs::read(path.join(name)) {
                if !data.is_empty() {
                    return Some(data);
                }
            }
        }
        return None;
    }
    if !path.is_file() {
        return None;
    }
    let file = std::fs::File::open(path).ok()?;
    let mut archive = zip::ZipArchive::new(file).ok()?;
    let mut target: Option<String> = None;
    for i in 0..archive.len() {
        if let Ok(entry) = archive.by_index(i) {
            let lower = entry.name().to_lowercase();
            if CANDIDATES
                .iter()
                .any(|c| lower == *c || lower.ends_with(&format!("/{}", c)))
            {
                target = Some(entry.name().to_string());
                break;
            }
        }
    }
    let target = target?;
    let mut entry = archive.by_name(&target).ok()?;
    if entry.size() > MAX_ICON_SIZE {
        return None;
    }
    let mut data = Vec::new();
    entry.read_to_end(&mut data).ok()?;
    if data.is_empty() {
        None
    } else {
        Some(data)
    }
}

/// 提取包图标为 base64 data URL（供前端直接作为 img src）
pub(crate) fn extract_pack_icon_data_url(path: &Path) -> Option<String> {
    let bytes = extract_pack_icon(path)?;
    let mime = if bytes.starts_with(&[0x89, b'P', b'N', b'G']) {
        "image/png"
    } else {
        "image/jpeg"
    };
    Some(format!(
        "data:{};base64,{}",
        mime,
        base64::engine::general_purpose::STANDARD.encode(&bytes)
    ))
}
