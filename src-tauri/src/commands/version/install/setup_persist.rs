//! 版本安装后的 setup.ini 持久化 + 隔离目录创建
//!
//! 安装完成后记录版本元数据到 setup.ini，
//! 并根据全局隔离设置创建 mods/saves/resourcepacks 等目录。

use crate::minecraft::isolation::{self, IsolationMode};
use crate::minecraft::version::{setup::VersionSetup, state::VersionType};
use crate::state::AppState;
use crate::{log_info, log_warn};

/// 保存 setup.ini + 创建隔离目录
pub(crate) async fn save_setup_and_create_isolation(
    state: &AppState,
    game_dir: &std::path::Path,
    actual_version_id: &str,
    mc_version: &str,
    version_type: VersionType,
) {
    let version_dir = game_dir.join("versions").join(actual_version_id);

    // 保存 setup.ini
    let setup = VersionSetup::new(
        mc_version,
        version_type,
        None, // Forge 版本号从目录名或 JSON 中提取
        None,
        None,
        None,
        None,
        None,
    );
    if let Err(e) = setup.save(&version_dir) {
        log_warn!("[Merged] 保存 setup.ini 失败: {}", e);
    } else {
        log_info!("[Merged] 已保存 setup.ini: {}", version_dir.display());
    }

    // 根据版本隔离设置创建隔离目录
    let isolation_mode = state.config.lock().await.isolation_mode;
    let mode = IsolationMode::from_u32(isolation_mode);
    if isolation::should_isolate(mode, version_type) {
        log_info!(
            "[Merged] 创建隔离目录: {} (模式: {}, 类型: {:?})",
            actual_version_id,
            isolation_mode,
            version_type
        );
        let result = if version_type.is_modded() {
            isolation::ensure_modded_dirs(&version_dir)
        } else {
            isolation::ensure_isolated_dirs(&version_dir)
        };
        if let Err(e) = result {
            log_warn!("[Merged] 创建隔离目录失败: {}", e);
        }
    }
}
