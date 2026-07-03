//! Assets 资源下载模块
//! 资源索引解析、哈希路径映射、批量下载

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use super::super::utils::file_checker::FileChecker;

/// 资源条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetEntry {
    /// 逻辑路径（如 minecraft/sounds/xxx.ogg）
    pub source_path: String,
    /// 本地存储路径
    pub local_path: String,
    /// 文件哈希
    pub hash: String,
    /// 文件大小
    pub size: i64,
}

/// 资源索引元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetIndexMeta {
    pub id: String,
    pub sha1: String,
    pub size: i64,
    pub url: String,
    pub total_size: Option<i64>,
}

/// 从版本 JSON 中获取资源索引元数据
pub fn get_asset_index_meta(json: &serde_json::Value) -> Option<AssetIndexMeta> {
    // 优先从 assetIndex 获取
    if let Some(index) = json.get("assetIndex") {
        let id = index["id"].as_str()?;
        let sha1 = index["sha1"].as_str().unwrap_or_default();
        let size = index["size"].as_i64().unwrap_or(0);
        let url = index["url"].as_str()?;

        return Some(AssetIndexMeta {
            id: id.to_string(),
            sha1: sha1.to_string(),
            size,
            url: url.to_string(),
            total_size: index["totalSize"].as_i64(),
        });
    }

    // 回退：从 assets 字段构建
    if let Some(assets) = json["assets"].as_str() {
        return Some(AssetIndexMeta {
            id: assets.to_string(),
            sha1: String::new(),
            size: 0,
            url: format!(
                "https://piston-meta.mojang.com/mc/game/assets/{}/{}",
                "2ec0cc96c44e5a76b9c8b7c39df7210883d12871", // 常见的索引版本
                assets
            ),
            total_size: None,
        });
    }

    None
}

/// 解析资源索引 JSON，获取所有资源条目
pub fn parse_asset_index(
    index_json: &serde_json::Value,
    game_dir: &Path,
) -> Vec<AssetEntry> {
    let mut entries = Vec::new();

    let objects = match index_json.get("objects").and_then(|o| o.as_object()) {
        Some(objs) => objs,
        None => return entries,
    };

    // 检查是否为 legacy 模式
    let is_legacy = index_json.get("virtual").and_then(|v| v.as_bool()).unwrap_or(false);
    let is_map_to_resources = index_json.get("map_to_resources").and_then(|v| v.as_bool()).unwrap_or(false);

    for (source_path, object) in objects {
        let hash = object["hash"].as_str().unwrap_or_default();
        let size = object["size"].as_i64().unwrap_or(0);

        let local_path = if is_map_to_resources {
            // 极老版本：resources 模式
            game_dir.join("resources").join(source_path).to_string_lossy().to_string()
        } else if is_legacy {
            // 旧版本：virtual 模式
            game_dir.join("assets").join("virtual").join("legacy").join(source_path).to_string_lossy().to_string()
        } else {
            // 正常模式：objects 目录
            let prefix = &hash[..2.min(hash.len())];
            game_dir.join("assets").join("objects").join(prefix).join(hash).to_string_lossy().to_string()
        };

        entries.push(AssetEntry {
            source_path: source_path.clone(),
            local_path,
            hash: hash.to_string(),
            size,
        });
    }

    entries
}

/// 获取资源索引的本地路径
pub fn get_asset_index_path(game_dir: &Path, index_id: &str) -> PathBuf {
    game_dir.join("assets").join("indexes").join(format!("{}.json", index_id))
}

/// 下载资源索引的 URL 列表
pub fn get_asset_index_urls(meta: &AssetIndexMeta) -> Vec<String> {
    let mut urls = vec![meta.url.clone()];

    // BMCLAPI 镜像
    let bmclapi_url = meta.url
        .replace("https://piston-data.mojang.com", "https://bmclapi2.bangbang93.com")
        .replace("https://piston-meta.mojang.com", "https://bmclapi2.bangbang93.com")
        .replace("https://launchermeta.mojang.com", "https://bmclapi2.bangbang93.com")
        .replace("https://launcher.mojang.com", "https://bmclapi2.bangbang93.com");

    if bmclapi_url != meta.url {
        urls.push(bmclapi_url);
    }

    urls
}

/// 检测缺失的资源文件
pub fn find_missing_assets(
    entries: &[AssetEntry],
) -> Vec<AssetEntry> {
    let mut missing = Vec::new();

    for entry in entries {
        let checker = FileChecker::new()
            .with_actual_size(if entry.size == 0 { -1 } else { entry.size })
            .with_hash(if entry.hash.is_empty() { None } else { Some(entry.hash.clone()) });

        if !checker.is_valid(&entry.local_path) {
            missing.push(entry.clone());
        }
    }

    missing
}

/// 构建资源文件的下载 URL 列表
pub fn build_asset_download_urls(entry: &AssetEntry, mirror_url: Option<&str>) -> Vec<String> {
    let hash = &entry.hash;
    let prefix = &hash[..2.min(hash.len())];

    let mut urls = Vec::new();

    // 官方源
    urls.push(format!(
        "https://resources.download.minecraft.net/{}/{}",
        prefix, hash
    ));

    // BMCLAPI 镜像
    urls.push(format!(
        "https://bmclapi2.bangbang93.com/assets/{}/{}",
        prefix, hash
    ));

    // 自定义镜像源
    if let Some(mirror) = mirror_url {
        let mirror_base = mirror.trim_end_matches('/');
        urls.push(format!("{}/assets/{}/{}", mirror_base, prefix, hash));
    }

    urls
}
