//! 社区资源下载安装 - 纯工具函数
//!
//! 包含字节数格式化、文件名格式拼接、安装目录解析、加载器 ID 解析等纯函数。
//! 这些函数无副作用、无 I/O，便于单元测试与跨子模块复用。

use crate::minecraft::community::types::ResourceType;
use std::path::PathBuf;

/// 格式化字节数为人类可读大小（如 29.6 MB），用于日志输出
pub(super) fn format_bytes(bytes: u64) -> String {
    if bytes >= 1_048_576 {
        format!("{:.1} MB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}

/// 根据 `community_filename_format` 拼接文件名
///
/// 格式：
/// - 0: 【译名】原名
/// - 1: [译名] 原名（默认）
/// - 2: 译名-原名
/// - 3: 原名-译名
/// - 4: 仅原名
///
/// 无译名时统一返回原名。扩展名（含 .jar.disabled 等多段后缀）原样保留。
pub(super) fn apply_filename_format(original: &str, translated: Option<&str>, format: u8) -> String {
    let translated = match translated {
        Some(t) if !t.is_empty() => t,
        _ => return original.to_string(),
    };

    // 分离扩展名（保留 .jar.disabled / .jar.old 等多段后缀）
    let (stem, ext) = match original.rfind('.') {
        Some(pos) => {
            // .disabled / .old 是禁用后缀，继续向前找主扩展名
            let first_ext = &original[pos..];
            if first_ext == ".disabled" || first_ext == ".old" {
                let base = &original[..pos];
                if let Some(p2) = base.rfind('.') {
                    (base[..p2].to_string(), original[p2..].to_string())
                } else {
                    (original.to_string(), String::new())
                }
            } else {
                (original[..pos].to_string(), first_ext.to_string())
            }
        }
        None => (original.to_string(), String::new()),
    };

    let new_stem = match format {
        0 => format!("【{}】{}", translated, stem),
        1 => format!("[{}] {}", translated, stem),
        2 => format!("{}-{}", translated, stem),
        3 => format!("{}-{}", stem, translated),
        _ => stem.clone(), // 4 = 仅原名
    };

    if ext.is_empty() {
        new_stem
    } else {
        format!("{}{}", new_stem, ext)
    }
}

/// 解析安装目录
pub(super) fn resolve_install_dir(
    game_dir: &PathBuf,
    resource_type: ResourceType,
    version_id: Option<&str>,
) -> PathBuf {
    let subdir = resource_type.install_subdir();
    if let Some(vid) = version_id {
        if !vid.is_empty() && !subdir.is_empty() {
            game_dir.join("versions").join(vid).join(subdir)
        } else if !subdir.is_empty() {
            game_dir.join(subdir)
        } else {
            game_dir.clone()
        }
    } else if !subdir.is_empty() {
        game_dir.join(subdir)
    } else {
        game_dir.clone()
    }
}

/// 解析 CF loader id（如 "forge-36.2.39"）→ (loader_name, version)
pub(super) fn parse_cf_loader_id(id: &str) -> (String, String) {
    if let Some(pos) = id.find('-') {
        (id[..pos].to_string(), id[pos + 1..].to_string())
    } else {
        (id.to_string(), String::new())
    }
}

/// 解析 MR loader key/value → (loader_name, version)
pub(super) fn parse_mr_loader(key: &str, value: &str) -> (&'static str, String) {
    match key {
        "fabric-loader" => ("fabric", value.split('/').next().unwrap_or("").to_string()),
        "quilt-loader" => ("quilt", value.split('/').next().unwrap_or("").to_string()),
        "forge" => ("forge", value.to_string()),
        "neoforge" => ("neoforge", value.to_string()),
        _ => ("", value.to_string()),
    }
}

/// 从 Modrinth 下载 URL 提取 project_id
///
/// URL 格式：`https://cdn.modrinth.com/data/<project_id>/versions/<version_id>/<filename>`
/// 或镜像源 `https://mod.mcimirror.top/.../data/<project_id>/...` 等。
/// 提取失败返回 None（不影响下载，只是无法应用文件名格式）。
pub(super) fn extract_mr_project_id(url: &str) -> Option<String> {
    // 匹配 /data/<id>/ 片段，id 为字母数字短串（Modrinth 项目 ID 格式）
    if let Some(start) = url.find("/data/") {
        let rest = &url[start + "/data/".len()..];
        if let Some(end) = rest.find('/') {
            let id = &rest[..end];
            if !id.is_empty() && id.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
                return Some(id.to_string());
            }
        }
    }
    None
}

/// 构造 CF edge 下载 URL（当 download_url 为空时的 fallback）
pub(super) fn construct_cf_edge_url(file_id: i64, file_name: &str) -> String {
    let id_str = file_id.to_string();
    if id_str.len() >= 6 {
        let (p1, p2) = id_str.split_at(id_str.len() - 4);
        format!("https://edge.forgecdn.net/files/{}/{}", p1, p2)
    } else {
        format!("https://edge.forgecdn.net/files/0/{}", file_name)
    }
}
