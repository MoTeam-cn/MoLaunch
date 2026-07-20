//! Assets 资源下载模块
//! 资源索引解析、哈希路径映射、批量下载

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use super::super::sources;
use super::super::sources::DownloadSourceMode;
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
                "{}/mc/game/assets/{}/{}",
                sources::MOJANG_PISTON_META,
                "2ec0cc96c44e5a76b9c8b7c39df7210883d12871", // 常见的索引版本
                assets
            ),
            total_size: None,
        });
    }

    // 最终回退：McAssetsGetIndex 函数
    // 当无法获取 assetIndex 时，使用硬编码的 legacy fallback
    let inherits_from = json
        .get("inheritsFrom")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    if !inherits_from.is_empty() {
        crate::log_warn!(
            "[Assets] No assetIndex found in JSON for version with inheritsFrom={}",
            inherits_from
        );
        crate::log_warn!("[Assets] Using legacy asset index as fallback");
    } else {
        crate::log_warn!("[Assets] No assetIndex found in JSON, using legacy fallback");
    }

    // 硬编码 legacy fallback
    // https://launchermeta.mojang.com/mc-staging/assets/legacy/c0fd82e8ce9fbc93119e40d96d5a4e62cfa3f729/legacy.json
    Some(AssetIndexMeta {
        id: "legacy".to_string(),
        sha1: "c0fd82e8ce9fbc93119e40d96d5a4e62cfa3f729".to_string(),
        size: 134284,
        url: format!(
            "{}/mc-staging/assets/legacy/{}/legacy.json",
            sources::MOJANG_LAUNCHERMETA,
            "c0fd82e8ce9fbc93119e40d96d5a4e62cfa3f729"
        ),
        total_size: Some(111220701),
    })
}

/// 解析资源索引 JSON，获取所有资源条目
pub fn parse_asset_index(index_json: &serde_json::Value, game_dir: &Path) -> Vec<AssetEntry> {
    let mut entries = Vec::new();

    let objects = match index_json.get("objects").and_then(|o| o.as_object()) {
        Some(objs) => objs,
        None => return entries,
    };

    // 检查是否为 legacy 模式
    let is_legacy = index_json
        .get("virtual")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let is_map_to_resources = index_json
        .get("map_to_resources")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    for (source_path, object) in objects {
        let hash = object["hash"].as_str().unwrap_or_default();
        let size = object["size"].as_i64().unwrap_or(0);

        // 路径遍历防护：拒绝含 ".." 的 source_path
        if source_path.contains("..") {
            crate::log_warn!(
                "[Assets] Skip path traversal in source_path: {}",
                source_path
            );
            continue;
        }

        let local_path = if is_map_to_resources {
            // 极老版本：resources 模式
            game_dir
                .join("resources")
                .join(source_path)
                .to_string_lossy()
                .to_string()
        } else if is_legacy {
            // 旧版本：virtual 模式
            game_dir
                .join("assets")
                .join("virtual")
                .join("legacy")
                .join(source_path)
                .to_string_lossy()
                .to_string()
        } else {
            // 正常模式：objects 目录
            let prefix = &hash[..2.min(hash.len())];
            game_dir
                .join("assets")
                .join("objects")
                .join(prefix)
                .join(hash)
                .to_string_lossy()
                .to_string()
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
    game_dir
        .join("assets")
        .join("indexes")
        .join(format!("{}.json", index_id))
}

/// 下载资源索引的 URL 列表
pub fn get_asset_index_urls(meta: &AssetIndexMeta, source_mode: DownloadSourceMode) -> Vec<String> {
    sources::build_replace_urls(&meta.url, None, sources::MOJANG_REPLACEMENTS, source_mode)
}

/// 检测缺失的资源文件
/// Find missing assets
///
/// ## 性能优化（与 `find_missing_libs` 一致）
///
/// - **并行检查**：使用 `std::thread::scope` 并行检查多个资源文件
/// - **快速检查模式**（`quick_check = true`）：只检查文件存在 + 大小匹配，不计算 SHA1
///   - 用于启动时的文件校验（assets 数量通常几百上千，串行哈希校验会非常慢）
///   - 启动时不做哈希校验
/// - **完整校验模式**（`quick_check = false`）：计算 SHA1 哈希，确保文件完整性
///   - 用于版本安装/修复时的严格校验
pub fn find_missing_assets(entries: &[AssetEntry], quick_check: bool) -> Vec<AssetEntry> {
    use std::sync::Mutex;

    let missing: Mutex<Vec<AssetEntry>> = Mutex::new(Vec::new());
    let num_threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
        .min(entries.len())
        .max(1);

    // 按索引取模分配到各线程
    let chunks: Vec<Vec<&AssetEntry>> = if num_threads > 1 {
        (0..num_threads)
            .map(|tid| {
                entries
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| i % num_threads == tid)
                    .map(|(_, e)| e)
                    .collect()
            })
            .collect()
    } else {
        vec![entries.iter().collect()]
    };

    std::thread::scope(|s| {
        for chunk in chunks {
            let missing = &missing;
            s.spawn(move || {
                for entry in chunk {
                    let is_ok = if quick_check {
                        // 快速检查：只检查文件存在 + 大小匹配
                        quick_check_asset(entry)
                    } else {
                        // 完整校验：文件存在 + 大小 + SHA1
                        let checker = FileChecker::new()
                            .with_actual_size(if entry.size == 0 { -1 } else { entry.size })
                            .with_hash(if entry.hash.is_empty() {
                                None
                            } else {
                                Some(entry.hash.clone())
                            });
                        checker.is_valid(&entry.local_path)
                    };
                    if !is_ok {
                        missing.lock().unwrap().push(entry.clone());
                    }
                }
            });
        }
    });

    // 保持原有顺序
    let mut result = missing.into_inner().unwrap();
    result.sort_by_key(|e| {
        entries
            .iter()
            .position(|x| x.local_path == e.local_path)
            .unwrap_or(usize::MAX)
    });
    result
}

/// 快速检查资源文件：只检查文件存在 + 大小匹配，不计算哈希
fn quick_check_asset(entry: &AssetEntry) -> bool {
    let path = Path::new(&entry.local_path);
    if !path.exists() {
        return false;
    }
    if entry.size > 0 {
        if let Ok(meta) = std::fs::metadata(path) {
            return meta.len() as i64 == entry.size;
        }
        return false;
    }
    true
}

/// 构建资源文件的下载 URL 列表
pub fn build_asset_download_urls(
    entry: &AssetEntry,
    mirror_url: Option<&str>,
    source_mode: DownloadSourceMode,
) -> Vec<String> {
    let hash = &entry.hash;
    let prefix = &hash[..2.min(hash.len())];

    let official_url = format!("{}/{}/{}", sources::MOJANG_RESOURCES, prefix, hash);
    let bmclapi_url = format!("{}/assets/{}/{}", sources::BMCLAPI_BASE, prefix, hash);

    match source_mode {
        DownloadSourceMode::Mirror => {
            let mut urls = Vec::new();
            if let Some(mirror) = mirror_url {
                urls.push(format!(
                    "{}/assets/{}/{}",
                    mirror.trim_end_matches('/'),
                    prefix,
                    hash
                ));
            }
            urls.push(bmclapi_url);
            urls
        }
        DownloadSourceMode::Official => {
            vec![official_url]
        }
        DownloadSourceMode::Smart => {
            vec![official_url, bmclapi_url]
        }
    }
}
