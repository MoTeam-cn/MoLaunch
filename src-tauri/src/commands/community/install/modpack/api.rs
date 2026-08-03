//! 整合包安装入口逻辑：重复安装 Guard / 本地整合包预览

use once_cell::sync::Lazy;
use std::collections::HashSet;
use std::sync::Mutex;

use crate::log_info;

use super::super::concurrent;
use super::super::modpack_stages::{extract_optional_mods, parse_modpack_info};
use super::super::types::ModpackPreview;

/// 当前正在安装的整合包实例名集合（用于重复任务检查）
///
/// InstallGuard 在入口 acquire，函数返回时通过 Drop 自动释放，无需手动清理。
static INSTALLING_INSTANCES: Lazy<Mutex<HashSet<String>>> =
    Lazy::new(|| Mutex::new(HashSet::new()));

/// 整合包安装占用 guard：构造时插入实例名，Drop 时自动移除
///
/// 用法：`let _guard = InstallGuard::acquire(&req.instance_name)?;`
pub struct InstallGuard {
    name: String,
}

impl InstallGuard {
    pub(super) fn acquire(name: &str) -> Result<Self, String> {
        let mut set = INSTALLING_INSTANCES.lock().unwrap();
        if set.contains(name) {
            return Err(format!(
                "整合包 \"{}\" 正在安装中，请等待当前安装完成或取消后再试",
                name
            ));
        }
        set.insert(name.to_string());
        Ok(InstallGuard {
            name: name.to_string(),
        })
    }
}

impl Drop for InstallGuard {
    fn drop(&mut self) {
        INSTALLING_INSTANCES.lock().unwrap().remove(&self.name);
    }
}

/// 预览本地整合包（拖拽安装前置步骤）
///
/// 仅打开 zip + 检测格式 + 解析 manifest/index，不下载、不复制 overrides。
/// 返回整合包基本信息 + 可选 Mod 列表，前端据弹窗询问用户是否下载可选 Mod。
pub async fn preview_local_modpack(file_path: String) -> Result<ModpackPreview, String> {
    log_info!("[Community] 预览本地整合包: {}", file_path);

    super::super::helpers::validate_modpack_extension(&file_path)?;

    let archive_path = std::path::PathBuf::from(&file_path);
    if !archive_path.exists() {
        return Err(format!("整合包文件不存在: {}", file_path));
    }

    let file = std::fs::File::open(&archive_path).map_err(|e| format!("打开整合包失败: {}", e))?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| format!("解析 zip 失败: {}（可能不是有效的整合包）", e))?;

    let detected = concurrent::detect_modpack_format(&mut archive)?;
    let info = parse_modpack_info(&detected)?;

    let optional_mods = extract_optional_mods(&info);

    log_info!(
        "[Community] 预览完成: format={:?} game={} loader={}{} optional_mods={}",
        info.format,
        info.game_version,
        info.loader,
        if info.loader_version.is_empty() {
            String::new()
        } else {
            format!("@{}", info.loader_version)
        },
        optional_mods.len()
    );

    Ok(ModpackPreview {
        format: info.format,
        game_version: info.game_version,
        loader: info.loader,
        loader_version: info.loader_version,
        optional_mods,
    })
}