//! 任务准备：解包到缓存工作区并分析，支持断点续传复用

use std::path::{Path, PathBuf};

use crate::storage::cache::Cache;
use crate::{log_info, log_warn};

use super::super::analyze;
use super::super::jar;
use super::super::resume::{self, Checkpoint};
use super::super::types::JarInspection;

/// 缓存工作区根目录（`.Molaunch/cache/mod-translation`）
const WORKSPACE_ROOT: &str = "mod-translation";
/// 续传身份标识（写入/匹配 resume marker 用）
const RESUME_IDENTITY: &str = "mod-translator-v1";

/// 已就绪的分析结果（analyze 与 start 之间传递工作区）
#[derive(Clone)]
pub(crate) struct Prepared {
    pub workspace: PathBuf,
    pub inspection: JarInspection,
    /// 断点续传检查点（复用续传工作区时存在）
    pub checkpoint: Option<Checkpoint>,
}

/// 解包到缓存工作区并分析；优先复用可续传工作区，否则新建 `job-<hash>` 并写续传标记
pub(crate) fn prepare(jar_path: &Path) -> Result<Prepared, String> {
    let hash = analyze::file_hash(jar_path)?;
    let root = Cache::instance()
        .ensure_dir(WORKSPACE_ROOT)
        .map_err(|e| format!("无法创建模组翻译缓存目录: {e}"))?;

    // 断点续传：命中同 hash + 身份的工作区则复用（跳过已翻译批次）
    if let Some(workspace) = resume::find_resumable_workspace(&root, &hash, RESUME_IDENTITY) {
        if let Some(marker) = resume::read_resume_marker(&workspace) {
            log_info!("[ModTranslation] 发现可续传工作区，跳过已翻译批次");
            return Ok(Prepared {
                checkpoint: resume::read_checkpoint(&workspace),
                workspace,
                inspection: marker.inspection,
            });
        }
    }

    let workspace = root.join(format!("job-{}", &hash[..16]));
    if workspace.exists() {
        std::fs::remove_dir_all(&workspace).map_err(|e| format!("无法清理旧工作区: {e}"))?;
    }
    std::fs::create_dir_all(&workspace).map_err(|e| format!("无法创建工作区: {e}"))?;

    let extracted = jar::extract_archive(jar_path, &workspace, &jar::ExtractionLimits::default())?;
    if extracted.signed {
        log_warn!("[ModTranslation] JAR 含签名文件，重打包后签名将失效");
    }
    let inspection = analyze::inspect_jar(&workspace, jar_path, extracted.signed);
    log_info!(
        "[ModTranslation] 分析完成：{}（{} 个语言源，{} 条目）",
        inspection.original_filename,
        inspection.language_sources.len(),
        inspection.language_entries
    );
    let _ = resume::write_resume_marker(
        &workspace,
        &resume::ResumeMarker {
            version: 1,
            input_hash: hash,
            resume_identity: RESUME_IDENTITY.to_string(),
            created_at: chrono::Local::now().to_rfc3339(),
            inspection: inspection.clone(),
        },
    );
    Ok(Prepared {
        checkpoint: None,
        workspace,
        inspection,
    })
}
