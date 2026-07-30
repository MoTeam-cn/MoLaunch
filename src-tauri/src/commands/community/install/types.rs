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
    /// 是否下载可选 Mod（CF required=false / MR env.client=optional）。
    /// None 时默认 true（在线资源页安装不弹窗）。
    #[serde(default)]
    pub include_optional: Option<bool>,
    /// 外部 Logo 文件本地路径（CF/MR 平台下载时缓存的缩略图，复制到 MoLaunch/Logo.png）。
    /// None 表示无外部 Logo，仅依赖 MMC iconKey 等内部图标迁移。
    #[serde(default)]
    pub logo_path: Option<String>,
    /// 平台工程 ID（CF project id / MR project id）。
    /// 在线资源页安装时由前端从工程详情响应传入；None 时跳过 modpack.meta.json 写入。
    #[serde(default)]
    pub project_id: Option<String>,
    /// 平台文件 ID（CF file id / MR version id）。
    /// 在线资源页安装时由前端从版本列表响应传入。
    #[serde(default)]
    pub file_id: Option<String>,
    /// 整合包自身版本号（如 `2.9.3`，来自平台版本列表的 version/displayName）。
    #[serde(default)]
    pub modpack_version: Option<String>,
    /// 整合包文件大小（字节，来自平台版本列表的 size/fileLength）。
    #[serde(default)]
    pub file_size: Option<u64>,
    /// 整合包名称（来自平台工程详情的 raw_name/translated_name）。
    #[serde(default)]
    pub name: Option<String>,
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
    /// 是否下载可选 Mod（CF required=false / MR env.client=optional）。
    /// 由前端 preview 后弹窗询问用户传入。None 时默认 true（保持向后兼容）。
    #[serde(default)]
    pub include_optional: Option<bool>,
    /// 外部 Logo 文件本地路径（拖拽安装时无外部 Logo，通常为 None）
    #[serde(default)]
    pub logo_path: Option<String>,
}

/// 可选 Mod 信息（用于前端弹窗显示）
///
/// CF: file_id + project_id（display_name 用 file_id 字符串，因 manifest 无 displayName 字段，
/// MoLaunch 在解析阶段预览，不调用 /mods/files API）
/// MR: path 末段作为 display_name
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OptionalModInfo {
    /// 显示名（CF: "CF File #{file_id}"，MR: path 末段）
    pub display_name: String,
    /// 文件大小（字节，CF 为 0 因为 manifest 不含大小，MR 从 file_size 取）
    pub file_size: u64,
    /// CurseForge file_id（仅 CF 格式有值）
    pub file_id: Option<i64>,
    /// CurseForge project_id（仅 CF 格式有值，可能为 None）
    pub project_id: Option<i64>,
    /// Modrinth 文件路径（仅 MR 格式有值）
    pub path: Option<String>,
}

/// 整合包预览信息（前端弹窗询问可选 Mod 用）
///
/// 由 `preview_local_modpack` 命令返回，前端根据 `optional_mods` 列表弹窗询问用户是否下载。
/// 用户选择后调用 `install_local_modpack` 传入 `include_optional` 参数。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModpackPreview {
    /// 识别出的整合包格式
    pub format: ModpackFormat,
    /// 游戏版本
    pub game_version: String,
    /// 加载器名称
    pub loader: String,
    /// 加载器版本
    pub loader_version: String,
    /// 可选 Mod 列表（CF required=false / MR env.client=optional）
    pub optional_mods: Vec<OptionalModInfo>,
}

/// 整合包格式
///
/// 识别优先级：
/// 1. mcbbs.packmeta → Mcbbs
/// 2. mmc-pack.json → Mmc
/// 3. modrinth.index.json → Modrinth
/// 4. manifest.json：有 addons → Mcbbs，无 addons → Curseforge
/// 5. modpack.json → Hmcl
/// 6. modpack.zip / modpack.mrpack → LauncherPack（带启动器整合包）
/// 7. 其他 → Compress（普通压缩包，需含 `.minecraft/` 目录）
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
    /// 带启动器整合包：zip 内含 `modpack.zip` 或 `modpack.mrpack`，
    /// 需先解压内层整合包到临时目录再递归安装
    LauncherPack,
    /// 普通压缩包兜底：无关键 manifest 文件，但含 `.minecraft/` 目录，
    /// 将该目录内容作为 overrides 解压到 instance 目录
    Compress,
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
    /// CurseForge manifest.overrides 字段（自定义覆写目录名，默认 None 表示 "overrides"）
    pub cf_overrides_name: Option<String>,
    /// CF manifest（仅 Curseforge 格式有值）
    pub cf_manifest: Option<super::curseforge::CfManifest>,
    /// MR index（仅 Modrinth 格式有值）
    pub mr_index: Option<super::modrinth::MrIndex>,
    /// HMCL manifest（仅 Hmcl 格式有值，保留供未来扩展：HMCL 整合包的 author/description 等元信息）
    #[allow(dead_code)]
    pub hmcl_manifest: Option<super::hmcl::HmclManifest>,
    /// MMC pack（仅 Mmc 格式有值，保留供未来扩展：components 元信息已用于 game_version/loader 解析）
    #[allow(dead_code)]
    pub mmc_pack: Option<super::mmc::MmcPack>,
    /// MMC instance.cfg 原始内容（仅 Mmc 格式有值，用于配置迁移）
    pub mmc_cfg_content: Option<String>,
    /// MCBBS manifest（仅 Mcbbs 格式有值，用于 launchInfo 迁移）
    pub mcbbs_manifest: Option<super::mcbbs::McbbsManifest>,
    /// LauncherPack 内层整合包在 zip 中的完整路径（仅 LauncherPack 有值，用于解压递归安装）
    #[allow(dead_code)]
    pub launcher_inner_path: Option<String>,
}
