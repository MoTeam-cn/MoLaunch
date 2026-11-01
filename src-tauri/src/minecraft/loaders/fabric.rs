//! Fabric loader module

use std::path::Path;
use std::sync::Arc;

use super::LoaderVersion;
use crate::minecraft::download::config::DownloadManagerConfig;
use crate::minecraft::download::manager::DownloadManager;
use crate::minecraft::download::types::{DownloadStatus, DownloadTask};
use crate::minecraft::sources::{self, DownloadSourceMode};

/// List Fabric versions
pub async fn list_versions(
    mirror_url: Option<&str>,
    source_mode: DownloadSourceMode,
) -> anyhow::Result<Vec<LoaderVersion>> {
    let urls = sources::build_urls(
        mirror_url,
        sources::FABRIC_META,
        sources::BMCLAPI_FABRIC_META,
        source_mode,
    );
    let content = sources::fetch_with_race(&urls).await?;
    let json: serde_json::Value = serde_json::from_str(&content)?;

    let mut versions = Vec::new();
    if let Some(versions_array) = json.as_array() {
        for version in versions_array {
            if let Some(version_str) = version["version"].as_str() {
                versions.push(LoaderVersion {
                    version: version_str.to_string(),
                    is_recommended: version["stable"].as_bool().unwrap_or(false),
                    release_time: None,
                });
            }
        }
    }

    Ok(versions)
}

/// Install Fabric
///
/// 进度由 `loader_helpers::start_progress_ticker` 统一管理（对数曲线伪进度），
/// 此函数内部不需要手写 progress_callback 回调。
pub async fn install(
    mc_version: &str,
    fabric_version: &str,
    game_dir: &Path,
    mirror_url: Option<&str>,
    progress_callback: Option<Arc<dyn Fn(f64) + Send + Sync>>,
    config: &DownloadManagerConfig,
) -> anyhow::Result<()> {
    crate::log_info!(
        "[Fabric] Installing {} for MC {}",
        fabric_version,
        mc_version
    );

    let version_id = format!("fabric-{}-{}", fabric_version, mc_version);
    let version_dir = game_dir.join("versions").join(&version_id);
    let mut record = install_log::InstallLog::begin(
        "Fabric",
        &version_id,
        mc_version,
        "(不需要 Java)",
        game_dir,
    );
    let report =
        |cb: &Option<Arc<dyn Fn(f64) + Send + Sync>>, stage: install_flow::InstallStage, f: f64| {
            if let Some(handler) = cb {
                handler(stage.at(f));
            }
        };
    record.stage(install_flow::InstallStage::Prepare);
    std::fs::create_dir_all(&version_dir)?;

    let url = sources::fabric_profile_url(mc_version, fabric_version);

    let urls = match mirror_url {
        Some(mirror) if !mirror.is_empty() => vec![
            format!(
                "{}/fabric-meta/v2/versions/loader/{}/{}/profile/json",
                mirror.trim_end_matches('/'),
                mc_version,
                fabric_version
            ),
            format!(
                "{}/fabric-meta/v2/versions/loader/{}/{}/profile/json",
                sources::BMCLAPI_BASE,
                mc_version,
                fabric_version
            ),
            url,
        ],
        _ => vec![
            format!(
                "{}/fabric-meta/v2/versions/loader/{}/{}/profile/json",
                sources::BMCLAPI_BASE,
                mc_version,
                fabric_version
            ),
            url,
        ],
    };

    let manager = DownloadManager::from_config(config);
    let task = DownloadTask {
        id: "fabric_profile".to_string(),
        urls,
        local_path: version_dir
            .join(format!("{}.json", version_id))
            .to_string_lossy()
            .to_string(),
        expected_size: 0,
        expected_hash: None,
    };

    record.stage(install_flow::InstallStage::FetchInstaller);
    report(
        &progress_callback,
        install_flow::InstallStage::FetchInstaller,
        0.0,
    );
    let results = manager.download_batch(vec![task], None).await;
    if let Some(result) = results.first() {
        if result.status == DownloadStatus::Failed {
            record.problem("下载 Fabric profile 失败");
            let path = record.finish(false, "下载 Fabric profile 失败");
            return Err(anyhow::anyhow!(
                "Failed to download Fabric profile{}",
                install_log::hint(path)
            ));
        }
    }

    // 验证 profile JSON 已写入
    let profile_path = version_dir.join(format!("{}.json", version_id));
    if !profile_path.exists() {
        return Err(anyhow::anyhow!(
            "Fabric profile JSON not found after download"
        ));
    }

    // profile 是 JSON：解析失败说明内容被截断或损坏（例如服务端按压缩长度切分 Range），
    // 此时宁可报错也不能写进 versions 目录让启动阶段莫名其妙失败
    let profile_text = std::fs::read_to_string(&profile_path)?;
    if let Err(e) = serde_json::from_str::<serde_json::Value>(&profile_text) {
        let _ = std::fs::remove_file(&profile_path);
        record.problem(format!("profile JSON 解析失败：{}", e));
        let path = record.finish(false, "profile JSON 损坏");
        return Err(anyhow::anyhow!(
            "Fabric profile JSON 解析失败：{}{}",
            e,
            install_log::hint(path)
        ));
    }
    report(
        &progress_callback,
        install_flow::InstallStage::FetchInstaller,
        1.0,
    );

    // Fabric 没有 processor，也没有安装器，审计只核对版本 JSON 声明的库
    record.stage(install_flow::InstallStage::AuditArtifacts);
    match install_audit::audit_install(
        game_dir,
        &profile_path,
        mc_version,
        None,
        "fabric",
        &[version_id.as_str()],
    ) {
        Ok(audit) => {
            record.note(audit.summary());
            for stray in &audit.stray_dirs {
                record.problem(format!("版本目录残留：{}", stray));
            }
            if !audit.missing_generated.is_empty() || !audit.missing_regular.is_empty() {
                record.problem("有声明的库文件缺失，启动时会自动补下载");
            }
        }
        Err(e) => record.problem(format!("审计失败：{}", e)),
    }

    record.stage(install_flow::InstallStage::WrapUp);
    let path = record.finish(true, "Fabric 安装完成");
    if let Some(p) = path {
        crate::log_info!("[Fabric] 安装记录：{}", p.display());
    }

    crate::log_info!("[Fabric] Installed: {}", version_id);

    Ok(())
}
