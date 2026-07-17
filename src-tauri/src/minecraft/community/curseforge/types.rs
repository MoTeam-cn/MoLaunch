//! CurseForge API 响应数据结构
//!
//! 仅 curseforge 模块内部使用，外部不应引用。

use serde::Deserialize;

/// CurseForge 搜索响应
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CfSearchResponse {
    pub data: Vec<CfModEntry>,
    pub pagination: CfPagination,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CfPagination {
    pub total_count: u32,
}

/// CurseForge 工程条目（搜索结果和详情共用）
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CfModEntry {
    pub id: i64,
    pub slug: Option<String>,
    pub name: String,
    pub summary: Option<String>,
    #[serde(default)]
    pub download_count: u64,
    #[serde(default)]
    pub date_released: String,
    #[serde(default)]
    pub logo: Option<CfLogo>,
    #[serde(default)]
    pub latest_files: Vec<CfFile>,
    pub links: Option<CfLinks>,
    #[serde(default)]
    pub categories: Vec<CfCategory>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CfLogo {
    #[serde(default)]
    pub thumbnail_url: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CfLinks {
    #[serde(default)]
    pub website_url: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CfCategory {
    #[serde(default)]
    pub id: Option<u32>,
    #[serde(default)]
    pub name: Option<String>,
}

/// CurseForge 文件
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CfFile {
    pub id: i64,
    /// 文件的 MurmurHash2 指纹（与请求 /fingerprints/432 时传入的指纹一致）
    ///
    /// 参考 PCL2 `Project("file")("fileFingerprint")`：用于反查 exactMatches[i] 对应哪个本地指纹。
    /// 注意：CF 的 fileFingerprint 是 uint32 number，不是字符串。
    #[serde(default)]
    pub file_fingerprint: u32,
    #[serde(default)]
    pub display_name: String,
    #[serde(default)]
    pub file_name: String,
    #[serde(default)]
    pub file_date: String,
    #[serde(default)]
    pub download_count: u64,
    #[serde(default)]
    pub release_type: u32,
    #[serde(default)]
    pub download_url: Option<String>,
    #[serde(default)]
    pub game_versions: Vec<String>,
    #[serde(default)]
    pub hashes: Vec<CfHash>,
    #[serde(default)]
    pub file_length: u64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CfHash {
    #[serde(default)]
    pub algo: u32, // 1=SHA1, 2=MD5
    pub value: String,
}

/// CurseForge 版本列表响应
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CfFilesResponse {
    pub data: Vec<CfFile>,
}
