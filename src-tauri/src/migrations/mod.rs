//! 启动时自动迁移模块
//!
//! 集中管理所有存储迁移逻辑，启动时由 `Storage::init` 调用 `run_all()` 一次性执行。
//! 类似数据库的自动迁移机制：每个迁移函数检测旧数据存在时自动迁移到新位置，
//! 迁移失败不阻塞启动（仅记录 WARN，下次启动再次尝试）。
//!
//! 迁移项：
//! 1. `appdata_naming`：AppData 根目录 .MolaLaunch → .Molaunch 命名统一
//! 2. `portable_to_appdata`：certs/providers/frp_auth 从便携式 .Molaunch 迁移到 AppData 全局共享
//! 3. `online_legacy`：online/device.json 从便携式旧路径迁移到 AppData
//! 4. `online_cleanup`：清理已迁移的 online 残留目录（由 `online_legacy` 内部完成）

pub mod appdata_naming;
pub mod online_legacy;
pub mod portable_to_appdata;

/// 执行所有启动时迁移
///
/// 由 `Storage::init` 调用，按依赖顺序执行：
/// 1. 先执行 appdata_naming（确保新路径就绪）
/// 2. 再执行 portable_to_appdata（依赖新路径）
/// 3. 最后执行 online_legacy + 清理
///
/// 任何迁移失败都不阻塞启动（仅记录 WARN）。
pub fn run_all() {
    appdata_naming::migrate();
    portable_to_appdata::migrate();
    online_legacy::migrate();
}

/// 判断目录是否非空（存在至少一个条目）
pub(super) fn dir_is_non_empty(dir: &std::path::Path) -> bool {
    std::fs::read_dir(dir)
        .map(|mut it| it.next().is_some())
        .unwrap_or(false)
}

/// 递归复制目录
pub(super) fn copy_dir_recursive(src: &std::path::Path, dst: &std::path::Path) -> Result<(), String> {
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
