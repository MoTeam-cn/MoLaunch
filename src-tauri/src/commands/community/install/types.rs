//! 社区资源下载安装 - 数据类型
//!
//! 包含下载请求/结果/进度、整合包请求/格式/结果 6 个对外可见的数据结构，
//! 以及 install_modpack 内部使用的 ModpackInfo 中间结构。

use crate::minecraft::community::types::{Platform, ResourceType};
use serde::{Deserialize, Serialize};

/// 下载安装请求
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadRequest {
    /// 下载 URL
    pub url: String,
    /// 文件名（原始名，后端会根据 community_filename_format 重命名）
    pub file_name: String,
    /// 资源类型
    pub resource_type: ResourceType,
    /// 目标版本 ID（安装到哪个版本目录）
    pub version_id: Option<String>,
    /// 文件 SHA1（用于校验）
    pub hash: Option<String>,
    /// 译名（可选，来自 mcmod 数据库，用于按 filename_format 拼接新文件名）
    pub translated_name: Option<String>,
}

/// 下载安装结果
#[derive(Debug, Serialize)]
pub struct DownloadResult {
    pub path: String,
    pub size: u64,
}

/// 社区资源下载进度事件
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommunityDownloadProgress {
    /// 文件名
    pub file_name: String,
    /// 已下载字节
    pub downloaded: u64,
    /// 总字节（未知则为 0）
    pub total: u64,
    /// 下载速度（字节/秒）
    pub speed: u64,
    /// 是否完成
    pub completed: bool,
    /// 错误信息
    pub error: Option<String>,
}

/// 整合包安装请求
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallModpackRequest {
    /// 来源平台
    pub platform: Platform,
    /// 下载 URL
    pub download_url: String,
    /// 原始文件名（如 MyModpack-1.0.zip / .mrpack）
    pub file_name: String,
    /// 整合包实例名（用于 versions/{instance_name}/ 目录）
    pub instance_name: String,
}

/// 本地整合包安装请求（拖拽安装）
///
/// 与 `InstallModpackRequest` 的差异：直接使用本地文件路径，跳过 Stage 0 下载。
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallLocalModpackRequest {
    /// 本地整合包文件绝对路径（.zip / .mrpack）
    pub file_path: String,
    /// 整合包实例名（用于 versions/{instance_name}/ 目录）
    pub instance_name: String,
}

/// 整合包格式
///
/// 识别优先级（参考 PCL2 ModModpack.vb）：
/// 1. mcbbs.packmeta → Mcbbs
/// 2. mmc-pack.json → Mmc
/// 3. modrinth.index.json → Modrinth
/// 4. manifest.json：有 addons → Mcbbs，无 addons → Curseforge
/// 5. modpack.json → Hmcl
/// 6. modpack.zip / modpack.mrpack → LauncherPack（暂未实现）
/// 7. 其他 → Compress（暂未实现）
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ModpackFormat {
    Curseforge,
    Modrinth,
    /// HMCL 整合包（modpack.json），overrides 在 `minecraft/` 目录
    Hmcl,
    /// MultiMC 整合包（mmc-pack.json + instance.cfg），overrides 在 `.minecraft/` 目录
    Mmc,
    /// MCBBS 整合包（mcbbs.packmeta 或带 addons 的 manifest.json），overrides 在 `overrides/` 目录
    Mcbbs,
}

/// 整合包安装结果
///
/// 完成整合包专属部分（下载原始包、下载依赖 mods、复制 overrides）后返回。
/// 前端拿到结果后调用 `install_merged` 安装游戏本体（使用返回的 mc_version + loader 信息）。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallModpackResult {
    /// 识别出的整合包格式
    pub format: ModpackFormat,
    /// 整合包内 minecraft.version
    pub game_version: String,
    /// 加载器名称（forge / fabric / quilt / neoforge / liteloader），空表示原版
    pub loader: String,
    /// 加载器版本
    pub loader_version: String,
    /// 原始整合包保存路径
    pub archive_path: String,
    /// instance 目录
    pub instance_dir: String,
}

/// install_modpack Stage 1 解析得到的整合包信息（中间结构）
///
/// 由 `modpack_stages::parse_modpack_info` 返回，供 install_modpack 后续阶段使用。
/// 跨 CF / MR / HMCL / MMC / MCBBS 五种格式统一为单一结构，
/// 避免 install_modpack 中 match 分支变量类型不一致。
pub(super) struct ModpackInfo {
    /// 识别出的整合包格式
    pub format: ModpackFormat,
    /// 整合包内 minecraft.version / dependencies["minecraft"] / gameVersion / addons.game
    pub game_version: String,
    /// 加载器名称（forge / fabric / quilt / neoforge / optifine），空表示原版
    pub loader: String,
    /// 加载器版本
    pub loader_version: String,
    /// manifest / index 中 files[] 长度（用于日志展示，HMCL/MMC/MCBBS 无依赖列表时为 0）
    pub mod_files_count: usize,
    /// 关键文件所在层级前缀（如 `""` 或 `"subfolder/"`），与 `format` 一起决定 overrides 前缀
    pub archive_base_folder: String,
    /// CF manifest（仅 Curseforge 格式有值）
    pub cf_manifest: Option<super::curseforge::CfManifest>,
    /// MR index（仅 Modrinth 格式有值）
    pub mr_index: Option<super::modrinth::MrIndex>,
    /// HMCL manifest（仅 Hmcl 格式有值，保留供未来扩展：HMCL 整合包的 author/description 等元信息）
    #[allow(dead_code)]
    pub hmcl_manifest: Option<super::hmcl::HmclManifest>,
    /// MMC pack（仅 Mmc 格式有值，保留供未来扩展：MMC instance.cfg 的 JVM 参数 / PreLaunchCommand 迁移）
    #[allow(dead_code)]
    pub mmc_pack: Option<super::mmc::MmcPack>,
    /// MCBBS manifest（仅 Mcbbs 格式有值，保留供未来扩展：MCBBS launchInfo 的 javaArgument / launchArgument 迁移）
    #[allow(dead_code)]
    pub mcbbs_manifest: Option<super::mcbbs::McbbsManifest>,
}
