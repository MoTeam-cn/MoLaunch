//! Modrinth API 响应数据结构
//!
//! 仅 modrinth 模块内部使用，外部不应引用。

use serde::Deserialize;

/// Modrinth 官方 API 基地址
pub(crate) const MR_OFFICIAL_BASE: &str = "https://api.modrinth.com/v2";

/// Modrinth 镜像 API 基地址（MCIM 镜像源）
pub(crate) const MR_MIRROR_BASE: &str = "https://mod.mcimirror.top/modrinth/v2";

/// Modrinth 搜索响应
#[derive(Debug, Deserialize)]
pub(crate) struct MrSearchResponse {
    pub hits: Vec<MrHit>,
    pub total_hits: u32,
}

/// Modrinth 搜索命中
#[derive(Debug, Deserialize)]
pub(crate) struct MrHit {
    #[serde(default)]
    pub project_id: String,
    #[serde(default)]
    pub slug: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub categories: Vec<String>,
    #[serde(default)]
    pub downloads: u64,
    #[serde(default)]
    pub icon_url: Option<String>,
    #[serde(default)]
    pub project_type: String,
    #[serde(default)]
    pub versions: Vec<String>,
    #[serde(default)]
    pub date_modified: Option<String>,
    #[serde(default)]
    pub display: Option<String>,
}

/// Modrinth 工程详情
#[derive(Debug, Deserialize)]
pub(crate) struct MrProject {
    pub id: String,
    pub slug: Option<String>,
    pub title: String,
    pub description: Option<String>,
    #[serde(default)]
    pub categories: Vec<String>,
    #[serde(default)]
    pub downloads: u64,
    pub icon_url: Option<String>,
    pub project_type: String,
    #[serde(default)]
    pub game_versions: Vec<String>,
    #[serde(default)]
    pub loaders: Vec<String>,
    pub updated: Option<String>,
}

/// Modrinth 版本
#[derive(Debug, Deserialize)]
pub(crate) struct MrVersion {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub version_number: String,
    #[serde(default)]
    pub date_published: String,
    #[serde(default)]
    pub downloads: u64,
    #[serde(default)]
    pub version_type: String,
    #[serde(default)]
    pub game_versions: Vec<String>,
    #[serde(default)]
    pub loaders: Vec<String>,
    #[serde(default)]
    pub files: Vec<MrFile>,
    #[serde(default)]
    pub dependencies: Vec<MrDependency>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct MrFile {
    #[serde(default)]
    pub url: String,
    pub filename: Option<String>,
    pub primary: Option<bool>,
    pub size: Option<u64>,
    pub hashes: Option<MrHashes>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct MrHashes {
    pub sha1: Option<String>,
    /// SHA512 hash（hex），MR API 在 hashes 字段中返回，导出整合包时需要
    #[serde(default)]
    pub sha512: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct MrDependency {
    pub project_id: Option<String>,
    pub dependency_type: Option<String>,
}
