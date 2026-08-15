//! 资源包可视化编辑器：打开（zip/folder → 工作目录 + 结构树）与单文件读取。
//!
//! - zip 包：打开时解压到临时工作目录（安全解压，复用 convert::unzip_to_dir），
//!   后续读取全部基于工作目录，保存/导出（M2）才重新打包；
//! - folder 包：直接以原目录作为工作目录；
//! - 路径安全：读取路径先拦截 `..` 跳转，再经 canonicalize 校验必须位于工作目录内。

use std::collections::{BTreeMap, HashSet, VecDeque};
use std::io::Read;
use std::path::{Component, Path, PathBuf};

use base64::Engine;

use crate::log_info;
use crate::log_warn;
use crate::state::AppState;

use super::super::types::{
    RpExportParams, RpExportResult, RpOpenParams, RpOpenResult, RpPackFormatInfoParams,
    RpPackFormatInfoResult, RpReadManyParams, RpReadManyResult, RpReadParams, RpReadResult,
    RpTreeNode, RpVersionPackFormatParams, RpVersionPackFormatResult, RpWriteParams, RpWriteResult,
};
use super::convert::unzip_to_dir;
use super::convert::zip_directory_with_comment;
use super::helpers::path_to_string;

use crate::error_util::log_err;

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
                src_path: None,
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

/// 批量读取模型与关联纹理：blockstate → 模型（含 parent 链）→ 被引用纹理 png，
/// 供前端构建 lodestone Resources 做 3D 预览。缺失/损坏的关联文件静默跳过，不阻塞整批。
pub async fn rp_read_many(
    _state: &AppState,
    params: RpReadManyParams,
) -> Result<serde_json::Value, String> {
    let result = match read_many_inner(&params) {
        Ok(r) => r,
        Err(e) => {
            log_warn!("[RpReadMany] failed: root={} err={}", params.rel_path, e);
            RpReadManyResult {
                root: params.rel_path.clone(),
                files: BTreeMap::new(),
                error: e,
            }
        }
    };
    if result.error.is_empty() {
        log_info!(
            "[RpReadMany] success: root={} files={}",
            result.root,
            result.files.len()
        );
    }
    serde_json::to_value(&result).map_err(|e| e.to_string())
}

/// 查询 pack_format 对应的 MC 版本（供前端表单输入时联动校验提示）
pub async fn rp_pack_format_info(
    _state: &AppState,
    params: RpPackFormatInfoParams,
) -> Result<serde_json::Value, String> {
    let result = pack_format_info_inner(params.pack_format);
    serde_json::to_value(&result).map_err(|e| e.to_string())
}

fn pack_format_info_inner(fmt: u32) -> RpPackFormatInfoResult {
    let version = pack_format_to_version(fmt);
    RpPackFormatInfoResult {
        mc_version: version.to_string(),
        known: version != "未知版本",
        error: String::new(),
    }
}

/// 由 MC 版本推导 pack_format（复用皮肤资源包的版本映射，前端下拉选版本后自动回填）
pub async fn rp_version_pack_format(
    _state: &AppState,
    params: RpVersionPackFormatParams,
) -> Result<serde_json::Value, String> {
    let result = version_pack_format_inner(&params.mc_version);
    serde_json::to_value(&result).map_err(|e| e.to_string())
}

fn version_pack_format_inner(mc_version: &str) -> RpVersionPackFormatResult {
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
        unzip_to_dir(&src_canon, &dir).inspect_err(|_| {
            let _ = std::fs::remove_dir_all(&dir);
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
        src_path: if is_zip {
            Some(src_canon.to_string_lossy().to_string())
        } else {
            None
        },
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

fn read_many_inner(params: &RpReadManyParams) -> Result<RpReadManyResult, String> {
    let work_dir = PathBuf::from(&params.work_dir);
    resolve_in_work_dir(&work_dir, &params.rel_path)?;
    let lower = params.rel_path.to_lowercase();
    if !lower.contains("/blockstates/") && !lower.contains("/models/") {
        return Err("仅支持 blockstates 或 models 目录下的 JSON 文件".to_string());
    }

    let mut files: BTreeMap<String, RpReadResult> = BTreeMap::new();
    let mut visited: HashSet<String> = HashSet::new();
    let mut queue: VecDeque<String> = VecDeque::new();
    queue.push_back(params.rel_path.clone());

    while let Some(rel) = queue.pop_front() {
        if !visited.insert(rel.clone()) {
            continue;
        }
        let Some(text) = read_text_limited(&work_dir, &rel) else {
            continue; // 缺失（如 vanilla parent）/超限/非文本 → 跳过
        };
        let parsed = serde_json::from_str::<serde_json::Value>(&text);
        // 起始文件 JSON 损坏时直接报错，关联文件损坏则跳过
        if rel == params.rel_path {
            if let Err(e) = &parsed {
                return Err(format!("JSON 解析失败: {}", e));
            }
        }
        let Ok(v) = parsed else { continue };
        let default_ns = namespace_of(&rel).unwrap_or("minecraft");

        files.insert(
            rel.clone(),
            RpReadResult {
                kind: "text".to_string(),
                content: text,
                error: String::new(),
            },
        );

        if rel.to_lowercase().contains("/blockstates/") {
            for model in blockstate_model_refs(&v) {
                queue_model(&mut queue, &model, default_ns);
            }
        } else {
            let (parent, textures) = model_refs(&v);
            if let Some(p) = parent {
                queue_model(&mut queue, &p, default_ns);
            }
            for tex in textures {
                let tex_rel = texture_rel(&tex, default_ns);
                if visited.contains(&tex_rel) {
                    continue;
                }
                visited.insert(tex_rel.clone());
                if let Some(content) = read_media_data_uri(&work_dir, &tex_rel) {
                    files.insert(
                        tex_rel,
                        RpReadResult {
                            kind: "data_uri".to_string(),
                            content,
                            error: String::new(),
                        },
                    );
                }
            }
        }
    }

    Ok(RpReadManyResult {
        root: params.rel_path.clone(),
        files,
        error: String::new(),
    })
}

/// 模型 id 可能无命名空间：优先当前文件命名空间，再补 minecraft，缺失项由读取时跳过
fn queue_model(queue: &mut VecDeque<String>, raw: &str, default_ns: &str) {
    let (ns, path) = split_id(raw, default_ns);
    queue.push_back(format!("assets/{}/models/{}.json", ns, path));
    if raw.find(':').is_none() && ns != "minecraft" {
        queue.push_back(format!("assets/minecraft/models/{}.json", path));
    }
}

fn texture_rel(raw: &str, default_ns: &str) -> String {
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
fn namespace_of(rel: &str) -> Option<&str> {
    let mut parts = rel.split('/');
    match (parts.next(), parts.next()) {
        (Some("assets"), Some(ns)) if !ns.is_empty() => Some(ns),
        _ => None,
    }
}

/// blockstate JSON 的模型引用：variants 与 multipart.apply
fn blockstate_model_refs(v: &serde_json::Value) -> Vec<String> {
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
fn model_refs(v: &serde_json::Value) -> (Option<String>, Vec<String>) {
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
fn read_text_limited(work_dir: &Path, rel: &str) -> Option<String> {
    let target = resolve_in_work_dir(work_dir, rel).ok()?;
    if std::fs::metadata(&target).ok()?.len() > MAX_TEXT_SIZE {
        return None;
    }
    std::fs::read_to_string(&target).ok()
}

/// 读图片为 base64 data URI（限 MAX_MEDIA_SIZE），缺失/超限/读取失败返回 None
fn read_media_data_uri(work_dir: &Path, rel: &str) -> Option<String> {
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

/// 写回包内单文件：text 原文 / base64 二进制（复用 rp_read 的路径安全校验）
pub async fn rp_write(
    _state: &AppState,
    params: RpWriteParams,
) -> Result<serde_json::Value, String> {
    let result = match write_inner(&params) {
        Ok(r) => r,
        Err(e) => {
            log_warn!("[RpWrite] failed: rel={} err={}", params.rel_path, e);
            RpWriteResult {
                success: false,
                message: e,
            }
        }
    };
    if result.success {
        log_info!(
            "[RpWrite] success: rel={} kind={}",
            params.rel_path,
            params.kind
        );
    }
    serde_json::to_value(&result).map_err(|e| e.to_string())
}

fn write_inner(params: &RpWriteParams) -> Result<RpWriteResult, String> {
    let work_dir = PathBuf::from(&params.work_dir);
    let target = resolve_in_work_dir(&work_dir, &params.rel_path)?;

    match params.kind.to_lowercase().as_str() {
        "text" => {
            if params.content.len() as u64 > MAX_TEXT_SIZE {
                return Err(format!(
                    "内容超过 {}MB，拒绝写入",
                    MAX_TEXT_SIZE / 1024 / 1024
                ));
            }
            std::fs::write(&target, &params.content).map_err(|e| format!("写入文件失败: {}", e))?;
        }
        "base64" => {
            let raw = params.content.trim();
            let b64 = raw
                .strip_prefix("data:")
                .and_then(|s| s.split_once(','))
                .map(|(_, data)| data)
                .unwrap_or(raw);
            let bytes = base64::engine::general_purpose::STANDARD
                .decode(b64)
                .map_err(|e| format!("base64 解码失败: {}", e))?;
            if bytes.len() as u64 > MAX_MEDIA_SIZE {
                return Err(format!(
                    "文件超过 {}MB，拒绝写入",
                    MAX_MEDIA_SIZE / 1024 / 1024
                ));
            }
            std::fs::write(&target, &bytes).map_err(|e| format!("写入文件失败: {}", e))?;
        }
        other => return Err(format!("不支持的写入类型: {}", other)),
    }
    Ok(RpWriteResult {
        success: true,
        message: "保存成功".to_string(),
    })
}

/// 导出资源包：zip 会话保存回原 zip / 另存为 zip（folder 会话保存即直写，仅另存为走此接口）
pub async fn rp_export(
    _state: &AppState,
    params: RpExportParams,
) -> Result<serde_json::Value, String> {
    let result = match export_inner(&params).await {
        Ok(r) => r,
        Err(e) => {
            log_warn!("[RpExport] failed: path={} err={}", params.path, e);
            RpExportResult {
                success: false,
                output_path: params.path.clone(),
                message: e,
            }
        }
    };
    if result.success {
        log_info!("[RpExport] success: path={}", result.output_path);
    }
    serde_json::to_value(&result).map_err(|e| e.to_string())
}

async fn export_inner(params: &RpExportParams) -> Result<RpExportResult, String> {
    if params.format.to_lowercase() != "zip" {
        return Err("当前仅支持导出为 zip".to_string());
    }
    let work_dir = PathBuf::from(&params.work_dir);
    let work_canon = work_dir
        .canonicalize()
        .map_err(|e| format!("工作目录不可用: {}", e))?;
    if !work_canon.is_dir() {
        return Err("工作目录不可用".to_string());
    }
    let target = PathBuf::from(&params.path);
    let parent = target.parent().ok_or_else(|| "目标路径无效".to_string())?;
    if !parent.exists() {
        return Err(format!("目标目录不存在: {}", parent.display()));
    }
    if target.is_dir() {
        return Err("目标路径不能是目录".to_string());
    }

    // 写临时文件后原子替换，避免打包失败损坏原 zip
    let final_target = target.clone();
    let tmp_target = final_target.with_extension("tmp");
    let src_zip: Option<PathBuf> = params.src_path.as_ref().map(PathBuf::from);
    let work_clone = work_canon.clone();
    let tmp_clone = tmp_target.clone();
    let export_result = tokio::task::spawn_blocking(move || {
        zip_directory_with_comment(&work_clone, &tmp_clone, src_zip.as_deref())
    })
    .await
    .map_err(log_err("资源包导出任务失败"))?;
    if let Err(e) = export_result {
        let _ = std::fs::remove_file(&tmp_target);
        return Err(e);
    }
    if final_target.exists() {
        std::fs::remove_file(&final_target).map_err(|e| format!("替换目标文件失败: {}", e))?;
    }
    std::fs::rename(&tmp_target, &final_target).map_err(|e| format!("写入目标文件失败: {}", e))?;

    Ok(RpExportResult {
        success: true,
        output_path: path_to_string(&target),
        message: "导出成功".to_string(),
    })
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
