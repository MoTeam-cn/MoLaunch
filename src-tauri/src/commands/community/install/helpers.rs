//! 社区资源下载安装 - 纯工具函数
//! 包含文件名格式拼接、安装目录解析、加载器 ID 解析等纯函数。
//! 这些函数无副作用、无 I/O，便于单元测试与跨子模块复用。
//! 注：字节数格式化已迁移到 `crate::utils::format::bytes`。

use crate::minecraft::community::types::ResourceType;
use std::path::{Path, PathBuf};

/// 根据 `community_filename_format` 拼接文件名
///
/// 格式：0=【译名】原名 / 1=[译名] 原名（默认）/ 2=译名-原名 / 3=原名-译名 / 4=仅原名。
/// 无译名返回原名；扩展名（含 .jar.disabled 等多段后缀）原样保留；译名中 Windows
/// 非法字符（< > : " / \ | ? *）替换为下划线，避免 `File::create` 报 os error。
pub(super) fn apply_filename_format(
    original: &str,
    translated: Option<&str>,
    format: u8,
) -> String {
    let translated = match translated {
        Some(t) if !t.is_empty() => t,
        _ => return original.to_string(),
    };

    // 过滤 Windows 文件名非法字符（mcmod 译名可能含 : ？ 等）
    // 控制字符（0-31）也一并过滤
    let sanitized: String = translated
        .chars()
        .map(|c| {
            if matches!(c, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*') || (c as u32) < 32
            {
                '_'
            } else {
                c
            }
        })
        .collect();

    let translated = if sanitized.is_empty() {
        return original.to_string();
    } else {
        sanitized.as_str()
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
    game_dir: &Path,
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
            game_dir.to_path_buf()
        }
    } else if !subdir.is_empty() {
        game_dir.join(subdir)
    } else {
        game_dir.to_path_buf()
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
/// 或镜像源 `https://cdn-modrinth.mocdn.net/.../data/<project_id>/...` 等。
/// 提取失败返回 None（不影响下载，只是无法应用文件名格式）。
pub(super) fn extract_mr_project_id(url: &str) -> Option<String> {
    // 匹配 /data/<id>/ 片段，id 为字母数字短串（Modrinth 项目 ID 格式）
    if let Some(start) = url.find("/data/") {
        let rest = &url[start + "/data/".len()..];
        if let Some(end) = rest.find('/') {
            let id = &rest[..end];
            if !id.is_empty()
                && id
                    .chars()
                    .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
            {
                return Some(id.to_string());
            }
        }
    }
    None
}

/// 构造 CF edge 下载 URL（download_url 为空、且推导开关已开启时的 fallback）
///
/// 实现收敛在社区资源层 [`crate::minecraft::community::curseforge::construct_edge_url`]：
/// 路径布局为 `files/{id/1000}/{id%1000}/{百分号编码后的文件名}`。始终用官方域名
/// 构造，镜像替换由调用方通过 `cdn_urls()` 处理（source=0 时替换为 mocdn 镜像）。
pub(super) fn construct_cf_edge_url(file_id: i64, file_name: &str) -> String {
    crate::minecraft::community::curseforge::construct_edge_url(file_id, file_name)
}

/// 解析 CurseForge 文件的可用下载地址
///
/// - 官方 API 返回了地址：原样使用
/// - 地址为空（作者禁止第三方分发）：`derive` 为 true 时按 fileId 推导 CDN 地址，
///   否则返回 `None`，由调用方给出手动下载提示
///
/// 推导是开发者选项（`download.cf_derive_url`）：它依赖 CurseForge 未公开的 CDN
/// 路径布局，默认关闭。
pub(super) fn resolve_cf_download_url(
    download_url: Option<&str>,
    file_id: i64,
    file_name: &str,
    derive: bool,
) -> Option<String> {
    match download_url.filter(|url| !url.is_empty()) {
        Some(url) => Some(url.to_string()),
        None if derive => Some(construct_cf_edge_url(file_id, file_name)),
        None => None,
    }
}

/// 校验整合包实例名是否合法
///
/// 入口校验规则：
/// - 不能为空
/// - 不能含 `!` 或 `;`（Java Classpath / 路径分隔符，Windows 路径含这些字符会导致 Java 启动失败）
/// - 不能含 Windows 文件名非法字符 `< > : " / \ | ? *`
/// - 不能含控制字符（0-31）
///
/// 返回 `Ok(())` 表示合法，`Err(reason)` 表示非法并给出具体原因。
pub(super) fn validate_instance_name(name: &str) -> Result<(), String> {
    if name.trim().is_empty() {
        return Err("整合包实例名不能为空".to_string());
    }
    for c in name.chars() {
        if c == '!' {
            return Err(
                "整合包实例名不能包含 \"!\" 字符（Java Classpath 分隔符，会导致启动失败）"
                    .to_string(),
            );
        }
        if c == ';' {
            return Err(
                "整合包实例名不能包含 \";\" 字符（路径分隔符，会导致启动失败）".to_string(),
            );
        }
        // / 与 \ 必须显式拒绝：实例名会 join 进 <game_dir>/versions/，
        // 漏掉分隔符即可用 ../ 逃出该目录（文档一直声称拒绝，实现曾漏掉）
        if matches!(c, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*') {
            return Err(format!("整合包实例名不能包含 Windows 非法字符 \"{}\"", c));
        }
        if (c as u32) < 32 {
            return Err("整合包实例名不能包含控制字符".to_string());
        }
    }
    // Windows 不允许文件夹名以 . 或空格结尾
    if name.ends_with('.') || name.ends_with(' ') {
        return Err("整合包实例名不能以 \".\" 或空格结尾".to_string());
    }
    // Windows 保留名（CON / PRN / AUX / NUL / COM1-9 / LPT1-9）
    let upper = name.to_uppercase();
    let reserved = [
        "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
        "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
    ];
    if reserved.contains(&upper.as_str()) {
        return Err(format!("整合包实例名不能使用 Windows 保留名 \"{}\"", name));
    }
    Ok(())
}

/// 校验整合包文件扩展名是否被支持
///
/// 拒绝 `.rar`（无开源解压库支持）和未识别扩展名。
/// 返回 `Ok(())` 表示可继续安装，`Err(reason)` 表示拒绝并给出原因。
pub(super) fn validate_modpack_extension(file_path: &str) -> Result<(), String> {
    let lower = file_path.to_lowercase();
    let ext = match lower.rfind('.') {
        Some(p) => &lower[p + 1..],
        None => "",
    };
    match ext {
        "zip" | "mrpack" => Ok(()),
        "rar" => {
            Err("MoLaunch 无法处理 rar 格式的压缩包，请解压后重新压缩为 zip 格式再试。".to_string())
        }
        _ => Err(format!(
            "不支持的整合包扩展名 .{}。仅支持 .zip / .mrpack。",
            ext
        )),
    }
}

/// 整合包安装失败/取消后清理版本目录
///
/// 保护逻辑：若版本目录下已存在 `saves/` 或 `versions/` 子目录，
/// 说明用户已独立启动过该版本，保留目录避免误删存档。
/// 否则删除整个版本目录，避免残留半安装文件污染版本列表。
///
/// 删除失败仅记录警告，不传播错误（清理失败不应阻塞错误传播）。
pub(super) fn cleanup_version_dir_on_failure(instance_dir: &std::path::Path) {
    let has_saves = instance_dir.join("saves").exists();
    let has_versions = instance_dir.join("versions").exists();
    if has_saves || has_versions {
        crate::log_warn!(
            "[Community] 版本目录已含 saves/versions 子目录，保留不清理: {}",
            instance_dir.display()
        );
        return;
    }
    // 失败后稍等 1 秒，让文件句柄释放（Windows 上文件可能仍被占用）
    std::thread::sleep(std::time::Duration::from_secs(1));
    match std::fs::remove_dir_all(instance_dir) {
        Ok(()) => {
            crate::log_warn!(
                "[Community] 安装失败/取消，已清理版本目录: {}",
                instance_dir.display()
            );
        }
        Err(e) => {
            crate::log_warn!(
                "[Community] 清理版本目录失败: {} ({})",
                instance_dir.display(),
                e
            );
        }
    }
}

#[cfg(test)]
#[path = "helpers_test.rs"]
mod helpers_test;
