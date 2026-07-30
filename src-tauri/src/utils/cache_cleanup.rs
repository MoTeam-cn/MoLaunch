//! 缓存定期清理模块
//!
//! 自动清理超过 24h 的不重要缓存文件（images / forge_installer / preload_mods /
//! launch / custom_layout / TaskTemp）。SDK 动态库和 Java Runtime 不清理。
//! 启动时执行一次，之后每 1h 检查一次。

use std::path::Path;
use std::time::{Duration, SystemTime};

use crate::utils::{cache, cache_temp, format};
use crate::{log_info, log_warn};

/// 文件最大保留时长（24 小时）
const MAX_AGE: Duration = Duration::from_secs(24 * 60 * 60);
/// 定时清理间隔（1 小时）
const CLEANUP_INTERVAL: Duration = Duration::from_secs(60 * 60);

/// 清理结果统计
#[derive(Debug, Default)]
struct CleanupResult {
    /// 删除的文件数
    removed_files: u64,
    /// 删除的目录数
    removed_dirs: u64,
    /// 释放的字节数
    freed_bytes: u64,
    /// 遇到的错误数
    errors: u64,
}

impl CleanupResult {
    fn merge(&mut self, other: CleanupResult) {
        self.removed_files += other.removed_files;
        self.removed_dirs += other.removed_dirs;
        self.freed_bytes += other.freed_bytes;
        self.errors += other.errors;
    }

    fn is_empty(&self) -> bool {
        self.removed_files == 0 && self.removed_dirs == 0 && self.errors == 0
    }
}

/// 执行一次完整的缓存清理
///
/// 遍历所有需要清理的缓存目录，删除 mtime 超过 24h 的文件和空目录。
/// 此函数是同步阻塞的，调用方应在 `spawn_blocking` 或独立线程中调用。
pub fn run_cleanup() {
    let start = std::time::Instant::now();
    let mut total = CleanupResult::default();

    // 1. .Molaunch/cache/images/（图片缓存）
    total.merge(cleanup_dir(&cache::path("images"), MAX_AGE));

    // 2. .Molaunch/cache/forge_installer/（Forge 安装器注入资源）
    total.merge(cleanup_dir(&cache::path("forge_installer"), MAX_AGE));

    // 3. .Molaunch/cache/preload_mods/（社区资源预加载缓存）
    total.merge(cleanup_dir(&cache::path("preload_mods"), MAX_AGE));

    // 4. .Molaunch/cache/launch/（嵌入 jar 释放）
    total.merge(cleanup_dir(&cache::path("launch"), MAX_AGE));

    // 5. .Molaunch/cache/custom_layout/（自定义布局 URL 下载缓存）
    total.merge(cleanup_dir(&cache::path("custom_layout"), MAX_AGE));

    // 6. <temp>/MoLaunch/TaskTemp/（安装包临时下载）
    total.merge(cleanup_dir(&cache_temp::task_temp_dir(), MAX_AGE));

    // 不清理：
    // - <temp>/MoLaunch/sdk/（SDK 动态库，有 sha256 校验）
    // - %APPDATA%/.minecraft/runtime/（Java Runtime，重要资源）

    if total.is_empty() {
        log_info!(
            "[CacheCleanup] 无过期文件需要清理（耗时 {}ms）",
            start.elapsed().as_millis()
        );
    } else {
        log_info!(
            "[CacheCleanup] 清理完成：删除 {} 个文件、{} 个目录，释放 {}（{} 个错误，耗时 {}ms）",
            total.removed_files,
            total.removed_dirs,
            format::bytes_with(total.freed_bytes, 2),
            total.errors,
            start.elapsed().as_millis()
        );
    }
}

/// 启动后台定时清理任务
///
/// 立即执行一次清理，然后每 1h 重复执行。
/// 应在 Tauri setup 中调用，任务在 tokio 运行时中异步执行。
pub fn spawn_cleanup_task() {
    tauri::async_runtime::spawn_blocking(|| {
        log_info!("[CacheCleanup] 启动定时清理任务（间隔 {}h）", CLEANUP_INTERVAL.as_secs() / 3600);
        // 启动时立即清理一次
        run_cleanup();

        // 定时循环
        loop {
            std::thread::sleep(CLEANUP_INTERVAL);
            run_cleanup();
        }
    });
}

/// 清理指定目录下超过 max_age 的文件和空目录
///
/// 仅清理一级条目（文件或子目录），不递归进入子目录内部。
/// 子目录本身会作为整体判断 mtime，过期则整个删除。
fn cleanup_dir(dir: &Path, max_age: Duration) -> CleanupResult {
    let mut result = CleanupResult::default();

    if !dir.exists() {
        return result;
    }

    let now = SystemTime::now();
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) => {
            log_warn!("[CacheCleanup] 读取目录失败 {}: {}", dir.display(), e);
            result.errors += 1;
            return result;
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let meta = match std::fs::metadata(&path) {
            Ok(m) => m,
            Err(_) => {
                // 文件可能已被其他进程删除，跳过
                continue;
            }
        };

        // 获取文件修改时间
        let mtime = match meta.modified() {
            Ok(t) => t,
            Err(_) => continue,
        };

        // 计算文件年龄
        let age = match now.duration_since(mtime) {
            Ok(d) => d,
            Err(_) => continue, // mtime 在未来（系统时间异常），跳过
        };

        if age <= max_age {
            continue; // 未过期，保留
        }

        // 过期，执行删除
        let file_size = meta.len();
        if path.is_dir() {
            match std::fs::remove_dir_all(&path) {
                Ok(()) => {
                    result.removed_dirs += 1;
                    result.freed_bytes += file_size;
                }
                Err(e) => {
                    log_warn!("[CacheCleanup] 删除目录失败 {}: {}", path.display(), e);
                    result.errors += 1;
                }
            }
        } else {
            match std::fs::remove_file(&path) {
                Ok(()) => {
                    result.removed_files += 1;
                    result.freed_bytes += file_size;
                }
                Err(e) => {
                    log_warn!("[CacheCleanup] 删除文件失败 {}: {}", path.display(), e);
                    result.errors += 1;
                }
            }
        }
    }

    result
}
