//! frpc 压缩包提取：从下载的 ZIP / tar.gz 中定位并提取 frpc 二进制；通用归档解压（Zip Slip 防护）。
//! 跨平台自探测：翻遍归档所有层级目录，匹配 basename 为 `frpc`/`frpc.exe` 的非目录条目。

use std::path::Path;

use crate::log_info;

/// frpc 二进制文件名（含扩展名）
fn frpc_filename() -> String {
    #[cfg(target_os = "windows")]
    {
        "frpc.exe".to_string()
    }
    #[cfg(not(target_os = "windows"))]
    {
        "frpc".to_string()
    }
}

/// 从 ZIP 字节流提取 frpc 二进制到目标路径
///
/// 跨平台自探测：翻遍 ZIP 所有层级目录，匹配 basename 为 `frpc`/`frpc.exe` 的非目录条目，
/// 按优先级选择：1.当前平台首选名优先（Windows=frpc.exe，macOS/Linux=frpc）；2.路径短优先
/// （顶层 > 子目录），避免命中 `*/utils/frpc.exe` 等辅助文件。basename 必须精确等于 `frpc`
/// 或 `frpc.exe`，其他文件（LICENSE/frpc.toml 等）一律跳过。兼容 GitHub Releases、apiServer
/// 分发、扁平打包、任意嵌套层级等格式。
pub(super) fn extract_frpc_from_zip(zip_bytes: &[u8], target_path: &Path) -> Result<(), String> {
    let cursor = std::io::Cursor::new(zip_bytes);
    let mut archive =
        zip::ZipArchive::new(cursor).map_err(|e| format!("解析 frpc ZIP 失败: {}", e))?;

    let preferred = frpc_filename(); // 当前平台期望名

    // 收集所有匹配条目：(索引, 路径, 是否为当前平台首选名)
    let mut candidates: Vec<(usize, String, bool)> = Vec::new();
    for i in 0..archive.len() {
        let file = archive
            .by_index(i)
            .map_err(|e| format!("读取 ZIP 条目失败: {}", e))?;
        let name = file.name().to_string();
        // 跳过目录条目
        if name.ends_with('/') {
            continue;
        }
        // 取路径最后一段作为文件名
        let basename = name.rsplit('/').next().unwrap_or(&name);
        // 精确匹配 frpc 或 frpc.exe（排除 frpc.toml / frpc.ini / frpc_full.ini 等）
        if basename == "frpc" || basename == "frpc.exe" {
            let is_preferred = basename == preferred;
            candidates.push((i, name, is_preferred));
        }
    }

    if candidates.is_empty() {
        return Err("ZIP 中未找到 frpc 二进制（期望文件名 frpc 或 frpc.exe）".to_string());
    }

    // 排序：首选名优先（is_preferred=true 排前），其次路径短优先（浅层目录）
    candidates.sort_by(|a, b| match b.2.cmp(&a.2) {
        std::cmp::Ordering::Equal => a.1.len().cmp(&b.1.len()),
        other => other,
    });
    let (best_idx, best_name, _) = &candidates[0];

    log_info!(
        "[Frp] 从 ZIP 提取: {}（共 {} 个候选，当前平台期望: {}）",
        best_name,
        candidates.len(),
        preferred
    );

    let mut file = archive
        .by_index(*best_idx)
        .map_err(|e| format!("读取 ZIP 条目失败: {}", e))?;

    if let Some(parent) = target_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("创建 frpc 目录失败: {}", e))?;
    }
    let mut out =
        std::fs::File::create(target_path).map_err(|e| format!("创建 frpc 文件失败: {}", e))?;
    std::io::copy(&mut file, &mut out).map_err(|e| format!("写入 frpc 文件失败: {}", e))?;

    Ok(())
}

/// 从 tar.gz 字节流提取 frpc 二进制到目标路径（frp 官方 macOS/Linux 分发包格式）
///
/// 匹配策略与 [`extract_frpc_from_zip`] 一致：翻遍所有条目，basename 精确等于
/// `frpc`/`frpc.exe` 的非目录条目，路径短优先（顶层 > 子目录）。
/// tar.gz 中 frpc 无 `.exe` 后缀（macOS/Linux 可执行文件）。
pub(super) fn extract_frpc_from_tar_gz(
    tar_gz_bytes: &[u8],
    target_path: &Path,
) -> Result<(), String> {
    let gz = flate2::read::GzDecoder::new(tar_gz_bytes);
    let mut archive = tar::Archive::new(gz);

    // 收集所有匹配条目：(路径, 内容)。frpc 二进制单条体积有限（10~20MB），
    // 直接读入内存简化遍历（tar 无按索引读取 API，需两次遍历）
    let mut candidates: Vec<(String, Vec<u8>)> = Vec::new();
    let entries = archive
        .entries()
        .map_err(|e| format!("解析 frpc tar.gz 失败: {}", e))?;
    for entry in entries {
        let mut entry = entry.map_err(|e| format!("读取 tar.gz 条目失败: {}", e))?;
        let path = entry
            .path()
            .map_err(|e| format!("读取条目路径失败: {}", e))?
            .to_string_lossy()
            .to_string();
        if path.ends_with('/') {
            continue;
        }
        let basename = path.rsplit('/').next().unwrap_or(&path);
        if basename == "frpc" || basename == "frpc.exe" {
            let mut buf = Vec::new();
            std::io::Read::read_to_end(&mut entry, &mut buf)
                .map_err(|e| format!("读取 tar.gz 条目内容失败: {}", e))?;
            candidates.push((path, buf));
        }
    }

    if candidates.is_empty() {
        return Err("tar.gz 中未找到 frpc 二进制（期望文件名 frpc 或 frpc.exe）".to_string());
    }
    // 路径短优先（浅层目录）
    candidates.sort_by(|a, b| a.0.len().cmp(&b.0.len()));
    let (best_name, bytes) = candidates.remove(0);
    log_info!(
        "[Frp] 从 tar.gz 提取: {}（共 {} 个候选）",
        best_name,
        candidates.len() + 1
    );

    if let Some(parent) = target_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("创建 frpc 目录失败: {}", e))?;
    }
    std::fs::write(target_path, &bytes).map_err(|e| format!("写入 frpc 文件失败: {}", e))?;

    Ok(())
}

/// 解压归档文件到目标目录（Zip Slip 防护）
pub(super) fn extract_archive(archive_path: &Path, dst: &Path) -> Result<(), String> {
    let file = std::fs::File::open(archive_path).map_err(|e| format!("打开归档失败: {}", e))?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("解析归档失败: {}", e))?;
    let canonical_dst = dst.canonicalize().unwrap_or_else(|_| dst.to_path_buf());
    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| format!("读取 ZIP 条目失败: {}", e))?;
        let name = entry.name().to_string();
        if name.ends_with('/') {
            std::fs::create_dir_all(dst.join(&name)).map_err(|e| format!("创建目录失败: {}", e))?;
            continue;
        }
        let file_path = dst.join(&name);
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("创建父目录失败: {}", e))?;
            let canonical_parent = parent
                .canonicalize()
                .map_err(|e| format!("canonicalize 失败: {}", e))?;
            if !canonical_parent.starts_with(&canonical_dst) {
                return Err(format!("Zip Slip 检测: {}", name));
            }
        }
        let mut out =
            std::fs::File::create(&file_path).map_err(|e| format!("创建文件失败: {}", e))?;
        std::io::copy(&mut entry, &mut out).map_err(|e| format!("写入文件失败: {}", e))?;
    }
    Ok(())
}
