//! CurseForge manifest.json 解析

use super::super::super::concurrent::DetectedModpack;
use super::super::super::curseforge::CfManifest;
use super::super::super::helpers::parse_cf_loader_id;
use super::super::super::types::{ModpackFormat, ModpackInfo};

/// 解析 CurseForge manifest.json
pub(crate) fn parse_cf(detected: &DetectedModpack) -> Result<ModpackInfo, String> {
    let manifest: CfManifest =
        serde_json::from_str(detected.manifest_content.as_deref().unwrap_or(""))
            .map_err(|e| format!("解析 manifest.json 失败: {}", e))?;
    let gv = manifest.minecraft.version.clone();
    // Quilt 加载器检测：id 以 "quilt-" 开头直接报错
    for l in &manifest.minecraft.mod_loaders {
        if l.id.starts_with("quilt-") || l.id.starts_with("quilt_") {
            return Err("CurseForge 整合包要求 Quilt 加载器，MoLaunch 暂不支持 Quilt".to_string());
        }
    }
    // Forge recommended 字段检测：旧版整合包格式，直接报错提示版本过老
    for l in &manifest.minecraft.mod_loaders {
        if l.id.starts_with("forge-") && l.id.contains("recommended") {
            return Err(
                "该整合包版本过老（使用旧版 Forge recommended 格式），请尝试更新版本的整合包"
                    .to_string(),
            );
        }
    }
    let (loader, ver) = manifest
        .minecraft
        .mod_loaders
        .iter()
        .find(|l| l.primary)
        .or_else(|| manifest.minecraft.mod_loaders.first())
        .map(|l| parse_cf_loader_id(&l.id))
        .unwrap_or((String::new(), String::new()));
    let count = manifest.files.len();
    let cf_overrides_name = manifest.overrides.clone();
    Ok(ModpackInfo {
        format: ModpackFormat::Curseforge,
        game_version: gv,
        loader,
        loader_version: ver,
        mod_files_count: count,
        archive_base_folder: detected.archive_base_folder.clone(),
        cf_overrides_name,
        cf_manifest: Some(manifest),
        mr_index: None,
        hmcl_manifest: None,
        mmc_pack: None,
        mmc_cfg_content: None,
        mcbbs_manifest: None,
        launcher_inner_path: None,
    })
}