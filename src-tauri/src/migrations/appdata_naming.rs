//! AppData 根目录命名统一迁移：`.MolaLaunch` → `.Molaunch`
//!
//! 历史命名不一致：早期 `personalization.rs`/`online/storage.rs`/`auth/storage` 误用
//! `.MolaLaunch`（多了一个 La），现统一为 `.Molaunch`。
//!
//! 迁移策略：
//! 1. 旧路径不存在 → 跳过
//! 2. 新路径已存在且非空 → 跳过（用户已有新路径数据，仅删除旧路径）
//! 3. 旧路径存在且新路径为空 → 递归复制旧路径内容到新路径，复制成功后删除旧路径
//! 4. 复制失败 → 保留旧路径，记录 WARN（下次启动再次尝试）
//!
//! Windows 文件系统不区分大小写，`.Molaunch` 与 `.MolaLaunch` 实为同一目录，
//! 迁移主要影响 macOS/Linux。Windows 上通过 canonicalize 比较检测同目录后 no-op。

use std::path::PathBuf;

use crate::log_info;
use crate::log_warn;
use crate::storage::appdata::appdata_root;

use super::{copy_dir_recursive, dir_is_non_empty};

/// 一次性迁移旧 AppData 根目录 `.MolaLaunch` → `.Molaunch`
pub fn migrate() {
    let new_root = match appdata_root() {
        Ok(p) => p,
        Err(_) => return,
    };

    // 解析旧路径（.MolaLaunch）
    let legacy_root = {
        #[cfg(windows)]
        {
            let appdata = match std::env::var("APPDATA") {
                Ok(v) => v,
                Err(_) => return,
            };
            PathBuf::from(appdata).join(".MolaLaunch")
        }

        #[cfg(not(windows))]
        {
            let home = match std::env::var("HOME") {
                Ok(v) => v,
                Err(_) => return,
            };
            PathBuf::from(home).join(".config").join("MolaLaunch")
        }
    };

    // Windows 大小写不敏感：旧路径与新路径实为同一目录，无需迁移
    // 用 canonicalize 比较是否指向同一目录
    if let (Ok(new_canon), Ok(legacy_canon)) = (new_root.canonicalize(), legacy_root.canonicalize())
    {
        if new_canon == legacy_canon {
            return;
        }
    }

    if !legacy_root.exists() {
        return;
    }

    // 新路径已存在且非空 → 用户已有新路径数据，仅删除旧路径
    if new_root.exists() && dir_is_non_empty(&new_root) {
        log_info!(
            "[Migrations] appdata_naming 跳过：新路径已有数据，删除旧路径 {}",
            legacy_root.display()
        );
        if let Err(e) = std::fs::remove_dir_all(&legacy_root) {
            log_warn!(
                "[Migrations] appdata_naming 删除旧路径 .MolaLaunch 失败（下次启动会再次尝试）: {}",
                e
            );
        }
        return;
    }

    // 递归复制旧路径 → 新路径
    log_info!(
        "[Migrations] appdata_naming 迁移旧 AppData 根目录: {} → {}",
        legacy_root.display(),
        new_root.display()
    );

    if let Err(e) = copy_dir_recursive(&legacy_root, &new_root) {
        log_warn!(
            "[Migrations] appdata_naming 迁移失败（复制失败），保留旧目录: {}",
            e
        );
        return;
    }

    // 复制成功，删除旧路径
    if let Err(e) = std::fs::remove_dir_all(&legacy_root) {
        log_warn!(
            "[Migrations] appdata_naming 迁移成功但旧路径删除失败（下次启动会再次尝试）: {}",
            e
        );
    }

    log_info!("[Migrations] appdata_naming 旧 AppData 根目录迁移完成");
}
