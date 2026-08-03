//! HMCL modpack.json 解析

use crate::log_info;

use super::super::super::concurrent::DetectedModpack;
use super::super::super::hmcl::HmclManifest;
use super::super::super::types::{ModpackFormat, ModpackInfo};

/// 解析 HMCL modpack.json
pub(crate) fn parse_hmcl(detected: &DetectedModpack) -> Result<ModpackInfo, String> {
    let manifest: HmclManifest =
        serde_json::from_str(detected.hmcl_content.as_deref().unwrap_or(""))
            .map_err(|e| format!("解析 modpack.json 失败: {}", e))?;
    let gv = manifest.game_version.clone();
    // HMCL 整合包不指定加载器版本，仅含游戏版本；加载器信息（如有）打包在 overrides 中
    log_info!(
        "[Community] HMCL 整合包: game={} name={}",
        gv,
        manifest.name
    );
    Ok(ModpackInfo {
        format: ModpackFormat::Hmcl,
        game_version: gv,
        loader: String::new(),
        loader_version: String::new(),
        mod_files_count: 0,
        archive_base_folder: detected.archive_base_folder.clone(),
        cf_overrides_name: None,
        cf_manifest: None,
        mr_index: None,
        hmcl_manifest: Some(manifest),
        mmc_pack: None,
        mmc_cfg_content: None,
        mcbbs_manifest: None,
        launcher_inner_path: None,
    })
}
