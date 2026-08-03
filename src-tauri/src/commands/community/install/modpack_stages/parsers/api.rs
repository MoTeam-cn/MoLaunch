//! 未拆分子文件的格式解析：MMC / MCBBS / LauncherPack / Compress
//! CF/MR/HMCL 见 curseforge.rs / modrinth.rs / hmcl.rs。

use crate::log_info;

use super::super::super::concurrent::DetectedModpack;
use super::super::super::mcbbs::McbbsManifest;
use super::super::super::mmc::MmcPack;
use super::super::super::types::{ModpackFormat, ModpackInfo};

/// 解析 MMC mmc-pack.json
pub fn parse_mmc(detected: &DetectedModpack) -> Result<ModpackInfo, String> {
    let pack: MmcPack = serde_json::from_str(detected.mmc_content.as_deref().unwrap_or(""))
        .map_err(|e| format!("解析 mmc-pack.json 失败: {}", e))?;
    // 从 components 提取 game_version 和 loader
    let mut gv = String::new();
    let mut loader = String::new();
    let mut loader_ver = String::new();
    for comp in &pack.components {
        match comp.uid.as_str() {
            "net.minecraft" => gv = comp.version.clone(),
            "net.minecraftforge" => {
                loader = "forge".to_string();
                loader_ver = comp.version.clone();
            }
            "net.neoforged" => {
                loader = "neoforge".to_string();
                loader_ver = comp.version.clone();
            }
            "net.fabricmc.fabric-loader" => {
                loader = "fabric".to_string();
                loader_ver = comp.version.clone();
            }
            _ => {
                // 跳过 org.lwjgl.* 等
                if !comp.uid.starts_with("org.lwjgl") {
                    log_info!(
                        "[Community] MMC 整合包跳过不支持的组件: uid={} version={}",
                        comp.uid,
                        comp.version
                    );
                }
            }
        }
    }
    if gv.is_empty() {
        return Err("MMC 整合包未提供 game 版本（缺少 net.minecraft 组件）".to_string());
    }
    log_info!(
        "[Community] MMC 整合包: game={} loader={}{}",
        gv,
        loader,
        if loader_ver.is_empty() {
            String::new()
        } else {
            format!("@{}", loader_ver)
        }
    );
    Ok(ModpackInfo {
        format: ModpackFormat::Mmc,
        game_version: gv,
        loader,
        loader_version: loader_ver,
        mod_files_count: 0,
        archive_base_folder: detected.archive_base_folder.clone(),
        cf_overrides_name: None,
        cf_manifest: None,
        mr_index: None,
        hmcl_manifest: None,
        mmc_pack: Some(pack),
        mmc_cfg_content: detected.mmc_cfg_content.clone(),
        mcbbs_manifest: None,
        launcher_inner_path: None,
    })
}

/// 解析 MCBBS mcbbs.packmeta/manifest.json
pub fn parse_mcbbs(detected: &DetectedModpack) -> Result<ModpackInfo, String> {
    let manifest: McbbsManifest =
        serde_json::from_str(detected.manifest_content.as_deref().unwrap_or(""))
            .map_err(|e| format!("解析 mcbbs.packmeta/manifest.json 失败: {}", e))?;
    // 从 addons 提取 game_version 和 loader
    let mut gv = String::new();
    let mut loader = String::new();
    let mut loader_ver = String::new();
    for addon in &manifest.addons {
        match addon.id.as_str() {
            "game" => gv = addon.version.clone(),
            "forge" => {
                loader = "forge".to_string();
                loader_ver = addon.version.clone();
            }
            "neoforge" => {
                loader = "neoforge".to_string();
                loader_ver = addon.version.clone();
            }
            "fabric" => {
                loader = "fabric".to_string();
                loader_ver = addon.version.clone();
            }
            "optifine" => {
                // OptiFine 作为独立加载器
                loader = "optifine".to_string();
                loader_ver = addon.version.clone();
            }
            "quilt" => {
                // MoLaunch 暂不支持 Quilt
                return Err("MCBBS 整合包要求 Quilt 加载器，MoLaunch 暂不支持 Quilt".to_string());
            }
            _ => {
                log_info!(
                    "[Community] MCBBS 整合包跳过未知 addon: id={} version={}",
                    addon.id,
                    addon.version
                );
            }
        }
    }
    if gv.is_empty() {
        return Err("MCBBS 整合包未提供 game 版本（addons 中缺少 id=game 项）".to_string());
    }
    log_info!(
        "[Community] MCBBS 整合包: game={} loader={}{} name={}",
        gv,
        loader,
        if loader_ver.is_empty() {
            String::new()
        } else {
            format!("@{}", loader_ver)
        },
        manifest.name
    );
    Ok(ModpackInfo {
        format: ModpackFormat::Mcbbs,
        game_version: gv,
        loader,
        loader_version: loader_ver,
        mod_files_count: 0,
        archive_base_folder: detected.archive_base_folder.clone(),
        cf_overrides_name: None,
        cf_manifest: None,
        mr_index: None,
        hmcl_manifest: None,
        mmc_pack: None,
        mmc_cfg_content: None,
        mcbbs_manifest: Some(manifest),
        launcher_inner_path: None,
    })
}

/// LauncherPack：记录内层整合包路径，实际递归安装由 install 流程处理
pub fn parse_launcher_pack(detected: &DetectedModpack) -> Result<ModpackInfo, String> {
    let inner_path = detected
        .launcher_inner_path
        .clone()
        .ok_or_else(|| "LauncherPack 检测异常：未记录内层整合包路径".to_string())?;
    log_info!(
        "[Community] LauncherPack 整合包: 内层整合包路径={}",
        inner_path
    );
    Ok(ModpackInfo {
        format: ModpackFormat::LauncherPack,
        game_version: String::new(),
        loader: String::new(),
        loader_version: String::new(),
        mod_files_count: 0,
        archive_base_folder: String::new(),
        cf_overrides_name: None,
        cf_manifest: None,
        mr_index: None,
        hmcl_manifest: None,
        mmc_pack: None,
        mmc_cfg_content: None,
        mcbbs_manifest: None,
        launcher_inner_path: Some(inner_path),
    })
}

/// Compress 普通压缩包兜底：archive_base_folder 已是 `.minecraft/` 前缀
pub fn parse_compress(detected: &DetectedModpack) -> Result<ModpackInfo, String> {
    log_info!(
        "[Community] Compress 整合包: archive_base_folder={}",
        detected.archive_base_folder
    );
    Ok(ModpackInfo {
        format: ModpackFormat::Compress,
        game_version: String::new(),
        loader: String::new(),
        loader_version: String::new(),
        mod_files_count: 0,
        archive_base_folder: detected.archive_base_folder.clone(),
        cf_overrides_name: None,
        cf_manifest: None,
        mr_index: None,
        hmcl_manifest: None,
        mmc_pack: None,
        mmc_cfg_content: None,
        mcbbs_manifest: None,
        launcher_inner_path: None,
    })
}