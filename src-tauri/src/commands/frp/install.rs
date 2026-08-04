//! 厂商安装/卸载：外部厂商安装（文件夹/ZIP + Zip Slip 防护）、卸载（路径遍历防护）。
//!
//! frpc 二进制下载见 [`super::binary`]，厂商列表/状态/启禁见 [`super::provider`]。

use super::provider::{
    frpc_platform_skip, is_external_frpc_ready, read_icon_as_data_url, read_providers_state,
    resolve_auth_type, write_provider_frpc_version, write_providers_state, SYSTEM_DEFAULT_ID,
};
use super::{ensure_dir, providers_root, validate_provider_id, ProviderInfo, ProviderManifest};
use crate::log_info;
use std::path::{Path, PathBuf};

// 安装 / 卸载
/// 读取目标目录已安装的 manifest 版本（目标不存在时返回 None）
fn read_installed_version(target_dir: &Path) -> Option<String> {
    let path = target_dir.join("manifest.json");
    let content = std::fs::read_to_string(path).ok()?;
    let manifest: serde_json::Value = serde_json::from_str(&content).ok()?;
    manifest
        .get("version")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

/// 判断版本是否变化（新旧版本字符串不一致视为变化）
///
/// 简单字符串比较：厂商版本一般为语义化字符串，版本号变（增/减/后缀变化）即视为更新。
fn version_changed(new_version: &str, old_version: Option<&str>) -> bool {
    match old_version {
        Some(old) => old != new_version,
        None => true,
    }
}

/// 从文件夹安装/更新外部厂商
///
/// 源目录必须包含 manifest.json。安装后校验 manifest.json 存在。
/// 若目标已存在：
/// - 版本号相同 → 返回"已是最新版本"，不执行合并
/// - 版本号变化 → 执行增量覆盖：逐文件内容比对，仅替换发生变更的文件，
///   未变化的文件与厂商用户数据（frpc 二进制、认证等）保持不变。
pub async fn install_provider_from_dir(source_dir: String) -> Result<ProviderInfo, String> {
    let src = Path::new(&source_dir);
    if !src.is_dir() {
        return Err(format!("源目录不存在: {}", source_dir));
    }
    let manifest_path = src.join("manifest.json");
    let content = std::fs::read_to_string(&manifest_path)
        .map_err(|e| format!("读取 manifest.json 失败: {}", e))?;
    let manifest: ProviderManifest =
        serde_json::from_str(&content).map_err(|e| format!("解析 manifest.json 失败: {}", e))?;

    validate_provider_id(&manifest.id)?;
    let target_dir = providers_root().join(&manifest.id);

    // 平台过滤：跳过其他平台的 frpc，只复制当前平台对应的二进制
    let (skip, _) = frpc_platform_skip(&manifest.binary);

    let is_install;
    let added;
    if target_dir.exists() {
        let old_version = read_installed_version(&target_dir);
        if !version_changed(&manifest.version, old_version.as_deref()) {
            log_info!(
                "[Frp] 厂商已是最新版本，跳过更新: {} (版本 {})",
                manifest.id,
                manifest.version
            );
            return Ok(build_provider_info(&manifest));
        }
        log_info!(
            "[Frp] 厂商版本变化，执行增量更新: {} ({} -> {})",
            manifest.id,
            old_version.unwrap_or_default(),
            manifest.version
        );
        ensure_dir(&target_dir)?;
        is_install = false;
        added = merge_dir_incremental(src, &target_dir, &skip, src)?.1;
    } else {
        ensure_dir(&providers_root())?;
        is_install = true;
        added = copy_dir_recursive(src, &target_dir, &skip, src)?;
    }

    let installed_manifest_path = target_dir.join("manifest.json");
    if !installed_manifest_path.exists() {
        if added == 0 {
            let _ = std::fs::remove_dir_all(&target_dir);
        }
        return Err("安装校验失败：manifest.json 不存在".to_string());
    }

    if !is_install {
        log_info!(
            "[Frp] 厂商已更新: {} ({}), 变更 {} 个文件",
            manifest.name,
            manifest.id,
            added
        );
    } else {
        log_info!(
            "[Frp] 厂商已安装: {} ({}), {} 个文件",
            manifest.name,
            manifest.id,
            added
        );
    }

    // 记录 manifest 声明的 frpc 版本（无论 bundled/url）。
    // 作为后续 ensure_frpc 判断是否更新的依据：版本没变则不重复下载/替换。
    if let Some(fv) = manifest.binary.frpc_version.as_deref() {
        write_provider_frpc_version(&manifest.id, fv);
    }

    Ok(build_provider_info(&manifest))
}

/// 从 ZIP 安装/更新外部厂商
///
/// 支持扁平结构（根直接含 manifest.json）和单根目录结构。
/// 解压带 Zip Slip 防护（canonicalize 父目录后校验目标在 dst 内）。
/// 若目标已存在则执行增量覆盖：先解压到临时目录，再逐文件内容比对合并，
/// 仅替换变更文件；未变化的文件与厂商用户数据保持不变。
pub async fn install_provider_from_zip(zip_path: String) -> Result<ProviderInfo, String> {
    // 解压与 manifest 解析放进独立作用域：ZipArchive/ZipFile 不是 Send，
    // 必须在后续 await（url 模式重刷 frpc）前结束生命周期。
    let (manifest, temp_dir) = {
        let zip_file = PathBuf::from(&zip_path);
        if !zip_file.exists() {
            return Err(format!("ZIP 文件不存在: {}", zip_path));
        }

        let file = std::fs::File::open(&zip_file).map_err(|e| format!("打开 ZIP 失败: {}", e))?;
        let mut archive =
            zip::ZipArchive::new(file).map_err(|e| format!("解析 ZIP 失败: {}", e))?;

        let names: Vec<String> = archive.file_names().map(|s| s.to_string()).collect();
        let prefix = determine_zip_prefix(&names)?;

        let manifest_entry = if prefix.is_empty() {
            "manifest.json".to_string()
        } else {
            format!("{}manifest.json", prefix)
        };
        let manifest_idx = names
            .iter()
            .position(|n| *n == manifest_entry)
            .ok_or_else(|| "ZIP 中未找到 manifest.json".to_string())?;
        let mut manifest_file = archive
            .by_index(manifest_idx)
            .map_err(|e| format!("读取 ZIP 内 manifest 失败: {}", e))?;
        let mut manifest_str = String::new();
        std::io::Read::read_to_string(&mut manifest_file, &mut manifest_str)
            .map_err(|e| format!("读取 manifest 内容失败: {}", e))?;
        // 立即释放 manifest_file（ZipFile 持有 archive 的可变借用），
        // 否则 extract_zip_safely(&mut archive) 无法再次借用
        drop(manifest_file);

        let manifest: ProviderManifest = serde_json::from_str(&manifest_str)
            .map_err(|e| format!("解析 manifest.json 失败: {}", e))?;
        validate_provider_id(&manifest.id)?;

        let temp_dir = std::env::temp_dir().join(format!(
            "molaunch-provider-extract-{}-{}",
            manifest.id,
            std::process::id()
        ));
        if temp_dir.exists() {
            let _ = std::fs::remove_dir_all(&temp_dir);
        }

        ensure_dir(&temp_dir)?;
        if let Err(e) = extract_zip_safely(&mut archive, &prefix, &temp_dir) {
            let _ = std::fs::remove_dir_all(&temp_dir);
            return Err(format!("解压失败: {}", e));
        }

        (manifest, temp_dir)
    };

    let target_dir = providers_root().join(&manifest.id);
    // 平台过滤：跳过其他平台的 frpc，只复制当前平台对应的二进制
    let (skip, _) = frpc_platform_skip(&manifest.binary);
    let is_install;
    let added;
    if target_dir.exists() {
        let old_version = read_installed_version(&target_dir);
        if !version_changed(&manifest.version, old_version.as_deref()) {
            let _ = std::fs::remove_dir_all(&temp_dir);
            log_info!(
                "[Frp] 厂商已是最新版本，跳过更新: {} (版本 {})",
                manifest.id,
                manifest.version
            );
            return Ok(build_provider_info(&manifest));
        }
        log_info!(
            "[Frp] 厂商版本变化，执行增量更新: {} ({} -> {})",
            manifest.id,
            old_version.unwrap_or_default(),
            manifest.version
        );
        ensure_dir(&target_dir)?;
        is_install = false;
        added = merge_dir_incremental(&temp_dir, &target_dir, &skip, &temp_dir)?.1;
    } else {
        ensure_dir(&providers_root())?;
        is_install = true;
        added = copy_dir_recursive(&temp_dir, &target_dir, &skip, &temp_dir)?;
    }
    let _ = std::fs::remove_dir_all(&temp_dir);

    let installed_manifest_path = target_dir.join("manifest.json");
    if !installed_manifest_path.exists() {
        if added == 0 {
            let _ = std::fs::remove_dir_all(&target_dir);
        }
        return Err("安装校验失败：manifest.json 不存在".to_string());
    }

    if !is_install {
        log_info!(
            "[Frp] 厂商已从 ZIP 更新: {} ({}), 变更 {} 个文件",
            manifest.name,
            manifest.id,
            added
        );
    } else {
        log_info!(
            "[Frp] 厂商已从 ZIP 安装: {} ({}), {} 个文件",
            manifest.name,
            manifest.id,
            added
        );
    }

    // 记录 manifest 声明的 frpc 版本（无论 bundled/url）。
    // 作为后续 ensure_frpc 判断是否更新的依据：版本没变则不重复下载/替换。
    if let Some(fv) = manifest.binary.frpc_version.as_deref() {
        write_provider_frpc_version(&manifest.id, fv);
    }

    Ok(build_provider_info(&manifest))
}

/// 从 URL 下载并安装外部厂商
///
/// 下载 ZIP 到临时文件，复用 `install_provider_from_zip` 安装逻辑。
/// 仅允许 HTTPS URL（用户主动提供，无域名白名单限制）。
/// 无论安装成功或失败，临时文件都会被清理。
pub async fn install_provider_from_url(url: String) -> Result<ProviderInfo, String> {
    if !url.starts_with("https://") {
        return Err("URL 必须使用 HTTPS".to_string());
    }

    log_info!("[Frp] 开始从 URL 下载厂商包: {}", url);

    // 复用 crate::http 统一管线（代理 / IP 版本 / TLS 信任源 / User-Agent 与全局一致），
    // 允许有限重定向（limited(5)）。
    let client = crate::http::build_client_with_redirect(
        reqwest::redirect::Policy::limited(5),
        Some(60_000),
    );

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("下载失败: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("下载失败: HTTP {}", response.status()));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("读取下载内容失败: {}", e))?;

    let temp_zip =
        std::env::temp_dir().join(format!("molaunch-provider-{}.zip", std::process::id()));

    std::fs::write(&temp_zip, &bytes).map_err(|e| format!("写入临时文件失败: {}", e))?;

    log_info!("[Frp] 厂商包下载完成，大小: {} 字节", bytes.len());

    // 复用 ZIP 安装逻辑（无论成功失败都清理临时文件）
    let result = install_provider_from_zip(temp_zip.to_string_lossy().to_string()).await;
    let _ = std::fs::remove_file(&temp_zip);
    result
}

/// 卸载外部厂商
///
/// 不允许卸载系统默认厂商。删除目录前用 canonicalize 校验路径不逃逸 providers/ 根。
pub async fn uninstall_provider(provider_id: String) -> Result<(), String> {
    if provider_id == SYSTEM_DEFAULT_ID {
        return Err("不能卸载系统默认厂商".to_string());
    }
    validate_provider_id(&provider_id)?;
    let dir = providers_root().join(&provider_id);
    if !dir.exists() {
        return Err(format!("厂商不存在: {}", provider_id));
    }
    let canonical_root = providers_root()
        .canonicalize()
        .map_err(|e| format!("路径校验失败: {}", e))?;
    let canonical_dir = dir
        .canonicalize()
        .map_err(|e| format!("路径校验失败: {}", e))?;
    if !canonical_dir.starts_with(&canonical_root) {
        return Err("路径遍历检测".to_string());
    }
    std::fs::remove_dir_all(&dir).map_err(|e| format!("卸载失败: {}", e))?;

    let mut state = read_providers_state();
    state.remove(&provider_id);
    write_providers_state(&state)?;

    log_info!("[Frp] 厂商已卸载: {}", provider_id);
    Ok(())
}

// 内部辅助
/// 从 manifest + 启用状态构建 ProviderInfo
fn build_provider_info(manifest: &ProviderManifest) -> ProviderInfo {
    let state = read_providers_state();
    let frpc_ready = is_external_frpc_ready(&manifest.id, manifest);
    let enabled = state.get(&manifest.id).copied().unwrap_or(true);
    let auth_type = resolve_auth_type(&manifest.id, manifest);
    let icon = manifest
        .icon
        .as_ref()
        .and_then(|icon_rel| read_icon_as_data_url(&manifest.id, icon_rel));
    ProviderInfo {
        id: manifest.id.clone(),
        name: manifest.name.clone(),
        description: manifest.description.clone(),
        version: manifest.version.clone(),
        author: manifest.author.clone(),
        builtin: false,
        auth_type,
        frpc_ready,
        enabled,
        distribution: manifest.binary.distribution.clone(),
        homepage: manifest.homepage.clone(),
        icon,
    }
}

/// 递归复制源目录到目标目录，返回复制的文件数
///
/// `skip` 为应跳过的相对路径集合（其他平台 frpc），命中则跳过不复制。
fn copy_dir_recursive(
    src: &Path,
    dst: &Path,
    skip: &std::collections::HashSet<String>,
    src_base: &Path,
) -> Result<u32, String> {
    if !dst.exists() {
        std::fs::create_dir_all(dst).map_err(|e| format!("创建目录失败: {}", e))?;
    }
    let mut count = 0u32;
    for entry in std::fs::read_dir(src).map_err(|e| format!("读取目录失败: {}", e))? {
        let entry = entry.map_err(|e| format!("读取目录项失败: {}", e))?;
        let path = entry.path();
        let file_name = entry.file_name();
        let dst_path = dst.join(&file_name);
        if path.is_dir() {
            count += copy_dir_recursive(&path, &dst_path, skip, src_base)?;
        } else {
            // 计算相对路径，命中跳过集则跳过（如其他平台的 frpc）
            let rel = path
                .strip_prefix(src_base)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            if skip.contains(&rel) {
                continue;
            }
            if let Some(parent) = dst_path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| format!("创建父目录失败: {}", e))?;
            }
            std::fs::copy(&path, &dst_path).map_err(|e| format!("复制文件失败: {}", e))?;
            count += 1;
        }
    }
    Ok(count)
}

/// 增量合并源目录到目标目录：仅替换内容发生变更的文件
///
/// 返回 `(更新的文件数, 新增的文件数)`。逐文件字节比对：
///
/// - 目标中不存在 → 复制（新增）
/// - 内容不同 → 覆盖（更新）
/// - 内容相同 → 跳过（保持不变）
///
/// `skip` 为应跳过的相对路径集合（其他平台 frpc），命中则跳过。
/// 目标中源目录不存在的文件（认证数据等用户数据）保留不动。
fn merge_dir_incremental(
    src: &Path,
    dst: &Path,
    skip: &std::collections::HashSet<String>,
    src_base: &Path,
) -> Result<(u32, u32), String> {
    if !dst.exists() {
        std::fs::create_dir_all(dst).map_err(|e| format!("创建目录失败: {}", e))?;
    }
    let mut updated = 0u32;
    let mut added = 0u32;
    for entry in std::fs::read_dir(src).map_err(|e| format!("读取目录失败: {}", e))? {
        let entry = entry.map_err(|e| format!("读取目录项失败: {}", e))?;
        let path = entry.path();
        let file_name = entry.file_name();
        let dst_path = dst.join(&file_name);
        if path.is_dir() {
            let (u, a) = merge_dir_incremental(&path, &dst_path, skip, src_base)?;
            updated += u;
            added += a;
        } else {
            let rel = path
                .strip_prefix(src_base)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            if skip.contains(&rel) {
                continue;
            }
            if let Some(parent) = dst_path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| format!("创建父目录失败: {}", e))?;
            }
            if !dst_path.exists() {
                std::fs::copy(&path, &dst_path).map_err(|e| format!("复制文件失败: {}", e))?;
                added += 1;
            } else {
                let src_bytes = std::fs::read(&path).map_err(|e| format!("读取文件失败: {}", e))?;
                let dst_bytes =
                    std::fs::read(&dst_path).map_err(|e| format!("读取目标文件失败: {}", e))?;
                if src_bytes != dst_bytes {
                    std::fs::copy(&path, &dst_path).map_err(|e| format!("覆盖文件失败: {}", e))?;
                    updated += 1;
                }
            }
        }
    }
    Ok((updated, added))
}

/// 探测 ZIP 前缀（扁平结构返回 ""，单根目录返回 "xxx/"）
///
/// - 若 ZIP 中存在根级文件（无 `/` 分隔），视为扁平结构
/// - 若所有文件都在同一根目录下，返回 "xxx/"
/// - 多根目录或无文件，视为扁平结构
fn determine_zip_prefix(names: &[String]) -> Result<String, String> {
    let mut root_dirs = std::collections::HashSet::new();
    let mut has_flat_files = false;
    for name in names {
        if name.contains('/') {
            let root = name.split('/').next().unwrap_or("");
            if !root.is_empty() {
                root_dirs.insert(root.to_string());
            }
        } else if !name.is_empty() {
            has_flat_files = true;
        }
    }
    if has_flat_files {
        return Ok(String::new());
    }
    if root_dirs.len() == 1 {
        let root = root_dirs.iter().next().ok_or_else(|| {
            "ZIP 前缀探测失败：根目录集合为空（不应发生，len 已校验为 1）".to_string()
        })?;
        return Ok(format!("{}/", root));
    }
    Ok(String::new())
}

/// 安全解压 ZIP（防 Zip Slip：canonicalize 父目录后校验目标在 dst 内）
fn extract_zip_safely<R: std::io::Read + std::io::Seek>(
    archive: &mut zip::ZipArchive<R>,
    prefix: &str,
    dst: &Path,
) -> Result<(), String> {
    let canonical_dst = dst.canonicalize().unwrap_or_else(|_| dst.to_path_buf());
    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| format!("读取 ZIP 条目失败: {}", e))?;
        let name = file.name().to_string();
        if !name.starts_with(prefix) {
            continue;
        }
        let rel = &name[prefix.len()..];
        if rel.is_empty() {
            continue;
        }
        if rel.ends_with('/') {
            std::fs::create_dir_all(dst.join(rel)).map_err(|e| format!("创建目录失败: {}", e))?;
            continue;
        }
        let file_path = dst.join(rel);
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("创建父目录失败: {}", e))?;
            let canonical_parent = parent
                .canonicalize()
                .map_err(|e| format!("canonicalize 失败: {}", e))?;
            if !canonical_parent.starts_with(&canonical_dst) {
                return Err(format!("Zip Slip 检测: {}", rel));
            }
        }
        let mut out =
            std::fs::File::create(&file_path).map_err(|e| format!("创建文件失败: {}", e))?;
        std::io::copy(&mut file, &mut out).map_err(|e| format!("写入文件失败: {}", e))?;
    }
    Ok(())
}
