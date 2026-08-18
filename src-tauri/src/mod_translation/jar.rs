//! 模组翻译：JAR 安全解包与重打包

use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use zip::write::SimpleFileOptions;
use zip::ZipArchive;

/// 解包时的资源上限，防止恶意压缩包打爆磁盘/内存
#[derive(Debug, Clone)]
pub struct ExtractionLimits {
    pub max_archive_bytes: u64,
    pub max_entries: usize,
    pub max_entry_bytes: u64,
    pub max_uncompressed_bytes: u64,
    pub max_compression_ratio: f64,
}

impl Default for ExtractionLimits {
    fn default() -> Self {
        Self {
            max_archive_bytes: 1_200 * 1024 * 1024,
            max_entries: 200_000,
            max_entry_bytes: 256 * 1024 * 1024,
            max_uncompressed_bytes: 2_000 * 1024 * 1024,
            max_compression_ratio: 200.0,
        }
    }
}

pub const ARCHIVE_MANIFEST: &str = ".mod-translator-archive-manifest.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveManifest {
    pub version: u32,
    pub entries: Vec<ArchiveEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveEntry {
    pub archive_path: String,
    pub workspace_path: String,
}

impl ArchiveManifest {
    pub fn read(directory: &Path) -> Option<Self> {
        let parsed: ArchiveManifest =
            serde_json::from_str(&std::fs::read_to_string(directory.join(ARCHIVE_MANIFEST)).ok()?)
                .ok()?;
        (parsed.version == 1).then_some(parsed)
    }

    pub fn write(&self, directory: &Path) -> std::io::Result<()> {
        let content = serde_json::to_string_pretty(self).unwrap_or_default();
        std::fs::write(directory.join(ARCHIVE_MANIFEST), content)
    }
}

/// 归档路径安全策略
pub struct PathPolicy;

impl PathPolicy {
    /// 规范化条目名并拒绝一切可能逃出工作区的写法
    pub fn safe_entry_name(raw: &str) -> Result<String, String> {
        let name = raw.replace('\\', "/");
        if name.contains('\0')
            || name.starts_with('/')
            || name.starts_with("//")
            || looks_like_drive_prefix(&name)
            || name.split('/').any(|segment| segment == "..")
        {
            return Err(format!("archive contains an unsafe path: {raw}"));
        }
        Ok(name)
    }

    /// 签名文件判定（META-INF/*.SF/.RSA/.DSA/.EC）
    pub fn is_signature_file(name: &str) -> bool {
        let lower = name.to_ascii_lowercase();
        lower.starts_with("meta-inf/")
            && lower
                .rsplit_once('/')
                .map(|(_, tail)| {
                    tail.ends_with(".sf")
                        || tail.ends_with(".rsa")
                        || tail.ends_with(".dsa")
                        || tail.ends_with(".ec")
                })
                .unwrap_or(false)
    }

    /// 解析工作区相对路径并保证不越界
    pub fn workspace_path(workspace: &Path, requested: &str) -> Result<PathBuf, String> {
        let mut result = workspace.to_path_buf();
        for segment in requested.replace('\\', "/").split('/') {
            if segment.is_empty() || segment == "." {
                continue;
            }
            if segment == ".." {
                return Err("workspace path escapes the task workspace".to_string());
            }
            result.push(segment);
        }
        if result != workspace && !result.starts_with(workspace) {
            return Err("workspace path escapes the task workspace".to_string());
        }
        Ok(result)
    }
}

fn looks_like_drive_prefix(name: &str) -> bool {
    let bytes = name.as_bytes();
    bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':'
}

#[derive(Debug)]
pub struct ExtractionResult {
    pub signed: bool,
    pub total_entries: u64,
}

/// 把 input_path 的每个条目安全解到 workspace（目录需已存在）
pub fn extract_archive(
    input_path: &Path,
    workspace: &Path,
    limits: &ExtractionLimits,
) -> Result<ExtractionResult, String> {
    let metadata =
        std::fs::metadata(input_path).map_err(|e| format!("unable to stat input JAR: {e}"))?;
    if !metadata.is_file() {
        return Err("input path is not a file".to_string());
    }
    if metadata.len() > limits.max_archive_bytes {
        return Err("JAR file exceeds the safe size limit".to_string());
    }

    let file = File::open(input_path).map_err(|e| format!("unable to open input JAR: {e}"))?;
    let mut archive = ZipArchive::new(file)
        .map_err(|e| format!("the file is not a valid JAR/ZIP archive: {e}"))?;

    let mut signed = false;
    let mut total_entries = 0u64;
    let mut uncompressed_bytes = 0u64;
    let mut manifest = ArchiveManifest {
        version: 1,
        entries: Vec::with_capacity(archive.len().min(100_000)),
    };

    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(|e| format!("unable to read archive entry {index}: {e}"))?;
        let raw_name = entry.name().to_string();
        let name = PathPolicy::safe_entry_name(&raw_name)?;
        total_entries += 1;
        if total_entries > limits.max_entries as u64 {
            return Err("JAR entry count exceeds the safety limit".to_string());
        }
        let entry_size = entry.size();
        uncompressed_bytes = uncompressed_bytes.saturating_add(entry_size);
        if entry_size > limits.max_entry_bytes {
            return Err(format!("JAR entry is too large: {name}"));
        }
        if uncompressed_bytes > limits.max_uncompressed_bytes {
            return Err("JAR expands beyond the safety limit".to_string());
        }
        if entry.compressed_size() > 0
            && entry_size as f64 / entry.compressed_size() as f64 > limits.max_compression_ratio
        {
            return Err(format!(
                "JAR entry has an abnormal compression ratio: {name}"
            ));
        }
        if entry.is_symlink() {
            return Err(format!("JAR contains a symbolic link: {name}"));
        }
        if PathPolicy::is_signature_file(&name) {
            signed = true;
        }
        if name.ends_with('/') {
            continue;
        }

        let output_path = PathPolicy::workspace_path(workspace, &name)?;
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("unable to create workspace directory for {name}: {e}"))?;
        }
        let mut output = File::create(&output_path)
            .map_err(|e| format!("unable to create workspace file for {name}: {e}"))?;
        std::io::copy(&mut entry, &mut output)
            .map_err(|e| format!("unable to extract {name}: {e}"))?;
        manifest.entries.push(ArchiveEntry {
            archive_path: name.clone(),
            workspace_path: name,
        });
    }

    manifest
        .write(workspace)
        .map_err(|e| format!("unable to write archive manifest: {e}"))?;
    Ok(ExtractionResult {
        signed,
        total_entries,
    })
}

/// 内部文件（不写入输出 JAR）
const INTERNAL_PREFIX: &str = ".mod-translator-";

fn is_internal_file(name: &str) -> bool {
    name == ARCHIVE_MANIFEST || name.starts_with(INTERNAL_PREFIX)
}

/// 把工作区重新打成 JAR，用 manifest 还原原始条目名；新生成的 zh_cn 文件按相对路径加入
pub fn package_archive(
    workspace: &Path,
    output_path: &Path,
    manifest: &ArchiveManifest,
) -> Result<(), String> {
    if let Some(parent) = output_path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("unable to create output directory: {e}"))?;
        }
    }
    if output_path.exists() {
        return Err(format!(
            "output file already exists: {}",
            output_path.display()
        ));
    }

    let temporary = output_path.with_extension("jar.partial");
    let _ = std::fs::remove_file(&temporary);
    let file =
        File::create(&temporary).map_err(|e| format!("unable to create temporary JAR: {e}"))?;
    let mut writer = zip::ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    for entry in &manifest.entries {
        let source = PathPolicy::workspace_path(workspace, &entry.workspace_path)?;
        if !source.is_file() {
            continue;
        }
        let bytes = std::fs::read(&source)
            .map_err(|e| format!("unable to read {}: {e}", source.display()))?;
        writer
            .start_file(&entry.archive_path, options)
            .map_err(|e| format!("unable to write archive entry {}: {e}", entry.archive_path))?;
        writer
            .write_all(&bytes)
            .map_err(|e| format!("unable to write archive entry {}: {e}", entry.archive_path))?;
    }

    // 新生成的 zh_cn 等文件（不在 manifest 中且非内部文件）一并打包
    for relative in collect_files(workspace)? {
        if is_internal_file(&relative) {
            continue;
        }
        if manifest
            .entries
            .iter()
            .any(|entry| entry.workspace_path == relative)
        {
            continue;
        }
        let bytes = std::fs::read(workspace.join(&relative))
            .map_err(|e| format!("unable to read {relative}: {e}"))?;
        writer
            .start_file(&relative, options)
            .map_err(|e| format!("unable to write archive entry {relative}: {e}"))?;
        writer
            .write_all(&bytes)
            .map_err(|e| format!("unable to write archive entry {relative}: {e}"))?;
    }

    writer
        .finish()
        .map_err(|e| format!("unable to finalize output JAR: {e}"))?;
    std::fs::rename(&temporary, output_path)
        .map_err(|e| format!("unable to move temporary JAR into place: {e}"))?;
    Ok(())
}

/// 递归收集工作区相对文件路径（排序）
pub fn collect_files(root: &Path) -> Result<Vec<String>, String> {
    let mut result = Vec::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        let entries = std::fs::read_dir(&directory)
            .map_err(|e| format!("unable to read workspace directory {directory:?}: {e}"))?;
        for entry in entries {
            let entry = entry.map_err(|e| format!("unable to read workspace entry: {e}"))?;
            let path = entry.path();
            if path.is_dir() {
                pending.push(path);
            } else if path.is_file() {
                let relative = path
                    .strip_prefix(root)
                    .map_err(|_| "workspace path prefix error")?;
                result.push(relative.to_string_lossy().replace('\\', "/"));
            }
        }
    }
    result.sort();
    Ok(result)
}
