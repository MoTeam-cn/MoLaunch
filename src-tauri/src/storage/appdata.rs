//! AppData 全局共享目录辅助模块
//!
//! 集中管理跨启动器实例共享的全局存储路径：
//! - Windows: `%APPDATA%/.MolaLaunch/`
//! - macOS/Linux: `~/.config/MolaLaunch/`
//!
//! 与 `<exe_dir>/.Molaunch/`（便携式、每实例独立）相对，本模块目录下的资源
//! 在同一用户的所有 MoLaunch 启动器实例间共享（如 TLS 证书、frpc 二进制、设备凭证等）。
//!
//! 历史上 `minecraft::online::storage::OnlineStorage::appdata_device_path` 与
//! `minecraft::auth::storage::AuthStorage::storage_path` 各自重复实现了同一套路径逻辑，
//! 现统一抽取到本模块，避免路径约定漂移。

use std::path::PathBuf;

use crate::log_info;
use crate::log_warn;

/// AppData 全局共享根目录
///
/// - Windows: `%APPDATA%/.MolaLaunch/`
/// - macOS/Linux: `~/.config/MolaLaunch/`
///
/// 环境变量缺失时返回 Err（调用方决定降级策略）。父目录不自动创建，
/// 由 [`ensure_appdata_subdir`] 或调用方按需 `create_dir_all`。
pub fn appdata_root() -> Result<PathBuf, String> {
    #[cfg(windows)]
    {
        let appdata = std::env::var("APPDATA")
            .map_err(|_| "APPDATA environment variable not set".to_string())?;
        Ok(PathBuf::from(appdata).join(".MolaLaunch"))
    }

    #[cfg(not(windows))]
    {
        let home =
            std::env::var("HOME").map_err(|_| "HOME environment variable not set".to_string())?;
        Ok(PathBuf::from(home).join(".config").join("MolaLaunch"))
    }
}

/// 解析 AppData 下指定子目录的完整路径（不自动创建）
///
/// 例如 `appdata_subdir("certs")` 返回 `%APPDATA%/.MolaLaunch/certs/`。
pub fn appdata_subdir(subdir: &str) -> Result<PathBuf, String> {
    Ok(appdata_root()?.join(subdir))
}

/// 确保 AppData 下指定子目录存在，返回其完整路径
///
/// 目录已存在则直接返回；不存在则递归创建。创建失败时返回 Err。
pub fn ensure_appdata_subdir(subdir: &str) -> Result<PathBuf, String> {
    let dir = appdata_subdir(subdir)?;
    if !dir.exists() {
        std::fs::create_dir_all(&dir).map_err(|e| format!("创建 AppData 子目录失败: {}", e))?;
        log_info!("[AppData] Created subdir: {}", dir.display());
    }
    Ok(dir)
}

/// 从便携式目录（`<exe_dir>/.Molaunch/<subdir>`）一次性迁移到 AppData 全局目录
///
/// 迁移策略：
/// 1. AppData 子目录已存在且非空 → 跳过迁移（用户已有全局数据，不覆盖）
/// 2. 便携式子目录不存在 → 跳过（无数据可迁移）
/// 3. 便携式子目录存在 → 递归复制到 AppData，复制成功后删除便携式旧目录
/// 4. 复制失败 → 保留便携式目录原样，记录 WARN（下次启动再次尝试）
///
/// 调用时机：启动器初始化阶段（`Storage::init`）调用一次即可。
pub fn migrate_from_portable(subdir: &str) -> Result<(), String> {
    let portable_dir = crate::storage::Storage::instance().base_dir().join(subdir);
    if !portable_dir.exists() {
        // 便携式目录不存在，无数据可迁移
        return Ok(());
    }

    let appdata_dir = appdata_subdir(subdir)?;

    // AppData 目录已存在且非空 → 用户已有全局数据，不覆盖
    if appdata_dir.exists() && dir_is_non_empty(&appdata_dir) {
        log_info!(
            "[AppData] 迁移跳过：{} 已存在全局数据，删除便携式旧目录 {}",
            appdata_dir.display(),
            portable_dir.display()
        );
        // 全局已有数据，便携式旧目录已无用，删除避免下次重复检测
        if let Err(e) = std::fs::remove_dir_all(&portable_dir) {
            log_warn!(
                "[AppData] 删除便携式旧目录失败（下次启动会再次尝试）: {}",
                e
            );
        }
        return Ok(());
    }

    // 确保 AppData 父目录存在
    if let Some(parent) = appdata_dir.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("创建 AppData 父目录失败: {}", e))?;
    }

    log_info!(
        "[AppData] 迁移目录: {} → {}",
        portable_dir.display(),
        appdata_dir.display()
    );

    // 递归复制便携式 → AppData
    if let Err(e) = copy_dir_recursive(&portable_dir, &appdata_dir) {
        log_warn!(
            "[AppData] 迁移失败（复制失败），保留便携式目录: {}",
            e
        );
        return Ok(());
    }

    // 复制成功，删除便携式旧目录
    if let Err(e) = std::fs::remove_dir_all(&portable_dir) {
        log_warn!(
            "[AppData] 迁移成功但旧目录删除失败（下次启动会再次尝试）: {}",
            e
        );
    }

    log_info!("[AppData] 目录迁移完成: {}", subdir);
    Ok(())
}

/// 判断目录是否非空（存在至少一个条目）
fn dir_is_non_empty(dir: &std::path::Path) -> bool {
    std::fs::read_dir(dir)
        .map(|mut it| it.next().is_some())
        .unwrap_or(false)
}

/// 递归复制目录
fn copy_dir_recursive(src: &std::path::Path, dst: &std::path::Path) -> Result<(), String> {
    if !dst.exists() {
        std::fs::create_dir_all(dst).map_err(|e| format!("创建目录失败: {}", e))?;
    }
    for entry in std::fs::read_dir(src).map_err(|e| format!("读取目录失败: {}", e))? {
        let entry = entry.map_err(|e| format!("读取目录项失败: {}", e))?;
        let path = entry.path();
        let file_name = entry.file_name();
        let dst_path = dst.join(&file_name);
        if path.is_dir() {
            copy_dir_recursive(&path, &dst_path)?;
        } else {
            std::fs::copy(&path, &dst_path).map_err(|e| format!("复制文件失败: {}", e))?;
        }
    }
    Ok(())
}
