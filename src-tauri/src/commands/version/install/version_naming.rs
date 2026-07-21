//! 版本目录命名与查找辅助函数
//!
//! - `resolve_unique_instance_name` 处理版本名冲突（追加 (1)(2) 后缀）
//!   整合包半成品目录（只有 mods/overrides，没有 MC 本体）直接复用而非追加后缀
//! - `find_loader_version_dir` 按加载器类型前缀匹配版本目录（成功路径用，与失败清理的精确匹配不同）

use crate::log_info;
use std::path::Path;

/// 解析唯一的实例名：如果版本目录已存在且包含完整 MC 本体，追加 (1)(2) 后缀；
/// 整合包半成品目录（缺 .json/.jar）直接复用。
pub(crate) fn resolve_unique_instance_name(game_dir: &Path, base_name: &str) -> String {
    let versions_dir = game_dir.join("versions");
    let target_dir = versions_dir.join(base_name);
    if !target_dir.exists() {
        // 目录不存在：直接使用
        log_info!(
            "[Merged] 版本目录不存在，直接使用: {}",
            target_dir.display()
        );
        return base_name.to_string();
    }

    // 目录已存在：检查是否为「完整版本」（有 <base_name>.json 或 <base_name>.jar）
    // 整合包安装后会创建同名目录，但里面只有 mods/overrides/configs，
    // 没有版本 JSON/JAR，这种情况下应直接复用目录，不要追加后缀
    let has_version_json = target_dir.join(format!("{}.json", base_name)).exists();
    let has_version_jar = target_dir.join(format!("{}.jar", base_name)).exists();
    let has_mods = target_dir.join("mods").exists();
    log_info!(
        "[Merged] 版本目录已存在: {} (json={}, jar={}, mods={})",
        target_dir.display(),
        has_version_json,
        has_version_jar,
        has_mods
    );
    if !has_version_json && !has_version_jar {
        // 整合包半成品目录（只有 mods/overrides，没有 MC 本体）→ 直接复用
        log_info!(
            "[Merged] 检测到整合包半成品目录，直接复用: {}",
            target_dir.display()
        );
        return base_name.to_string();
    }

    // 完整版本已存在：追加后缀 (1), (2) 等
    log_info!("[Merged] 版本已存在，追加后缀: {}", base_name);
    let mut counter = 1;
    loop {
        let candidate = format!("{}({})", base_name, counter);
        if !versions_dir.join(&candidate).exists() {
            return candidate;
        }
        counter += 1;
    }
}

/// 按加载器类型前缀匹配版本目录（成功路径用）。
///
/// 匹配规则（与原 install_merged 成功路径内联逻辑一致）：
/// - Forge:      `{mc}-forge-`
/// - NeoForge:   `{mc}-neoforge-`
/// - Fabric:     `fabric-` 前缀 + `-{mc}` 后缀
/// - OptiFine:   `{mc}-OptiFine`
/// - LiteLoader: `{mc}-LiteLoader`
///
/// 返回第一个匹配的目录名。失败路径的精确 fabric 匹配见 `cleanup::cleanup_failed_install`。
pub(crate) fn find_loader_version_dir(versions_dir: &Path, mc_version: &str) -> Option<String> {
    let entries = std::fs::read_dir(versions_dir).ok()?;
    for entry in entries.flatten() {
        let dir_name = entry.file_name().to_string_lossy().to_string();
        if dir_name.starts_with(&format!("{}-forge-", mc_version))
            || dir_name.starts_with(&format!("{}-neoforge-", mc_version))
            || (dir_name.starts_with("fabric-") && dir_name.ends_with(&format!("-{}", mc_version)))
            || dir_name.starts_with(&format!("{}-OptiFine", mc_version))
            || dir_name.starts_with(&format!("{}-LiteLoader", mc_version))
        {
            return Some(dir_name);
        }
    }
    None
}
