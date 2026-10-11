//! ZIP 解压共享工具：顶层目录探测 + 防 Zip Slip 的安全解压
//!
//! 此前 `determine_zip_prefix` 有三份拷贝、安全解压有五份拷贝，改一处漏两处。
//! 插件安装 / frp 厂商包 / easytier 内核 / 资源包归档等所有解压点统一走这里。

use std::io::{Read, Seek};
use std::path::Path;

/// 探测 ZIP 顶层前缀
///
/// - 存在根级文件（条目名不含 `/`）→ 扁平结构，返回 `""`
/// - 所有条目都在同一根目录下 → 返回 `"xxx/"`
/// - 多根目录 / 空条目 → 返回 `""`
pub fn determine_zip_prefix(names: &[String]) -> String {
    let mut roots = std::collections::HashSet::new();
    let mut flat = false;
    for name in names {
        if name.contains('/') {
            if let Some(root) = name.split('/').next().filter(|v| !v.is_empty()) {
                roots.insert(root);
            }
        } else if !name.is_empty() {
            flat = true;
        }
    }
    if flat || roots.len() != 1 {
        return String::new();
    }
    match roots.into_iter().next() {
        Some(root) => format!("{}/", root),
        None => String::new(),
    }
}

/// 安全解压 ZIP 条目（防 Zip Slip）
///
/// 两道防线：
/// 1. **段级校验** `ensure_safe_relative_path` —— 目录条目与文件条目都要过，且必须
///    在创建任何目录**之前**执行（目录分支一旦先 `create_dir_all` 再 `continue`，
///    后续的文件分支校验就再也走不到了）
/// 2. canonicalize 父目录后校验目标在 `dst` 内（兜底，防符号链接等绕过段级校验）
///
/// - `prefix`：只解压以此前缀开头的条目并剥掉该前缀（`""` 表示全部）
/// - `should_keep`：剥前缀后的相对路径过滤器，返回 `false` 的条目跳过
pub fn extract_zip_safely<R, F>(
    archive: &mut zip::ZipArchive<R>,
    prefix: &str,
    dst: &Path,
    mut should_keep: F,
) -> Result<(), String>
where
    R: Read + Seek,
    F: FnMut(&str) -> bool,
{
    let canonical_dst = dst.canonicalize().unwrap_or_else(|_| dst.to_path_buf());
    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| format!("读取 ZIP 条目失败: {}", e))?;
        // 跳过 symlink 条目：本实现只创建普通文件，symlink 条目会把链接目标当文件
        // 内容写出来（纯垃圾文件）；解压出真实链接反而有指向 dst 之外的风险
        if file.is_symlink() {
            continue;
        }
        let name = file.name().to_string();
        if !name.starts_with(prefix) {
            continue;
        }
        let rel = &name[prefix.len()..];
        if rel.is_empty() || !should_keep(rel) {
            continue;
        }
        crate::utils::path::ensure_safe_relative_path(rel)
            .map_err(|e| format!("Zip Slip 检测: {} ({})", rel, e))?;
        let path = dst.join(rel);
        // 用 is_dir() 而非仅看结尾 '/'：目录条目可以不以 '/' 结尾
        if file.is_dir() || rel.ends_with('/') {
            std::fs::create_dir_all(&path).map_err(|e| format!("创建目录失败: {}", e))?;
            continue;
        }
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("创建父目录失败: {}", e))?;
            let canonical_parent = parent
                .canonicalize()
                .map_err(|e| format!("canonicalize 失败: {}", e))?;
            if !canonical_parent.starts_with(&canonical_dst) {
                return Err(format!("Zip Slip 检测: {}", rel));
            }
        }
        let mut out = std::fs::File::create(&path).map_err(|e| format!("创建文件失败: {}", e))?;
        std::io::copy(&mut file, &mut out).map_err(|e| format!("写入文件失败: {}", e))?;
    }
    Ok(())
}

#[cfg(test)]
#[path = "archive_test.rs"]
mod archive_test;
