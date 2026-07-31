//! 各格式整合包 manifest 解析（CF / MR / HMCL / MMC / MCBBS / LauncherPack / Compress）

use crate::log_info;

use super::super::concurrent::DetectedModpack;
use super::super::curseforge::CfManifest;
use super::super::helpers::{parse_cf_loader_id, parse_mr_loader};
use super::super::hmcl::HmclManifest;
use super::super::mcbbs::McbbsManifest;
use super::super::mmc::MmcPack;
use super::super::modrinth::MrIndex;
use super::super::types::{ModpackFormat, ModpackInfo};

/// 解析 CurseForge manifest.json
pub(super) fn parse_cf(detected: &DetectedModpack) -> Result<ModpackInfo, String> {
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
                "该整合包版本过老（使用旧版 Forge recommended 格式），请尝试更新版本的整合包".to_string(),
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

/// 解析 Modrinth modrinth.index.json
pub(super) fn parse_mr(detected: &DetectedModpack) -> Result<ModpackInfo, String> {
    let index: MrIndex =
        serde_json::from_str(detected.index_content.as_deref().unwrap_or(""))
            .map_err(|e| format!("解析 modrinth.index.json 失败: {}", e))?;
    let gv = index
        .dependencies
        .get("minecraft")
        .cloned()
        .unwrap_or_default();
    // Quilt 加载器检测：dependencies 含 quilt-loader 直接报错
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

/// 解析 HMCL modpack.json
pub(super) fn parse_hmcl(detected: &DetectedModpack) -> Result<ModpackInfo, String> {
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

/// 解析 MMC mmc-pack.json
pub(super) fn parse_mmc(detected: &DetectedModpack) -> Result<ModpackInfo, String> {
    let pack: MmcPack =
        serde_json::from_str(detected.mmc_content.as_deref().unwrap_or(""))
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
pub(super) fn parse_mcbbs(detected: &DetectedModpack) -> Result<ModpackInfo, String> {
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
pub(super) fn parse_launcher_pack(detected: &DetectedModpack) -> Result<ModpackInfo, String> {
    let inner_path = detected.launcher_inner_path.clone().ok_or_else(|| {
        "LauncherPack 检测异常：未记录内层整合包路径".to_string()
    })?;
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
pub(super) fn parse_compress(detected: &DetectedModpack) -> Result<ModpackInfo, String> {
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
