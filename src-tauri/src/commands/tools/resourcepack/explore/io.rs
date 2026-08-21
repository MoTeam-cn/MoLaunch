//! 资源包编辑器读写实现（打开 / 单文件读 / 批量读 / 写回 / 导出）

use std::collections::{BTreeMap, HashSet, VecDeque};
use std::io::Read;
use std::path::PathBuf;

use base64::Engine;

use super::super::convert::unzip_to_dir;
use super::super::convert::zip_directory_with_comment;
use super::super::helpers::path_to_string;
use super::parse::{
    blockstate_model_refs, model_refs, namespace_of, pack_format_to_version, queue_model,
    read_media_data_uri, read_pack_meta, read_text_limited, texture_rel,
};
use super::tree::{
    build_tree, cleanup_work_dir, create_session_dir, dir_size, resolve_in_work_dir,
};
use super::{
    RpExportParams, RpExportResult, RpOpenParams, RpOpenResult, RpReadManyParams, RpReadManyResult,
    RpReadParams, RpReadResult, RpWriteParams, RpWriteResult,
};

use crate::error_util::log_err;

/// 文本类文件读取上限
const MAX_TEXT_SIZE: u64 = 2 * 1024 * 1024;
/// 图片/音频 data URI 读取上限
const MAX_MEDIA_SIZE: u64 = 20 * 1024 * 1024;

pub(crate) fn open_inner(params: &RpOpenParams) -> Result<RpOpenResult, String> {
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

pub(crate) fn read_inner(params: &RpReadParams) -> Result<RpReadResult, String> {
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

pub(crate) fn read_many_inner(params: &RpReadManyParams) -> Result<RpReadManyResult, String> {
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

pub(crate) fn write_inner(params: &RpWriteParams) -> Result<RpWriteResult, String> {
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

pub(crate) async fn export_inner(params: &RpExportParams) -> Result<RpExportResult, String> {
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
    // 路径安全：canonicalize 父目录后取文件名，解析真实路径（防符号链接 / `..` 逃逸）
    let parent_canon = parent
        .canonicalize()
        .map_err(|e| format!("目标目录不可用: {}", e))?;
    let file_name = target
        .file_name()
        .ok_or_else(|| "目标路径无效".to_string())?;
    let target = parent_canon.join(file_name);

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
