//! online/device.json 旧路径（便携式）→ AppData 迁移

use std::path::Path;
use std::path::PathBuf;

use crate::log_info;
use crate::log_warn;
use crate::storage::appdata::appdata_subdir;
use crate::storage::Storage;

/// 旧路径设备凭证文件在便携式目录下的相对路径
const LEGACY_DEVICE_FILE: &str = "online/device.json";

/// 解析设备凭证旧存储路径（便携式 `<exe_dir>/.Molaunch/online/device.json`）
///
/// 供 `OnlineStorage::save/clear` 清理旧路径残留时复用，确保路径口径与启动迁移一致。
pub(crate) fn legacy_device_path() -> PathBuf {
    Storage::instance().base_dir().join(LEGACY_DEVICE_FILE)
}

/// 迁移 online/device.json 从便携式旧路径到 AppData
///
/// 仅文件搬移（原样转写，不解密/不加密）。迁移成功后删除整个旧 `online/` 目录。
/// 失败不阻塞启动（仅记录 WARN，下次启动再次尝试）。
pub fn migrate() {
    let legacy_path = legacy_device_path();

    // 1. 旧 device.json 存在 → 迁移到 AppData
    if legacy_path.exists() {
        let new_path = match appdata_subdir("online") {
            Ok(dir) => dir.join("device.json"),
            Err(e) => {
                log_warn!(
                    "[Migrations] online_legacy 跳过：解析 AppData 路径失败: {}",
                    e
                );
                return;
            }
        };

        if new_path.exists() {
            // 新路径已存在 → 已迁移过，仅清理旧目录
            log_info!(
                "[Migrations] online_legacy 跳过：新路径已存在 {}，清理旧目录",
                new_path.display()
            );
        } else {
            log_info!(
                "[Migrations] online_legacy 迁移 device.json: {} → {}",
                legacy_path.display(),
                new_path.display()
            );

            // 读取旧文件原内容（不解密，原样转写到新路径）
            let raw = match std::fs::read_to_string(&legacy_path) {
                Ok(s) => s,
                Err(e) => {
                    log_warn!(
                        "[Migrations] online_legacy 读取旧文件失败（保留旧目录）: {}",
                        e
                    );
                    return;
                }
            };

            // 写入新路径（确保父目录存在）
            if let Some(parent) = new_path.parent() {
                if let Err(e) = std::fs::create_dir_all(parent) {
                    log_warn!(
                        "[Migrations] online_legacy 创建新路径父目录失败（保留旧目录）: {}",
                        e
                    );
                    return;
                }
            }
            if let Err(e) = std::fs::write(&new_path, &raw) {
                log_warn!(
                    "[Migrations] online_legacy 写入新路径失败（保留旧目录）: {}",
                    e
                );
                return;
            }

            log_info!("[Migrations] online_legacy 迁移完成");
        }
    }

    // 2. 清理整个旧 online/ 目录（device.json 已迁移或不存在时清理残留）
    cleanup_legacy_dir(&legacy_path);
}

/// 清理旧 online 目录（device.json 所在目录）
fn cleanup_legacy_dir(legacy_file: &Path) {
    if let Some(dir) = legacy_file.parent() {
        if dir.exists() {
            if let Err(e) = std::fs::remove_dir_all(dir) {
                log_warn!(
                    "[Migrations] online_legacy 旧目录删除失败（下次启动会再次尝试）: {}",
                    e
                );
            }
        }
    }
}
