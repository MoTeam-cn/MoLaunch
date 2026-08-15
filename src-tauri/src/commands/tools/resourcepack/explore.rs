//! 资源包可视化编辑器：打开（zip/folder → 工作目录 + 结构树）与单文件读取。
//!
//! - zip 包：打开时解压到临时工作目录（安全解压，复用 convert::unzip_to_dir），
//!   后续读取全部基于工作目录，保存/导出（M2）才重新打包；
//! - folder 包：直接以原目录作为工作目录；
//! - 路径安全：读取路径先拦截 `..` 跳转，再经 canonicalize 校验必须位于工作目录内。

use std::io::Read;
use std::path::{Component, Path, PathBuf};

use base64::Engine;

use crate::log_info;
use crate::log_warn;
use crate::state::AppState;

use super::super::types::{RpOpenParams, RpOpenResult, RpReadParams, RpReadResult, RpTreeNode};
use super::convert::unzip_to_dir;

/// 文本类文件读取上限
const MAX_TEXT_SIZE: u64 = 2 * 1024 * 1024;
/// 图片/音频 data URI 读取上限
const MAX_MEDIA_SIZE: u64 = 20 * 1024 * 1024;

/// 打开资源包：zip 解压到临时工作目录 / folder 直接使用原目录，返回包信息与结构树
///
/// 失败时返回带 `error` 字段的 `RpOpenResult`（而非 Err），前端在页面内展示原因。
pub async fn rp_open(_state: &AppState, params: RpOpenParams) -> Result<serde_json::Value, String> {
    let result = match open_inner(&params) {
        Ok(r) => r,
        Err(e) => {
            log_warn!("[RpOpen] failed: path={} err={}", params.path, e);
            RpOpenResult {
                work_dir: String::new(),
                is_zip: false,
                name: String::new(),
                format: String::new(),
                size: 0,
                icon_data_url: None,
                pack_format: None,
                mc_version: None,
                description: None,
                tree: empty_tree(),
                error: e,
            }
        }
    };
    if result.error.is_empty() {
        log_info!(
            "[RpOpen] success: name={} format={} size={}",
            result.name,
            result.format,
            result.size
        );
    }
    serde_json::to_value(&result).map_err(|e| e.to_string())
}

/// 读取包内单文件：png/ogg → base64 data URI，json/lang/文本 → 原文
pub async fn rp_read(_state: &AppState, params: RpReadParams) -> Result<serde_json::Value, String> {
    let result = match read_inner(&params) {
        Ok(r) => r,
        Err(e) => {
            log_warn!("[RpRead] failed: rel={} err={}", params.rel_path, e);
            RpReadResult {
                kind: String::new(),
                content: String::new(),
                error: e,
            }
        }
    };
    serde_json::to_value(&result).map_err(|e| e.to_string())
}

fn open_inner(params: &RpOpenParams) -> Result<RpOpenResult, String> {
    let src = PathBuf::from(&params.path);
    if !src.exists() {
        return Err(format!("路径不存在: {}", params.path));
    }

    // 打开新包前清理上一 zip 会话的临时工作目录（folder 会话位于包目录内，前缀校验会跳过）
    if let Some(prev) = &params.previous_work_dir {
        cleanup_work_dir(prev);
    }

    let src_canon = src
        .canonicalize()
        .map_err(|e| format!("路径解析失败: {}", e))?;
    let is_zip = src_canon.is_file()
        && src_canon
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.eq_ignore_ascii_case("zip"))
            .unwrap_or(false);

    let (work_dir, format, name) = if is_zip {
        let dir = create_session_dir()?;
        unzip_to_dir(&src_canon, &dir).map_err(|e| {
            let _ = std::fs::remove_dir_all(&dir);
            e
        })?;
        (
            dir,
            "zip".to_string(),
            src_canon
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("pack")
                .to_string(),
        )
    } else if src_canon.is_dir() {
        (
            src_canon.clone(),
            "folder".to_string(),
            src_canon
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("pack")
                .to_string(),
        )
    } else {
        return Err("仅支持 .zip 压缩包或文件夹资源包".to_string());
    };

    let size = if is_zip {
        src_canon.metadata().map(|m| m.len()).unwrap_or(0)
    } else {
        dir_size(&work_dir)
    };
    let (pack_format, description) = read_pack_meta(&work_dir);
    let mc_version = pack_format.map(|f| pack_format_to_version(f).to_string());
    let icon_data_url =
        crate::commands::version::packs::icon::extract_pack_icon_data_url(&work_dir);
    let mut tree = build_tree(&work_dir, "");
    tree.name = name.clone();

    Ok(RpOpenResult {
        work_dir: work_dir.to_string_lossy().to_string(),
        is_zip,
        name,
        format,
        size,
        icon_data_url,
        pack_format,
        mc_version,
        description,
        tree,
        error: String::new(),
    })
}

fn read_inner(params: &RpReadParams) -> Result<RpReadResult, String> {
    let work_dir = PathBuf::from(&params.work_dir);
    let target = resolve_in_work_dir(&work_dir, &params.rel_path)?;

    let lower = params.rel_path.to_lowercase();
    let is_media = lower.ends_with(".png") || lower.ends_with(".ogg");
    let limit = if is_media {
        MAX_MEDIA_SIZE
    } else {
        MAX_TEXT_SIZE
    };
    let meta = std::fs::metadata(&target).map_err(|e| format!("读取文件信息失败: {}", e))?;
    if meta.len() > limit {
        let mb = limit / 1024 / 1024;
        return Err(format!("文件超过 {}MB，跳过预览", mb));
    }

    if is_media {
        let mut bytes = Vec::new();
        std::fs::File::open(&target)
            .map_err(|e| format!("打开文件失败: {}", e))?
            .read_to_end(&mut bytes)
            .map_err(|e| format!("读取文件失败: {}", e))?;
        let mime = if lower.ends_with(".png") {
            "image/png"
        } else {
            "audio/ogg"
        };
        Ok(RpReadResult {
            kind: "data_uri".to_string(),
            content: format!(
                "data:{};base64,{}",
                mime,
                base64::engine::general_purpose::STANDARD.encode(&bytes)
            ),
            error: String::new(),
        })
    } else {
        let text =
            std::fs::read_to_string(&target).map_err(|e| format!("文件不是有效文本: {}", e))?;
        Ok(RpReadResult {
            kind: "text".to_string(),
            content: text,
            error: String::new(),
        })
    }
}

/// 路径安全：拦截绝对路径与 `..` 跳转，canonicalize 后必须位于工作目录内
fn resolve_in_work_dir(work_dir: &Path, rel_path: &str) -> Result<PathBuf, String> {
    let rel = Path::new(rel_path);
    if rel.is_absolute() {
        return Err("路径必须为包内相对路径".to_string());
    }
    if rel.components().any(|c| matches!(c, Component::ParentDir)) {
        return Err("路径包含非法跳转 (..)".to_string());
    }
    let work_canon = work_dir
        .canonicalize()
        .map_err(|e| format!("工作目录不可用: {}", e))?;
    let target = work_dir
        .join(rel)
        .canonicalize()
        .map_err(|e| format!("文件不存在: {}", e))?;
    if !target.starts_with(&work_canon) {
        return Err("路径超出资源包范围".to_string());
    }
    if !target.is_file() {
        return Err("目标不是文件".to_string());
    }
    Ok(target)
}

/// 创建 zip 会话的临时工作目录（temp_dir/molaunch-rp/rp-{pid}-{纳秒时间戳}）
fn create_session_dir() -> Result<PathBuf, String> {
    let root = std::env::temp_dir().join("molaunch-rp");
    std::fs::create_dir_all(&root).map_err(|e| format!("创建临时目录失败: {}", e))?;
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let dir = root.join(format!("rp-{}-{}", std::process::id(), nanos));
    std::fs::create_dir_all(&dir).map_err(|e| format!("创建临时目录失败: {}", e))?;
    Ok(dir)
}

/// 清理 zip 会话临时工作目录（仅允许删除 molaunch-rp 根下的会话目录）
fn cleanup_work_dir(dir: &str) {
    let root = std::env::temp_dir().join("molaunch-rp");
    let root_str = root.to_string_lossy().replace('\\', "/");
    let dir_str = dir.replace('\\', "/");
    if dir_str.starts_with(&root_str) && dir_str.len() > root_str.len() {
        let _ = std::fs::remove_dir_all(dir);
    }
}

/// 递归统计目录总大小
fn dir_size(dir: &Path) -> u64 {
    let mut total = 0u64;
    if let Ok(rd) = std::fs::read_dir(dir) {
        for entry in rd.flatten() {
            let path = entry.path();
            if path.is_dir() {
                total += dir_size(&path);
            } else if let Ok(meta) = std::fs::metadata(&path) {
                total += meta.len();
            }
        }
    }
    total
}

/// 读取 pack.mcmeta 的 pack_format 与 description
fn read_pack_meta(work_dir: &Path) -> (Option<u32>, Option<String>) {
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
fn pack_format_to_version(fmt: u32) -> &'static str {
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

/// 递归构建结构树（目录在前，文件按类型分类；动画纹理标记同名 .png.mcmeta 存在）
fn build_tree(dir: &Path, rel: &str) -> RpTreeNode {
    let name = dir
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_string();
    let mut node = RpTreeNode {
        name,
        rel_path: rel.to_string(),
        kind: "dir".to_string(),
        file_type: String::new(),
        size: 0,
        animated: false,
        children: Vec::new(),
    };
    let mut entries = std::fs::read_dir(dir)
        .map(|r| r.flatten().collect::<Vec<_>>())
        .unwrap_or_default();
    entries.sort_by_key(|e| (e.path().is_file(), e.file_name()));
    for entry in entries {
        let path = entry.path();
        let fname = entry.file_name().to_string_lossy().to_string();
        let child_rel = if rel.is_empty() {
            fname.clone()
        } else {
            format!("{}/{}", rel, fname)
        };
        if path.is_dir() {
            let mut child = build_tree(&path, &child_rel);
            node.size += child.size;
            child.name = fname;
            node.children.push(child);
        } else if path.is_file() {
            let size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
            node.size += size;
            let is_png = path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.eq_ignore_ascii_case("png"))
                .unwrap_or(false);
            let animated =
                is_png && Path::new(&format!("{}.mcmeta", path.to_string_lossy())).exists();
            let file_type = classify_file(&child_rel).to_string();
            node.children.push(RpTreeNode {
                name: fname,
                rel_path: child_rel,
                kind: "file".to_string(),
                file_type,
                size,
                animated,
                children: Vec::new(),
            });
        }
    }
    node
}

/// 按相对路径分类文件类型
fn classify_file(rel: &str) -> &'static str {
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

/// 打开失败时的空结构树
fn empty_tree() -> RpTreeNode {
    RpTreeNode {
        name: String::new(),
        rel_path: String::new(),
        kind: "dir".to_string(),
        file_type: String::new(),
        size: 0,
        animated: false,
        children: Vec::new(),
    }
}

#[cfg(test)]
#[path = "explore_test.rs"]
mod explore_test;
