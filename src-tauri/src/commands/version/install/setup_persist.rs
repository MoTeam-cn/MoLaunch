//! 版本安装后的 setup.ini 持久化 + 隔离目录创建
//!
//! 安装完成后记录版本元数据到 setup.ini，
//! 并根据全局隔离设置创建 mods/saves/resourcepacks 等目录。

use crate::minecraft::isolation::{self, IsolationMode};
use crate::minecraft::version::{setup::VersionSetup, state::VersionType};
use crate::state::AppState;
use crate::{log_info, log_warn};

/// 保存 setup.ini + 创建隔离目录
#[allow(clippy::too_many_arguments)]
pub(crate) async fn save_setup_and_create_isolation(
    state: &AppState,
    game_dir: &std::path::Path,
    actual_version_id: &str,
    mc_version: &str,
    version_type: VersionType,
    forge_version: Option<&str>,
    neoforge_version: Option<&str>,
    fabric_version: Option<&str>,
    optifine_version: Option<&str>,
    liteloader_version: Option<&str>,
) {
    let version_dir = game_dir.join("versions").join(actual_version_id);

    // 保存 setup.ini（type 同时写入加载器对应版本，便于后续版本信息查询）
    let setup = VersionSetup::new(
        mc_version,
        version_type,
        forge_version,
        neoforge_version,
        fabric_version,
        None, // quilt 版本未从 install_merged 传入
        optifine_version,
        liteloader_version,
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
