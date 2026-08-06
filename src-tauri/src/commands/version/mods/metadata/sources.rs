//! Mod 元数据的 4 个来源读取函数
//! 依据各加载器官方文件格式规范读取：1. mcmod.info（Forge 1.12-）；
//! 2. fabric.mod.json（Fabric/Quilt）；3. META-INF/mods.toml（Forge 1.13+/NeoForge）；
//! 4. META-INF/fml_cache_annotation.json（Forge 1.7-1.12 注解缓存）。

use std::collections::HashMap;
use std::io::Read;

use super::builder::MetaBuilder;

/// 合并 mcmod.info（Forge 1.12-）
///
/// 读取 mcmod.info，从第一个 mod 对象获取 modid/description/version
pub(super) fn merge_mcmod_info<R: Read + std::io::Seek>(
    archive: &mut zip::ZipArchive<R>,
    builder: &mut MetaBuilder,
) {
    let content = {
        let mut file = match archive.by_name("mcmod.info") {
            Ok(f) => f,
            Err(_) => return,
        };
        let mut s = String::new();
        if file.read_to_string(&mut s).is_err() || s.len() < 15 {
            return;
        }
        s
    };
    let json: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => return,
    };
    let arr = json
        .as_array()
        .or_else(|| json.get("modList").and_then(|v| v.as_array()));
    let first = match arr.and_then(|a| a.first()) {
        Some(f) => f,
        None => return,
    };

    let modid = first
        .get("modid")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    builder.set_slug(modid);

    if let Some(desc) = first.get("description").and_then(|v| v.as_str()) {
        builder.set_description(desc.to_string());
    }
    if let Some(ver) = first.get("version").and_then(|v| v.as_str()) {
        builder.set_version(ver.to_string());
    }
}

/// 合并 fabric.mod.json（Fabric/Quilt）
///
/// 必须包含 "schemaVersion" 才视为有效 fabric.mod.json
pub(super) fn merge_fabric_mod_json<R: Read + std::io::Seek>(
    archive: &mut zip::ZipArchive<R>,
    builder: &mut MetaBuilder,
) {
    let content = {
        let mut file = match archive.by_name("fabric.mod.json") {
            Ok(f) => f,
            Err(_) => return,
        };
        let mut s = String::new();
        if file.read_to_string(&mut s).is_err() {
            return;
        }
        s
    };
    // 检查：必须包含 "schemaVersion" 才是有效的 fabric.mod.json
    if !content.contains("schemaVersion") {
        return;
    }
    let json: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => return,
    };

    let id = json
        .get("id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    builder.set_slug(id);

    if let Some(desc) = json.get("description").and_then(|v| v.as_str()) {
        builder.set_description(desc.to_string());
    }
    if let Some(ver) = json.get("version").and_then(|v| v.as_str()) {
        builder.set_version(ver.to_string());
    }

    // 解析 depends 对象（key 是 mod_id），收集依赖列表
    if let Some(depends) = json.get("depends").and_then(|v| v.as_object()) {
        let dep_ids = depends.keys().map(|k| k.to_string());
        builder.add_dependencies(dep_ids);
    }
}

/// 合并 META-INF/mods.toml（Forge 1.13+/NeoForge）
///
/// 完整 TOML 解析，从 [[mods]] 块获取 modId/description/version
pub(super) fn merge_mods_toml<R: Read + std::io::Seek>(
    archive: &mut zip::ZipArchive<R>,
    builder: &mut MetaBuilder,
) {
    let content = {
        let mut file = match archive.by_name("META-INF/mods.toml") {
            Ok(f) => f,
            Err(_) => return,
        };
        let mut s = String::new();
        if file.read_to_string(&mut s).is_err() || s.len() < 15 {
            return;
        }
        s
    };

    // 文件标准化：去除注释、头尾空格、空行
    let lines: Vec<String> = content
        .lines()
        .filter_map(|line| {
            let line = if line.starts_with('#') {
                return None;
            } else if let Some(pos) = line.find('#') {
                &line[..pos]
            } else {
                line
            };
            let trimmed = line.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        })
        .collect();

    // 按段落分组
    let mut current_section = String::new();
    let mut current_fields: HashMap<String, String> = HashMap::new();
    let mut sections: Vec<(String, HashMap<String, String>)> = Vec::new();

    for line in &lines {
        if line.starts_with('[') && line.ends_with(']') {
            if !current_section.is_empty() || !current_fields.is_empty() {
                sections.push((current_section.clone(), current_fields.clone()));
            }
            current_section = line.trim_matches(|c| c == '[' || c == ']').to_string();
            current_fields.clear();
        } else if let Some(eq_pos) = line.find('=') {
            let key = line[..eq_pos].trim().to_string();
            let raw_value = line[eq_pos + 1..].trim();
            let value = raw_value.trim_matches('"').trim_matches('\'').to_string();
            if !key.is_empty() {
                current_fields.insert(key, value);
            }
        }
    }
    if !current_section.is_empty() || !current_fields.is_empty() {
        sections.push((current_section, current_fields));
    }

    // 从 [[mods]] 块获取信息
    let mod_entry = sections
        .iter()
        .find(|(header, _)| header == "mods")
        .map(|(_, fields)| fields);

    if let Some(fields) = mod_entry {
        if let Some(mod_id) = fields.get("modId") {
            builder.set_slug(Some(mod_id.clone()));
        }
        if let Some(desc) = fields.get("description") {
            builder.set_description(desc.clone());
        }
        if let Some(ver) = fields.get("version") {
            builder.set_version(ver.clone());
        }
    }

    // 从所有 [[dependencies]] 块收集 modId
    let dep_ids = sections
        .iter()
        .filter(|(header, _)| header == "dependencies")
        .filter_map(|(_, fields)| fields.get("modId").cloned());
    builder.add_dependencies(dep_ids);
}

/// 合并 META-INF/fml_cache_annotation.json（Forge 1.7-1.12 注解缓存）
///
/// 查找 @Mod 注解，从 values 中获取 version
pub(super) fn merge_fml_cache_annotation<R: Read + std::io::Seek>(
    archive: &mut zip::ZipArchive<R>,
    builder: &mut MetaBuilder,
) {
    let content = {
        let mut file = match archive.by_name("META-INF/fml_cache_annotation.json") {
            Ok(f) => f,
            Err(_) => return,
        };
        let mut s = String::new();
        if file.read_to_string(&mut s).is_err() {
            return;
        }
        s
    };
    if !content.contains("Lnet/minecraftforge/fml/common/Mod;") {
        return;
    }
    let json: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => return,
    };
    let obj = match json.as_object() {
        Some(o) => o,
        None => return,
    };

    // 遍历所有文件条目，查找 @Mod 注解
    for (_, file_value) in obj {
        let annotations = match file_value.get("annotations").and_then(|v| v.as_array()) {
            Some(a) => a,
            None => continue,
        };
        for anno in annotations {
            let name = anno.get("name").and_then(|v| v.as_str()).unwrap_or("");
            if name == "Lnet/minecraftforge/fml/common/Mod;" {
                let values = match anno.get("values").and_then(|v| v.as_object()) {
                    Some(o) => o,
                    None => continue,
                };
                if let Some(ver_obj) = values.get("version") {
                    if let Some(ver) = ver_obj.get("value").and_then(|v| v.as_str()) {
                        builder.set_version(ver.to_string());
                    }
                }
                return;
            }
        }
    }
}

/// 从 META-INF/MANIFEST.MF 读取 Implementation-Version
/// 用于替换 mods.toml 中的 ${file.jarVersion} 占位符
pub(super) fn read_manifest_version<R: Read + std::io::Seek>(
    archive: &mut zip::ZipArchive<R>,
) -> Option<String> {
    let mut file = archive.by_name("META-INF/MANIFEST.MF").ok()?;
    let mut content = String::new();
    file.read_to_string(&mut content).ok()?;

    let content = content.replace(" :", ":").replace(": ", ":");
    let prefix = "Implementation-Version:";
    for line in content.lines() {
        if let Some(rest) = line.trim().strip_prefix(prefix) {
            let v = rest.trim();
            if !v.is_empty() {
                return Some(v.to_string());
            }
        }
    }
    None
}
