//! 便携式目录 → AppData 全局共享目录迁移
//!
//! 将设备级资源（certs/providers）从便携式 `<exe_dir>/.Molaunch/<subdir>` 迁移到
//! AppData 全局目录 `%APPDATA%/.Molaunch/<subdir>`，实现多启动器实例共享。
//!
//! 迁移策略：
//! 1. AppData 子目录已存在且非空 → 跳过（用户已有全局数据，删除便携式旧目录）
//! 2. 便携式子目录不存在 → 跳过
//! 3. 便携式子目录存在 → 递归复制到 AppData，成功后删除便携式旧目录
//! 4. 复制失败 → 保留便携式目录，记录 WARN（下次启动再次尝试）

use crate::log_info;
use crate::log_warn;
use crate::storage::appdata::appdata_subdir;
use crate::storage::Storage;

use super::{copy_dir_recursive, dir_is_non_empty};

/// 执行 certs 和 providers 两个子目录的便携式 → AppData 迁移
pub fn migrate() {
    if let Err(e) = migrate_subdir("certs") {
        log_warn!("[Migrations] portable_to_appdata certs 迁移失败: {}", e);
    }
    if let Err(e) = migrate_subdir("providers") {
        log_warn!(
            "[Migrations] portable_to_appdata providers 迁移失败: {}",
            e
        );
    }
}

/// 从便携式目录迁移单个子目录到 AppData
fn migrate_subdir(subdir: &str) -> Result<(), String> {
    let portable_dir = Storage::instance().base_dir().join(subdir);
    if !portable_dir.exists() {
        // 便携式目录不存在，无数据可迁移
        return Ok(());
    }

    let appdata_dir = appdata_subdir(subdir)?;

    // AppData 目录已存在且非空 → 用户已有全局数据，不覆盖
    if appdata_dir.exists() && dir_is_non_empty(&appdata_dir) {
        log_info!(
            "[Migrations] portable_to_appdata {} 跳过：AppData 已存在全局数据，删除便携式旧目录 {}",
            subdir,
            portable_dir.display()
        );
        // 全局已有数据，便携式旧目录已无用，删除避免下次重复检测
        if let Err(e) = std::fs::remove_dir_all(&portable_dir) {
            log_warn!(
                "[Migrations] portable_to_appdata 删除便携式旧目录失败（下次启动会再次尝试）: {}",
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
        "[Migrations] portable_to_appdata 迁移目录: {} → {}",
        portable_dir.display(),
        appdata_dir.display()
    );

    // 递归复制便携式 → AppData
    if let Err(e) = copy_dir_recursive(&portable_dir, &appdata_dir) {
        log_warn!(
            "[Migrations] portable_to_appdata 迁移失败（复制失败），保留便携式目录: {}",
            e
        );
        return Ok(());
    }

    // 复制成功，删除便携式旧目录
    if let Err(e) = std::fs::remove_dir_all(&portable_dir) {
        log_warn!(
            "[Migrations] portable_to_appdata 迁移成功但旧目录删除失败（下次启动会再次尝试）: {}",
            e
        );
    }

    log_info!(
        "[Migrations] portable_to_appdata 目录迁移完成: {}",
        subdir
    );
    Ok(())
}
