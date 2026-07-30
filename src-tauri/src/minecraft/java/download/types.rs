//! Java 下载相关数据类型

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Mojang all.json 中的单个 Java 条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JavaRuntimeEntry {
    #[serde(rename = "manifest")]
    pub manifest: ManifestRef,
    pub version: VersionInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestRef {
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    pub name: String,
}

/// manifest.json 中的文件清单
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeManifest {
    pub files: HashMap<String, RuntimeFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeFile {
    #[serde(rename = "type")]
    pub file_type: String,
    pub downloads: Option<Downloads>,
    #[serde(default)]
    pub executable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Downloads {
    pub raw: DownloadInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadInfo {
    pub url: String,
    pub size: u64,
    pub sha1: String,
}
