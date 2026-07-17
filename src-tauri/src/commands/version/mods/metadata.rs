//! Jar 内 Mod 元数据读取流水线
//!
//! 参考 PCL2 LocalMod.Read:
//! 1. fabric.mod.json → id / description / version / iconPath（Fabric/Quilt）
//! 2. META-INF/mods.toml → modId / description / version / logoFile（Forge 1.13+/NeoForge）
//! 3. mcmod.info → modid / description / version / logoFile（Forge 1.12-）
//!
//! 查到 slug 后用 mcmod 数据库查询译名，查不到返回空字符串
//! logo 从 jar 内提取并编码为 base64 data URL，未找到则返回 None
//! slug 也一并返回，用于前端关联 CF/MR 平台工程和查 mcmod.cn 直链

use std::io::Read;

use super::types::{ModMeta, ModMetadata};

/// 从 jar 文件内读取 mod 元数据：译名、描述、版本号、logo data URL、slug
///
/// 读取顺序（参考 PCL2 LocalMod.Read):
/// 1. fabric.mod.json → id / description / version / iconPath（Fabric/Quilt）
/// 2. META-INF/mods.toml → modId / description / version / logoFile（Forge 1.13+/NeoForge）
/// 3. mcmod.info → modid / description / version / logoFile（Forge 1.12-）
pub(crate) fn read_mod_metadata(path: &std::path::Path) -> ModMetadata {
    let file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return ModMetadata::default(),
    };
    let mut archive = match zip::ZipArchive::new(file) {
        Ok(a) => a,
        Err(_) => return ModMetadata::default(),
    };

    // 尝试 fabric.mod.json
    if let Some(meta) = read_fabric_mod_meta(&mut archive) {
        return finalize_metadata(meta, &mut archive);
    }
    // 尝试 META-INF/mods.toml（Forge 1.13+/NeoForge）
    if let Some(meta) = read_forge_mods_toml_meta(&mut archive) {
        return finalize_metadata(meta, &mut archive);
    }
    // 尝试 mcmod.info（Forge 1.12-）
    if let Some(meta) = read_mcmod_info_meta(&mut archive) {
        return finalize_metadata(meta, &mut archive);
    }

    ModMetadata::default()
}

/// 把中间结构 ModMeta 转换为最终 ModMetadata（提取 logo + 查译名）
fn finalize_metadata<R: std::io::Read + std::io::Seek>(
    meta: ModMeta,
    archive: &mut zip::ZipArchive<R>,
) -> ModMetadata {
    let slug = meta.slug.clone().unwrap_or_default();
    let translated = meta
        .slug
        .as_deref()
        .and_then(lookup_translated)
        .unwrap_or_default();
    // fabric 用 icon_path，forge/mcmod 用 logo_file
    let logo_path = meta.icon_path.or(meta.logo_file);
    let logo = logo_path
        .as_deref()
        .and_then(|p| extract_logo_data_url(archive, p));
    ModMetadata {
        slug,
        description: meta.description,
        version: meta.version,
        logo_data: logo,
        translated_name: translated,
    }
}

/// 从 jar 内提取 logo 文件并编码为 base64 data URL
/// 支持 png/jpg/jpeg/gif，根据扩展名推断 MIME
/// logo 路径可能是绝对路径（jar 内）或相对路径（fabric.mod.json 的 iconPath 通常相对于 jar 根）
fn extract_logo_data_url<R: std::io::Read + std::io::Seek>(
    archive: &mut zip::ZipArchive<R>,
    logo_path: &str,
) -> Option<String> {
    // 清理路径：去除前导 /
    let clean_path = logo_path.trim_start_matches('/');

    // 尝试直接路径
    let mut logo_bytes = None;
    let mut mime = "image/png";

    // 尝试原路径
    if let Ok(mut file) = archive.by_name(clean_path) {
        let mut buf = Vec::new();
        if file.read_to_end(&mut buf).is_ok() && !buf.is_empty() {
            logo_bytes = Some(buf);
            mime = guess_mime(clean_path);
        }
    }

    // 如果原路径失败，尝试常见 logo 路径
    if logo_bytes.is_none() {
        let candidates = [
            clean_path.to_string(),
            format!("assets/{}", clean_path),
            format!("META-INF/{}", clean_path),
            "logo.png".to_string(),
            "icon.png".to_string(),
            "pack.png".to_string(),
        ];
        for path in &candidates {
            if let Ok(mut file) = archive.by_name(path) {
                let mut buf = Vec::new();
                if file.read_to_end(&mut buf).is_ok() && !buf.is_empty() {
                    logo_bytes = Some(buf);
                    mime = guess_mime(path);
                    break;
                }
            }
        }
    }

    let bytes = logo_bytes?;
    // 限制 256KB 防止过大图标
    if bytes.len() > 256 * 1024 {
        return None;
    }

    use base64::{engine::general_purpose, Engine as _};
    let b64 = general_purpose::STANDARD.encode(&bytes);
    Some(format!("data:{};base64,{}", mime, b64))
}

/// 根据文件扩展名猜测 MIME 类型
fn guess_mime(path: &str) -> &'static str {
    let lower = path.to_lowercase();
    if lower.ends_with(".png") {
        "image/png"
    } else if lower.ends_with(".jpg") || lower.ends_with(".jpeg") {
        "image/jpeg"
    } else if lower.ends_with(".gif") {
        "image/gif"
    } else if lower.ends_with(".webp") {
        "image/webp"
    } else {
        "image/png"
    }
}

/// 读取 fabric.mod.json 的 id / description / version / iconPath
fn read_fabric_mod_meta<R: std::io::Read + std::io::Seek>(
    archive: &mut zip::ZipArchive<R>,
) -> Option<ModMeta> {
    let mut file = archive.by_name("fabric.mod.json").ok()?;
    let mut content = String::new();
    file.read_to_string(&mut content).ok()?;
    let json: serde_json::Value = serde_json::from_str(&content).ok()?;

    let slug = json.get("id")?.as_str()?.trim().to_lowercase();
    let slug = if slug.is_empty() { None } else { Some(slug) };

    let description = json
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let version = json
        .get("version")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let icon_path = json
        .get("iconPath")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    Some(ModMeta {
        slug,
        description,
        version,
        icon_path,
        logo_file: None,
    })
}

/// 读取 META-INF/mods.toml 的 modId / description / version / logoFile（Forge 1.13+/NeoForge）
fn read_forge_mods_toml_meta<R: std::io::Read + std::io::Seek>(
    archive: &mut zip::ZipArchive<R>,
) -> Option<ModMeta> {
    // 先把 mods.toml 内容读到 String，drop 掉 ZipFile 借用，避免后续 read_manifest_version 二次借用
    let content = {
        let mut file = archive.by_name("META-INF/mods.toml").ok()?;
        let mut s = String::new();
        file.read_to_string(&mut s).ok()?;
        s
    };

    let mut slug: Option<String> = None;
    let mut description = String::new();
    let mut version = String::new();
    let mut logo_file: Option<String> = None;

    // 简化解析 TOML（避免引入 toml crate）
    // 检查 [[mods]] 块内的字段
    let mut in_mods_block = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("[[") {
            in_mods_block = trimmed == "[[mods]]";
            continue;
        }
        if !in_mods_block {
            continue;
        }
        // 解析 key = "value" 或 key = "value" # comment
        if let Some((key, value)) = parse_toml_kv(trimmed) {
            match key.as_str() {
                "modId" => {
                    if !value.is_empty() {
                        slug = Some(value.to_lowercase());
                    }
                }
                "description" => description = value,
                "version" => version = value,
                "logoFile" => logo_file = Some(value),
                _ => {}
            }
        }
    }

    // mods.toml 中 version 常为 "${file.jarVersion}" 占位符
    // 需从 JAR 内 META-INF/MANIFEST.MF 的 Implementation-Version 解析
    if version.contains("${") {
        if let Some(manifest_ver) = read_manifest_version(archive) {
            version = manifest_ver;
        } else {
            version = String::new();
        }
    }

    Some(ModMeta {
        slug,
        description,
        version,
        icon_path: None,
        logo_file,
    })
}

/// 从 META-INF/MANIFEST.MF 读取 Implementation-Version
/// 用于替换 mods.toml 中的 ${file.jarVersion} 占位符
fn read_manifest_version<R: std::io::Read + std::io::Seek>(
    archive: &mut zip::ZipArchive<R>,
) -> Option<String> {
    let mut file = archive.by_name("META-INF/MANIFEST.MF").ok()?;
    let mut content = String::new();
    file.read_to_string(&mut content).ok()?;

    // MANIFEST.MF 格式：每行 "Key: Value"
    // Implementation-Version 可能跨行续行（前导空格），但简化处理只看单行
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("Implementation-Version:") {
            let v = rest.trim();
            if !v.is_empty() {
                return Some(v.to_string());
            }
        }
    }
    None
}

/// 读取 mcmod.info 的 modid / description / version / logoFile（Forge 1.12-）
fn read_mcmod_info_meta<R: std::io::Read + std::io::Seek>(
    archive: &mut zip::ZipArchive<R>,
) -> Option<ModMeta> {
    let mut file = archive.by_name("mcmod.info").ok()?;
    let mut content = String::new();
    file.read_to_string(&mut content).ok()?;
    let json: serde_json::Value = serde_json::from_str(&content).ok()?;
    let arr = json
        .as_array()
        .or_else(|| json.get("modList")?.as_array())?;
    let first = arr.first()?;

    let slug = first
        .get("modid")
        .and_then(|v| v.as_str())
        .map(|s| s.trim().to_lowercase());
    let slug = slug.filter(|s| !s.is_empty());

    let description = first
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let version = first
        .get("version")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let logo_file = first
        .get("logoFile")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    Some(ModMeta {
        slug,
        description,
        version,
        icon_path: None,
        logo_file,
    })
}

/// 简化解析 TOML 单行 key = "value"（去除注释）
fn parse_toml_kv(line: &str) -> Option<(String, String)> {
    // 去除行尾注释（# 不在字符串内时才视为注释）
    let line = line.split('#').next()?.trim();
    let eq_pos = line.find('=')?;
    let key = line[..eq_pos].trim().to_string();
    let value_raw = line[eq_pos + 1..].trim();
    // 去除引号
    let value = value_raw
        .trim_matches('"')
        .trim_matches('\'')
        .to_string();
    if key.is_empty() {
        return None;
    }
    Some((key, value))
}

/// 查询 mcmod 译名（先查 CurseForge slug，再查 Modrinth slug）
fn lookup_translated(slug: &str) -> Option<String> {
    let slug = slug.trim().to_lowercase();
    if let Some(name) = crate::minecraft::community::mcmod::lookup_cf(&slug) {
        return Some(name.to_string());
    }
    if let Some(name) = crate::minecraft::community::mcmod::lookup_mr(&slug) {
        return Some(name.to_string());
    }
    None
}
