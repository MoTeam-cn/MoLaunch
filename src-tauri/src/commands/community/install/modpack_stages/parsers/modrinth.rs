//! Modrinth modrinth.index.json 解析

use super::super::super::concurrent::DetectedModpack;
use super::super::super::helpers::parse_mr_loader;
use super::super::super::modrinth::MrIndex;
use super::super::super::types::{ModpackFormat, ModpackInfo};

/// 解析 Modrinth modrinth.index.json
pub(crate) fn parse_mr(detected: &DetectedModpack) -> Result<ModpackInfo, String> {
    let index: MrIndex = serde_json::from_str(detected.index_content.as_deref().unwrap_or(""))
        .map_err(|e| format!("解析 modrinth.index.json 失败: {}", e))?;
    let gv = index
        .dependencies
        .get("minecraft")
        .cloned()
        .unwrap_or_default();
    // Quilt 加载器特判：本项目暂不支持 Quilt 加载器（功能性决策，非格式限制），
    // 整合包要求 Quilt 时直接拒绝安装
    if index.dependencies.contains_key("quilt-loader") {
        return Err("Modrinth 整合包要求 Quilt 加载器，MoLaunch 暂不支持 Quilt".to_string());
    }
    let (loader, ver) = ["fabric-loader", "forge", "neoforge"]
        .iter()
        .find_map(|key| {
            index.dependencies.get(*key).map(|v| {
                let (ln, vv) = parse_mr_loader(key, v);
                (ln.to_string(), vv)
            })
        })
        .unwrap_or((String::new(), String::new()));
    let count = index.files.len();
    Ok(ModpackInfo {
        format: ModpackFormat::Modrinth,
        game_version: gv,
        loader,
        loader_version: ver,
        mod_files_count: count,
        archive_base_folder: detected.archive_base_folder.clone(),
        cf_overrides_name: None,
        cf_manifest: None,
        mr_index: Some(index),
        hmcl_manifest: None,
        mmc_pack: None,
        mmc_cfg_content: None,
        mcbbs_manifest: None,
        launcher_inner_path: None,
    })
}
