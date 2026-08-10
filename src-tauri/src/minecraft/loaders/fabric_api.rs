//! Fabric API 自动补充模块
//!
//! 从 Modrinth（project_id = P7dR8mSH）获取 fabric-api 版本列表，按 MC 版本筛选并自动下载到 mods 目录

use std::path::Path;
use std::sync::Arc;

use crate::minecraft::community::modrinth;
use crate::minecraft::community::types::{ResourceType, ResourceVersion};
use crate::minecraft::download::config::DownloadManagerConfig;
use crate::minecraft::download::manager::DownloadManager;
use crate::minecraft::download::types::{DownloadStatus, DownloadTask};

/// Fabric API 在 Modrinth 上的 project_id（slug = fabric-api）
const FABRIC_API_PROJECT_ID: &str = "P7dR8mSH";

/// Fabric API 版本信息（精简版，用于前端展示和后端安装）
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FabricApiVersion {
    /// Modrinth version ID
    pub version_id: String,
    /// 版本号（如 0.92.2+1.20.4）
    pub version_number: String,
    /// 显示名（如 Fabric API 0.92.2+1.20.4）
    pub display_name: String,
    /// 支持的 MC 版本列表
    pub game_versions: Vec<String>,
    /// 发布日期（ISO 8601）
    pub release_date: String,
    /// 下载 URL
    pub download_url: String,
    /// 文件名（如 fabric-api-0.92.2+1.20.4.jar）
    pub file_name: String,
    /// 文件大小（字节）
    pub size: u64,
    /// SHA1 哈希
    pub hash: Option<String>,
}

/// 列出与指定 MC 版本兼容的 Fabric API 版本
///
/// 流程：
/// 1. 从 Modrinth 获取 fabric-api 全部版本列表
/// 2. 用 is_compatible 筛选兼容版本
/// 3. 按 ReleaseDate 降序排序（最新在前）
pub async fn list_versions(mc_version: &str) -> Result<Vec<FabricApiVersion>, String> {
    crate::log_info!("[FabricAPI] 查询兼容 MC {} 的 Fabric API 版本", mc_version);

    let versions = modrinth::get_versions(FABRIC_API_PROJECT_ID, ResourceType::Mod).await?;

    let mut compatible: Vec<FabricApiVersion> = versions
        .iter()
        .filter(|v| is_compatible(v, mc_version))
        .map(|v| FabricApiVersion {
            version_id: v.id.clone(),
            version_number: v.version.clone(),
            display_name: v.display.clone(),
            game_versions: v.game_versions.clone(),
            release_date: v.release_date.clone(),
            download_url: v.download_url.clone(),
            file_name: v.file_name.clone(),
            size: v.size,
            hash: v.hash.clone(),
        })
        .collect();

    // 按发布日期降序排序（最新在前）
    compatible.sort_by(|a, b| b.release_date.cmp(&a.release_date));

    crate::log_info!(
        "[FabricAPI] 找到 {} 个兼容 MC {} 的 Fabric API 版本",
        compatible.len(),
        mc_version
    );

    Ok(compatible)
}

/// 检查 Fabric API 版本是否与指定 MC 版本兼容
///
/// 使用 Modrinth API 的 game_versions 字段精确匹配
fn is_compatible(version: &ResourceVersion, mc_version: &str) -> bool {
    // 精确匹配 game_versions 列表
    if version.game_versions.iter().any(|gv| gv == mc_version) {
        return true;
    }
    // 字符串匹配逻辑（作为 fallback）
    // 某些 Fabric API 版本可能用 "1.20.4/1.20.5" 这样的格式
    let display_l = version.display.to_lowercase();
    let mc_l = mc_version.to_lowercase();
    if display_l.contains(&format!("[{}]", mc_l)) {
        return true;
    }
    // 检查显示名中 [x/y] 格式是否包含目标版本
    if let Some(bracket_start) = display_l.find('[') {
        if let Some(bracket_end) = display_l[bracket_start..].find(']') {
            let bracket_content = &display_l[bracket_start + 1..bracket_start + bracket_end];
            for part in bracket_content.split('/') {
                if part.trim() == mc_l {
                    return true;
                }
            }
        }
    }
    false
}

/// 安装 Fabric API 到指定 mods 目录
///
/// - 作为 DownloadReason.Dependency 类型下载
/// - 下载到 mods 目录（考虑版本隔离路径）
pub async fn install(
    download_url: &str,
    file_name: &str,
    mods_dir: &Path,
    hash: Option<&str>,
    config: &DownloadManagerConfig,
    progress_callback: Option<Arc<dyn Fn(f64) + Send + Sync>>,
) -> anyhow::Result<()> {
    if let Some(ref cb) = progress_callback {
        cb(0.0);
    }

    crate::log_info!("[FabricAPI] 安装 {} -> {}", file_name, mods_dir.display());

    // 确保 mods 目录存在
    std::fs::create_dir_all(mods_dir)?;

    // 构建下载 URL 列表（根据 source 策略：0=镜像，1=官方+镜像fallback，2=官方）
    let urls = crate::minecraft::sources::cdn_urls(download_url);

    let local_path = mods_dir.join(file_name);
    let manager = DownloadManager::from_config(config);
    let task = DownloadTask {
        id: "fabric_api".to_string(),
        urls,
        local_path: local_path.to_string_lossy().to_string(),
        expected_size: 0,
        expected_hash: hash.map(|h| h.to_string()),
    };

    let results = manager.download_batch(vec![task], None).await;
    if let Some(result) = results.first() {
        if result.status == DownloadStatus::Failed {
            return Err(anyhow::anyhow!("Failed to download Fabric API"));
        }
    }

    crate::log_info!("[FabricAPI] 安装完成: {}", file_name);

    if let Some(ref cb) = progress_callback {
        cb(100.0);
    }

    Ok(())
}
