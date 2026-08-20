//! 资源包解析：pack 元数据 / 文件分类 / 模型引用提取 / 受限读取

use std::collections::VecDeque;
use std::io::Read;
use std::path::Path;

use base64::Engine;

use super::tree::resolve_in_work_dir;
use super::{RpPackFormatInfoResult, RpVersionPackFormatResult};

/// 文本类文件读取上限
const MAX_TEXT_SIZE: u64 = 2 * 1024 * 1024;
/// 图片/音频 data URI 读取上限
const MAX_MEDIA_SIZE: u64 = 20 * 1024 * 1024;

pub(crate) fn pack_format_info_inner(fmt: u32) -> RpPackFormatInfoResult {
    let version = pack_format_to_version(fmt);
    RpPackFormatInfoResult {
        mc_version: version.to_string(),
        known: version != "未知版本",
        error: String::new(),
    }
}

pub(crate) fn version_pack_format_inner(mc_version: &str) -> RpVersionPackFormatResult {
    let pack_format = crate::minecraft::launch::skin_resourcepack::get_pack_format(mc_version);
    let known = !mc_version.trim().is_empty()
        && crate::utils::version::parse_number(mc_version)
            .first()
            .copied()
            .unwrap_or(0)
            > 0;
    RpVersionPackFormatResult {
        pack_format,
        known,
        error: String::new(),
    }
}

/// 模型 id 可能无命名空间：优先当前文件命名空间，再补 minecraft，缺失项由读取时跳过
pub(crate) fn queue_model(queue: &mut VecDeque<String>, raw: &str, default_ns: &str) {
    let (ns, path) = split_id(raw, default_ns);
    queue.push_back(format!("assets/{}/models/{}.json", ns, path));
    if raw.find(':').is_none() && ns != "minecraft" {
        queue.push_back(format!("assets/minecraft/models/{}.json", path));
    }
}

pub(crate) fn texture_rel(raw: &str, default_ns: &str) -> String {
    let (ns, path) = split_id(raw, default_ns);
    format!("assets/{}/textures/{}.png", ns, path)
}

/// "ns:path" 或 "path"（用默认命名空间补全）
fn split_id(raw: &str, default_ns: &str) -> (String, String) {
    let trimmed = raw.trim();
    match trimmed.find(':') {
        Some(idx) => (trimmed[..idx].to_string(), trimmed[idx + 1..].to_string()),
        None => (default_ns.to_string(), trimmed.to_string()),
    }
}

/// 从相对路径取命名空间（assets/<ns>/...）
pub(crate) fn namespace_of(rel: &str) -> Option<&str> {
    let mut parts = rel.split('/');
    match (parts.next(), parts.next()) {
        (Some("assets"), Some(ns)) if !ns.is_empty() => Some(ns),
        _ => None,
    }
}

/// blockstate JSON 的模型引用：variants 与 multipart.apply
pub(crate) fn blockstate_model_refs(v: &serde_json::Value) -> Vec<String> {
    let mut refs = Vec::new();
    if let Some(variants) = v.get("variants").and_then(|x| x.as_object()) {
        for entry in variants.values() {
            collect_model_entry(entry, &mut refs);
        }
    }
    if let Some(multipart) = v.get("multipart").and_then(|x| x.as_array()) {
        for part in multipart {
            if let Some(apply) = part.get("apply") {
                collect_model_entry(apply, &mut refs);
            }
        }
    }
    refs
}

/// model 引用的三种形态：字符串 / {model,..} / 数组
fn collect_model_entry(v: &serde_json::Value, out: &mut Vec<String>) {
    match v {
        serde_json::Value::String(s) => out.push(s.clone()),
        serde_json::Value::Object(o) => {
            if let Some(m) = o.get("model").and_then(|x| x.as_str()) {
                out.push(m.to_string());
            }
        }
        serde_json::Value::Array(arr) => {
            for item in arr {
                if let Some(m) = item.get("model").and_then(|x| x.as_str()) {
                    out.push(m.to_string());
                }
            }
        }
        _ => {}
    }
}

/// model JSON 的 parent 与纹理引用（跳过 `#key` 内部引用）
pub(crate) fn model_refs(v: &serde_json::Value) -> (Option<String>, Vec<String>) {
    let parent = v
        .get("parent")
        .and_then(|x| x.as_str())
        .map(|s| s.to_string());
    let mut textures = Vec::new();
    if let Some(tex) = v.get("textures").and_then(|x| x.as_object()) {
        for val in tex.values() {
            if let Some(s) = val.as_str() {
                if !s.is_empty() && !s.starts_with('#') {
                    textures.push(s.to_string());
                }
            }
        }
    }
    (parent, textures)
}

/// 读文本（限 MAX_TEXT_SIZE），缺失/超限/非 UTF-8 返回 None
pub(crate) fn read_text_limited(work_dir: &Path, rel: &str) -> Option<String> {
    let target = resolve_in_work_dir(work_dir, rel).ok()?;
    if std::fs::metadata(&target).ok()?.len() > MAX_TEXT_SIZE {
        return None;
    }
    std::fs::read_to_string(&target).ok()
}

/// 读图片为 base64 data URI（限 MAX_MEDIA_SIZE），缺失/超限/读取失败返回 None
pub(crate) fn read_media_data_uri(work_dir: &Path, rel: &str) -> Option<String> {
    let target = resolve_in_work_dir(work_dir, rel).ok()?;
    if std::fs::metadata(&target).ok()?.len() > MAX_MEDIA_SIZE {
        return None;
    }
    let mut bytes = Vec::new();
    std::fs::File::open(&target)
        .ok()?
        .read_to_end(&mut bytes)
        .ok()?;
    Some(format!(
        "data:image/png;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(&bytes)
    ))
}

/// 读取 pack.mcmeta 的 pack_format 与 description
pub(crate) fn read_pack_meta(work_dir: &Path) -> (Option<u32>, Option<String>) {
    let content = match std::fs::read_to_string(work_dir.join("pack.mcmeta")) {
        Ok(c) => c,
        Err(_) => return (None, None),
    };
    let v: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => return (None, None),
    };
    let pack = v.get("pack");
    let pf = pack
        .and_then(|p| p.get("pack_format"))
        .and_then(|x| x.as_u64())
        .map(|x| x as u32);
    let desc = pack
        .and_then(|p| p.get("description"))
        .and_then(|d| match d {
            serde_json::Value::String(s) => Some(s.clone()),
            serde_json::Value::Array(arr) => Some(
                arr.iter()
                    .filter_map(|x| x.as_str())
                    .collect::<Vec<_>>()
                    .join(""),
            ),
            serde_json::Value::Object(obj) => obj
                .get("text")
                .and_then(|t| t.as_str())
                .map(|s| s.to_string()),
            _ => None,
        });
    (pf, desc)
}

/// pack_format → MC 版本范围描述（与 skin_resourcepack/generate.rs 的 get_pack_format 映射互补）
pub(crate) fn pack_format_to_version(fmt: u32) -> &'static str {
    match fmt {
        1 => "1.6–1.8",
        2 => "1.9–1.10",
        3 => "1.11",
        4 => "1.12",
        5 => "1.13",
        6 => "1.14–1.16.1",
        7 => "1.16.2–1.16.5",
        8 => "1.17",
        9 => "1.18–1.19.2",
        12 => "1.19.3",
        13 => "1.19.4",
        15 => "1.19.5–1.20.1",
        18 => "1.20.2",
        22 => "1.20.3–1.20.4",
        34 => "1.20.5–1.21.x",
        _ => "未知版本",
    }
}

/// 按相对路径分类文件类型
pub(crate) fn classify_file(rel: &str) -> &'static str {
    if rel == "pack.mcmeta" {
        return "mcmeta";
    }
    let lower = rel.to_lowercase();
    if lower.ends_with(".png") {
        return "png";
    }
    if lower.ends_with(".ogg") {
        return "ogg";
    }
    if lower.ends_with(".json") {
        if lower.contains("/lang/") {
            return "lang";
        }
        if lower.contains("/models/") {
            return "model";
        }
        return "json";
    }
    if lower.ends_with(".txt") || lower.ends_with(".properties") || lower.ends_with(".mcmeta") {
        return "text";
    }
    "other"
}
