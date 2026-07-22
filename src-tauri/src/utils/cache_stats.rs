//! 缓存统计工具
//!
//! 提供统一的缓存目录统计能力，返回每个子目录的文件数、占用大小、TTL 等信息。
//! 供 IPC 命令 `get_cache_stats` 使用，开发者页和未来可能的外部暴露接口复用。
//!
//! ## 统计范围
//!
//! | 类别 | 子目录 | TTL | 说明 |
//!------|--------|-----|------|
//! | `cache` | images/ | 24h | 图片缓存 |
//! | `cache` | forge_installer/ | 24h | Forge 安装器注入资源 |
//! | `cache` | preload_mods/ | 24h | 社区资源预加载缓存 |
//! | `cache` | launch/ | 24h | 嵌入 jar 释放 |
//! | `cache` | custom_layout/ | 24h | 自定义布局 URL 下载缓存 |
//! | `cache_temp` | TaskTemp/ | 24h | 安装包临时下载 |
//! | `cache_temp` | sdk/ | - | SDK 动态库（不清理，有 sha256 校验） |
//! | `cache_app` | runtime/ | - | Java Runtime（不清理，重要资源） |
//!
//! 注意：`cache_app/runtime/` 统计的是其下的 component 子目录，
//! 因为 runtime/ 本身是 Mojang 官方目录，可能包含其他启动器的内容。

use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::utils::{cache, cache_app, cache_temp};

/// 单个缓存子目录的统计信息
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CacheStat {
    /// 显示名称（如 "图片缓存"）
    pub name: String,
    /// 所属类别（"cache" / "cacheTemp" / "cacheApp"）
    pub category: String,
    /// 子目录相对路径（如 "images" / "TaskTemp" / "runtime"）
    pub sub_dir: String,
    /// 完整路径
    pub path: String,
    /// 文件数量（递归统计）
    pub file_count: u64,
    /// 占用字节数（递归统计）
    pub total_size: u64,
    /// 自动清理 TTL（小时），null 表示不清理
    pub ttl_hours: Option<u64>,
}

/// 缓存统计结果（按类别分组）
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CacheStatsResult {
    /// 运行路径缓存（.Molaunch/cache/）
    pub cache: Vec<CacheStat>,
    /// 系统临时目录缓存（<temp>/MoLaunch/）
    pub cache_temp: Vec<CacheStat>,
    /// AppData 缓存（%APPDATA%/.minecraft/）
    pub cache_app: Vec<CacheStat>,
}

/// 统计所有缓存目录
///
/// 此函数是同步阻塞的，调用方应在 `spawn_blocking` 或独立线程中调用。
pub fn collect_all() -> CacheStatsResult {
    CacheStatsResult {
        cache: collect_cache(),
        cache_temp: collect_cache_temp(),
        cache_app: collect_cache_app(),
    }
}

/// 统计 `.Molaunch/cache/` 下的子目录
fn collect_cache() -> Vec<CacheStat> {
    let ttl = Some(24);
    vec![
        CacheStat {
            name: "图片缓存".to_string(),
            category: "cache".to_string(),
            sub_dir: "images".to_string(),
            path: cache::path("images").to_string_lossy().to_string(),
            ..stat_dir(&cache::path("images"), ttl)
        },
        CacheStat {
            name: "Forge 安装器".to_string(),
            category: "cache".to_string(),
            sub_dir: "forge_installer".to_string(),
            path: cache::path("forge_installer").to_string_lossy().to_string(),
            ..stat_dir(&cache::path("forge_installer"), ttl)
        },
        CacheStat {
            name: "社区资源预加载".to_string(),
            category: "cache".to_string(),
            sub_dir: "preload_mods".to_string(),
            path: cache::path("preload_mods").to_string_lossy().to_string(),
            ..stat_dir(&cache::path("preload_mods"), ttl)
        },
        CacheStat {
            name: "嵌入 jar 释放".to_string(),
            category: "cache".to_string(),
            sub_dir: "launch".to_string(),
            path: cache::path("launch").to_string_lossy().to_string(),
            ..stat_dir(&cache::path("launch"), ttl)
        },
        CacheStat {
            name: "自定义布局缓存".to_string(),
            category: "cache".to_string(),
            sub_dir: "custom_layout".to_string(),
            path: cache::path("custom_layout").to_string_lossy().to_string(),
            ..stat_dir(&cache::path("custom_layout"), ttl)
        },
    ]
}

/// 统计 `<temp>/MoLaunch/` 下的子目录
fn collect_cache_temp() -> Vec<CacheStat> {
    vec![
        CacheStat {
            name: "安装包临时下载".to_string(),
            category: "cacheTemp".to_string(),
            sub_dir: "TaskTemp".to_string(),
            path: cache_temp::task_temp_dir().to_string_lossy().to_string(),
            ..stat_dir(&cache_temp::task_temp_dir(), Some(24))
        },
        CacheStat {
            name: "SDK 动态库".to_string(),
            category: "cacheTemp".to_string(),
            sub_dir: "sdk".to_string(),
            path: cache_temp::sdk_dir().to_string_lossy().to_string(),
            ..stat_dir(&cache_temp::sdk_dir(), None)
        },
    ]
}

/// 统计 `%APPDATA%/.minecraft/` 下的子目录
fn collect_cache_app() -> Vec<CacheStat> {
    // runtime/ 目录下每个 component 单独统计
    let runtime_base = cache_app::runtime_base_dir();
    let mut stats = Vec::new();

    if runtime_base.exists() {
        if let Ok(entries) = std::fs::read_dir(&runtime_base) {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }
                let name = match path.file_name().and_then(|n| n.to_str()) {
                    Some(s) => s.to_string(),
                    None => continue,
                };
                stats.push(CacheStat {
                    name: format!("Java Runtime · {}", name),
                    category: "cacheApp".to_string(),
                    sub_dir: format!("runtime/{}", name),
                    path: path.to_string_lossy().to_string(),
                    ..stat_dir(&path, None)
                });
            }
        }
    }

    // 若 runtime/ 不存在或为空，仍返回一个占位项便于 UI 展示路径
    if stats.is_empty() {
        stats.push(CacheStat {
            name: "Java Runtime".to_string(),
            category: "cacheApp".to_string(),
            sub_dir: "runtime".to_string(),
            path: runtime_base.to_string_lossy().to_string(),
            file_count: 0,
            total_size: 0,
            ttl_hours: None,
        });
    }

    stats
}

/// 递归统计目录的文件数和总大小
///
/// 目录不存在时返回零值。
fn stat_dir(path: &Path, ttl_hours: Option<u64>) -> CacheStat {
    let mut file_count = 0u64;
    let mut total_size = 0u64;

    if path.exists() {
        walk_dir(path, &mut file_count, &mut total_size);
    }

    CacheStat {
        name: String::new(),
        category: String::new(),
        sub_dir: String::new(),
        path: String::new(),
        file_count,
        total_size,
        ttl_hours,
    }
}

/// 递归遍历目录，累加文件数和大小
fn walk_dir(dir: &Path, file_count: &mut u64, total_size: &mut u64) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let meta = match std::fs::metadata(&path) {
            Ok(m) => m,
            Err(_) => continue,
        };

        if meta.is_dir() {
            walk_dir(&path, file_count, total_size);
        } else if meta.is_file() {
            *file_count += 1;
            *total_size += meta.len();
        }
    }
}

/// 获取缓存根目录路径列表（供 UI 展示父目录用）
pub fn root_dirs() -> Vec<(String, PathBuf)> {
    vec![
        ("cache".to_string(), cache::dir()),
        ("cacheTemp".to_string(), cache_temp::dir()),
        ("cacheApp".to_string(), cache_app::dir()),
    ]
}
