//! 安装产物审计
//!
//! 查两头：该有的有没有、不该有的有没有。前者看 libraries 与 processor 的产出，
//! 后者防安装器在 versions/ 下留自己的目录——那会让版本列表出现两个同名条目。

use std::path::Path;

use serde_json::Value;

use super::{artifact_check, processor_run};

/// 一次审计的结果
#[derive(Debug, Default)]
pub struct AuditReport {
    /// 只能本地生成、但没有的文件
    pub missing_generated: Vec<String>,
    /// 有下载地址、但没落地的普通库
    pub missing_regular: Vec<String>,
    /// 应当由 processor 产出、但没有的文件
    pub missing_outputs: Vec<String>,
    /// versions/ 下安装器留下的临时版本目录
    pub stray_dirs: Vec<String>,
}

impl AuditReport {
    /// 什么都没缺
    pub fn is_clean(&self) -> bool {
        self.missing_generated.is_empty()
            && self.missing_regular.is_empty()
            && self.missing_outputs.is_empty()
    }

    /// 可否自动补：有缺的产物就说明 processor 该重跑
    pub fn needs_processor_rerun(&self) -> bool {
        !self.missing_generated.is_empty() || !self.missing_outputs.is_empty()
    }

    /// 拼一句能直接给用户看的话
    pub fn summary(&self) -> String {
        let mut parts: Vec<String> = Vec::new();
        if !self.missing_generated.is_empty() {
            parts.push(format!(
                "缺少 {} 个本地生成文件",
                self.missing_generated.len()
            ));
        }
        if !self.missing_outputs.is_empty() {
            parts.push(format!(
                "processor 有 {} 个产物没生成",
                self.missing_outputs.len()
            ));
        }
        if !self.missing_regular.is_empty() {
            parts.push(format!("缺少 {} 个普通库文件", self.missing_regular.len()));
        }
        if !self.stray_dirs.is_empty() {
            parts.push(format!("有 {} 个临时版本目录残留", self.stray_dirs.len()));
        }
        if parts.is_empty() {
            return "产物完整".to_string();
        }
        parts.join("；")
    }
}

/// 审计一次安装
///
/// `installer` 为空时跳过 processor 产物核对（Fabric 这类没有 processor 的加载器）。
pub fn audit_install(
    game_dir: &Path,
    json_path: &Path,
    mc_version: &str,
    installer: Option<&Path>,
    loader_keyword: &str,
    keep_versions: &[&str],
) -> Result<AuditReport, String> {
    let text = std::fs::read_to_string(json_path)
        .map_err(|e| format!("读不到版本 JSON（{}）：{}", json_path.display(), e))?;
    let doc: Value = serde_json::from_str(&text)
        .map_err(|e| format!("版本 JSON 解析失败（{}）：{}", json_path.display(), e))?;

    let mut report = AuditReport::default();
    for file in artifact_check::required_library_files(&doc, game_dir) {
        if file.path.is_file() {
            continue;
        }
        let entry = format!("{}（{}）", file.name, file.path.display());
        if file.generated {
            report.missing_generated.push(entry);
        } else {
            report.missing_regular.push(entry);
        }
    }

    if let Some(installer_path) = installer {
        match processor_run::collect_missing_outputs(installer_path, game_dir, mc_version) {
            Ok(missing) => report.missing_outputs = missing,
            Err(e) => {
                crate::log_warn!("[Audit] 核对 processor 产物失败：{}", e);
            }
        }
    }

    report.stray_dirs = find_stray_loader_dirs(game_dir, loader_keyword, keep_versions);
    Ok(report)
}

/// 找出版本目录里由安装器留下的临时目录
///
/// 只认安装器自己的命名习惯（`neoforge-<ver>`、`forge-<ver>`、`fabric-loader-<ver>`），
/// 不用「名字里含关键字」这种宽判据——用户给实例起的名字可能就带加载器名。
pub fn find_stray_loader_dirs(
    game_dir: &Path,
    loader_keyword: &str,
    keep_versions: &[&str],
) -> Vec<String> {
    let versions_dir = game_dir.join("versions");
    let Ok(entries) = std::fs::read_dir(&versions_dir) else {
        return Vec::new();
    };
    let prefix = format!("{}-", loader_keyword.to_ascii_lowercase());
    let mut found = Vec::new();
    for entry in entries.flatten() {
        if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if keep_versions
            .iter()
            .any(|keep| keep.eq_ignore_ascii_case(&name))
        {
            continue;
        }
        if !name.to_ascii_lowercase().starts_with(&prefix) {
            continue;
        }
        // 只认确实是版本目录的（有同名 JSON）
        if entry.path().join(format!("{}.json", name)).is_file() {
            found.push(name);
        }
    }
    found.sort();
    found
}

#[cfg(test)]
#[path = "install_audit_test.rs"]
mod install_audit_test;
