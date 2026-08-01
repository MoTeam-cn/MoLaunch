//! 社区资源数据类型定义
//!
//! 统一 CurseForge 和 Modrinth 两个平台的数据结构

use serde::{Deserialize, Serialize};

/// 资源类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceType {
    Mod,
    ModPack,
    ResourcePack,
    Shader,
    DataPack,
}

impl ResourceType {
    /// CurseForge classId
    pub fn curseforge_class_id(&self) -> u32 {
        match self {
            ResourceType::Mod => 6,
            ResourceType::ModPack => 4471,
            ResourceType::DataPack => 6945,
            ResourceType::Shader => 6552,
            ResourceType::ResourcePack => 12,
        }
    }

    /// Modrinth project_type
    pub fn modrinth_project_type(&self) -> &'static str {
        match self {
            ResourceType::Mod => "mod",
            ResourceType::ModPack => "modpack",
            ResourceType::ResourcePack => "resourcepack",
            ResourceType::Shader => "shader",
            ResourceType::DataPack => "mod", // Modrinth 数据包归入 mod
        }
    }

    /// 安装子目录
    pub fn install_subdir(&self) -> &'static str {
        match self {
            ResourceType::Mod => "mods",
            ResourceType::ResourcePack => "resourcepacks",
            ResourceType::Shader => "shaderpacks",
            ResourceType::DataPack => "datapacks",
            ResourceType::ModPack => "", // 整合包走特殊安装流程
        }
    }
}

/// 资源来源平台
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Platform {
    CurseForge,
    Modrinth,
}

impl Platform {
    pub fn as_str(&self) -> &'static str {
        match self {
            Platform::CurseForge => "CurseForge",
            Platform::Modrinth => "Modrinth",
        }
    }
}

/// 加载器类型（Flags 枚举）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ModLoaders(pub u32);

impl ModLoaders {
    pub const NONE: u32 = 0;
    pub const FORGE: u32 = 1;
    pub const LITELOADER: u32 = 2;
    pub const FABRIC: u32 = 4;
    pub const QUILT: u32 = 8;
    pub const NEOFORGE: u32 = 16;

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> u32 {
        match s.to_lowercase().as_str() {
            "forge" => Self::FORGE,
            "liteloader" => Self::LITELOADER,
            "fabric" => Self::FABRIC,
            "quilt" => Self::QUILT,
            "neoforge" => Self::NEOFORGE,
            _ => 0,
        }
    }

    pub fn contains(&self, flag: u32) -> bool {
        self.0 & flag != 0
    }

    /// 返回包含的加载器名称列表
    pub fn to_names(&self) -> Vec<String> {
        let mut names = Vec::new();
        if self.contains(Self::FORGE) {
            names.push("Forge".into());
        }
        if self.contains(Self::NEOFORGE) {
            names.push("NeoForge".into());
        }
        if self.contains(Self::FABRIC) {
            names.push("Fabric".into());
        }
        if self.contains(Self::QUILT) {
            names.push("Quilt".into());
        }
        if self.contains(Self::LITELOADER) {
            names.push("LiteLoader".into());
        }
        names
    }
}

/// 发布类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReleaseType {
    Release,
    Beta,
    Alpha,
}

impl ReleaseType {
    pub fn from_curseforge(n: u32) -> Self {
        match n {
            1 => ReleaseType::Release,
            2 => ReleaseType::Beta,
            3 => ReleaseType::Alpha,
            _ => ReleaseType::Release,
        }
    }

    pub fn from_modrinth(s: &str) -> Self {
        match s {
            "release" => ReleaseType::Release,
            "beta" => ReleaseType::Beta,
            "alpha" => ReleaseType::Alpha,
            _ => ReleaseType::Release,
        }
    }
}

/// 资源工程
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceProject {
    pub platform: Platform,
    pub resource_type: ResourceType,
    pub id: String,
    pub slug: String,
    pub raw_name: String,
    /// 中文译名（来自 mcmod.cn 数据库），为空则无翻译
    #[serde(default)]
    pub translated_name: String,
    pub description: String,
    pub website: String,
    pub last_update: String,
    pub download_count: u64,
    pub mod_loaders: u32,
    pub tags: Vec<String>,
    pub logo_url: Option<String>,
    pub game_versions: Vec<String>,
}

/// 资源版本/文件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceVersion {
    pub id: String,
    pub display: String,
    pub version: String,
    pub release_date: String,
    pub download_count: u64,
    pub mod_loaders: u32,
    pub game_versions: Vec<String>,
    pub release_type: ReleaseType,
    pub file_name: String,
    pub download_url: String,
    pub hash: Option<String>,
    pub size: u64,
    pub dependencies: Vec<String>,
}

/// 联网查询得到的 mod 文件下载信息（导出整合包专用）
///
/// 由 `modrinth::version_files_search_with_downloads` 和
/// `curseforge::fingerprint_search_with_downloads` 返回，
/// 用于直接写入 `modrinth.index.json` 的 files 数组。
#[derive(Debug, Clone)]
pub struct FileDownloadInfo {
    /// 下载地址（与平台返回的 URL 一致，CF 已包含 CDN 路径）
    pub download_url: String,
    /// 文件大小（字节）
    pub file_size: u64,
    /// SHA1 hash（hex，MR 来自 API 响应；CF 可能为空，由调用方本地计算）
    pub sha1: String,
    /// SHA512 hash（hex，MR 来自 API 响应；CF 不提供，由调用方本地计算）
    pub sha512: Option<String>,
    /// CurseForge project id（仅 CF 查询结果设置，MR 为 None）
    /// 用于导出 CurseForge 格式整合包时写入 manifest.files[].projectID
    pub project_id: Option<i64>,
    /// CurseForge file id（仅 CF 查询结果设置，MR 为 None）
    /// 用于导出 CurseForge 格式整合包时写入 manifest.files[].fileID
    pub file_id: Option<i64>,
}

/// 搜索请求参数
#[derive(Debug, Clone, Deserialize)]
pub struct SearchParams {
    /// 搜索词
    pub query: String,
    /// 资源类型
    pub resource_type: ResourceType,
    /// 游戏版本（空=不限）
    pub game_version: Option<String>,
    /// 加载器（0=不限，1=Forge, 4=Fabric, 16=NeoForge, 8=Quilt, 2=LiteLoader）
    pub mod_loader: u32,
    /// 来源筛选：0=全部, 1=仅CurseForge, 2=仅Modrinth
    pub source: u32,
    /// 分类标签（CurseForge 数字 ID / Modrinth slug，空=不限）
    pub category: Option<String>,
    /// 页码（0-based）
    pub page: u32,
}

/// 搜索结果
#[derive(Debug, Clone, Serialize)]
pub struct SearchResult {
    pub projects: Vec<ResourceProject>,
    pub total_count: u32,
    pub page: u32,
    pub page_size: u32,
}
